use crate::{ClientConfig, Request, Response, Result};

#[cfg(test)]
mod tests;

pub mod reqwest;

pub trait HttpClient {
    fn create(config: ClientConfig) -> Self
    where
        Self: Sized;

    fn execute(&self, request: &Request) -> Result<Response>;
}
