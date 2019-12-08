use crate::scripter::ExecuteError;
use crate::scripter::ParseError;
use crate::scripter::ScriptEngine;
use crate::Handler;
use crate::Processed;
use crate::RequestScript;
use crate::Response;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Map;
use serde_json::Value;
use std::fmt::Display;
use std::fmt::Formatter;

pub(crate) mod boa;

#[derive(Debug)]
pub struct Error;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl From<ParseError> for Error {
    fn from(_: ParseError) -> Self {
        unimplemented!()
    }
}

impl From<ExecuteError> for Error {
    fn from(_: ExecuteError) -> Self {
        unimplemented!()
    }
}

pub(crate) trait ResponseHandler {
    type Engine: ScriptEngine;
    type Outputter: Outputter<Response = Self::Response>;
    type Response: Into<ScriptResponse>;
    fn engine(&mut self) -> &mut Self::Engine;
    fn outputter(&mut self) -> &mut Self::Outputter;
    fn handle(
        &mut self,
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
            self.inject(script_response)?;
            let expr = self.engine().parse(script.clone())?;
            self.engine().execute(expr)?;
        }
        Ok(())
    }

    fn inject(&mut self, script_response: ScriptResponse) -> Result<(), Error> {
        let script = format!(
            "var response = {};",
            serde_json::to_string(&script_response).unwrap()
        );
        let expr = self.engine().parse(script)?;
        self.engine().execute(expr)?;
        if let Ok(Value::Object(response_body)) =
            serde_json::from_str(script_response.body.as_str())
        {
            let script = format!(
                "response.body = {};",
                serde_json::to_string(&response_body).unwrap()
            );
            let expr = self.engine().parse(script).unwrap();
            self.engine().execute(expr).unwrap();
        }
        Ok(())
    }
}

pub(crate) trait Outputter {
    type Response;
    fn output_response(&mut self, response: &Self::Response) -> Result<(), Error>;
}

pub(crate) struct DefaultOutputter;

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

pub(crate) struct DefaultResponse(Response);

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
pub(crate) struct ScriptResponse {
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
