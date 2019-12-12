use crate::request_script::*;
use std::convert::From;
use std::fmt::{Debug, Formatter};

pub mod boa;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Script;
#[derive(Debug)]
pub struct Execute;
#[derive(Debug)]
pub struct Parse;

#[derive(Debug)]
pub struct Error<K> {
    kind: K,
}

impl<T: Debug> std::error::Error for Error<T> {}
impl<T: Debug> std::fmt::Display for Error<T> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        unimplemented!()
    }
}

impl From<Error<Parse>> for Error<Script> {
    fn from(_: Error<Parse>) -> Self {
        unimplemented!()
    }
}

impl From<Error<Execute>> for Error<Script> {
    fn from(_: Error<Execute>) -> Self {
        unimplemented!()
    }
}

pub trait Processable<T: ScriptEngine> {
    type Output;
    fn process(&self, _engine: &mut T) -> Result<Self::Output, Error<Script>>;
}

impl<T: ScriptEngine> Processable<T> for RequestScript<Unprocessed> {
    type Output = RequestScript<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error<Script>> {
        Ok(RequestScript {
            request: self.request.process(engine)?,
            handler: self.handler.clone(),
            selection: self.selection.clone(),
        })
    }
}

impl<T: ScriptEngine> Processable<T> for Request<Unprocessed> {
    type Output = Request<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error<Script>> {
        let mut headers = vec![];
        for header in &self.headers {
            headers.push(header.process(engine)?);
        }

        let body = match &self.body {
            Some(value) => Some(value.process(engine)?),
            None => None,
        };

        Ok(Request {
            method: self.method.clone(),
            target: self.target.process(engine)?,
            headers,
            body,
            selection: self.selection.clone(),
        })
    }
}

impl<T: ScriptEngine> Processable<T> for Header<Unprocessed> {
    type Output = Header<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error<Script>> {
        Ok(Header {
            field_name: self.field_name.clone(),
            field_value: self.field_value.process(engine)?,
            selection: self.selection.clone(),
        })
    }
}

pub trait ScriptEngine {
    type Expression;
    type EnvExpression;

    fn execute(&mut self, expression: Self::Expression) -> Result<String, Error<Execute>>;
    fn parse(&mut self, script: String) -> Result<Self::Expression, Error<Parse>>;
    fn parse_env(&mut self, script: String) -> Result<Self::EnvExpression, Error<Parse>>;
    fn execute_env(
        &mut self,
        expression: Self::EnvExpression,
        env: String,
    ) -> Result<String, Error<Execute>>;
}

impl<E, T: ScriptEngine<Expression = E>> Processable<T> for Value<Unprocessed> {
    type Output = Value<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error<Script>> {
        if let Value {
            state:
                Unprocessed::WithInline {
                    value,
                    inline_scripts,
                    selection: _selection,
                },
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
                .collect::<Result<Vec<(&String, E)>, Error<Parse>>>()?;

            let mut interpolated = value.clone();
            for (placeholder, expr) in evaluated {
                let result = engine.execute(expr)?.to_string();
                interpolated = interpolated.replacen(placeholder.as_str(), result.as_str(), 1);
            }

            return Ok(Value {
                state: Processed {
                    value: interpolated,
                },
            });
        }

        if let Value {
            state: Unprocessed::WithoutInline(value, _),
        } = self
        {
            return Ok(Value {
                state: Processed {
                    value: value.clone(),
                },
            });
        }
        Err(Error { kind: Script {} })
    }
}
