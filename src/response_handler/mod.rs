use crate::request_script::*;
use crate::scripter;
use crate::scripter::{Execute, Parse, ScriptEngine};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;
use std::fmt::Display;
use std::fmt::Formatter;

pub mod boa;

#[derive(Debug)]
pub struct Error;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl From<scripter::Error<Parse>> for Error {
    fn from(_: scripter::Error<Parse>) -> Self {
        unimplemented!()
    }
}

impl From<scripter::Error<Execute>> for Error {
    fn from(_: scripter::Error<Execute>) -> Self {
        unimplemented!()
    }
}

pub trait ResponseHandler {
    type Engine: ScriptEngine;
    type Outputter: Outputter<Response = Self::Response>;
    type Response: Into<ScriptResponse>;
    fn outputter(&mut self) -> &mut Self::Outputter;
    fn handle(
        &mut self,
        engine: &mut Self::Engine,
        request_script: &RequestScript<Processed>,
        response: Self::Response,
    ) -> Result<(), Error> {
        self.outputter().output_response(&response).unwrap();
        if let Some(Handler {
            script,
            selection: _,
        }) = &request_script.handler
        {
            let script_response: ScriptResponse = response.into();
            self.inject(engine, script_response)?;
            let expr = engine.parse(script.clone())?;
            engine.execute(expr)?;
        }
        Ok(())
    }

    fn inject(
        &self,
        engine: &mut Self::Engine,
        script_response: ScriptResponse,
    ) -> Result<(), Error> {
        let script = format!(
            "var response = {};",
            serde_json::to_string(&script_response).unwrap()
        );
        let expr = engine.parse(script)?;
        engine.execute(expr)?;
        if let Ok(Value::Object(response_body)) =
            serde_json::from_str(script_response.body.as_str())
        {
            let script = format!(
                "response.body = {};",
                serde_json::to_string(&response_body).unwrap()
            );
            let expr = engine.parse(script).unwrap();
            engine.execute(expr).unwrap();
        }
        Ok(())
    }
}

pub trait Outputter {
    type Response;
    fn output_response(&mut self, response: &Self::Response) -> Result<(), Error>;
}

pub struct DefaultOutputter;

impl DefaultOutputter {
    pub fn new() -> DefaultOutputter {
        DefaultOutputter {}
    }
}

impl Outputter for DefaultOutputter {
    type Response = DefaultResponse;

    fn output_response(&mut self, response: &Self::Response) -> Result<(), Error> {
        println!("{}", response);
        Ok(())
    }
}

pub struct DefaultResponse(Response);

impl Display for DefaultResponse {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let DefaultResponse(response) = self;
        let headers: String = response
            .headers
            .iter()
            .map(|(key, value)| format!("\n{}: {}", key, value))
            .collect();
        formatter.write_fmt(format_args!(
            "{http_version} {status}\
             {headers}
             {body}",
            http_version = response.version,
            status = response.status,
            headers = headers,
            body = format!("\n{}", response.body),
        ))
    }
}

#[derive(Deserialize, Serialize)]
pub struct ScriptResponse {
    body: String,
    headers: Map<String, Value>,
    status: u16,
    content_type: String,
}

impl From<Response> for DefaultResponse {
    fn from(response: Response) -> Self {
        DefaultResponse(response)
    }
}

impl From<DefaultResponse> for ScriptResponse {
    fn from(response: DefaultResponse) -> Self {
        let DefaultResponse(response) = response;
        ScriptResponse {
            body: response.body,
            headers: Map::new(),
            status: response.status_code,
            content_type: String::new(),
        }
    }
}
