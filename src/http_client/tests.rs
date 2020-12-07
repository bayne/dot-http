use http::Method;
use httpmock::{Mock, MockServer};

use crate::http_client::reqwest::ReqwestHttpClient;
use crate::http_client::HttpClient;

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

    let request = http::Request::builder()
        .method(Method::POST)
        .uri(format_args!("http://localhost:{port}/defaults", port = server.port()).to_string())
        .header("Content-Type", "application/json")
        .header("X-Custom-Header", "test_validate_verify")
        .body(Some(String::from(body)))
        .unwrap();
    let client = ReqwestHttpClient::default();
    let res = client.execute(request).unwrap();

    assert_eq!(mock.times_called(), 1);
    assert_eq!(res.status().as_u16(), 200);
}
