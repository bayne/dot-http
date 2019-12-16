use crate::response_handler::{DefaultOutputter, DefaultResponse, ResponseHandler};
use crate::script_engine::boa::BoaScriptEngine;

pub struct DefaultResponseHandler;

impl ResponseHandler for DefaultResponseHandler {
    type Engine = BoaScriptEngine;
    type Outputter = DefaultOutputter;
    type Response = DefaultResponse;
}
