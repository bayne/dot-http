use crate::executor::Executor;
use crate::response_handler::ResponseHandler;
use crate::response_handler::DefaultResponseHandler;
use crate::parser::parse;
use crate::scripter::{parser_expr, Processable};
use crate::{Error, ErrorKind};
use boa::exec::Executor as BoaExecutor;
use boa::exec::Interpreter;
use boa::realm::Realm;
use boa::syntax::ast::expr::Expr;
use boa::syntax::ast::expr::ExprDef::{Block, Local, ObjectDecl, VarDecl};
use clap::{App, Arg};
use futures::executor::block_on;
use std::fs::read_to_string;

pub fn app() -> Result<(), Error> {
    // stop on error flag
    // initial data (http-client.env)
    // offset
    // run-all vs only on offset
    // save state
    // environment

    let matches = App::new("dot_http")
        .version("0.1.0")
        .about("Executes HTTP stuff")
        .author("Brian Payne")
        .arg(
            Arg::with_name("env_file")
                .long("environment-file")
                .default_value("http-client.env.json"),
        )
        .arg(
            Arg::with_name("env")
                .short("e")
                .long("environment")
                .default_value("dev"),
        )
        .arg(Arg::with_name("INPUT").required(true).index(1))
        .arg(
            Arg::with_name("OFFSET")
                .default_value("0")
                .required(true)
                .index(2),
        )
        .get_matches();

    let file = matches.value_of("INPUT").unwrap();
    let file = read_to_string(file).map_err(|err| ErrorKind::CannotReadRequestScriptFile(err))?;
    let mut file = parse(file.as_str())?;

    let offset: usize = matches.value_of("OFFSET").unwrap().parse().unwrap();

    let realm = Realm::create();
    let mut engine: Interpreter = BoaExecutor::new(realm);
    {
        let env_file = matches.value_of("env_file").unwrap();
        let env_file = read_to_string(env_file).map_err(|err| ErrorKind::CannotReadEnvFile(err))?;
        let env_file = parser_expr(env_file.as_str())
            .map_err(|err| ErrorKind::CannotParseEnvFile(Box::new(err)))?;
        let env_file = match &env_file {
            Expr { def: Block(expr) } => match &expr[..] {
                [Expr {
                    def: expr @ ObjectDecl(_),
                }] => Ok(expr),
                _ => Err(ErrorKind::InvalidEnvFile(env_file)),
            },
            _ => Err(ErrorKind::InvalidEnvFile(env_file)),
        }?;

        let env_file = Expr {
            def: Block(vec![Expr {
                def: VarDecl(vec![(
                    "env_file".to_string(),
                    Some(Expr {
                        def: env_file.clone(),
                    }),
                )]),
            }]),
        };
        engine.run(&env_file).unwrap();

        let env = {
            let env = matches.value_of("env").unwrap();
            let env = parser_expr(env).unwrap();
            match &env.def {
                Block(expr) => match &expr[..] {
                    [Expr { def: Local(expr) }] => Ok(expr.clone()),
                    _ => Err(ErrorKind::UnexpectedEnvironment(env)),
                },
                _ => Err(ErrorKind::UnexpectedEnvironment(env)),
            }?
        };

        let init = format!("var client = env_file[\"{}\"];", env);
        let init = parser_expr(&init).unwrap();

        engine.run(&init).unwrap();

        let init = include_str!("scripter/init.js");
        let init = parser_expr(init).unwrap();

        engine.run(&init).unwrap();
    }
    file.process(&mut engine)?;
    let executor = Executor::new();
    let mut response_handler = DefaultResponseHandler {engine: &mut engine};
    block_on(async {
        let request_script = file.request_scripts.get(offset).unwrap();
        let executable_result = executor.execute(request_script);
        let response = executable_result.await.unwrap();
        response_handler.handle(request_script, response).unwrap();
    });

    Ok(())
}
