use crate::*;
use boa::builtins::value::ValueData;
use boa::exec::{Executor, Interpreter};
use boa::realm::Realm;
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

trait Processable {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        Ok(())
    }
}

//impl Processable for ValueWithInline {
//    type Evaluated = String;

//    fn script(&self) -> &str {
//        self.value.as_str()
//    }
//}

impl Processable for Handler {
    //    type Evaluated = ();

    //    fn script(&self) -> &str {
    //        unimplemented!()
    //    }
}

impl Processable for File {
    //    type Evaluated = File;

    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        for mut request_script in &mut self.request_scripts {
            request_script.process(engine)?;
        }
        Ok(())
        //        Ok(File {
        //            request_scripts: self
        //                .request_scripts
        //                .iter()
        //                .map(|request_script| request_script.process())
        //                .collect::<Result<Vec<RequestScript>, Error>>()?,
        //        })
    }
}

impl Processable for RequestScript {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        self.request.process(engine)
        //        Ok(RequestScript {
        //            request: self.request.process()?,
        //        })
    }
}

impl Processable for Request {
    fn process(&mut self, engine: &mut Interpreter) -> Result<(), Error> {
        for mut header in &mut self.headers {
            header.process(engine)?;
        }

        self.target.process(engine)?;

        if let Some(value) = &mut self.body {
            value.process(engine)?;
        }

        Ok(())
        //        unimplemented!()
        //        Ok(Request {
        //            method: self.method,
        //            target: self.target,
        //            headers: vec![],
        //            body: None,
        //            handler: None,
        //        })
    }
}

impl Processable for Header {}

fn parser_expr(src: &str) -> Result<Expr, Error> {
    let mut lexer = Lexer::new(src);
    lexer.lex()?;
    let tokens = lexer.tokens;
    // need to remove unwrap
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
                        // pull this out so we can ?
                        parser_expr(inline_script.script.as_str())?,
                    ))
                })
                .collect::<Result<Vec<(&String, Expr)>, Error>>()?;

            let mut interpolated = value.clone();
            for (placeholder, expr) in evaluated {
                dbg!(placeholder);
                dbg!(&expr);
                let result = engine.run(&expr)?.to_string();
                interpolated = interpolated.replacen(placeholder.as_str(), result.as_str(), 1);
            }

            *self = Value::WithoutInline(interpolated);
        }
        Ok(())
    }
}

pub(crate) fn pre_process(file: &mut File) -> Result<(), Error> {
    let realm = Realm::create();
    let mut engine: Interpreter = Executor::new(realm);

    file.process(&mut engine)?;
    println!("{:#?}", file);
    Ok(())
    //    file.requests.iter().map(|request_script| request_script.request.body.map(|value_with_inline| {
    //        value_with_inline.value = "What".to_string()
    //    }));
    //    Ok(file)
}
