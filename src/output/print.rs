use crate::output::{prettify_response_body, Outputter};
use crate::{Request, Response, Result};
use std::io::Write;
use std::ops::Add;

pub struct PrintOutputter<'a, W: Write> {
    writer: &'a mut W,
}

impl<'a, W: Write> PrintOutputter<'a, W> {
    pub fn new(writer: &mut W) -> PrintOutputter<W> {
        PrintOutputter { writer }
    }
}

impl<'a, W: Write> Outputter for PrintOutputter<'a, W> {
    fn response(&mut self, response: &Response) -> Result<()> {
        let Response {
            headers,
            version,
            status,
            body,
            ..
        } = response;

        let headers: String = headers
            .iter()
            .map(|(key, value)| format!("\n{}: {}", key, value))
            .collect();
        let response = format!(
            "\
{http_version} {status}\
{headers}\
{body}\
            ",
            http_version = version,
            status = status,
            headers = headers,
            body = match body {
                Some(body) => String::from("\n\n")
                    .add(&prettify_response_body(&body))
                    .add("\n"),
                None => String::from(""),
            }
        );

        self.writer.write_all(response.as_bytes())?;

        Ok(())
    }

    fn request(&mut self, request: &Request) -> Result<()> {
        let Request { method, target, .. } = request;
        let request = format!("{method} {target}\n", method = method, target = target);
        self.writer.write_all(request.as_bytes())?;
        Ok(())
    }
}
