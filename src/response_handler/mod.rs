use crate::model::*;
use crate::script_engine;
use crate::script_engine::{Script, ScriptEngine};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use std::fmt::Display;
use std::fmt::Formatter;

pub mod boa;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.message))
    }
}

impl From<script_engine::Error> for Error {
    fn from(e: script_engine::Error) -> Self {
        Error {
            message: e.to_string(),
        }
    }
}

pub trait ResponseHandler {
    type Engine: ScriptEngine;
    type Outputter: Outputter<Response = Self::Response>;
    type Response: Into<ScriptResponse>;
    fn handle(
        &mut self,
        engine: &mut Self::Engine,
        outputter: &mut Self::Outputter,
        request_script: &RequestScript<Processed>,
        response: Self::Response,
    ) -> Result<(), Error> {
        outputter.output_response(&response).unwrap();
        if let Some(Handler { script, selection }) = &request_script.handler {
            let script_response: ScriptResponse = response.into();
            self.inject(engine, script_response)?;
            let expr = engine.parse(Script {
                selection: selection.clone(),
                src: script.clone(),
            })?;
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
        let expr = engine.parse(Script::internal_script(script))?;
        engine.execute(expr)?;
        if let Ok(serde_json::Value::Object(response_body)) =
            serde_json::from_str(script_response.body.as_str())
        {
            let script = format!(
                "response.body = {};",
                serde_json::to_string(&response_body).unwrap()
            );
            let expr = engine.parse(Script::internal_script(script)).unwrap();
            engine.execute(expr).unwrap();
        }
        Ok(())
    }
}

pub trait Outputter {
    type Response;
    fn output_response(&mut self, response: &Self::Response) -> Result<(), Error>;
    fn output_request(&mut self, request: &Request<Processed>) -> Result<(), Error>;
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

    fn output_request(&mut self, request: &Request<Processed>) -> Result<(), Error> {
        println!("{}", request);
        Ok(())
    }
}

pub struct DefaultResponse(Response);

impl Display for Method {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let method = match self {
            Method::Get(_) => "GET",
            Method::Post(_) => "POST",
            Method::Delete(_) => "DELETE",
            Method::Put(_) => "PUT",
            Method::Patch(_) => "PATCH",
            Method::Options(_) => "OPTIONS",
        };
        f.write_str(method)
    }
}

impl Display for Value<Processed> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(self.state.value.as_str())
    }
}

impl Display for Request<Processed> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_fmt(format_args!(
            "{method} {target}\n",
            method = self.method,
            target = self.target
        ))
    }
}

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
    headers: Map<String, serde_json::Value>,
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
