use crate::response_handler::{DefaultOutputter, DefaultResponse, QuietOutputter, ResponseHandler};
use crate::script_engine::boa::BoaScriptEngine;

pub struct DefaultResponseHandler;

impl ResponseHandler for DefaultResponseHandler {
    type Engine = BoaScriptEngine;
    type Outputter = DefaultOutputter;
    type Response = DefaultResponse;
}

pub struct QuietResponseHandler;
impl ResponseHandler for QuietResponseHandler {
    type Engine = BoaScriptEngine;
    type Outputter = QuietOutputter;
    type Response = DefaultResponse;
}
