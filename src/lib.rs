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

#[derive(Debug)]
struct File {
    request_scripts: Vec<RequestScript>,
}

#[derive(Debug)]
struct RequestScript {
    request: Request,
    handler: Option<Handler>,
}

#[derive(Debug)]
struct Request {
    method: Method,
    target: Value,
    headers: Vec<Header>,
    body: Option<Value>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Method {
    Get,
    Post,
    Delete,
    Put,
    Patch,
}

#[derive(Debug)]
struct Header {
    field_name: String,
    field_value: Value,
}

#[derive(Debug)]
struct Handler {
    script: String,
}

#[derive(Debug)]
struct ValueWithInline {
    value: String,
    inline_scripts: Vec<InlineScript>,
}

#[derive(Debug)]
enum Value {
    WithInline {
        value: String,
        inline_scripts: Vec<InlineScript>,
    },
    WithoutInline(String),
}

#[derive(Debug)]
struct InlineScript {
    script: String,
    placeholder: String,
}

struct Response {
    version: String,
    status_code: u16,
    status: String,
    headers: Vec<(String, String)>,
    body: String,
}
