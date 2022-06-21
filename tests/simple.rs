use crate::common::{create_file, DebugWriter};
use dot_http::output::parse_format;
use dot_http::output::print::FormattedOutputter;
use dot_http::{ClientConfig, Runtime};
use httpmock::MockServer;
use std::borrow::BorrowMut;

mod common;

#[test]
fn simple_get() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(httpmock::Method::GET).path("/simple_get/30");
        then.status(200).header("date", "");
    });

    let env = "dev";

    let snapshot_file = create_file("{}");
    let env_file = create_file(r#"{"dev": {"id": 30}}"#);
    let script_file = create_file(&format!(
        "GET http://localhost:{port}/simple_get/{{{{id}}}}",
        port = server.port()
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
\n\n\n",
            server.port()
        )
    );
}

#[test]
fn simple_post() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(httpmock::Method::POST).path("/simple_post");
        then.status(200)
            .header("date", "")
            .body(r#"{"value": true}"#);
    });

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
