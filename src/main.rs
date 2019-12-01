use boa::exec;
//use std::{env, fs::read_to_string, process::exit};
//use std::io::Lines;
//use boa::syntax::ast::punc::Punctuator::StrictNotEq;
use dot_http::executor::execute;

#[runtime::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
//    let buffer = read_to_string("../tests/test.http")?;
//    let buffer = "\
//GET https://httpbin.org/get
//Accept */*
//    ";
////    dbg!(exec(&buffer));
//    let script = "\
//var config = {
//    stuff: function () {
//        console.log('yep');
//    }
//}
//config.stuff();
//return 'a';
//    ";
//
//    let result = parse(buffer);
//
//    let mut res = surf::get("https://httpbin.org/get").await?;
////    dbg!(exec(script));
//    dbg!(res.body_string().await?);
//    Ok(())
    execute().await?;
    Ok(())
}

//enum HttpMethod {
//    OPTIONS,
//    GET,
//    HEAD,
//    POST,
//    PUT,
//    DELETE,
//    TRACE,
//    CONNECT,
//}
//
//struct Request {
//    request_line: RequestLine,
//    headers: Headers,
//    body: Body,
//    request_handler: RequestHandler
//}
//
//struct Body {
//
//}
//
//struct Headers {
//
//}
//
//struct RequestLine {
//    method: HttpMethod,
//    url: Value
//}
//
//struct Value {
//
//}
//
//struct RequestHandler {
//
//}
//
//fn file() {
//
//}
//
//fn requests() {
//
//}
//
//fn request_line() {
//
//}
//
//fn request_header_fields() {
//
//}
//
//fn request_message_body() {
//
//}
