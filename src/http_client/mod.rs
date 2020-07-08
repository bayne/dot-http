use crate::{Request, Response, Result};

#[cfg(test)]
mod tests;

pub mod reqwest;

pub trait HttpClient {
    fn execute(&self, request: &Request) -> Result<Response>;
}
