#![feature(slice_patterns)]
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate pest;

mod http_client;
mod parser;
mod request_script;
mod response_handler;
mod scripter;

use crate::http_client::execute;
use crate::parser::parse;
use crate::request_script::*;
use crate::response_handler::boa::DefaultResponseHandler;
use crate::response_handler::{DefaultOutputter, ResponseHandler};
use crate::scripter::boa::BoaScriptEngine;
use crate::scripter::{Parse, Processable, ScriptEngine};
use futures::executor::block_on;
use serde::export::Formatter;
use std::fs::read_to_string;

pub struct Config {
    pub env_file: String,
}

pub struct Parameters {
    pub script_file: String,
    pub offset: usize,
    pub env: String,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    MissingArgument(&'static str),
    CannotReadEnvFile(std::io::Error),
    CannotParseEnvFile(scripter::Error<Parse>),
    //    InvalidEnvFile(Expr),
    CannotReadRequestScriptFile(std::io::Error),
    //    UnexpectedEnvironment(Expr),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, _jf: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        unimplemented!()
    }
}

impl<T> From<scripter::Error<T>> for Error {
    fn from(_: scripter::Error<T>) -> Self {
        unimplemented!()
    }
}

impl From<parser::Error> for Error {
    fn from(_: parser::Error) -> Self {
        unimplemented!()
    }
}

pub struct DotHttp {
    engine: BoaScriptEngine,
    response_handler: DefaultResponseHandler,
    config: Config,
}

impl DotHttp {
    pub fn new(config: Config) -> DotHttp {
        let outputter: DefaultOutputter = DefaultOutputter::new();
        DotHttp {
            config,
            engine: BoaScriptEngine::new(),
            response_handler: DefaultResponseHandler::new(outputter),
        }
    }
    pub fn execute(&mut self, parameters: Parameters) -> Result<(), Error> {
        let file = read_to_string(parameters.script_file.clone()).map_err(|err| Error {
            kind: ErrorKind::CannotReadRequestScriptFile(err),
        })?;
        let file = &mut parse(file.as_str())?;

        let env_file = read_to_string(self.config.env_file.clone()).map_err(|err| Error {
            kind: ErrorKind::CannotReadEnvFile(err),
        })?;

        let engine = &mut self.engine;
        let env_file = engine.parse_env(env_file).map_err(|err| Error {
            kind: ErrorKind::CannotParseEnvFile(err),
        })?;
        engine
            .execute_env(env_file, parameters.env)
            .unwrap_or_default();
        let request_script = file.request_script(parameters.offset);
        let request_script = request_script.process(engine)?;

        let response = block_on(async {
            let executable_result = execute(&request_script.request);
            executable_result.await.unwrap()
        });

        self.response_handler
            .handle(engine, &request_script, response.into())
            .unwrap();

        Ok(())
    }
}

impl File {
    fn request_script(&self, offset: usize) -> &RequestScript<Unprocessed> {
        self.request_scripts
            .iter()
            .find(|request_script| {
                request_script.selection.start.line <= offset
                    && request_script.selection.end.line > offset
            })
            .unwrap()
    }
}
