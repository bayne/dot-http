#![feature(slice_patterns)]
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate pest;

use crate::scripter::ParseError;
use boa::syntax::ast::expr::Expr;
use surf::Exception;

pub mod commands;
mod executor;
mod parser;
mod response_handler;
mod scripter;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug)]
pub enum ErrorKind {
    Parse,
    ScriptRun,
    InvalidPair,
    UnexpectedMethod,
    InvalidRequest,
    InvalidHeader,
    RequestFailed(Exception),
    MissingArgument(&'static str),
    CannotReadEnvFile(std::io::Error),
    CannotParseEnvFile(ParseError),
    InvalidEnvFile(Expr),
    CannotReadRequestScriptFile(std::io::Error),
    UnexpectedEnvironment(Expr),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Selection {
    start: Position,
    end: Position,
}

#[cfg(test)]
impl Selection {
    pub fn none() -> Selection {
        Selection {
            start: Position { line: 0, col: 0 },
            end: Position { line: 0, col: 0 },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
struct Position {
    line: usize,
    col: usize,
}

#[derive(Debug)]
struct File {
    request_scripts: Vec<RequestScript<Unprocessed>>,
}

#[derive(Debug)]
struct RequestScript<S> {
    request: Request<S>,
    handler: Option<Handler>,
    selection: Selection,
}

#[derive(Debug)]
struct Request<S> {
    method: Method,
    target: Value<S>,
    headers: Vec<Header<S>>,
    body: Option<Value<S>>,
    selection: Selection,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Method {
    Get(Selection),
    Post(Selection),
    Delete(Selection),
    Put(Selection),
    Patch(Selection),
}

#[derive(Debug)]
struct Header<S> {
    field_name: String,
    field_value: Value<S>,
    selection: Selection,
}

#[derive(Debug, Clone)]
struct Handler {
    script: String,
    selection: Selection,
}

#[derive(Debug)]
struct Value<S> {
    state: S,
}

#[derive(Debug)]
struct Processed {
    value: String,
}

#[derive(Debug)]
enum Unprocessed {
    WithInline {
        value: String,
        inline_scripts: Vec<InlineScript>,
        selection: Selection,
    },
    WithoutInline(String, Selection),
}

#[derive(Debug)]
struct InlineScript {
    script: String,
    placeholder: String,
    selection: Selection,
}

#[derive(Debug)]
struct Response {
    version: String,
    status_code: u16,
    status: String,
    headers: Vec<(String, String)>,
    body: String,
}
