use crate::response_handler::DefaultOutputter;
use crate::response_handler::DefaultResponse;
use crate::response_handler::ResponseHandler;
use crate::scripter::boa::BoaScriptEngine;

pub(crate) struct DefaultResponseHandler<'a> {
    pub engine: &'a mut BoaScriptEngine,
    pub outputter: &'a mut DefaultOutputter,
}

impl<'a> DefaultResponseHandler<'a> {
    pub fn new(
        engine: &'a mut BoaScriptEngine,
        outputter: &'a mut DefaultOutputter,
    ) -> DefaultResponseHandler<'a> {
        DefaultResponseHandler { engine, outputter }
    }
}

impl ResponseHandler for DefaultResponseHandler<'_> {
    type Engine = BoaScriptEngine;
    type Outputter = DefaultOutputter;
    type Response = DefaultResponse;

    fn engine(&mut self) -> &mut Self::Engine {
        self.engine
    }

    fn outputter(&mut self) -> &mut Self::Outputter {
        self.outputter
    }
}
