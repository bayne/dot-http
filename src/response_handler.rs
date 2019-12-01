use crate::executor::Response;
use crate::scripter::parser_expr;
use crate::{Error, RequestScript};
use boa::exec::{Executor, Interpreter};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};

pub(crate) struct DefaultResponseHandler<'a> {
    pub engine: &'a mut Interpreter,
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

impl ScriptResponse {
    fn inject(&self, engine: &mut Interpreter) {
        let script = format!("var response = {};", serde_json::to_string(self).unwrap());
        let expr = parser_expr(script.as_str()).unwrap();
        engine.run(&expr).unwrap();
        if let Ok(Value::Object(response_body)) = serde_json::from_str(self.body.as_str()) {
            let script = format!(
                "response.body = {};",
                serde_json::to_string(&response_body).unwrap()
            );
            let expr = parser_expr(script.as_str()).unwrap();
            engine.run(&expr).unwrap();
        }
    }
}

pub(crate) trait ResponseHandler {
    type Response: Display + Into<ScriptResponse>;
    fn handle(&mut self, request_script: &RequestScript, response: Response) -> Result<(), Error>;
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

impl ResponseHandler for DefaultResponseHandler<'_> {
    type Response = DefaultResponse;

    fn handle(&mut self, request_script: &RequestScript, response: Response) -> Result<(), Error> {
        let response: Self::Response = response.into();
        println!("{}", response);
        if let Some(handler_script) = &request_script.handler {
            let script_response: ScriptResponse = response.into();
            script_response.inject(self.engine);
            let expr = parser_expr(handler_script.script.as_str())?;
            self.engine.run(&expr)?;
        }
        Ok(())
    }
}
