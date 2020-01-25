use crate::model::{Response, Version};
use crate::response_handler::boa::DefaultResponseHandler;
use crate::response_handler::{DefaultResponse, ResponseHandler, ScriptResponse};
use crate::script_engine::boa::BoaScriptEngine;
use crate::script_engine::{Script, ScriptEngine};
use serde_json::{Map, Value};

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

#[test]
fn test_headers_for_script_response() {
    let response = DefaultResponse(Response {
        version: Version::Http11,
        status_code: 0,
        status: "".to_string(),
        headers: vec![(
            String::from("Content-Type"),
            String::from("application/json"),
        )],
        body: "".to_string(),
    });

    let script_response: ScriptResponse = response.into();

    assert_eq!(
        script_response.headers.get("Content-Type"),
        Some(&Value::String(String::from("application/json")))
    )
}
