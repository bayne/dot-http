use crate::controller::Controller;
use http_test_server::TestServer;
use std::io::Write;
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
    let mut dot_http = Controller::default();

    let offset = 1;
    let env = String::from("dev");

    dot_http
        .execute(offset, env, &script_file, &snapshot_file, &env_file)
        .unwrap();
}

#[test]
fn test_last_line() {
    let server = TestServer::new().unwrap();
    server.create_resource("/first");
    server.create_resource("/second");
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
        "\
POST http://localhost:{port}/first HTTP/1.1
Accept: */*
Content-Type: application/json

{{
    \"id\": 1
}}

> {{%
    console.log('test');
%}}

###
GET http://localhost:{port}/second HTTP/1.1
Accept: */*

",
        port = server.port()
    )
    .unwrap();

    let script_file = script_file.into_temp_path();
    let mut dot_http = Controller::default();

    let offset = 17;
    let env = String::from("dev");

    dot_http
        .execute(offset, env, &script_file, &snapshot_file, &env_file)
        .unwrap();
}
