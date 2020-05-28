use crate::controller::Controller;
use http_test_server::{Request, TestServer};
use std::io::Write;
use std::sync::mpsc::Receiver;
use tempfile::NamedTempFile;

#[test]
fn test() {
    let server = TestServer::new().unwrap();
    let _resource = server.create_resource("/defaults");
    let _requests = server.requests();

    let mut snapshot_file = NamedTempFile::new().unwrap();

    writeln!(snapshot_file, "{{}}").unwrap();

    let snapshot_file = snapshot_file.into_temp_path();

    let mut env_file = NamedTempFile::new().unwrap();

    writeln!(env_file, "{{}}").unwrap();

    let env_file = env_file.into_temp_path();

    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(
        script_file,
        "POST http://localhost:{port} HTTP/1.1",
        port = server.port()
    )
    .unwrap();

    let script_file = script_file.into_temp_path();
    let mut controller = Controller::default();

    let offset = 1;
    let env = String::from("dev");

    controller
        .execute(offset, false, env, &script_file, &snapshot_file, &env_file)
        .unwrap();
}

#[test]
fn test_last_line() {
    let mut requests = multi_line_setup(
        17,
        false,
        r#"
POST http://localhost:{{port}}/first HTTP/1.1
Accept: */*
Content-Type: application/json

{
    "id": 1
}

> {%
    console.log('test');
%}

###
GET http://localhost:{{port}}/second HTTP/1.1
Accept: */*
    "#,
    )
    .into_iter();

    let _second = requests.next().expect("We should have a first request");
    assert_eq!(None, requests.next(), "We should only have 1 request");
}

#[test]
fn test_all_requests() {
    let mut requests = multi_line_setup(
        0,
        true,
        r#"
GET http://localhost:{{port}}/first HTTP/1.1
Accept: */*
Content-Type: application/json

{
    "id": 1
}

###
GET http://localhost:{{port}}/second HTTP/1.1
Accept: */*
    "#,
    )
    .into_iter();

    let _first = requests.next().expect("We should have a first request");
    let _second = requests.next().expect("We should have a second request");
    assert_eq!(None, requests.next(), "We should only have 2 requests");
}

#[test]
fn test_all_global_object() {
    let mut requests = multi_line_setup(
        0,
        true,
        r#"
GET http://localhost:{{port}}/first HTTP/1.1
Accept: */*
Content-Type: application/json

> {%
    client.global.set('global_state', response.body.response);
%}

###

GET http://localhost:{{port}}/{{global_state}} HTTP/1.1
Accept: */*
    "#,
    )
    .into_iter();

    let _first = requests.next().expect("We should have a first request");
    let second = requests.next().expect("We should have a second request");
    assert_eq!(None, requests.next(), "We should only have 2 requests");

    assert_eq!(
        "/some_response", second.url,
        "We should be able to pass state via the global object"
    );
}

/// This test ensures that we must operate through the global object and that we don't propagate state
/// via global variables
#[test]
fn test_all_global_state() {
    let mut requests = multi_line_setup(
        0,
        true,
        r#"
GET http://localhost:{{port}}/first HTTP/1.1

> {%
    var someGlobal = "global";
    client.global.set('global_state', response.body.response);
%}

###

GET http://localhost:{{port}}/second HTTP/1.1

> {%
    var found = (typeof someGlobal) !== "undefined";

    client.global.set('found', found);
%}

###

GET http://localhost:{{port}}/{{found}}
    "#,
    )
    .into_iter();

    let _first = requests.next().expect("We should have a first request");
    let _second = requests.next().expect("We should have a second request");
    let third = requests.next().expect("We should have a second request");
    assert_eq!(None, requests.next(), "We should only have 2 requests");

    assert_eq!(
        "/false", third.url,
        "We should not persist global variables across runs"
    );
}

fn multi_line_setup(offset: usize, all: bool, scripts: &str) -> Receiver<Request> {
    let server = TestServer::new().unwrap();
    server.create_resource("/first").body(
        r#"{
            "response": "some_response"
        }"#,
    );
    server.create_resource("/second");
    let requests = server.requests();

    let mut snapshot_file = NamedTempFile::new().unwrap();

    writeln!(snapshot_file, "{{}}").unwrap();

    let snapshot_file = snapshot_file.into_temp_path();

    let mut env_file = NamedTempFile::new().unwrap();

    writeln!(
        env_file,
        r#"{{
        "dev": {{
            "port": {port}
        }}
    }}"#,
        port = server.port()
    )
    .unwrap();

    let env_file = env_file.into_temp_path();

    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(script_file, "{}", scripts,).unwrap();

    let script_file = script_file.into_temp_path();
    let mut controller = Controller::default();

    let env = String::from("dev");

    controller
        .execute(offset, all, env, &script_file, &snapshot_file, &env_file)
        .unwrap();

    requests
}
