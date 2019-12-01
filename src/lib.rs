#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate pest;

pub mod executor;
mod parser;
mod scripter;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

#[derive(Debug)]
enum ErrorKind {
    Parse,
    ScriptRun,
    InvalidPair,
    UnexpectedMethod,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

#[derive(Debug)]
struct File {
    request_scripts: Vec<RequestScript>,
}

#[derive(Debug)]
struct RequestScript {
    request: Request,
}

#[derive(Debug)]
struct Request {
    method: Method,
    target: Value,
    headers: Vec<Header>,
    body: Option<Value>,
    handler: Option<Handler>,
}

#[derive(PartialEq, Debug)]
enum Method {
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
enum ProcessState {
    Unprocessed,
    Processed,
}

#[derive(Debug)]
struct ValueWithInline {
    state: ProcessState,
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
