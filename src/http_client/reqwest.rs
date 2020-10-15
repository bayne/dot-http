use crate::http_client::{ClientConfig, HttpClient};
use crate::{Method, Request, Response, Result, Version};
use reqwest::blocking::{Client, RequestBuilder};
use reqwest::header::HeaderMap;
use std::convert::{TryFrom, TryInto};

pub struct ReqwestHttpClient {
    client: Client,
}

impl Default for ReqwestHttpClient {
    fn default() -> Self {
        Self::create(ClientConfig::default())
    }
}

impl HttpClient for ReqwestHttpClient {
    fn create(config: ClientConfig) -> ReqwestHttpClient
    where
        Self: Sized,
    {
        let client = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(config.ssl_check)
            .build()
            .unwrap();

        ReqwestHttpClient { client }
    }

    fn execute(&self, request: &Request) -> Result<Response> {
        let Request {
            method,
            target,
            headers,
            body,
        } = request;
        let mut request_builder = self.client.request(method.into(), target);
        request_builder = set_headers(headers, request_builder);
        if let Some(body) = body {
            request_builder = set_body(body, request_builder);
        }
        let response = request_builder.send()?;

        response.try_into()
    }
}

fn set_headers(
    headers: &[(String, String)],
    mut request_builder: RequestBuilder,
) -> RequestBuilder {
    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }
    request_builder
}

impl From<&Method> for reqwest::Method {
    fn from(method: &Method) -> Self {
        match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Delete => reqwest::Method::DELETE,
            Method::Put => reqwest::Method::PUT,
            Method::Patch => reqwest::Method::PATCH,
            Method::Options => reqwest::Method::OPTIONS,
        }
    }
}

impl From<reqwest::Version> for Version {
    fn from(version: reqwest::Version) -> Self {
        match version {
            reqwest::Version::HTTP_09 => Version::Http09,
            reqwest::Version::HTTP_2 => Version::Http2,
            reqwest::Version::HTTP_10 => Version::Http10,
            reqwest::Version::HTTP_11 => Version::Http11,
            _ => panic!("Non-exhaustive"),
        }
    }
}

struct Headers(Vec<(String, String)>);
impl TryFrom<reqwest::blocking::Response> for Response {
    type Error = anyhow::Error;

    fn try_from(response: reqwest::blocking::Response) -> Result<Self> {
        let Headers(headers) = response.headers().try_into()?;
        Ok(Response {
            version: response.version().into(),
            status_code: response.status().as_u16(),
            status: response.status().to_string(),
            headers,
            body: match response.text()? {
                body if !body.is_empty() => Some(body),
                _ => None,
            },
        })
    }
}

impl TryFrom<&HeaderMap> for Headers {
    type Error = anyhow::Error;

    fn try_from(value: &HeaderMap) -> Result<Self> {
        let mut headers = vec![];
        for (header_name, header_value) in value.iter() {
            headers.push((header_name.to_string(), header_value.to_str()?.to_string()))
        }
        Ok(Headers(headers))
    }
}

fn set_body(body: &str, mut request_builder: RequestBuilder) -> RequestBuilder {
    let body = body.trim();
    request_builder = request_builder.body::<reqwest::blocking::Body>(body.to_string().into());
    request_builder
}
