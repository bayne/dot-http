use crate::output::{prettify_response_body, FormatItem, Outputter};
use crate::{Request, Response, Result};
use std::io::Write;
use std::ops::Add;

pub struct FormattedOutputter<'a, W: Write> {
    writer: &'a mut W,
    request_format: Vec<FormatItem>,
    response_format: Vec<FormatItem>,
}

impl<'a, W: Write> FormattedOutputter<'a, W> {
    pub fn new(
        writer: &mut W,
        request_format: Vec<FormatItem>,
        response_format: Vec<FormatItem>,
    ) -> FormattedOutputter<W> {
        FormattedOutputter {
            writer,
            request_format,
            response_format,
        }
    }
}

fn format_headers(headers: &[(String, String)]) -> String {
    headers
        .iter()
        .map(|(key, value)| format!("{}: {}\n", key, value))
        .collect()
}

fn format_body(body: &Option<String>) -> String {
    match body {
        Some(body) => prettify_response_body(&body),
        None => String::from(""),
    }
}

impl<'a, W: Write> Outputter for FormattedOutputter<'a, W> {
    fn response(&mut self, response: &Response) -> Result<()> {
        let Response {
            headers,
            version,
            status,
            body,
            ..
        } = response;

        for format_item in &self.response_format {
            let to_write = match format_item {
                FormatItem::FirstLine => format!("{} {}", version, status),
                FormatItem::Headers => format!("{}", format_headers(headers)),
                FormatItem::Body => format!("{}", format_body(body)),
                FormatItem::Chars(s) => s.clone(),
            };

            self.writer.write_all(to_write.as_bytes())?;
        }
        Ok(())
    }
    fn request(&mut self, request: &Request) -> Result<()> {
        let Request {
            method,
            target,
            headers,
            body,
            ..
        } = request;

        for format_item in &self.request_format {
            let to_write = match format_item {
                FormatItem::FirstLine => format!("{} {}", method, target),
                FormatItem::Headers => format!("{}", format_headers(headers)),
                FormatItem::Body => format!("{}", format_body(body)),
                FormatItem::Chars(s) => s.clone(),
            };

            self.writer.write_all(to_write.as_bytes())?;
        }
        Ok(())
    }
}

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
