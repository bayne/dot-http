use crate::response_handler::{DefaultOutputter, DefaultResponse, ResponseHandler};

pub struct DefaultResponseHandler;

impl ResponseHandler for DefaultResponseHandler {
    type Outputter = DefaultOutputter;
    type Response = DefaultResponse;
}
