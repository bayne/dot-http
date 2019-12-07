use crate::{File, Header, Request, RequestScript, Value};
use std::convert::From;

pub(crate) mod boa;

#[cfg(test)]
mod tests;

impl From<ParseError> for ScriptError {
    fn from(e: ParseError) -> Self {
        unimplemented!()
        //        Error {
        //            kind: ErrorKind::Parse,
        //            message: format!("{:?}", e),
        //        }
    }
}

impl From<ExecuteError> for ScriptError {
    fn from(e: ExecuteError) -> Self {
        unimplemented!()
        //        Error {
        //            kind: ErrorKind::Parse,
        //            message: format!("{:?}", e),
        //        }
    }
}

#[derive(Debug)]
pub struct ScriptError;
#[derive(Debug)]
pub struct ExecuteError;
#[derive(Debug)]
pub struct ParseError;

impl std::error::Error for ScriptError {}
impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for ExecuteError {}
impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

pub trait Processable<T: ScriptEngine> {
    fn process(&mut self, _engine: &mut T) -> Result<(), ScriptError>;
}

impl<T: ScriptEngine> Processable<T> for File {
    fn process(&mut self, engine: &mut T) -> Result<(), ScriptError> {
        for request_script in &mut self.request_scripts {
            request_script.process(engine)?;
        }
        Ok(())
    }
}

impl<T: ScriptEngine> Processable<T> for RequestScript {
    fn process(&mut self, engine: &mut T) -> Result<(), ScriptError> {
        self.request.process(engine)
    }
}

impl<T: ScriptEngine> Processable<T> for Request {
    fn process(&mut self, engine: &mut T) -> Result<(), ScriptError> {
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

impl<T: ScriptEngine> Processable<T> for Header {
    fn process(&mut self, engine: &mut T) -> Result<(), ScriptError> {
        self.field_value.process(engine)?;
        Ok(())
    }
}

pub trait ScriptEngine {
    type Expression;
    type EnvExpression;

    fn execute(&mut self, expression: Self::Expression) -> Result<String, ExecuteError>;
    fn parse(&mut self, script: String) -> Result<Self::Expression, ParseError>;
    fn parse_env(&mut self, script: String) -> Result<Self::EnvExpression, ParseError>;
    fn execute_env(
        &mut self,
        expression: Self::EnvExpression,
        env: String,
    ) -> Result<String, ExecuteError>;
}

impl<E, T: ScriptEngine<Expression = E>> Processable<T> for Value {
    fn process(&mut self, engine: &mut T) -> Result<(), ScriptError> {
        if let Value::WithInline {
            value,
            inline_scripts,
        } = self
        {
            let evaluated = inline_scripts
                .iter()
                .map(|inline_script| {
                    Ok((
                        &inline_script.placeholder,
                        engine.parse(inline_script.script.clone())?,
                    ))
                })
                .collect::<Result<Vec<(&String, E)>, ParseError>>()?;

            let mut interpolated = value.clone();
            for (placeholder, expr) in evaluated {
                let result = engine.execute(expr)?.to_string();
                interpolated = interpolated.replacen(placeholder.as_str(), result.as_str(), 1);
            }

            *self = Value::WithoutInline(interpolated);
        }
        Ok(())
    }
}
