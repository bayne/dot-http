use crate::http_client::reqwest::ReqwestHttpClient;
use crate::http_client::HttpClient;
use crate::{Method, Request};
use httpmock::Method::POST;
use httpmock::MockServer;

#[test]
fn execute() {
    let body = "{\"result\": \"content\"}";

    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/defaults")
            .body(body)
            .header("X-Custom-Header", "test_validate_verify")
            .header("Content-Type", "application/json");
        then.status(200);
    });

    let request = Request {
        method: Method::Post,
        target: format_args!("http://localhost:{port}/defaults", port = server.port()).to_string(),
        headers: vec![
            (
                String::from("Content-Type"),
                String::from("application/json"),
            ),
            (
                String::from("X-Custom-Header"),
                String::from("test_validate_verify"),
            ),
        ],
        body: Some(String::from(body)),
    };
    let client = ReqwestHttpClient::default();
    let res = client.execute(&request).unwrap();

    mock.assert();
    assert_eq!(res.status_code, 200);
}
