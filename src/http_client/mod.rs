use crate::model::*;
use std::fmt::Formatter;
use surf::middleware::HttpClient;
use surf::{http, url, Exception};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        unimplemented!()
    }
}

#[derive(Debug)]
enum ErrorKind {
    RequestFailed(Exception),
    RequestBodyFailed(Exception),
}

impl From<url::ParseError> for Error {
    fn from(_e: url::ParseError) -> Self {
        unimplemented!()
    }
}

impl Into<http::method::Method> for &Method {
    fn into(self) -> http::method::Method {
        match self {
            Method::Get(_) => http::Method::GET,
            Method::Post(_) => http::Method::POST,
            Method::Delete(_) => http::Method::DELETE,
            Method::Put(_) => http::Method::PUT,
            Method::Patch(_) => http::Method::PATCH,
        }
    }
}

fn set_headers<E: std::error::Error + Send + Sync, C: HttpClient<Error = E>>(
    headers: &[Header<Processed>],
    mut request: surf::Request<C>,
) -> surf::Request<C> {
    for header in headers {
        let Value {
            state: Processed { value: field_value },
        } = &header.field_value;
        let field_name = Box::leak(header.field_name.clone().into_boxed_str());
        request = request.set_header(field_name, field_value);
    }
    request
}

fn set_body<E: std::error::Error + Send + Sync, C: HttpClient<Error = E>>(
    body: &Option<Value<Processed>>,
    mut request: surf::Request<C>,
) -> surf::Request<C> {
    if let Some(Value {
        state: Processed { value: body },
    }) = body
    {
        request = request.body_string(body.clone());
    }
    request
}

impl Request<Processed> {
    pub async fn execute(&self) -> Result<Response, Error> {
        let Request {
            method,
            target: Value {
                state: Processed { value: target },
            },
            headers,
            body,
            selection: _selection,
        } = &self;

        let url = url::Url::parse(target)?;
        let request = surf::Request::new(method.into(), url);
        let request = set_headers(headers, request);
        let request = set_body(body, request);

        let mut response = request.await.map_err(|e| Error {
            kind: ErrorKind::RequestFailed(e),
        })?;
        let response_body = response.body_string().await.map_err(|e| Error {
            kind: ErrorKind::RequestBodyFailed(e),
        })?;
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
