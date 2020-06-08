use crate::model::Selection;
use crate::script_engine::{Error, ErrorKind, Script, ScriptEngine};
use rusty_v8::{
    inspector::{
        StringView, V8Inspector, V8InspectorClientBase, V8InspectorClientImpl, V8StackTrace,
    },
    scope::Entered,
    Context, ContextScope, Exception, Global, HandleScope, Isolate, OwnedIsolate,
    Script as V8Script, String as V8String, TryCatch, V8,
};
use std::convert::From;
use std::sync::Once;

static V8_INIT: Once = Once::new();
pub struct V8ScriptEngine {
    isolate: OwnedIsolate,
    global: Global<Context>,
    env_file: String,
    env: String,
}

impl From<serde_json::error::Error> for Error {
    fn from(_: serde_json::error::Error) -> Self {
        unimplemented!()
    }
}

impl V8ScriptEngine {
    pub fn new(
        env_script: &str,
        env: &str,
        snapshot_script: &str,
    ) -> Result<V8ScriptEngine, Error> {
        V8_INIT.call_once(|| {
            let platform = rusty_v8::new_default_platform().unwrap();
            V8::initialize_platform(platform);
            V8::initialize();
        });

        let mut isolate = Isolate::new(Default::default());
        let mut global = Global::<Context>::new();
        let mut handle_scope = HandleScope::new(&mut isolate);
        let scope = handle_scope.enter();
        let context = Context::new(scope);
        global.set(scope, context);

        let mut engine = V8ScriptEngine {
            isolate,
            global,
            env_file: env_script.to_string(),
            env: env.to_string(),
        };

        let environment: serde_json::Value = serde_json::from_str(env_script)?;
        if let Some(environment) = environment.get(env) {
            declare(&mut engine, environment)?;
            let script = format!(
                r#"
            var _env_file = {};
            var _env = _env_file['{}'];
            "#,
                &env_script, &env
            );
            engine.execute_script(&Script::internal_script(script.as_str()))?;
        }

        let snapshot: serde_json::Value = serde_json::from_str(snapshot_script).unwrap();
        declare(&mut engine, &snapshot)?;
        let snapshot = format!("var _snapshot = {};", snapshot);
        engine.execute_script(&Script::internal_script(snapshot.as_str()))?;

        let script = include_str!("init.js");
        engine.execute_script(&Script::internal_script(script))?;

        Ok(engine)
    }
}

fn catch(
    script: &Script,
    tc: &mut TryCatch,
    scope: &mut Entered<ContextScope, Entered<HandleScope, OwnedIsolate>>,
) -> Error {
    let exception = tc.exception().unwrap();
    let msg = Exception::create_message(scope, exception);
    Error {
        selection: script.selection.clone(),
        kind: ErrorKind::Execute(msg.get(scope).to_rust_string_lossy(scope)),
    }
}

impl ScriptEngine for V8ScriptEngine {
    fn execute_script(&mut self, script: &Script) -> Result<String, Error> {
        let isolate = &mut self.isolate;
        let mut logger = ConsoleLogger::new();
        let mut inspector = V8Inspector::create(isolate, &mut logger);
        let mut scope = HandleScope::new(isolate);
        let scope = scope.enter();

        let context = self.global.get(scope).unwrap();

        let mut scope = ContextScope::new(scope, context);
        let scope = scope.enter();

        let name = b"";
        let name_view = StringView::from(&name[..]);
        inspector.context_created(context, 1, name_view);

        let mut try_catch = TryCatch::new(scope);
        let try_catch = try_catch.enter();
        let source = V8String::new(scope, script.src).unwrap();

        let mut compiled = V8Script::compile(scope, context, source, None)
            .ok_or_else(|| catch(script, try_catch, scope))?;
        let result = compiled
            .run(scope, context)
            .ok_or_else(|| catch(script, try_catch, scope))?;

        let result = result.to_string(scope).unwrap();

        Ok(result.to_rust_string_lossy(scope))
    }

    fn empty(&self) -> String {
        "{}".to_string()
    }

    fn reset(&mut self) -> Result<(), Error> {
        let snapshot = self.snapshot()?;
        *self = V8ScriptEngine::new(self.env_file.as_str(), self.env.as_str(), snapshot.as_str())?;
        Ok(())
    }

    fn snapshot(&mut self) -> Result<String, Error> {
        let script = "JSON.stringify(_snapshot)";
        let out = self.execute_script(&Script::internal_script(script))?;
        Ok(out)
    }
}

fn declare(engine: &mut V8ScriptEngine, variables_object: &serde_json::Value) -> Result<(), Error> {
    if let serde_json::Value::Object(map) = variables_object {
        for (key, value) in map {
            let script = format!("this['{}'] = {};", key, serde_json::to_string(&value)?);
            engine.execute_script(&Script::internal_script(script.as_str()))?;
        }
        Ok(())
    } else {
        Err(Error {
            selection: Selection::none(),
            kind: ErrorKind::ParseInitializeObject("Env error".to_string()),
        })
    }
}

struct ConsoleLogger {
    base: V8InspectorClientBase,
}
impl ConsoleLogger {
    fn new() -> Self {
        Self {
            base: V8InspectorClientBase::new::<Self>(),
        }
    }
}

impl V8InspectorClientImpl for ConsoleLogger {
    fn base(&self) -> &V8InspectorClientBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut V8InspectorClientBase {
        &mut self.base
    }

    fn console_api_message(
        &mut self,
        _context_group_id: i32,
        _level: i32,
        message: &StringView,
        _url: &StringView,
        _line_number: u32,
        _column_number: u32,
        _stack_trace: &mut V8StackTrace,
    ) {
        println!("{}", message.to_string());
    }
}