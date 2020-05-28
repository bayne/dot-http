use crate::model::Selection;
use crate::script_engine::ErrorKind::Execute;
use crate::script_engine::{Error, ErrorKind, Script, ScriptEngine};
use boa::builtins::value::ValueData;
use boa::exec::Executor;
use boa::exec::Interpreter;
use boa::realm::Realm;
use boa::syntax::ast::constant;
use boa::syntax::ast::expr::ExprDef::Local;
use boa::syntax::ast::expr::ExprDef::ObjectDecl;
use boa::syntax::ast::expr::ExprDef::VarDecl;
use boa::syntax::ast::expr::ExprDef::{Block, Call, GetConstField};
use boa::syntax::ast::expr::{Expr, ExprDef};
use boa::syntax::lexer::Lexer;
use boa::syntax::parser::Parser;
use serde_json::{to_string_pretty, Map};
use serde_json::{Number as JSONNumber, Value as JSONValue};
use std::convert::From;

pub struct BoaScriptEngine {
    engine: Interpreter,
    initial_state: Option<InitialState>,
}

pub struct Expression<T> {
    selection: Selection,
    expr: T,
}

struct InitialState {
    env_file: Expr,
    env: Expression<Expr>,
    init: Expression<Expr>,
}

/// Capture the main interactions with our [[Interpreter]] as a trait so we can disjointly borrow
/// against our [[BoaScriptEngine]] struct
trait InternalBoaScriptEngine {
    fn execute(&mut self, expression: &Expression<Expr>) -> Result<String, Error>;
    fn parse(&self, script: &Script) -> Result<Expression<Expr>, Error>;
}

impl BoaScriptEngine {
    pub fn new() -> BoaScriptEngine {
        let realm = Realm::create();
        let engine: Interpreter = Executor::new(realm);
        BoaScriptEngine {
            engine,
            initial_state: None,
        }
    }
}

fn get_global(var: String) -> Expr {
    Expr {
        def: Block(vec![Expr {
            def: Call(
                Box::new(Expr {
                    def: GetConstField(
                        Box::new(Expr {
                            def: GetConstField(
                                Box::new(Expr {
                                    def: Local(String::from("client")),
                                }),
                                String::from("global"),
                            ),
                        }),
                        String::from("get"),
                    ),
                }),
                vec![Expr {
                    def: ExprDef::Const(constant::Const::String(var)),
                }],
            ),
        }]),
    }
}

impl BoaScriptEngine {
    fn process_script(&self, expression: Expression<Expr>) -> Expression<Expr> {
        match &expression {
            Expression {
                expr: Expr { def: Block(expr) },
                ..
            } => match &expr[..] {
                [Expr {
                    def: Local(var_name),
                }] => Expression {
                    selection: Selection::none(),
                    expr: get_global(var_name.clone()),
                },
                _ => expression,
            },
            _ => expression,
        }
    }

    fn execute(&mut self, expression: &Expression<Expr>) -> Result<String, Error> {
        self.engine.execute(expression)
    }

    fn parse(&self, script: &Script) -> Result<Expression<Expr>, Error> {
        self.engine.parse(script)
    }
}

impl ScriptEngine for BoaScriptEngine {
    fn execute_script(&mut self, script: &Script) -> Result<String, Error> {
        let expr = self.parse(&script)?;

        let expr = self.process_script(expr);
        self.execute(&expr)
    }

    fn empty(&self) -> String {
        String::from("{}")
    }

    fn initialize(&mut self, env_script: &str, env: &str) -> Result<(), Error> {
        let env_file = declare_object(&self.engine, env_script, "_env_file")?;

        let env = format!("var _env = _env_file[\"{}\"];", env);
        let env = self.parse(&Script::internal_script(&env))?;

        let init = include_str!("init.js");
        let init = self.parse(&Script::internal_script(&String::from(init)))?;

        // build up our initial state
        self.initial_state = Some(InitialState {
            env_file,
            env,
            init,
        });

        Ok(())
    }

    fn reset(&mut self, snapshot_script: &str) -> Result<(), Error> {
        let BoaScriptEngine {
            engine,
            initial_state,
        } = self;

        match initial_state {
            Some(InitialState {
                env_file,
                env,
                init,
            }) => {
                // drop our existing realm, clearing our out current JS state
                engine.realm = Realm::create();

                let snapshot = declare_object(engine, snapshot_script, "_snapshot")?;

                engine.run(&env_file).map_err(|err_value| Error {
                    selection: Selection::none(),
                    kind: Execute(format!("Error when executing javascript: {}", err_value)),
                })?;
                engine.execute(env)?;
                engine.run(&snapshot).map_err(|err_value| Error {
                    selection: Selection::none(),
                    kind: Execute(format!("Error when executing javascript: {}", err_value)),
                })?;

                engine.execute(init)?;

                Ok(())
            }
            None => panic!("We must initialize our engine before we can use it"),
        }
    }

    fn snapshot(&mut self) -> Result<String, Error> {
        let Expression { expr, .. } =
            self.parse(&Script::internal_script(&String::from("_snapshot")))?;
        let result = self.engine.run(&expr).unwrap();
        Ok(to_string_pretty(&to_json(&result)).expect(""))
    }
}

impl InternalBoaScriptEngine for Interpreter {
    fn execute(&mut self, expression: &Expression<Expr>) -> Result<String, Error> {
        self.run(&expression.expr)
            .map_err(|_err| Error {
                selection: expression.selection.clone(),
                kind: Execute(String::from(
                    "Unknown error when trying to execute javascript",
                )),
            })
            .map(|value| value.to_string())
    }

    fn parse(&self, script: &Script) -> Result<Expression<Expr>, Error> {
        let Script {
            src: script,
            selection,
        } = script;
        let mut lexer = Lexer::new(script);
        lexer.lex().map_err(|err| Error {
            selection: selection.clone(),
            kind: ErrorKind::Execute(err.to_string()),
        })?;
        let tokens = lexer.tokens;
        Ok(Expression {
            selection: selection.clone(),
            expr: Parser::new(tokens).parse_all().map_err(|_err| Error {
                selection: selection.clone(),
                kind: ErrorKind::Execute("Error while parsing".to_string()),
            })?,
        })
    }
}

fn to_json(value: &ValueData) -> JSONValue {
    match *value {
        ValueData::Null | ValueData::Symbol(_) | ValueData::Undefined | ValueData::Function(_) => {
            JSONValue::Null
        }
        ValueData::Boolean(b) => JSONValue::Bool(b),
        ValueData::Object(ref obj) => {
            let mut new_obj = Map::new();
            for (k, v) in obj.borrow().properties.iter() {
                new_obj.insert(k.clone(), to_json(v.value.as_ref().unwrap()));
            }
            JSONValue::Object(new_obj)
        }
        ValueData::String(ref str) => JSONValue::String(str.clone()),
        ValueData::Number(num) => {
            JSONValue::Number(JSONNumber::from_f64(num).expect("Could not convert to JSONNumber"))
        }
        ValueData::Integer(val) => JSONValue::Number(JSONNumber::from(val)),
    }
}

fn declare_object<E>(engine: &E, script: &str, var_name: &'static str) -> Result<Expr, Error>
where
    E: InternalBoaScriptEngine,
{
    let Expression { expr: env_file, .. } = engine
        .parse(&Script::internal_script(script))
        .map_err(|_| initialize_error(ErrorKind::ParseInitializeObject(var_name.to_string())))?;
    let env_file = match &env_file {
        Expr { def: Block(expr) } => match &expr[..] {
            [Expr {
                def: expr @ ObjectDecl(_),
            }] => Ok(expr),
            _ => Err(initialize_error(ErrorKind::ParseInitializeObject(
                var_name.to_string(),
            ))),
        },
        _ => Err(initialize_error(ErrorKind::ParseInitializeObject(
            var_name.to_string(),
        ))),
    }?;

    Ok(Expr {
        def: Block(vec![Expr {
            def: VarDecl(vec![(
                String::from(var_name),
                Some(Expr {
                    def: env_file.clone(),
                }),
            )]),
        }]),
    })
}

fn initialize_error(kind: ErrorKind) -> Error {
    Error {
        selection: Selection::none(),
        kind,
    }
}
