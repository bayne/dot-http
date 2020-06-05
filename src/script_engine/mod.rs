use crate::model::*;
use crate::script_engine::ErrorKind::ParseInitializeObject;
use std::fmt::{Debug, Formatter};

#[cfg(feature = "boa")]
pub mod boa;

#[cfg(feature = "rusty_v8")]
pub mod v8;

#[cfg(test)]
mod tests;

pub fn create_script_engine() -> Box<dyn ScriptEngine> {
    if let Some(engine) = create_script_boa_engine() {
        engine
    } else if let Some(engine) = create_script_v8_engine() {
        engine
    } else {
        panic!("No Script Engine compiled in the binary");
    }
}

#[cfg(feature = "boa")]
fn create_script_boa_engine() -> Option<Box<dyn ScriptEngine>> {
    use crate::script_engine::boa::BoaScriptEngine;
    Some(Box::new(BoaScriptEngine::new()))
}
#[cfg(not(feature = "boa"))]
fn create_script_boa_engine() -> Option<Box<dyn ScriptEngine>> {
    None
}

#[cfg(feature = "rusty_v8")]
fn create_script_v8_engine() -> Option<Box<dyn ScriptEngine>> {
    use crate::script_engine::v8::V8ScriptEngine;
    Some(Box::new(V8ScriptEngine::new()))
}
#[cfg(not(feature = "rusty_v8"))]
fn create_script_v8_engine() -> Option<Box<dyn ScriptEngine>> {
    None
}

#[derive(Debug)]
pub struct Error {
    selection: Selection,
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    ParseInitializeObject(String),
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

pub trait Processable {
    type Output;
    fn process(&self, _engine: &mut dyn ScriptEngine) -> Result<Self::Output, Error>;
}

impl Processable for RequestScript<Unprocessed> {
    type Output = RequestScript<Processed>;
    fn process(&self, engine: &mut dyn ScriptEngine) -> Result<Self::Output, Error> {
        Ok(RequestScript {
            request: self.request.process(engine)?,
            handler: self.handler.clone(),
            selection: self.selection.clone(),
        })
    }
}

impl Processable for Request<Unprocessed> {
    type Output = Request<Processed>;
    fn process(&self, engine: &mut dyn ScriptEngine) -> Result<Self::Output, Error> {
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

impl Processable for Header<Unprocessed> {
    type Output = Header<Processed>;
    fn process(&self, engine: &mut dyn ScriptEngine) -> Result<Self::Output, Error> {
        Ok(Header {
            field_name: self.field_name.clone(),
            field_value: self.field_value.process(engine)?,
            selection: self.selection.clone(),
        })
    }
}

pub struct Script<'a> {
    pub selection: Selection,
    pub src: &'a str,
}

impl<'a> Script<'a> {
    pub fn internal_script(src: &str) -> Script {
        Script {
            src,
            selection: Selection::none(),
        }
    }
}

pub trait ScriptEngine {
    fn execute_script(&mut self, script: &Script) -> Result<String, Error>;

    fn empty(&self) -> String;

    fn initialize(&mut self, env_script: &str, env: &str) -> Result<(), Error>;
    fn reset(&mut self, snapshot_script: &str) -> Result<(), Error>;

    fn snapshot(&mut self) -> Result<String, Error>;
}

impl Processable for Value<Unprocessed> {
    type Output = Value<Processed>;
    fn process(&self, engine: &mut dyn ScriptEngine) -> Result<Self::Output, Error> {
        match self {
            Value {
                state:
                    Unprocessed::WithInline {
                        value,
                        inline_scripts,
                        selection: _selection,
                    },
            } => {
                let mut interpolated = value.clone();
                for inline_script in inline_scripts {
                    let placeholder = inline_script.placeholder.clone();
                    let result = engine.execute_script(&Script {
                        selection: inline_script.selection.clone(),
                        src: &inline_script.script,
                    })?;
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
