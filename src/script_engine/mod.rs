use crate::parser::Selection;
use crate::Result;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use std::fmt::Debug;

#[cfg(feature = "boa")]
pub mod boa;

#[cfg(feature = "rusty_v8")]
pub mod v8;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Value<S> {
    pub state: S,
}

#[derive(Debug)]
pub struct Processed {
    pub value: String,
}

#[derive(Debug)]
pub enum Unprocessed {
    WithInline {
        value: String,
        inline_scripts: Vec<InlineScript>,
        selection: Selection,
    },
    WithoutInline(String, Selection),
}

#[derive(Debug)]
pub struct InlineScript {
    pub script: String,
    pub placeholder: String,
    pub selection: Selection,
}

pub fn create_script_engine(
    env_script: &str,
    env: &str,
    snapshot_script: &str,
) -> Box<dyn ScriptEngine> {
    if let Some(engine) = create_script_boa_engine(env_script, env, snapshot_script) {
        engine
    } else if let Some(engine) = create_script_v8_engine(env_script, env, snapshot_script) {
        engine
    } else {
        panic!("No Script Engine compiled in the binary");
    }
}

#[cfg(feature = "boa")]
fn create_script_boa_engine(
    env_script: &str,
    env: &str,
    snapshot_script: &str,
) -> Option<Box<dyn ScriptEngine>> {
    use crate::script_engine::boa::BoaScriptEngine;
    Some(Box::new(
        BoaScriptEngine::new(env_script, env, snapshot_script).unwrap(),
    ))
}
#[cfg(not(feature = "boa"))]
fn create_script_boa_engine(
    _env_script: &str,
    _env: &str,
    _snapshot_script: &str,
) -> Option<Box<dyn ScriptEngine>> {
    None
}

#[cfg(feature = "rusty_v8")]
fn create_script_v8_engine(
    env_script: &str,
    env: &str,
    snapshot_script: &str,
) -> Option<Box<dyn ScriptEngine>> {
    use crate::script_engine::v8::V8ScriptEngine;
    Some(Box::new(
        V8ScriptEngine::new(env_script, env, snapshot_script).unwrap(),
    ))
}
#[cfg(not(feature = "rusty_v8"))]
fn create_script_v8_engine(
    _env_script: &str,
    _env: &str,
    _snapshot_script: &str,
) -> Option<Box<dyn ScriptEngine>> {
    None
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
    fn execute_script(&mut self, script: &Script) -> Result<String>;

    fn empty(&self) -> String;

    fn reset(&mut self) -> Result<()>;

    fn snapshot(&mut self) -> Result<String>;

    fn handle(&mut self, script: &Script, response: &crate::Response) -> Result<()>;

    fn process(&mut self, value: Value<Unprocessed>) -> Result<Value<Processed>> {
        match value {
            Value {
                state:
                    Unprocessed::WithInline {
                        value,
                        inline_scripts,
                        selection: _selection,
                    },
            } => {
                let mut interpolated = value;
                for inline_script in inline_scripts {
                    let placeholder = inline_script.placeholder.clone();
                    let result = self.execute_script(&Script {
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
                state: Processed { value },
            }),
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Response {
    body: Option<String>,
    headers: Map<String, serde_json::Value>,
    status: u16,
}

impl From<&crate::Response> for Response {
    fn from(response: &crate::Response) -> Self {
        let mut headers = Map::new();
        for (key, value) in response.headers.as_slice() {
            headers.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        Response {
            body: response.body.clone(),
            headers,
            status: response.status_code,
        }
    }
}

fn handle(
    engine: &mut dyn ScriptEngine,
    script: &Script,
    response: &crate::Response,
) -> Result<()> {
    inject(engine, response)?;
    engine.execute_script(script)?;
    Ok(())
}

fn inject(engine: &mut dyn ScriptEngine, response: &crate::Response) -> Result<()> {
    let response: Response = response.into();

    let script = format!(
        "var response = {};",
        serde_json::to_string(&response).unwrap()
    );
    engine.execute_script(&Script::internal_script(&script))?;
    if let Some(body) = response.body {
        if let Ok(serde_json::Value::Object(response_body)) = serde_json::from_str(body.as_str()) {
            let script = format!(
                "response.body = {};",
                serde_json::to_string(&response_body).unwrap()
            );
            engine
                .execute_script(&Script::internal_script(&script))
                .unwrap();
        }
    }
    Ok(())
}
