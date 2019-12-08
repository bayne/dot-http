use crate::Error;
use crate::ErrorKind;
use crate::Method;
use crate::Processed;
use crate::Request;
use crate::RequestScript;
use crate::Response;
use crate::Value;

use crate::ErrorKind::RequestFailed;
use std::convert::TryInto;
use surf::middleware::HttpClient;
use surf::{http, url};

#[cfg(test)]
mod tests;

type ExecutableResult = Result<Response, Error>;

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        match kind {
            RequestFailed(ref e) => Error {
                message: e.to_string(),
                kind,
            },
            _ => Error {
                message: format!("{:?}", kind),
                kind,
            },
        }
    }
}

impl From<url::ParseError> for Error {
    fn from(_e: url::ParseError) -> Self {
        unimplemented!()
    }
}

impl TryInto<http::method::Method> for &Method {
    type Error = Error;

    fn try_into(self) -> Result<http::method::Method, Self::Error> {
        match self {
            Method::Get(_) => Ok(http::Method::GET),
            Method::Post(_) => Ok(http::Method::POST),
            Method::Delete(_) => Ok(http::Method::DELETE),
            Method::Put(_) => Ok(http::Method::PUT),
            Method::Patch(_) => Ok(http::Method::PATCH),
        }
    }
}

struct RequestFactory;

impl RequestFactory {
    fn request(
        &self,
        target: &str,
        method: &Method,
    ) -> Result<surf::Request<impl HttpClient>, Error> {
        let url = url::Url::parse(target)?;
        Ok(surf::Request::new(method.try_into()?, url))
    }
}

pub(crate) struct Executor {
    request_factory: RequestFactory,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            request_factory: RequestFactory {},
        }
    }

    pub async fn execute(&self, script: &RequestScript<Processed>) -> ExecutableResult {
        let Request {
            method,
            target: Value {
                state: Processed { value: target },
            },
            headers,
            body,
            selection: _selection,
        } = &script.request;
        let mut request = self.request_factory.request(target, method)?;
        for header in headers {
            let Value {
                state: Processed { value: field_value },
            } = &header.field_value;
            let field_name = Box::leak(header.field_name.clone().into_boxed_str());
            request = request.set_header(field_name, field_value);
        }
        if let Some(Value {
            state: Processed { value: body },
        }) = body
        {
            request = request.body_string(body.clone());
        }

        let mut response = request.await.map_err(|e| ErrorKind::RequestFailed(e))?;
        let response_body = response
            .body_string()
            .await
            .map_err(|e| ErrorKind::RequestFailed(e))?;
        Ok(Response {
            status: format!("{}", response.status()),
            status_code: response.status().as_u16(),
            version: format!("{:?}", response.version()),
            headers: response
                .headers()
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            body: response_body,
        })
    }
}
