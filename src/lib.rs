#![feature(slice_patterns)]
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate pest;

mod http_client;
mod parser;
mod response_handler;
mod script_engine;

use crate::model::*;
use crate::parser::parse;
use crate::response_handler::boa::DefaultResponseHandler;
use crate::response_handler::{DefaultOutputter, ResponseHandler, Outputter};
use crate::script_engine::boa::BoaScriptEngine;
use crate::script_engine::{Processable, ScriptEngine};
use futures::executor::block_on;
use serde::export::Formatter;
use std::fs::read_to_string;
use crate::ErrorKind::CannotParseRequestScript;

pub struct Config {
    pub env_file: String,
    pub snapshot_file: String,
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
    CannotReadSnapshotFile(std::io::Error),
    CannotParseRequestScript(parser::Error),
    CannotReadRequestScriptFile(std::io::Error),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, _jf: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        unimplemented!()
    }
}

impl From<script_engine::Error> for Error {
    fn from(_: script_engine::Error) -> Self {
        unimplemented!()
    }
}

impl From<parser::Error> for Error {
    fn from(e: parser::Error) -> Self {
        Error {
            kind: CannotParseRequestScript(e)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        dbg!(e);
        unimplemented!()
    }
}

pub struct DotHttp {
    engine: BoaScriptEngine,
    outputter: DefaultOutputter,
    response_handler: DefaultResponseHandler,
    config: Config,
}

impl DotHttp {
    pub fn new(config: Config) -> DotHttp {
        let outputter: DefaultOutputter = DefaultOutputter::new();
        DotHttp {
            config,
            outputter,
            engine: BoaScriptEngine::new(),
            response_handler: DefaultResponseHandler{},
        }
    }
    pub fn execute(&mut self, parameters: Parameters) -> Result<(), Error> {
        let file = read_to_string(parameters.script_file).map_err(|err| Error {
            kind: ErrorKind::CannotReadRequestScriptFile(err),
        })?;
        let file = &mut parse(file.as_str())?;

        let env_file = read_to_string(self.config.env_file.clone()).map_err(|err| Error {
            kind: ErrorKind::CannotReadEnvFile(err),
        })?;

        let snapshot_script = match read_to_string(self.config.snapshot_file.clone()) {
            Ok(script) => Ok(script),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(String::from(BoaScriptEngine::empty()))
            },
            Err(e) => Err(Error {
                kind: ErrorKind::CannotReadSnapshotFile(e)
            })
        }?;

        let engine = &mut self.engine;
        let outputter = &mut self.outputter;

        engine.initialize(env_file, parameters.env, snapshot_script).unwrap();

        let request_script = file.request_script(parameters.offset);
        let request_script = request_script.process(engine)?;

        outputter.output_request(&request_script.request).unwrap();

        let response = block_on(async {
            let result = request_script.request.execute();
            result.await.unwrap()
        });

        self.response_handler
            .handle(engine, outputter, &request_script, response.into())
            .unwrap();
        let snapshot = engine.snapshot().unwrap();

        std::fs::write(self.config.snapshot_file.clone(), snapshot).unwrap();

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

mod model {
    #[derive(Debug)]
    pub struct Response {
        pub version: String,
        pub status_code: u16,
        pub status: String,
        pub headers: Vec<(String, String)>,
        pub body: String,
    }

    #[derive(Debug)]
    pub struct File {
        pub request_scripts: Vec<RequestScript<Unprocessed>>,
    }

    #[derive(Debug)]
    pub struct RequestScript<S> {
        pub request: Request<S>,
        pub handler: Option<Handler>,
        pub selection: Selection,
    }

    #[derive(Debug)]
    pub struct Request<S> {
        pub method: Method,
        pub target: Value<S>,
        pub headers: Vec<Header<S>>,
        pub body: Option<Value<S>>,
        pub selection: Selection,
    }

    #[derive(PartialEq, Debug, Clone)]
    pub enum Method {
        Get(Selection),
        Post(Selection),
        Delete(Selection),
        Put(Selection),
        Patch(Selection),
    }

    #[derive(Debug)]
    pub struct Header<S> {
        pub field_name: String,
        pub field_value: Value<S>,
        pub selection: Selection,
    }

    #[derive(Debug, Clone)]
    pub struct Handler {
        pub script: String,
        pub selection: Selection,
    }

    #[derive(Debug)]
    pub struct Value<S> {
        pub state: S,
    }

    #[derive(Debug)]
    pub struct Processed {
        pub value: String,
    }

    #[derive(Debug)]
    pub enum Unprocessed {
        WithInline {
            value: String,
            inline_scripts: Vec<InlineScript>,
            selection: Selection,
        },
        WithoutInline(String, Selection),
    }

    #[derive(Debug)]
    pub struct InlineScript {
        pub script: String,
        pub placeholder: String,
        pub selection: Selection,
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Selection {
        pub start: Position,
        pub end: Position,
    }

    impl Selection {
        pub fn none() -> Selection {
            Selection {
                start: Position { line: 0, col: 0 },
                end: Position { line: 0, col: 0 },
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Position {
        pub line: usize,
        pub col: usize,
    }
}
