#[macro_use]
extern crate anyhow;

use std::borrow::BorrowMut;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use anyhow::Context;

use dot_http_lib::output::Outputter;
use dot_http_lib::parser::parse;
use dot_http_lib::script_engine::ScriptEngine;
use dot_http_lib::{parser, process, Result};

use crate::http_client::reqwest::ReqwestHttpClient;
use crate::http_client::HttpClient;
use crate::script_engine::create_script_engine;

mod http_client;
mod script_engine;

pub struct ClientConfig {
    pub ssl_check: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self { ssl_check: true }
    }
}

impl ClientConfig {
    pub fn new(ssl_check: bool) -> Self {
        Self { ssl_check }
    }
}

pub struct Runtime<'a> {
    engine: Box<dyn ScriptEngine>,
    snapshot_file: PathBuf,
    outputter: &'a mut dyn Outputter,
    client: Box<dyn HttpClient>,
}

impl<'a> Runtime<'a> {
    pub fn new(
        env: &str,
        snapshot_file: &Path,
        env_file: &Path,
        outputter: &'a mut dyn Outputter,
        config: ClientConfig,
    ) -> Result<Runtime<'a>> {
        let env_file = match read_to_string(env_file) {
            Ok(script) => Ok(script),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                std::fs::write(env_file, "{}").unwrap();
                Ok("{}".to_string())
            }
            Err(e) => Err(e),
        }?;

        let snapshot = match read_to_string(snapshot_file) {
            Ok(script) => Ok(script),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok("{}".to_string()),
            Err(e) => Err(e),
        }?;

        let engine = create_script_engine(&env_file, env, &snapshot);
        let client = Box::new(ReqwestHttpClient::create(config));

        Ok(Runtime {
            outputter,
            snapshot_file: PathBuf::from(snapshot_file),
            engine,
            client,
        })
    }

    pub fn execute(&mut self, script_file: &Path, offset: usize, all: bool) -> Result<()> {
        let file = read_to_string(&script_file)
            .with_context(|| format!("Failed opening script file: {:?}", script_file))?;
        let file = &mut parse(script_file.to_path_buf(), file.as_str())
            .with_context(|| format!("Failed parsing file: {:?}", script_file))?;

        let request_scripts = file.request_scripts(offset, all);

        let engine = &mut *self.engine;
        let outputter = self.outputter.borrow_mut();
        let client = &self.client;

        for request_script in request_scripts {
            let request = process(engine, &request_script.request)
                .with_context(|| format!("Failed processing request found on line {}", offset))?;
            outputter
                .request(&request)
                .with_context(|| format!("Failed outputting request found on line {}", offset))?;

            let response = client
                .execute(request)
                .with_context(|| format!("Error executing request found on line {}", offset))?;
            outputter.response(&response).with_context(|| {
                format!(
                    "Error outputting response for request found on line {}",
                    offset
                )
            })?;

            if let Some(parser::Handler { script, selection }) = &request_script.handler {
                engine
                    .handle(
                        &dot_http_lib::script_engine::Script {
                            selection: selection.clone(),
                            src: script.as_str(),
                        },
                        &response,
                    )
                    .with_context(|| {
                        format!(
                            "Error handling response for request found on line {}",
                            offset
                        )
                    })?;
            }

            engine.reset().unwrap();
        }
        let snapshot = engine
            .snapshot()
            .with_context(|| "Error creating snapshot")?;

        std::fs::write(self.snapshot_file.as_path(), snapshot)
            .with_context(|| "Error writing snapshot")?;

        Ok(())
    }
}
