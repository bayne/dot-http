use crate::response_handler::boa::DefaultResponseHandler;
use crate::response_handler::{ResponseHandler, ScriptResponse};
use crate::script_engine::boa::BoaScriptEngine;
use crate::script_engine::{Script, ScriptEngine};
use serde_json::Map;

#[test]
fn test_headers_available_in_response() {
    let mut engine = BoaScriptEngine::new();
    engine
        .initialize(String::from("{}"), String::from("dev"), String::from("{}"))
        .unwrap();
    let response_handler = DefaultResponseHandler;

    let mut headers = Map::new();
    headers.insert(
        "X-Auth-Token".to_string(),
        serde_json::to_value("SomeTokenValue").unwrap(),
    );

    let script_response = ScriptResponse {
        body: "{}".to_string(),
        headers,
        status: 200,
        content_type: "application/json".to_string(),
    };

    response_handler
        .inject(&mut engine, script_response)
        .unwrap();

    let expr = engine
        .parse(Script::internal_script(String::from(
            "response.headers['X-Auth-Token']",
        )))
        .unwrap();
    let result = engine.execute(expr).unwrap();

    assert_eq!("SomeTokenValue", result);
}
