#[derive(Debug)]
pub struct Response {
    pub version: String,
    pub status_code: u16,
    pub status: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug)]
pub struct File {
    pub request_scripts: Vec<RequestScript<Unprocessed>>,
}

#[derive(Debug)]
pub struct RequestScript<S> {
    pub request: Request<S>,
    pub handler: Option<Handler>,
    pub selection: Selection,
}

#[derive(Debug)]
pub struct Request<S> {
    pub method: Method,
    pub target: Value<S>,
    pub headers: Vec<Header<S>>,
    pub body: Option<Value<S>>,
    pub selection: Selection,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Method {
    Get(Selection),
    Post(Selection),
    Delete(Selection),
    Put(Selection),
    Patch(Selection),
}

#[derive(Debug)]
pub struct Header<S> {
    pub field_name: String,
    pub field_value: Value<S>,
    pub selection: Selection,
}

#[derive(Debug, Clone)]
pub struct Handler {
    pub script: String,
    pub selection: Selection,
}

#[derive(Debug)]
pub struct Value<S> {
    pub state: S,
}

#[derive(Debug)]
pub struct Processed {
    pub value: String,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
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
pub struct Position {
    pub line: usize,
    pub col: usize,
}
