use crate::controller::ErrorKind;
use crate::model::*;
use reqwest::blocking::Body;
use reqwest::blocking::RequestBuilder;
use reqwest::blocking::Response as HttpResponse;
use reqwest::{Error, Method as HttpMethod, Version as HttpVersion};

#[cfg(test)]
mod tests;

fn set_headers(
    headers: &[Header<Processed>],
    mut request_builder: RequestBuilder,
) -> RequestBuilder {
    for header in headers {
        let Value {
            state: Processed { value: field_value },
        } = &header.field_value;
        request_builder = request_builder.header(&header.field_name, field_value);
    }
    request_builder
}

impl From<&Method> for HttpMethod {
    fn from(method: &Method) -> Self {
        match method {
            Method::Get(_) => HttpMethod::GET,
            Method::Post(_) => HttpMethod::POST,
            Method::Delete(_) => HttpMethod::DELETE,
            Method::Put(_) => HttpMethod::PUT,
            Method::Patch(_) => HttpMethod::PATCH,
            Method::Options(_) => HttpMethod::OPTIONS,
        }
    }
}

impl From<HttpVersion> for Version {
    fn from(version: HttpVersion) -> Self {
        match version {
            HttpVersion::HTTP_09 => Version::Http09,
            HttpVersion::HTTP_2 => Version::Http2,
            HttpVersion::HTTP_10 => Version::Http10,
            HttpVersion::HTTP_11 => Version::Http11,
            _ => panic!("Non-exhaustive"),
        }
    }
}

impl From<HttpResponse> for Response {
    fn from(response: HttpResponse) -> Self {
        Response {
            version: response.version().into(),
            status_code: response.status().as_u16(),
            status: response.status().to_string(),
            headers: response
                .headers()
                .iter()
                .map(|(header_name, header_value)| {
                    (
                        header_name.to_string(),
                        header_value.to_str().unwrap().to_string(),
                    )
                })
                .collect(),
            body: response.text().unwrap(),
        }
    }
}

impl From<reqwest::Error> for crate::controller::Error {
    fn from(e: reqwest::Error) -> Self {
        crate::controller::Error {
            kind: ErrorKind::HttpClient(e),
        }
    }
}

fn set_body(
    body: &Option<Value<Processed>>,
    mut request_builder: RequestBuilder,
) -> RequestBuilder {
    if let Some(Value {
        state: Processed { value: body },
    }) = body
    {
        let body = String::from(body.trim());
        request_builder = request_builder.body::<Body>(body.into());
    }
    request_builder
}

impl Request<Processed> {
    pub fn execute(&self) -> Result<Response, Error> {
        let Request {
            method,
            target: Value {
                state: Processed { value: target },
            },
            headers,
            body,
            selection: _selection,
        } = &self;

        let client = reqwest::blocking::Client::new();
        let mut request_builder = client.request(method.into(), target);
        request_builder = set_headers(headers, request_builder);
        request_builder = set_body(body, request_builder);
        let response = request_builder.send()?;

        Ok(response.into())
    }
}
