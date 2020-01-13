use dot_http::DotHttp;
use http_test_server::TestServer;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::net::TcpStream;
use tempfile::{tempfile, NamedTempFile, TempPath};

#[test]
fn test() {
    let server = TestServer::new().unwrap();
    let resource = server.create_resource("/defaults");
    let requests = server.requests();

    let mut snapshot_file = NamedTempFile::new().unwrap();
    writeln!(snapshot_file, "{{}}");
    let snapshot_file = snapshot_file.into_temp_path();

    let mut env_file = NamedTempFile::new().unwrap();
    writeln!(env_file, "{{}}");
    let env_file = env_file.into_temp_path();

    let mut script_file = NamedTempFile::new().unwrap();
    writeln!(
        script_file,
        "POST http://localhost:{port} HTTP/1.1",
        port = server.port()
    );
    let script_file = script_file.into_temp_path();
    let mut dot_http = DotHttp::new();

    let offset = 1;
    let env = String::from("dev");

    dot_http
        .execute(offset, env, &script_file, &snapshot_file, &env_file)
        .unwrap();
}
