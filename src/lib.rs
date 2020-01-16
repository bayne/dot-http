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
use crate::response_handler::{DefaultOutputter, Outputter, ResponseHandler};
use crate::script_engine::boa::BoaScriptEngine;
use crate::script_engine::{Processable, ScriptEngine};
use crate::ErrorKind::{
    ParseRequestScript, ReadEnvFile, ReadRequestScriptFile, ReadSnapshotFile, ScriptEngineError,
};
use serde::export::Formatter;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    ReadEnvFile(PathBuf, std::io::Error),
    ReadSnapshotFile(PathBuf, std::io::Error),
    ParseRequestScript(parser::Error),
    ReadRequestScriptFile(PathBuf, std::io::Error),
    ScriptEngineError(PathBuf, script_engine::Error),
    HttpClient(reqwest::Error),
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.kind {
            ReadEnvFile(filename, e) => f.write_fmt(format_args!(
                "Could not read environment file - {}: {}",
                filename.display(),
                e
            )),
            ReadSnapshotFile(filename, e) => f.write_fmt(format_args!(
                "Could not read the snapshot file - {}: {}",
                filename.display(),
                e
            )),
            ParseRequestScript(e) => f.write_fmt(format_args!("{}", e.message)),
            ReadRequestScriptFile(filename, e) => {
                f.write_fmt(format_args!("{}: {}", filename.display(), e))
            }
            ScriptEngineError(filename, e) => {
                f.write_fmt(format_args!("{}: {}", filename.display(), e))
            }
            ErrorKind::HttpClient(e) => f.write_fmt(format_args!("{}", e)),
        }
    }
}

pub struct DotHttp {
    engine: BoaScriptEngine,
    outputter: DefaultOutputter,
    response_handler: DefaultResponseHandler,
}

impl Default for DotHttp {
    fn default() -> Self {
        let outputter: DefaultOutputter = DefaultOutputter::new();
        DotHttp {
            outputter,
            engine: BoaScriptEngine::new(),
            response_handler: DefaultResponseHandler {},
        }
    }
}

impl DotHttp {
    pub fn execute(
        &mut self,
        offset: usize,
        env: String,
        script_file: &Path,
        snapshot_file: &Path,
        env_file: &Path,
    ) -> Result<(), Error> {
        let file = read_to_string(&script_file).map_err(|err| Error {
            kind: ErrorKind::ReadRequestScriptFile(script_file.to_path_buf(), err),
        })?;
        let file = &mut parse(script_file.to_path_buf(), file.as_str()).map_err(|err| Error {
            kind: ParseRequestScript(err),
        })?;
        let env_file = read_to_string(env_file).map_err(|err| Error {
            kind: ErrorKind::ReadEnvFile(env_file.to_path_buf(), err),
        })?;

        let snapshot_script = match read_to_string(snapshot_file) {
            Ok(script) => Ok(script),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(String::from(BoaScriptEngine::empty()))
            }
            Err(e) => Err(Error {
                kind: ErrorKind::ReadSnapshotFile(snapshot_file.to_path_buf(), e),
            }),
        }?;

        let engine = &mut self.engine;
        let outputter = &mut self.outputter;

        engine.initialize(env_file, env, snapshot_script).unwrap();

        let request_script = file.request_script(offset);
        let request_script = request_script.process(engine).map_err(|err| Error {
            kind: ScriptEngineError(script_file.to_path_buf(), err),
        })?;

        outputter.output_request(&request_script.request).unwrap();

        let response = request_script.request.execute()?;

        self.response_handler
            .handle(engine, outputter, &request_script, response.into())
            .unwrap();
        let snapshot = engine.snapshot().unwrap();

        std::fs::write(snapshot_file, snapshot).unwrap();

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
    use serde::export::fmt::Error;
    use serde::export::Formatter;
    use std::fmt::Display;
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct Response {
        pub version: Version,
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
        Options(Selection),
    }

    #[derive(Debug)]
    pub enum Version {
        Http09,
        Http2,
        Http10,
        Http11,
    }

    impl Display for Version {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            let version = match *self {
                Version::Http09 => "HTTP/0.9",
                Version::Http2 => "HTTP/2.0",
                Version::Http10 => "HTTP/1.0",
                Version::Http11 => "HTTP/1.1",
            };
            f.write_str(version)
        }
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
        pub filename: PathBuf,
        pub start: Position,
        pub end: Position,
    }

    impl Display for Selection {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
            f.write_fmt(format_args!(
                "{filename}:{line}:{col}",
                filename = self.filename.display(),
                line = self.start.line,
                col = self.start.col
            ))
        }
    }

    impl Selection {
        pub fn none() -> Selection {
            Selection {
                filename: PathBuf::default(),
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
