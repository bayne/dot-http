#[cfg(test)]
pub mod tests;

use crate::Result;
use pest::error::LineColLocation;
use pest::iterators::Pair;
use pest::Parser;
use pest::Span;
use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Parser)]
#[grammar = "parser/parser.pest"]
struct ScriptParser;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub selection: Selection,
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.message.as_str())
    }
}

fn invalid_pair(expected: Rule, got: Rule) -> ! {
    panic!("Wrong pair. Expected: {:?}, Got: {:?}", expected, got)
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
            Rule::response_handler => Handler {
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
            _ => invalid_pair(Rule::response_handler, pair.as_rule()),
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
                _ => panic!("Unsupported method: {}", pair.as_str()),
            },
            _ => invalid_pair(Rule::method, pair.as_rule()),
        }
    }
}

impl FromPair for Value {
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

impl FromPair for Header {
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

impl FromPair for Request {
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
                        .collect::<Vec<Header>>(),
                    body: {
                        let pair = pairs.find_map(|pair| match pair.as_rule() {
                            Rule::request_body => Some(pair),
                            _ => None,
                        });
                        pair.map(|pair| Value::from_pair(filename, pair))
                    },
                }
            }
            _ => invalid_pair(Rule::request_script, pair.as_rule()),
        }
    }
}

impl FromPair for RequestScript {
    fn from_pair(filename: PathBuf, pair: Pair<'_, Rule>) -> Self {
        match pair.as_rule() {
            Rule::request_script => {
                let mut pairs = pair.clone().into_inner();
                RequestScript {
                    selection: pair.as_span().to_selection(filename.clone()),
                    request: Request::from_pair(filename.clone(), pair),
                    handler: {
                        let pair = pairs.find_map(|pair| match pair.as_rule() {
                            Rule::response_handler => Some(pair),
                            _ => None,
                        });
                        pair.map(|pair| Handler::from_pair(filename, pair))
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
                    .collect::<Vec<RequestScript>>(),
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

pub fn parse(filename: PathBuf, source: &str) -> Result<File> {
    Ok(ScriptParser::parse(Rule::file, source)
        .map_err(|error| Error {
            message: error.to_string(),
            selection: error.line_col.to_selection(filename.clone()),
        })?
        .map(|pair| File::from_pair(filename.clone(), pair))
        .last()
        .unwrap())
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.state {
            Unprocessed::WithInline { value, .. } => f.write_str(value),
            Unprocessed::WithoutInline(value, _) => f.write_str(value),
        }
    }
}

#[derive(Debug)]
pub struct Value {
    pub state: Unprocessed,
}

#[derive(Debug)]
pub enum Unprocessed {
    WithInline {
        value: String,
        inline_scripts: Vec<InlineScript>,
        selection: Selection,
    },
    WithoutInline(String, Selection),
}

#[derive(Debug)]
pub struct InlineScript {
    pub script: String,
    pub placeholder: String,
    pub selection: Selection,
}

#[derive(Debug)]
pub struct File {
    pub request_scripts: Vec<RequestScript>,
}

#[derive(Debug)]
pub struct RequestScript {
    pub request: Request,
    pub handler: Option<Handler>,
    pub selection: Selection,
}

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub target: Value,
    pub headers: Vec<Header>,
    pub body: Option<Value>,
    pub selection: Selection,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Method {
    Get(Selection),
    Post(Selection),
    Delete(Selection),
    Put(Selection),
    Patch(Selection),
    Options(Selection),
}

#[derive(Debug)]
pub struct Header {
    pub field_name: String,
    pub field_value: Value,
    pub selection: Selection,
}

#[derive(Debug, Clone)]
pub struct Handler {
    pub script: String,
    pub selection: Selection,
}

impl Selection {
    pub fn none() -> Selection {
        Selection {
            filename: PathBuf::default(),
            start: Position { line: 0, col: 0 },
            end: Position { line: 0, col: 0 },
        }
    }
}

impl File {
    pub fn request_scripts(
        &self,
        offset: usize,
        all: bool,
    ) -> impl Iterator<Item = &RequestScript> {
        let mut scripts = self
            .request_scripts
            .iter()
            .filter(move |request_script| {
                (all || request_script.selection.start.line <= offset)
                    && request_script.selection.end.line > offset
            })
            .peekable();

        match scripts.peek() {
            Some(_) => scripts,
            None => panic!("Couldn't find any scripts in our file at the given line number"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Selection {
    pub filename: PathBuf,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}
