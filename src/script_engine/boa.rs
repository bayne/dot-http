use crate::model::Selection;
use crate::script_engine::ErrorKind::Execute;
use crate::script_engine::{Error, ErrorKind, Expression, Script, ScriptEngine};
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
}

impl BoaScriptEngine {
    pub fn new() -> BoaScriptEngine {
        let realm = Realm::create();
        let engine: Interpreter = Executor::new(realm);
        BoaScriptEngine { engine }
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

impl ScriptEngine for BoaScriptEngine {
    type Expr = Expr;

    fn process_script(&mut self, expression: Expression<Self::Expr>) -> Expression<Self::Expr> {
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

    fn execute(&mut self, expression: Expression<Self::Expr>) -> Result<String, Error> {
        Ok(self
            .engine
            .run(&expression.expr)
            .map_err(|_err| Error {
                selection: expression.selection,
                kind: Execute(String::from(
                    "Unknown error when trying to execute javascript",
                )),
            })?
            .to_string())
    }

    fn parse(&mut self, script: Script) -> Result<Expression<Self::Expr>, Error> {
        let Script {
            src: script,
            selection,
        } = script;
        let mut lexer = Lexer::new(script.as_str());
        lexer.lex().map_err(|err| Error {
            selection: selection.clone(),
            kind: ErrorKind::Execute(err.to_string()),
        })?;
        let tokens = lexer.tokens;
        Ok(Expression {
            selection: selection.clone(),
            expr: Parser::new(tokens).parse_all().map_err(|_err| Error {
                selection,
                kind: ErrorKind::Execute("Error while parsing".to_string()),
            })?,
        })
    }

    fn empty() -> &'static str {
        "{}"
    }

    fn initialize(
        &mut self,
        env_script: &str,
        env: &str,
        snapshot_script: &str,
    ) -> Result<(), Error> {
        // create a new realm to prevent state leaking across requests
        self.engine.realm = Realm::create();

        let env_file = declare_object(self, env_script.to_string(), "_env_file")?;
        self.engine.run(&env_file).unwrap();

        let env = format!("var _env = _env_file[\"{}\"];", env);
        let env = self.parse(Script::internal_script(env)).unwrap();
        self.execute(env).unwrap_or_default();

        let snapshot = declare_object(self, snapshot_script.to_string(), "_snapshot").unwrap();
        self.engine.run(&snapshot).unwrap();

        let init = include_str!("init.js");
        let init = self
            .parse(Script::internal_script(String::from(init)))
            .unwrap();

        self.execute(init).unwrap();
        Ok(())
    }

    fn snapshot(&mut self) -> Result<String, Error> {
        let Expression { expr, .. } =
            self.parse(Script::internal_script(String::from("_snapshot")))?;
        let result = self.engine.run(&expr).unwrap();
        Ok(to_string_pretty(&to_json(&result)).expect(""))
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

fn declare_object(
    engine: &mut BoaScriptEngine,
    script: String,
    var_name: &'static str,
) -> Result<Expr, Error> {
    let Expression { expr: env_file, .. } = engine
        .parse(Script::internal_script(script))
        .map_err(|_| initialize_error(ErrorKind::ParseInitializeObject(var_name)))?;
    let env_file = match &env_file {
        Expr { def: Block(expr) } => match &expr[..] {
            [Expr {
                def: expr @ ObjectDecl(_),
            }] => Ok(expr),
            _ => Err(initialize_error(ErrorKind::ParseInitializeObject(var_name))),
        },
        _ => Err(initialize_error(ErrorKind::ParseInitializeObject(var_name))),
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
