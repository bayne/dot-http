use crate::model::*;
use crate::script_engine::ErrorKind::ParseInitializeObject;
use std::fmt::{Debug, Formatter};

pub mod boa;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Error {
    selection: Selection,
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    ParseInitializeObject(&'static str),
    Execute(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.kind {
            ParseInitializeObject(e) => f.write_fmt(format_args!(
                "{selection}:, Could not parse initialize object, {error}",
                selection = self.selection,
                error = e
            )),
            ErrorKind::Execute(e) => f.write_fmt(format_args!(
                "{selection}: {error}",
                selection = self.selection,
                error = e.as_str()
            )),
        }
    }
}

pub trait Processable<T: ScriptEngine> {
    type Output;
    fn process(&self, _engine: &mut T) -> Result<Self::Output, Error>;
}

impl<T: ScriptEngine> Processable<T> for RequestScript<Unprocessed> {
    type Output = RequestScript<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error> {
        Ok(RequestScript {
            request: self.request.process(engine)?,
            handler: self.handler.clone(),
            selection: self.selection.clone(),
        })
    }
}

impl<T: ScriptEngine> Processable<T> for Request<Unprocessed> {
    type Output = Request<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error> {
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
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error> {
        Ok(Header {
            field_name: self.field_name.clone(),
            field_value: self.field_value.process(engine)?,
            selection: self.selection.clone(),
        })
    }
}

pub struct Expression<T> {
    selection: Selection,
    expr: T,
}

pub struct Script {
    pub selection: Selection,
    pub src: String,
}

impl Script {
    pub fn internal_script(src: String) -> Script {
        Script {
            src,
            selection: Selection::none(),
        }
    }
}

pub trait ScriptEngine {
    type Expr;

    fn process_script(&mut self, expression: Expression<Self::Expr>) -> Expression<Self::Expr>;
    fn execute(&mut self, expression: Expression<Self::Expr>) -> Result<String, Error>;
    fn parse(&mut self, script: Script) -> Result<Expression<Self::Expr>, Error>;
    fn empty() -> &'static str;

    fn initialize(
        &mut self,
        env_script: &str,
        env: &str,
        snapshot_script: &str,
    ) -> Result<(), Error>;

    fn snapshot(&mut self) -> Result<String, Error>;
}

impl<E, T: ScriptEngine<Expr = E>> Processable<T> for Value<Unprocessed> {
    type Output = Value<Processed>;
    fn process(&self, engine: &mut T) -> Result<Self::Output, Error> {
        match self {
            Value {
                state:
                    Unprocessed::WithInline {
                        value,
                        inline_scripts,
                        selection: _selection,
                    },
            } => {
                let evaluated = inline_scripts
                    .iter()
                    .map(|inline_script| {
                        Ok((
                            &inline_script.placeholder,
                            engine.parse(Script {
                                selection: inline_script.selection.clone(),
                                src: inline_script.script.clone(),
                            })?,
                        ))
                    })
                    .collect::<Result<Vec<(&String, Expression<E>)>, Error>>()?;

                let mut interpolated = value.clone();
                for (placeholder, expr) in evaluated {
                    let expr = engine.process_script(expr);
                    let result = engine.execute(expr)?;
                    interpolated = interpolated.replacen(placeholder.as_str(), result.as_str(), 1);
                }

                Ok(Value {
                    state: Processed {
                        value: interpolated,
                    },
                })
            }
            Value {
                state: Unprocessed::WithoutInline(value, _),
            } => Ok(Value {
                state: Processed {
                    value: value.clone(),
                },
            }),
        }
    }
}
