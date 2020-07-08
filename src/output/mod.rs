pub mod print;

#[cfg(test)]
mod tests;

use crate::{Method, Request, Response, Result, Version};
use std::fmt;

fn prettify_response_body(body: &str) -> String {
    match serde_json::from_str(body) {
        Ok(serde_json::Value::Object(response_body)) => {
            serde_json::to_string_pretty(&response_body).unwrap()
        }
        _ => String::from(body),
    }
}

pub trait Outputter {
    fn response(&mut self, response: &Response) -> Result<()>;
    fn request(&mut self, request: &Request) -> Result<()>;
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = match *self {
            Version::Http09 => "HTTP/0.9",
            Version::Http2 => "HTTP/2.0",
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
        };
        f.write_str(version)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method = match *self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Delete => "DELETE",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Options => "OPTIONS",
        };
        f.write_str(method)
    }
}
