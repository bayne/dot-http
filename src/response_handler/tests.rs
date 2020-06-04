use crate::model::{Response, Version};
use crate::response_handler::DefaultResponseHandler;
use crate::response_handler::{
    prettify_response_body, DefaultResponse, ResponseHandler, ScriptResponse,
};
use crate::script_engine::{create_script_engine, Script};
use serde_json::{Map, Value};

#[test]
fn test_headers_available_in_response() {
    let mut engine = create_script_engine();
    engine.initialize("{}", "dev").unwrap();
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
        .inject(&mut *engine, script_response)
        .unwrap();

    let result = engine
        .execute_script(&Script::internal_script("response.headers['X-Auth-Token']"))
        .unwrap();

    assert_eq!("SomeTokenValue", result);
}

#[test]
fn test_output_is_prettified() {
    let pretty_body = prettify_response_body("simple");

    assert_eq!("simple", pretty_body);

    let pretty_body = prettify_response_body("{\"stuff\": \"andThings\"}");

    assert_eq!(
        "\
{
  \"stuff\": \"andThings\"
}\
",
        pretty_body
    );
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

#[test]
#[should_panic]
fn test_reset_before_initialize() {
    let mut engine = create_script_engine();
    let _ = engine.reset("{}");
}
