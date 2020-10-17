use crate::common::{create_file, DebugWriter};
use dot_http::output::parse_format;
use dot_http::output::print::FormattedOutputter;
use dot_http::{ClientConfig, Runtime};
use httpmock::{Mock, MockServer};
use std::borrow::BorrowMut;

mod common;

#[test]
fn multi_post() {
    let server = MockServer::start();

    Mock::new()
        .expect_method(httpmock::Method::POST)
        .expect_path("/multi_post_first")
        .return_status(200)
        .return_header("date", "")
        .return_body(r#"{"value": true}"#)
        .create_on(&server);

    Mock::new()
        .expect_method(httpmock::Method::GET)
        .expect_path("/multi_get_second")
        .return_status(200)
        .return_header("date", "")
        .return_body(r#"{"value": false}"#)
        .create_on(&server);

    Mock::new()
        .expect_method(httpmock::Method::GET)
        .expect_path("/multi_get_third")
        .return_status(204)
        .return_header("date", "")
        .create_on(&server);

    let env = "dev";

    let snapshot_file = create_file("{}");
    let env_file = create_file("{}");
    let script_file = create_file(&format!(
        "\
POST http://localhost:{port}/multi_post_first

{{
    \"test\": \"body\"
}}

###

GET http://localhost:{port}/multi_get_second
###
GET http://localhost:{port}/multi_get_third\
        ",
        port = server.port(),
    ));
    let writer = &mut DebugWriter(String::new());
    let request_format = "%R\n";
    let response_format = "%R\n%H\n%B\n";
    let mut outputter = FormattedOutputter::new(
        writer,
        parse_format(request_format).unwrap(),
        parse_format(response_format).unwrap(),
    );

    let mut runtime = Runtime::new(
        env,
        &snapshot_file,
        &env_file,
        outputter.borrow_mut(),
        ClientConfig::default(),
    )
    .unwrap();

    runtime.execute(&script_file, 1, true).unwrap();

    let DebugWriter(buf) = writer;

    debug_assert_eq!(
        *buf,
        format!(
            "\
POST http://localhost:{port}/multi_post_first
HTTP/1.1 200 OK
date: \n\
content-length: 15\
\n\n\
{{
  \"value\": true
}}
GET http://localhost:{port}/multi_get_second
HTTP/1.1 200 OK
date: \n\
content-length: 16\
\n\n\
{{
  \"value\": false
}}
GET http://localhost:{port}/multi_get_third
HTTP/1.1 204 No Content
date: \n\
content-length: 0\
\n\n\n",
            port = server.port()
        )
    );
}
