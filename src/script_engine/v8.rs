use crate::script_engine::{Error, ErrorKind, Script, ScriptEngine};
use rusty_v8::{
    inspector::{
        StringView, V8Inspector, V8InspectorClientBase, V8InspectorClientImpl, V8StackTrace,
    },
    Context, ContextScope, Exception, Global, HandleScope, Isolate, OwnedIsolate,
    Script as V8Script, String as V8String, TryCatch, V8,
};
use std::convert::From;
use std::sync::Once;

static V8_INIT: Once = Once::new();
pub struct V8ScriptEngine {
    isolate: OwnedIsolate,
    global: Global<Context>,
    init: Option<InitialState>,
}
#[derive(Clone)]
struct InitialState {
    env_file: String,
    env: String,
}

impl V8ScriptEngine {
    pub fn new() -> V8ScriptEngine {
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
        V8ScriptEngine {
            isolate,
            global,
            init: None,
        }
    }
    fn init(&mut self) -> Result<(), Error> {
        if let Some(init) = self.init.clone() {
            let env_script = format!("var _env_file = {} ", &init.env_file);
            self.execute_script(&Script::internal_script(&env_script))?;

            let env = format!(
                r#"
                            (function() {{
                            let config = _env_file["{}"];
                            for (prop in config) {{
                                this[prop] = config[prop];
                            }}
                            }})()
                          "#,
                init.env
            );
            self.execute_script(&Script::internal_script(&env))?;

            let init = include_str!("init.js");
            self.execute_script(&Script::internal_script(&String::from(init)))?;
            Ok(())
        } else {
            panic!("We must initialize our engine before we can use it")
        }
    }
}

impl Default for V8ScriptEngine {
    fn default() -> Self {
        V8ScriptEngine::new()
    }
}

impl ScriptEngine for V8ScriptEngine {
    fn execute_script(&mut self, script: &Script) -> Result<String, Error> {
        let isolate = &mut self.isolate;
        let mut logger = ConsoleLogger::new();
        let mut inspector = V8Inspector::create(isolate, &mut logger);
        let mut handle_scope = HandleScope::new(isolate);
        let isolate_scope = handle_scope.enter();

        let context = self.global.get(isolate_scope).unwrap();

        let name = b"";
        let name_view = StringView::from(&name[..]);
        inspector.context_created(context, 1, name_view);

        let mut context_scope = ContextScope::new(isolate_scope, context);
        let scope = context_scope.enter();

        let mut try_catch = TryCatch::new(scope);
        let tc = try_catch.enter();
        let source = V8String::new(scope, script.src).unwrap();
        if let Some(mut compiled) = V8Script::compile(scope, context, source, None) {
            if let Some(result) = compiled.run(scope, context) {
                let result = result.to_string(scope).unwrap();
                Ok(result.to_rust_string_lossy(scope))
            } else {
                let exception = tc.exception().unwrap();
                let msg = Exception::create_message(scope, exception);
                Err(Error {
                    selection: script.selection.clone(),
                    kind: ErrorKind::Execute(msg.get(scope).to_rust_string_lossy(scope)),
                })
            }
        } else {
            let exception = tc.exception().unwrap();
            let msg = Exception::create_message(scope, exception);
            Err(Error {
                selection: script.selection.clone(),
                kind: ErrorKind::Execute(msg.get(scope).to_rust_string_lossy(scope)),
            })
        }
    }

    fn empty(&self) -> String {
        "{}".to_string()
    }

    fn initialize(&mut self, env_script: &str, env: &str) -> Result<(), Error> {
        self.init = Some(InitialState {
            env_file: env_script.to_string(),
            env: env.to_string(),
        });
        // TODO: Double check this error mapping, is only for compatibility with boa script
        // executor
        self.init().map_err(|e| Error {
            selection: e.selection,
            kind: ErrorKind::ParseInitializeObject(format!("{:?}", e.kind)),
        })
    }

    fn reset(&mut self, snapshot_script: &str) -> Result<(), Error> {
        let mut handle_scope = HandleScope::new(&mut self.isolate);
        let scope = handle_scope.enter();
        let context = Context::new(scope);
        self.global.set(scope, context);
        self.init()?;
        let script = if snapshot_script.trim().is_empty() {
            "var _snapshot = {}".to_string()
        } else {
            format!(
                r#" var _snapshot = {};
                        (function() {{
                            for (prop in _snapshot) {{
                                this[prop] = _snapshot[prop];
                            }}
                            }})()
                          "#,
                snapshot_script
            )
        };
        self.execute_script(&Script::internal_script(&script))?;
        Ok(())
    }

    fn snapshot(&mut self) -> Result<String, Error> {
        let script = "JSON.stringify(_snapshot)";
        let out = self.execute_script(&Script::internal_script(script))?;
        Ok(out)
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
