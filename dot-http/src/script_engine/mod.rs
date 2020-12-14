use dot_http_lib::script_engine::{Script, ScriptEngine};

#[cfg(feature = "boa")]
pub mod boa;

#[cfg(feature = "rusty_v8")]
pub mod v8;

#[cfg(test)]
mod tests;

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
