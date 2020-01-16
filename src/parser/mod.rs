extern crate pest;

#[cfg(test)]
pub mod tests;

use crate::model::*;
use pest::error::LineColLocation;
use pest::iterators::Pair;
use pest::Parser;
use pest::Span;
use std::path::PathBuf;

#[derive(Parser)]
#[grammar = "parser/parser.pest"]
struct ScriptParser;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    pub selection: Selection,
}

#[derive(Debug)]
pub enum ErrorKind {
    Parse,
}

fn invalid_pair(expected: Rule, got: Rule) -> ! {
    panic!(format!(
        "Wrong pair. Expected: {:?}, Got: {:?}",
        expected, got
    ))
}

trait FromPair {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self;
}

trait ToSelection {
    fn to_selection(self, filename: PathBuf) -> Selection;
}

impl ToSelection for LineColLocation {
    fn to_selection(self, filename: PathBuf) -> Selection {
        match self {
            LineColLocation::Pos((line, col)) => Selection {
                filename,
                start: Position { line, col },
                end: Position { line, col },
            },
            LineColLocation::Span((start_line, start_col), (end_line, end_col)) => Selection {
                filename,
                start: Position {
                    line: start_line,
                    col: start_col,
                },
                end: Position {
                    line: end_line,
                    col: end_col,
                },
            },
        }
    }
}

impl FromPair for Handler {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::request_handler => Handler {
                selection: pair.as_span().to_selection(filename),
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
            },
            _ => invalid_pair(Rule::request_handler, pair.as_rule()),
        }
    }
}

impl FromPair for Method {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        let selection = pair.as_span().to_selection(filename);
        match pair.as_rule() {
            Rule::method => match pair.as_str() {
                "GET" => Method::Get(selection),
                "POST" => Method::Post(selection),
                "DELETE" => Method::Delete(selection),
                "PUT" => Method::Put(selection),
                "PATCH" => Method::Patch(selection),
                "OPTIONS" => Method::Options(selection),
                _ => panic!(format!("Unsupported method: {}", pair.as_str())),
            },
            _ => invalid_pair(Rule::method, pair.as_rule()),
        }
    }
}

impl FromPair for Value<Unprocessed> {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match (pair.as_rule(), pair.as_str()) {
            (Rule::request_target, string)
            | (Rule::field_value, string)
            | (Rule::request_body, string) => {
                let selection = pair.as_span().clone().to_selection(filename.clone());
                let inline_scripts = pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::inline_script)
                    .map(|pair| InlineScript::from_pair(filename.clone(), pair))
                    .collect::<Vec<InlineScript>>();

                if !inline_scripts.is_empty() {
                    Value {
                        state: Unprocessed::WithInline {
                            value: string.to_string(),
                            inline_scripts,
                            selection,
                        },
                    }
                } else {
                    Value {
                        state: Unprocessed::WithoutInline(string.to_string(), selection),
                    }
                }
            }
            _ => invalid_pair(Rule::request_target, pair.as_rule()),
        }
    }
}

impl FromPair for InlineScript {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::inline_script => InlineScript {
                selection: pair.as_span().to_selection(filename),
                placeholder: pair.as_str().to_string(),
                script: pair
                    .into_inner()
                    .map(|pair| pair.as_str())
                    .last()
                    .unwrap()
                    .to_string(),
            },
            _ => invalid_pair(Rule::inline_script, pair.as_rule()),
        }
    }
}

impl FromPair for Header<Unprocessed> {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::header_field => {
                let selection = pair.as_span().clone().to_selection(filename.clone());
                let mut pairs = pair.into_inner();
                Header {
                    selection,
                    field_name: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::field_name => Some(pair.as_str().to_string()),
                            _ => None,
                        })
                        .unwrap(),
                    field_value: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::field_value => Some(Value::from_pair(filename.clone(), pair)),
                            _ => None,
                        })
                        .unwrap(),
                }
            }
            _ => invalid_pair(Rule::header_field, pair.as_rule()),
        }
    }
}

impl FromPair for Request<Unprocessed> {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::request_script => {
                let selection = pair.as_span().clone().to_selection(filename.clone());
                let mut pairs = pair.into_inner();
                Request {
                    selection,
                    method: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::method => Some(Method::from_pair(filename.clone(), pair)),
                            _ => None,
                        })
                        .unwrap(),
                    target: pairs
                        .find_map(|pair| match pair.as_rule() {
                            Rule::request_target => Some(Value::from_pair(filename.clone(), pair)),
                            _ => None,
                        })
                        .unwrap(),
                    headers: pairs
                        .clone()
                        .filter_map(|pair| match pair.as_rule() {
                            Rule::header_field => Some(Header::from_pair(filename.clone(), pair)),
                            _ => None,
                        })
                        .collect::<Vec<Header<Unprocessed>>>(),
                    body: {
                        let pair = pairs.find_map(|pair| match pair.as_rule() {
                            Rule::request_body => Some(pair),
                            _ => None,
                        });
                        if let Some(pair) = pair {
                            Some(Value::from_pair(filename, pair))
                        } else {
                            None
                        }
                    },
                }
            }
            _ => invalid_pair(Rule::request_script, pair.as_rule()),
        }
    }
}

impl FromPair for RequestScript<Unprocessed> {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::request_script => {
                let mut pairs = pair.clone().into_inner();
                RequestScript {
                    selection: pair.as_span().to_selection(filename.clone()),
                    request: Request::from_pair(filename.clone(), pair),
                    handler: {
                        let pair = pairs.find_map(|pair| match pair.as_rule() {
                            Rule::request_handler => Some(pair),
                            _ => None,
                        });
                        if let Some(pair) = pair {
                            Some(Handler::from_pair(filename, pair))
                        } else {
                            None
                        }
                    },
                }
            }
            _ => invalid_pair(Rule::request_script, pair.as_rule()),
        }
    }
}

impl FromPair for File {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::file => File {
                request_scripts: pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::request_script)
                    .map(|pair| RequestScript::from_pair(filename.clone(), pair))
                    .collect::<Vec<RequestScript<Unprocessed>>>(),
            },
            _ => invalid_pair(Rule::file, pair.as_rule()),
        }
    }
}

impl ToSelection for Span<'_> {
    fn to_selection(self, filename: PathBuf) -> Selection {
        let (start_line, start_col) = self.start_pos().line_col();
        let (end_line, end_col) = self.end_pos().line_col();
        Selection {
            filename,
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

pub fn parse(filename: PathBuf, source: &str) -> Result<File, Error> {
    Ok(ScriptParser::parse(Rule::file, source)
        .map_err(|error| Error {
            kind: ErrorKind::Parse,
            message: error.to_string(),
            selection: error.line_col.to_selection(filename.clone()),
        })?
        .map(|pair| File::from_pair(filename.clone(), pair))
        .last()
        .unwrap())
}
