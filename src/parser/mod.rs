extern crate pest;

#[cfg(test)]
pub(crate) mod tests;

use crate::parser::Method::{Get, Post};
use crate::ErrorKind::*;
use crate::*;
use pest::iterators::Pair;
use pest::Parser;
use std::convert::TryFrom;

#[derive(Parser)]
#[grammar = "parser/parser.pest"]
struct ScriptParser;

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
                script: pair
                    .into_inner()
                    .into_iter()
                    .find_map(|pair| match pair.as_rule() {
                        Rule::handler_script => Some(
                            pair.into_inner()
                                .into_iter()
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
        match pair.as_rule() {
            Rule::method => match pair.as_str() {
                "GET" => Ok(Get),
                "POST" => Ok(Post),
                _ => Err(Error {
                    kind: UnexpectedMethod,
                    message: format!("Unsupported method: {}", pair.as_str()),
                }),
            },
            _ => Err(Error::invalid_pair(Rule::method, pair.as_rule())),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Value {
    type Error = Error;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match (pair.as_rule(), pair.as_str()) {
            (Rule::request_target, string)
            | (Rule::field_value, string)
            | (Rule::request_body, string) => {
                let inline_scripts = pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::inline_script)
                    .map(|pair| InlineScript::try_from(pair))
                    .collect::<Result<Vec<InlineScript>, Error>>()?;

                if inline_scripts.len() > 0 {
                    Ok(Value::WithInline {
                        value: string.to_string(),
                        inline_scripts,
                    })
                } else {
                    Ok(Value::WithoutInline(string.to_string()))
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

impl TryFrom<Pair<'_, Rule>> for Header {
    type Error = Error;

    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match pair.as_rule() {
            Rule::header_field => {
                let mut pairs = pair.into_inner();
                Ok(Header {
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

impl TryFrom<Pair<'_, Rule>> for Request {
    type Error = Error;
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::request_script => {
                let mut pairs = pair.into_inner();
                Ok(Request {
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
                        .collect::<Result<Vec<Header>, Error>>()?,
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

impl TryFrom<Pair<'_, Rule>> for RequestScript {
    type Error = Error;
    fn try_from(pair: Pair<'_, Rule>) -> Result<Self, Error> {
        match pair.as_rule() {
            Rule::request_script => Ok(RequestScript {
                request: Request::try_from(pair)?,
            }),
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
                    .map(|pair| RequestScript::try_from(pair))
                    .collect::<Result<Vec<RequestScript>, Error>>()?,
            }),
            _ => Err(Error::invalid_pair(Rule::file, pair.as_rule())),
        }
    }
}

pub(crate) fn parse(source: &str) -> Result<File, Error> {
    Ok(ScriptParser::parse(Rule::file, source)?
        .map(|pair| File::try_from(pair))
        .last()
        .unwrap()?)
}
