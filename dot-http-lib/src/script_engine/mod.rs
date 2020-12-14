use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;

use crate::parser::Selection;
use crate::Result;

pub const INIT_SCRIPT: &str = include_str!("init.js");

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
    fn execute_script(&mut self, script: &Script) -> crate::Result<String>;

    fn reset(&mut self) -> crate::Result<()>;

    fn snapshot(&mut self) -> crate::Result<String>;

    fn handle(&mut self, script: &Script, response: &crate::Response) -> crate::Result<()> {
        inject(self, response)?;
        self.execute_script(&script)?;
        Ok(())
    }

    fn process(&mut self, value: Value<Unprocessed>) -> crate::Result<Value<Processed>> {
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

    fn declare(&mut self, variables_object: &serde_json::Value) -> Result<()> {
        if let serde_json::Value::Object(map) = variables_object {
            for (key, value) in map {
                let script = serde_json::to_string(&value)?;
                let script = format!("this['{}'] = {};", key, script);
                self.execute_script(&Script::internal_script(script.as_str()))?;
            }
            Ok(())
        } else {
            Err(anyhow!("Failed to declare object: {:?}", variables_object))
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
        for (key, value) in response.headers() {
            headers.insert(
                key.to_string(),
                serde_json::Value::String(String::from_utf8_lossy(value.as_bytes()).to_string()),
            );
        }
        Response {
            body: response.body().clone(),
            headers,
            status: response.status().as_u16(),
        }
    }
}

pub fn inject<S>(engine: &mut S, response: &crate::Response) -> Result<()>
where
    S: ScriptEngine + ?Sized,
{
    let response: Response = response.into();

    let script = format!(
        "var response = {};",
        serde_json::to_string(&response).map_err(|e| anyhow!("Failed to serialize response: {}", e))?
    );
    engine.execute_script(&Script::internal_script(&script))?;
    if let Some(body) = response.body {
        if let Ok(serde_json::Value::Object(response_body)) = serde_json::from_str(body.as_str()) {
            let script = format!(
                "response.body = {};",
                serde_json::to_string(&response_body).map_err(|e| anyhow!("Failed to serialize body: {}", e))?
            );
            engine
                .execute_script(&Script::internal_script(&script))?;
        }
    }
    Ok(())
}
