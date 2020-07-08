use crate::http_client::reqwest::ReqwestHttpClient;
use crate::http_client::HttpClient;
use crate::{Method, Request};
use httpmock::{Mock, MockServer};

#[test]
fn execute() {
    let body = "{\"result\": \"content\"}";

    let server = MockServer::start();

    let mock = Mock::new()
        .expect_method(httpmock::Method::POST)
        .expect_path("/defaults")
        .expect_body(body)
        .expect_header("X-Custom-Header", "test_validate_verify")
        .expect_header("Content-Type", "application/json")
        .return_status(200)
        .create_on(&server);

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

    assert_eq!(mock.times_called(), 1);
    assert_eq!(res.status_code, 200);
}
