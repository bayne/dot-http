use crate::common::{create_file, DebugWriter};
use dot_http::output::print::PrintOutputter;
use dot_http::Runtime;
use httpmock::{Mock, MockServer};
use std::borrow::BorrowMut;

mod common;

#[test]
fn simple_get() {
    let server = MockServer::start();
    Mock::new()
        .expect_method(httpmock::Method::GET)
        .expect_path("/simple_get/30")
        .return_status(200)
        .return_header("date", "")
        .create_on(&server);

    let env = "dev";

    let snapshot_file = create_file("{}");
    let env_file = create_file(r#"{"dev": {"id": 30}}"#);
    let script_file = create_file(&format!(
        "GET http://localhost:{port}/simple_get/{{{{id}}}}",
        port = server.port()
    ));
    let writer = &mut DebugWriter(String::new());
    let mut outputter = PrintOutputter::new(writer);

    let mut runtime = Runtime::new(env, &snapshot_file, &env_file, outputter.borrow_mut()).unwrap();
    runtime.execute(&script_file, 1, false).unwrap();

    let DebugWriter(buf) = writer;

    debug_assert_eq!(
        *buf,
        format!(
            "\
GET http://localhost:{}/simple_get/30
HTTP/1.1 200 OK
date: \n\
content-length: 0\
            ",
            server.port()
        )
    );
}

#[test]
fn simple_post() {
    let server = MockServer::start();
    Mock::new()
        .expect_method(httpmock::Method::POST)
        .expect_path("/simple_post")
        .return_status(200)
        .return_header("date", "")
        .return_body(r#"{"value": true}"#)
        .create_on(&server);

    let env = "dev";

    let snapshot_file = create_file("{}");
    let env_file = create_file("{}");
    let script_file = create_file(&format!(
        "\
POST http://localhost:{port}/simple_post

{{
    \"test\": \"body\"
}}\
        ",
        port = server.port(),
    ));
    let writer = &mut DebugWriter(String::new());
    let mut outputter = PrintOutputter::new(writer);

    let mut runtime = Runtime::new(env, &snapshot_file, &env_file, outputter.borrow_mut()).unwrap();

    runtime.execute(&script_file, 1, false).unwrap();

    let DebugWriter(buf) = writer;

    debug_assert_eq!(
        *buf,
        format!(
            "\
POST http://localhost:{port}/simple_post
HTTP/1.1 200 OK
date: \n\
content-length: 15\
\n\n\
{{
  \"value\": true
}}\n\
            ",
            port = server.port()
        )
    );
}
