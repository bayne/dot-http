use crate::response_handler::{DefaultOutputter, DefaultResponse, ResponseHandler};
use crate::scripter::boa::BoaScriptEngine;

pub struct DefaultResponseHandler {
    pub outputter: DefaultOutputter,
}

impl DefaultResponseHandler {
    pub fn new(outputter: DefaultOutputter) -> DefaultResponseHandler {
        DefaultResponseHandler { outputter }
    }
}

impl ResponseHandler for DefaultResponseHandler {
    type Engine = BoaScriptEngine;
    type Outputter = DefaultOutputter;
    type Response = DefaultResponse;

    fn outputter(&mut self) -> &mut Self::Outputter {
        &mut self.outputter
    }
}
