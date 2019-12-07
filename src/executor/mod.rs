use crate::{Error, ErrorKind, Method, Request, RequestScript, Value, Response};

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
            Method::Get => Ok(http::Method::GET),
            Method::Post => Ok(http::Method::POST),
            Method::Delete => Ok(http::Method::DELETE),
            Method::Put => Ok(http::Method::PUT),
            Method::Patch => Ok(http::Method::PATCH),
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

    pub async fn execute(&self, script: &RequestScript) -> ExecutableResult {
        if let Request {
            method,
            target: Value::WithoutInline(target),
            headers,
            body,
        } = &script.request
        {
            let mut request = self.request_factory.request(target, method)?;
            for header in headers {
                if let Value::WithoutInline(field_value) = &header.field_value {
                    let field_name = Box::leak(header.field_name.clone().into_boxed_str());
                    request = request.set_header(field_name, field_value);
                } else {
                    return Err(Error {
                        kind: ErrorKind::InvalidHeader,
                        message: "Could not use the header's value".to_string(),
                    });
                }
            }
            if let Some(Value::WithoutInline(body)) = body {
                request = request.body_string(body.clone());
            }

            let mut response = request.await.map_err(|e| ErrorKind::RequestFailed(e))?;
            let response_body = response.body_string().await.map_err(|e| ErrorKind::RequestFailed(e))?;
            Ok(Response {
                status: format!("{}", response.status()),
                status_code: response.status().as_u16(),
                version: format!("{:?}", response.version()),
                headers: response.headers().iter().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
                body: response_body
            })
        } else {
            Err(Error {
                kind: ErrorKind::InvalidRequest,
                message: "".to_string(),
            })
        }
    }
}
