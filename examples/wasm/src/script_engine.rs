use core::mem;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use dot_http_lib::script_engine::{Script, ScriptEngine, INIT_SCRIPT};
use dot_http_lib::Result;

/// THIS IS COMPLETELY UN-SANDBOXED, it runs everything in the global scope
pub struct BrowserScriptEngine {
    env: serde_json::Value,
}

impl BrowserScriptEngine {
    pub fn new(env: serde_json::Value, snapshot_script: &str) -> Result<Self> {
        let mut engine = BrowserScriptEngine {
            // set null while we still use our env
            env: serde_json::Value::Null,
        };
        engine.declare(&env)?;

        let script = format!(
            r#"
                var _env = {};
            "#,
            &env
        );
        engine.env = env;

        engine.execute_script(&Script::internal_script(&script))?;

        let snapshot: serde_json::Value = serde_json::from_str(snapshot_script)?;
        engine.declare(&snapshot)?;
        let snapshot = format!("var _snapshot = {};", snapshot);
        engine.execute_script(&Script::internal_script(snapshot.as_str()))?;

        engine.execute_script(&Script::internal_script(INIT_SCRIPT))?;

        Ok(engine)
    }
}

impl ScriptEngine for BrowserScriptEngine {
    fn execute_script(&mut self, script: &Script) -> Result<String> {
        let result =
            global_eval(script.src).map_err(|err| anyhow!("Error calling eval: {:?}", err))?;

        Ok(result.as_string().unwrap_or_else(|| String::new()))
    }

    fn reset(&mut self) -> Result<()> {
        let snapshot = self.snapshot()?;
        let env = mem::replace(&mut self.env, serde_json::Value::Null);
        *self = BrowserScriptEngine::new(env, &snapshot)?;

        Ok(())
    }

    fn snapshot(&mut self) -> Result<String> {
        let script = "JSON.stringify(_snapshot)";
        let out = self.execute_script(&Script::internal_script(script))?;
        Ok(out)
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = globalEval, catch)]
    fn global_eval(script: &str) -> core::result::Result<JsValue, JsValue>;
}
