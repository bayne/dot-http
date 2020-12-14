#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate pest;
#[macro_use]
extern crate pest_derive;

use http::Method;

use crate::parser::Header;
use crate::script_engine::ScriptEngine;
use anyhow::Context;

pub mod output;
pub mod parser;
pub mod script_engine;

pub type Result<T> = anyhow::Result<T>;

pub type Request = http::Request<Option<String>>;

pub type Response = http::Response<Option<String>>;

pub fn process(engine: &mut dyn ScriptEngine, request: &parser::Request) -> Result<Request> {
    let parser::Request {
        method,
        target,
        headers,
        body,
        ..
    } = request;

    let mut builder = http::Request::builder()
        .method(http::Method::from(method))
        .uri(
            engine
                .process(target.into())
                .with_context(|| format!("Failed processing: {}", target))?
                .state
                .value,
        );

    for (name, value) in process_headers(engine, headers)? {
        builder = builder.header(&name, &value);
    }

    builder
        .body(match body {
            None => None,
            Some(body) => Some(engine.process(body.into())?.state.value),
        })
        .map_err(|e| anyhow!("Http Request Error: {}", e))
}

fn process_header(engine: &mut dyn ScriptEngine, header: &Header) -> Result<(String, String)> {
    let parser::Header {
        field_name,
        field_value,
        ..
    } = header;
    engine
        .process(field_value.into())
        .map(|value| (field_name.clone(), value.state.value))
}

fn process_headers(
    engine: &mut dyn ScriptEngine,
    headers: &[Header],
) -> Result<Vec<(String, String)>> {
    headers
        .iter()
        .map(|header| process_header(engine, header))
        .collect()
}

impl From<&parser::Method> for Method {
    fn from(method: &parser::Method) -> Self {
        match method {
            parser::Method::Get(_) => http::Method::GET,
            parser::Method::Post(_) => http::Method::POST,
            parser::Method::Delete(_) => http::Method::DELETE,
            parser::Method::Put(_) => http::Method::PUT,
            parser::Method::Patch(_) => http::Method::PATCH,
            parser::Method::Options(_) => http::Method::OPTIONS,
        }
    }
}

impl From<&parser::InlineScript> for script_engine::InlineScript {
    fn from(inline_script: &parser::InlineScript) -> Self {
        let parser::InlineScript {
            script,
            placeholder,
            selection,
        } = inline_script;
        script_engine::InlineScript {
            script: script.clone(),
            placeholder: placeholder.clone(),
            selection: selection.clone(),
        }
    }
}

impl From<&parser::Unprocessed> for script_engine::Unprocessed {
    fn from(state: &parser::Unprocessed) -> Self {
        match state {
            parser::Unprocessed::WithInline {
                value,
                inline_scripts,
                selection,
            } => script_engine::Unprocessed::WithInline {
                value: value.clone(),
                inline_scripts: inline_scripts.iter().map(|script| script.into()).collect(),
                selection: selection.clone(),
            },
            parser::Unprocessed::WithoutInline(value, selection) => {
                script_engine::Unprocessed::WithoutInline(value.clone(), selection.clone())
            }
        }
    }
}

impl From<&parser::Value> for script_engine::Value<script_engine::Unprocessed> {
    fn from(value: &parser::Value) -> Self {
        let parser::Value { state } = value;
        script_engine::Value {
            state: state.into(),
        }
    }
}
