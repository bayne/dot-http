use crate::scripter::{ExecuteError, ParseError, ScriptEngine};
use crate::Error;
use boa::builtins::value::ValueData;
use boa::exec::Executor;
use boa::exec::Interpreter;
use boa::realm::Realm;
use boa::syntax::ast::expr::Expr;
use boa::syntax::ast::expr::ExprDef::Block;
use boa::syntax::ast::expr::ExprDef::Local;
use boa::syntax::ast::expr::ExprDef::ObjectDecl;
use boa::syntax::ast::expr::ExprDef::VarDecl;
use boa::syntax::lexer::Lexer;
use boa::syntax::lexer::LexerError;
use boa::syntax::parser::ParseError as BoaParseError;
use boa::syntax::parser::Parser;
use gc::Gc;
use std::convert::From;

impl From<LexerError> for Error {
    fn from(_e: LexerError) -> Self {
        unimplemented!()
    }
}

impl From<LexerError> for ParseError {
    fn from(_e: LexerError) -> Self {
        unimplemented!()
    }
}

impl From<Gc<ValueData>> for ExecuteError {
    fn from(_e: Gc<ValueData>) -> Self {
        unimplemented!()
    }
}

impl From<BoaParseError> for Error {
    fn from(_e: BoaParseError) -> Self {
        unimplemented!()
    }
}

impl From<BoaParseError> for ParseError {
    fn from(_e: BoaParseError) -> Self {
        unimplemented!()
    }
}

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

impl ScriptEngine for BoaScriptEngine {
    type Expression = Expr;
    type EnvExpression = Expr;

    fn execute(&mut self, expression: Self::Expression) -> Result<String, ExecuteError> {
        Ok(self.engine.run(&expression)?.to_string())
    }

    fn parse(&mut self, script: String) -> Result<Self::Expression, ParseError> {
        let mut lexer = Lexer::new(script.as_str());
        lexer.lex()?;
        let tokens = lexer.tokens;
        Ok(Parser::new(tokens).parse_all()?)
    }

    fn parse_env(&mut self, script: String) -> Result<Self::EnvExpression, ParseError> {
        let env_file = self.parse(String::from(script.as_str()))?;
        //            .map_err(|err| ErrorKind::CannotParseEnvFile(err))?;
        let env_file = match &env_file {
            Expr { def: Block(expr) } => match &expr[..] {
                [Expr {
                    def: expr @ ObjectDecl(_),
                }] => Ok(expr),
                //                _ => Err(ErrorKind::InvalidEnvFile(env_file)),
                _ => Err(ParseError {}),
            },
            //            _ => Err(ErrorKind::InvalidEnvFile(env_file)),
            _ => Err(ParseError {}),
        }?;

        Ok(Expr {
            def: Block(vec![Expr {
                def: VarDecl(vec![(
                    "env_file".to_string(),
                    Some(Expr {
                        def: env_file.clone(),
                    }),
                )]),
            }]),
        })
    }

    fn execute_env(
        &mut self,
        expression: Self::EnvExpression,
        env: String,
    ) -> Result<String, ExecuteError> {
        self.engine.run(&expression)?;
        let env = {
            let env = self.parse(env).unwrap();
            match &env.def {
                Block(expr) => match &expr[..] {
                    [Expr { def: Local(expr) }] => Ok(expr.clone()),
                    //                    _ => Err(ErrorKind::UnexpectedEnvironment(env)),
                    _ => Err(ExecuteError {}),
                },
                _ => Err(ExecuteError {}),
                //                _ => Err(ErrorKind::UnexpectedEnvironment(env)),
            }?
        };

        let init = format!("var client = env_file[\"{}\"];", env);
        let init = self.parse(init).unwrap();

        self.execute(init).unwrap_or_default();

        let init = include_str!("init.js");
        let init = self.parse(String::from(init)).unwrap();

        self.execute(init)
    }
}
