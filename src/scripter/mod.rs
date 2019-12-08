use crate::Header;
use crate::Processed;
use crate::Request;
use crate::RequestScript;
use crate::Unprocessed;
use crate::Unprocessed::WithInline;
use crate::Unprocessed::WithoutInline;
use crate::Value;
use std::convert::From;

pub(crate) mod boa;

#[cfg(test)]
mod tests;

impl From<ParseError> for ScriptError {
    fn from(_e: ParseError) -> Self {
        unimplemented!()
    }
}

impl From<ExecuteError> for ScriptError {
    fn from(_e: ExecuteError) -> Self {
        unimplemented!()
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
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for ExecuteError {}
impl std::fmt::Display for ExecuteError {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

impl std::error::Error for ParseError {}
impl std::fmt::Display for ParseError {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}

pub trait Processable<T: ScriptEngine> {
    type Output;
    fn process(&self, _engine: &mut T) -> Result<Self::Output, ScriptError>;
}

impl<T: ScriptEngine> Processable<T> for RequestScript<Unprocessed> {
    type Output = RequestScript<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, ScriptError> {
        Ok(RequestScript {
            request: self.request.process(engine)?,
            handler: self.handler.clone(),
            selection: self.selection.clone(),
        })
    }
}

impl<T: ScriptEngine> Processable<T> for Request<Unprocessed> {
    type Output = Request<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, ScriptError> {
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
    fn process(&self, engine: &mut T) -> Result<Self::Output, ScriptError> {
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

    fn execute(&mut self, expression: Self::Expression) -> Result<String, ExecuteError>;
    fn parse(&mut self, script: String) -> Result<Self::Expression, ParseError>;
    fn parse_env(&mut self, script: String) -> Result<Self::EnvExpression, ParseError>;
    fn execute_env(
        &mut self,
        expression: Self::EnvExpression,
        env: String,
    ) -> Result<String, ExecuteError>;
}

impl<E, T: ScriptEngine<Expression = E>> Processable<T> for Value<Unprocessed> {
    type Output = Value<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, ScriptError> {
        if let Value {
            state:
                WithInline {
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
                .collect::<Result<Vec<(&String, E)>, ParseError>>()?;

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
            state: WithoutInline(value, _),
        } = self
        {
            return Ok(Value {
                state: Processed {
                    value: value.clone(),
                },
            });
        }
        Err(ScriptError {})
    }
}
