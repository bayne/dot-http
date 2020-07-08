use crate::script_engine::{handle, Script, ScriptEngine};

use boa::exec::Executor;
use boa::exec::Interpreter;
use boa::realm::Realm;
use boa::syntax::ast::node::Node;
use boa::syntax::lexer::Lexer;
use boa::syntax::parser::Parser;

use crate::{Response, Result};
use std::convert::From;

pub struct BoaScriptEngine {
    interpreter: Interpreter,
    env_script: String,
    env: String,
}

impl BoaScriptEngine {
    pub fn new(env_script: &str, env: &str, snapshot_script: &str) -> Result<BoaScriptEngine> {
        let realm = Realm::create();
        let interpreter: Interpreter = Executor::new(realm);

        let mut engine = BoaScriptEngine {
            interpreter,
            env_script: env_script.to_string(),
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

        let snapshot: serde_json::Value = serde_json::from_str(snapshot_script)?;
        declare(&mut engine, &snapshot)?;
        let snapshot = format!("var _snapshot = {};", snapshot);
        engine.execute_script(&Script::internal_script(snapshot.as_str()))?;

        let script = include_str!("init.js");
        engine.execute_script(&Script::internal_script(script))?;

        Ok(engine)
    }
}

fn parser_expr(src: &str) -> Result<Node> {
    let mut lexer = Lexer::new(src);
    lexer.lex()?;
    let tokens = lexer.tokens;
    let node = Parser::new(&tokens)
        .parse_all()
        .map_err(|e| anyhow!("ParsingError: {}", e))?;
    Ok(node)
}

impl ScriptEngine for BoaScriptEngine {
    fn execute_script(&mut self, script: &Script) -> Result<String> {
        // Setup executor
        let expr = parser_expr(script.src)?;
        let result = self
            .interpreter
            .run(&expr)
            .map_err(|err| anyhow!("Error executing expression: {}", err))?
            .to_string();
        Ok(result)
    }

    fn empty(&self) -> String {
        String::from("{}")
    }

    fn reset(&mut self) -> Result<()> {
        let snapshot = self.snapshot()?;
        *self = BoaScriptEngine::new(
            self.env_script.as_str(),
            self.env.as_str(),
            snapshot.as_str(),
        )?;
        Ok(())
    }

    fn snapshot(&mut self) -> Result<String> {
        let script = "JSON.stringify(_snapshot)";
        let out = self.execute_script(&Script::internal_script(script))?;
        Ok(out)
    }

    fn handle(&mut self, request_script: &Script, response: &Response) -> Result<()> {
        handle(self, request_script, response)
    }
}

fn declare(engine: &mut BoaScriptEngine, variables_object: &serde_json::Value) -> Result<()> {
    if let serde_json::Value::Object(map) = variables_object {
        for (key, value) in map {
            let script = serde_json::to_string(&value)?;
            let script = format!("this['{}'] = {};", key, script);
            engine.execute_script(&Script::internal_script(script.as_str()))?;
        }
        Ok(())
    } else {
        Err(anyhow!("Failed to declare object: {:?}", variables_object))
    }
}
