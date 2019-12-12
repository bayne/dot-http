extern crate pest;

#[cfg(test)]
pub mod tests;

use crate::parser::ErrorKind::{InvalidPair, UnexpectedMethod};
use crate::request_script::*;
use pest::iterators::Pair;
use pest::Parser;
use pest::Span;
use std::convert::TryFrom;

#[derive(Parser)]
#[grammar = "parser/parser.pest"]
struct ScriptParser;

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
}

impl Error {
    fn invalid_pair(expected: Rule, got: Rule) -> Error {
        Error {
            kind: InvalidPair,
            message: format!("Wrong pair. Expected: {:?}, Got: {:?}", expected, got),
        }
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(e: pest::error::Error<Rule>) -> Self {
        Error {
            message: e.to_string(),
            kind: ErrorKind::Parse,
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Handler {
    type Error = Error;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::request_handler => Ok(Handler {
                selection: pair.as_span().into(),
                script: pair
                    .into_inner()
                    .find_map(|pair| match pair.as_rule() {
                        Rule::handler_script => Some(
                            pair.into_inner()
                                .find_map(|pair| match pair.as_rule() {
                                    Rule::handler_script_string => Some(pair.as_str()),
                                    _ => None,
                                })
                                .unwrap(),
                        ),
                        _ => None,
                    })
                    .unwrap()
                    .to_string(),
            }),
            _ => Err(Error::invalid_pair(Rule::request_handler, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Method {
    type Error = Error;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let selection = pair.as_span().into();
        match pair.as_rule() {
            Rule::method => match pair.as_str() {
                "GET" => Ok(Method::Get(selection)),
                "POST" => Ok(Method::Post(selection)),
                "DELETE" => Ok(Method::Delete(selection)),
                "PUT" => Ok(Method::Put(selection)),
                "PATCH" => Ok(Method::Patch(selection)),
                _ => Err(Error {
                    kind: UnexpectedMethod,
                    message: format!("Unsupported method: {}", pair.as_str()),
                }),
            },
            _ => Err(Error::invalid_pair(Rule::method, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Value<Unprocessed> {
    type Error = Error;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match (pair.as_rule(), pair.as_str()) {
            (Rule::request_target, string)
            | (Rule::field_value, string)
            | (Rule::request_body, string) => {
                let selection = pair.as_span().clone().into();
                let inline_scripts = pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::inline_script)
                    .map(InlineScript::try_from)
                    .collect::<Result<Vec<InlineScript>, Error>>()?;

                if !inline_scripts.is_empty() {
                    Ok(Value {
                        state: Unprocessed::WithInline {
                            value: string.to_string(),
                            inline_scripts,
                            selection,
                        },
                    })
                } else {
                    Ok(Value {
                        state: Unprocessed::WithoutInline(string.to_string(), selection),
                    })
                }
            }
            _ => Err(Error::invalid_pair(Rule::request_target, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for InlineScript {
    type Error = Error;
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::inline_script => Ok(InlineScript {
                selection: pair.as_span().into(),
                placeholder: pair.as_str().to_string(),
                script: pair
                    .into_inner()
                    .map(|pair| pair.as_str())
                    .last()
                    .unwrap()
                    .to_string(),
            }),
            _ => Err(Error::invalid_pair(Rule::inline_script, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Header<Unprocessed> {
    type Error = Error;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::header_field => {
                let selection = pair.as_span().clone().into();
                let mut pairs = pair.into_inner();
                Ok(Header {
                    selection,
                    field_name: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::field_name => Some(pair.as_str().to_string()),
                            _ => None,
                        })
                        .unwrap(),
                    field_value: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::field_value => Some(Value::try_from(pair)),
                            _ => None,
                        })
                        .unwrap()?,
                })
            }
            _ => Err(Error::invalid_pair(Rule::header_field, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Request<Unprocessed> {
    type Error = Error;
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::request_script => {
                let selection = pair.as_span().clone().into();
                let mut pairs = pair.into_inner();
                Ok(Request {
                    selection,
                    method: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::method => Some(Method::try_from(pair)),
                            _ => None,
                        })
                        .unwrap()?,
                    target: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::request_target => Some(Value::try_from(pair)),
                            _ => None,
                        })
                        .unwrap()?,
                    headers: pairs
                        .clone()
                        .filter_map(|pair| match pair.as_rule() {
                            Rule::header_field => Some(Header::try_from(pair)),
                            _ => None,
                        })
                        .collect::<Result<Vec<Header<Unprocessed>>, Error>>()?,
                    body: {
                        let pair = pairs.find_map(|pair| match pair.as_rule() {
                            Rule::request_body => Some(pair),
                            _ => None,
                        });
                        if let Some(pair) = pair {
                            Some(Value::try_from(pair)?)
                        } else {
                            None
                        }
                    },
                })
            }
            _ => Err(Error::invalid_pair(Rule::request_script, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for RequestScript<Unprocessed> {
    type Error = Error;
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::request_script => {
                let mut pairs = pair.clone().into_inner();
                Ok(RequestScript {
                    selection: pair.as_span().into(),
                    request: Request::try_from(pair)?,
                    handler: {
                        let pair = pairs.find_map(|pair| match pair.as_rule() {
                            Rule::request_handler => Some(pair),
                            _ => None,
                        });
                        if let Some(pair) = pair {
                            Some(Handler::try_from(pair)?)
                        } else {
                            None
                        }
                    },
                })
            }
            _ => Err(Error::invalid_pair(Rule::request_script, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for File {
    type Error = Error;
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::file => Ok(File {
                request_scripts: pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::request_script)
                    .map(RequestScript::try_from)
                    .collect::<Result<Vec<RequestScript<Unprocessed>>, Error>>()?,
            }),
            _ => Err(Error::invalid_pair(Rule::file, pair.as_rule())),
        }
    }
}

impl From<Span<'_>> for Selection {
    fn from(span: Span<'_>) -> Self {
        let (start_line, start_col) = span.start_pos().line_col();
        let (end_line, end_col) = span.end_pos().line_col();
        Selection {
            start: Position {
                line: start_line,
                col: start_col,
            },
            end: Position {
                line: end_line,
                col: end_col,
            },
        }
    }
}

pub fn parse(source: &str) -> Result<File, Error> {
    Ok(ScriptParser::parse(Rule::file, source)?
        .map(File::try_from)
        .last()
        .unwrap()?)
}
