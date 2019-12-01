use crate::*;
use boa::builtins::value::ValueData;
use boa::exec::{Executor, Interpreter};
use boa::syntax::ast::expr::Expr;
use boa::syntax::lexer::{Lexer, LexerError};
use boa::syntax::parser::ParseError;
use boa::syntax::parser::Parser;
use gc::Gc;
use std::convert::From;

#[cfg(test)]
mod tests;

impl From<LexerError> for Error {
    fn from(e: LexerError) -> Self {
        Error {
            kind: ErrorKind::ScriptRun,
            message: format!("{}", e),
        }
    }
}

impl From<Gc<ValueData>> for Error {
    fn from(e: Gc<ValueData>) -> Self {
        Error {
            kind: ErrorKind::ScriptRun,
            message: format!("{}", e),
        }
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error {
            kind: ErrorKind::Parse,
            message: format!("{:?}", e),
        }
    }
}

pub(crate) trait Processable {
    fn process(&mut self, _engine: &mut Interpreter) -> Result<(), Error>;
}

impl Processable for File {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        for request_script in &mut self.request_scripts {
            request_script.process(engine)?;
        }
        Ok(())
    }
}

impl Processable for RequestScript {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        self.request.process(engine)
    }
}

impl Processable for Request {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        for header in &mut self.headers {
            header.process(engine)?;
        }

        self.target.process(engine)?;

        if let Some(value) = &mut self.body {
            value.process(engine)?;
        }

        Ok(())
    }
}

impl Processable for Header {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        self.field_value.process(engine)?;
        Ok(())
    }
}

pub fn parser_expr(src: &str) -> Result<Expr, Error> {
    let mut lexer = Lexer::new(src);
    lexer.lex()?;
    let tokens = lexer.tokens;
    Ok(Parser::new(tokens).parse_all()?)
}

impl Processable for Value {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        if let Value::WithInline {
            value,
            inline_scripts,
        } = self
        {
            let evaluated: Vec<(&String, Expr)> = inline_scripts
                .iter()
                .map(|inline_script| {
                    Ok((
                        &inline_script.placeholder,
                        parser_expr(inline_script.script.as_str())?,
                    ))
                })
                .collect::<Result<Vec<(&String, Expr)>, Error>>()?;

            let mut interpolated = value.clone();
            for (placeholder, expr) in evaluated {
                let result = engine.run(&expr)?.to_string();
                interpolated = interpolated.replacen(placeholder.as_str(), result.as_str(), 1);
            }

            *self = Value::WithoutInline(interpolated);
        }
        Ok(())
    }
}
