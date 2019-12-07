use crate::executor::Executor;
use crate::response_handler::{ResponseHandler, DefaultOutputter};
use crate::response_handler::boa::DefaultResponseHandler;
use crate::parser::parse;
use crate::scripter::{ScriptEngine};
use crate::scripter::boa::BoaScriptEngine;
use crate::{Error, ErrorKind};
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
    let file = parse(file.as_str())?;

    let offset: usize = matches.value_of("OFFSET").unwrap().parse().unwrap();

    let mut engine = BoaScriptEngine::new();
    {
        let env = matches.value_of("env").unwrap();

        let env_file = matches.value_of("env_file").unwrap();
        let env_file = read_to_string(env_file).map_err(|err| ErrorKind::CannotReadEnvFile(err))?;
        let env_file = engine.parse_env(env_file)
            .map_err(|err| ErrorKind::CannotParseEnvFile(err))?;
        engine.execute_env(env_file, String::from(env)).unwrap_or_default();
    }
    let executor = Executor::new();
    let mut outputter = DefaultOutputter::new();
    let mut response_handler = DefaultResponseHandler::new(&mut engine, &mut outputter);
    block_on(async {
        let request_script = file.request_scripts.get(offset).unwrap();
        let executable_result = executor.execute(request_script);
        let response = executable_result.await.unwrap();
        response_handler.handle(request_script, response.into()).unwrap();
    });

    Ok(())
}
