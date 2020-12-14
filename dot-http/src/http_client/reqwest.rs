use crate::http_client::{ClientConfig, HttpClient};
use dot_http_lib::{Request, Response, Result};
use reqwest::blocking::Client;
use reqwest::blocking::Request as Reqwest;
use std::convert::TryInto;

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

    fn execute(&self, request: Request) -> Result<Response> {
        // reqwest doesn't deal with the non-existence of a body in their TryFrom method
        // so take it out an deal with it separately
        let (parts, body) = request.into_parts();
        let mut request: Reqwest = http::Request::from_parts(parts, "").try_into()?;
        // override our filler body
        *request.body_mut() = body.map(Into::into);

        let response = self.client.execute(request)?;

        let mut response_builder = http::Response::builder()
            .version(response.version())
            .status(response.status());

        for (name, value) in response.headers() {
            response_builder = response_builder.header(name, value);
        }

        let body = response.text()?;
        let response = if !body.is_empty() {
            response_builder.body(Some(body))?
        } else {
            response_builder.body(None)?
        };

        Ok(response)
    }
}
