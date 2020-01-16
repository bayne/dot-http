use crate::*;
use http_test_server::TestServer;

#[test]
fn test_execute() {
    let server = TestServer::new().unwrap();
    server.create_resource("/defaults");

    let script = RequestScript {
        request: Request {
            method: Method::Get(Selection::none()),
            target: Value {
                state: Processed {
                    value: format_args!("http://localhost:{port}/defaults", port = server.port())
                        .to_string(),
                },
            },
            headers: vec![],
            body: None,
            selection: Selection::none(),
        },
        handler: None,
        selection: Selection::none(),
    };
    let res = script.request.execute().unwrap();
    assert_eq!(200, res.status_code);
}
