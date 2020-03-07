use crate::controller::ErrorKind::{
    ParseRequestScript, ReadEnvFile, ReadRequestScriptFile, ReadSnapshotFile, ScriptEngineError,
};
use crate::model::*;
use crate::parser::parse;
use crate::response_handler::boa::{DefaultResponseHandler, QuietResponseHandler};
use crate::response_handler::{
    DefaultOutputter, DefaultResponse, Outputter, QuietOutputter, ResponseHandler, ScriptResponse,
};
use crate::script_engine::boa::BoaScriptEngine;
use crate::script_engine::{Processable, ScriptEngine};
use crate::{parser, script_engine};
use serde::export::Formatter;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

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

pub trait Controller {
    fn execute(
        &mut self,
        offset: usize,
        all: bool,
        env: String,
        script_file: &Path,
        snapshot_file: &Path,
        env_file: &Path,
    ) -> Result<(), Error>;
}

impl<T, R, O, E, Exp> Controller for T
where
    T: ControllerImpl<Engine = E, Outputter = O, Response = R>,
    R: Into<ScriptResponse> + From<Response>,
    O: Outputter<Response = R>,
    E: ScriptEngine<Expr = Exp>,
{
    fn execute(
        &mut self,
        offset: usize,
        all: bool,
        env: String,
        script_file: &Path,
        snapshot_file: &Path,
        env_file: &Path,
    ) -> Result<(), Error> {
        self.execute_impl(offset, all, env, script_file, snapshot_file, env_file)
    }
}

pub struct QuietController {
    engine: BoaScriptEngine,
    outputter: QuietOutputter,
    response_handler: QuietResponseHandler,
}

impl Default for QuietController {
    fn default() -> Self {
        QuietController {
            outputter: QuietOutputter::default(),
            engine: BoaScriptEngine::new(),
            response_handler: QuietResponseHandler {},
        }
    }
}
impl ControllerImpl for QuietController {
    type Engine = BoaScriptEngine;
    type Outputter = QuietOutputter;
    type Response = DefaultResponse;
    type ResponseHandler = QuietResponseHandler;
    fn components(
        &mut self,
    ) -> (
        &mut Self::Engine,
        &mut Self::Outputter,
        &mut Self::ResponseHandler,
    ) {
        (
            &mut self.engine,
            &mut self.outputter,
            &mut self.response_handler,
        )
    }
}

pub struct DefaultController {
    engine: BoaScriptEngine,
    outputter: DefaultOutputter,
    response_handler: DefaultResponseHandler,
}

impl Default for DefaultController {
    fn default() -> Self {
        DefaultController {
            outputter: DefaultOutputter::default(),
            engine: BoaScriptEngine::new(),
            response_handler: DefaultResponseHandler {},
        }
    }
}
impl ControllerImpl for DefaultController {
    type Engine = BoaScriptEngine;
    type Outputter = DefaultOutputter;
    type Response = DefaultResponse;
    type ResponseHandler = DefaultResponseHandler;
    fn components(
        &mut self,
    ) -> (
        &mut Self::Engine,
        &mut Self::Outputter,
        &mut Self::ResponseHandler,
    ) {
        (
            &mut self.engine,
            &mut self.outputter,
            &mut self.response_handler,
        )
    }
}

pub trait ControllerImpl {
    type Engine: ScriptEngine;
    type Outputter: Outputter<Response = Self::Response>;
    type Response: Into<ScriptResponse> + From<Response>;
    type ResponseHandler: ResponseHandler<
        Engine = Self::Engine,
        Outputter = Self::Outputter,
        Response = Self::Response,
    >;

    fn execute_impl(
        &mut self,
        offset: usize,
        all: bool,
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

        let env_file = match read_to_string(env_file) {
            Ok(script) => Ok(script),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let env = String::from(BoaScriptEngine::empty());
                std::fs::write(env_file, &env).unwrap();
                Ok(env)
            }
            Err(e) => Err(Error {
                kind: ErrorKind::ReadEnvFile(env_file.to_path_buf(), e),
            }),
        }?;

        let mut snapshot = match read_to_string(snapshot_file) {
            Ok(script) => Ok(script),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(String::from(BoaScriptEngine::empty()))
            }
            Err(e) => Err(Error {
                kind: ErrorKind::ReadSnapshotFile(snapshot_file.to_path_buf(), e),
            }),
        }?;

        let (engine, outputter, response_handler) = self.components();

        engine.initialize(&env_file, &env).unwrap();

        let request_scripts = file.request_scripts(offset, all);

        for request_script in request_scripts {
            engine.reset(&snapshot).unwrap();

            let request_script = request_script.process(engine).map_err(|err| Error {
                kind: ScriptEngineError(script_file.to_path_buf(), err),
            })?;

            outputter.output_request(&request_script.request).unwrap();

            let response = request_script.request.execute()?;

            response_handler
                .handle(engine, outputter, &request_script, response.into())
                .unwrap();
            snapshot = engine.snapshot().unwrap();
        }

        std::fs::write(snapshot_file, snapshot).unwrap();

        Ok(())
    }
    fn components(
        &mut self,
    ) -> (
        &mut Self::Engine,
        &mut Self::Outputter,
        &mut Self::ResponseHandler,
    );
}

impl File {
    fn request_scripts(
        &self,
        offset: usize,
        all: bool,
    ) -> impl Iterator<Item = &RequestScript<Unprocessed>> {
        let mut scripts = self
            .request_scripts
            .iter()
            .filter(move |request_script| {
                (all || request_script.selection.start.line <= offset)
                    && request_script.selection.end.line > offset
            })
            .peekable();

        match scripts.peek() {
            Some(_) => scripts,
            None => panic!("Couldn't find any scripts in our file at the given line number"),
        }
    }
}
