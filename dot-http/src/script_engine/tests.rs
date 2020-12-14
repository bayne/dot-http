use http::{StatusCode, Version};

use dot_http_lib::script_engine::{inject, Script, ScriptEngine};

use crate::script_engine::create_script_engine;

#[cfg(test)]
fn setup(src: &'static str) -> Box<dyn ScriptEngine> {
    let engine = create_script_engine("{}", "dev", src);
    return engine;
}

#[test]
fn test_syntax_error() {
    let mut engine = setup("{}");

    let result = engine.execute_script(&Script::internal_script("..test"));

    assert!(
        result.is_err(),
        "Should've been an error, but instead got:\n {:#?}",
        result
    );
    if let Err(error) = result {
        assert!(
            error.to_string().to_lowercase().contains("token"),
            "Should've been a lexer error, but instead got:\n {:#?}",
            error
        );
    }
}

#[test]
fn test_parse_error() {
    let mut engine = setup("{}");

    let result = engine.execute_script(&Script::internal_script(".test"));

    assert!(
        result.is_err(),
        "Should've been an error, but instead got:\n {:#?}",
        result
    );
    if let Err(error) = result {
        // Different engine return different errors, checking both
        assert!(
            error.to_string().contains("ParsingError") || error.to_string().contains("SyntaxError"),
            "Should've been a parsing error, but instead got:\n {:#?}",
            error
        );
    }
}

#[test]
#[should_panic]
fn test_initialize_error() {
    let _engine = create_script_engine("invalid", "dev", "{}");
}

#[test]
fn test_initialize() {
    let mut engine = create_script_engine(r#"{"dev": {"a": 1}}"#, "dev", "{}");

    let result = engine.execute_script(&Script::internal_script("a"));

    assert!(result.is_ok());

    if let Ok(result_value) = result {
        assert_eq!(result_value, "1");
    }
}

#[test]
fn test_reset() {
    let mut engine = create_script_engine(r#"{"dev": {"a": 1}}"#, "dev", "{}");
    engine
        .execute_script(&Script::internal_script(
            r#"client.global.set("test", "test_value")"#,
        ))
        .unwrap();

    engine.reset().unwrap();

    let result = engine.execute_script(&Script::internal_script("test"));

    assert!(result.is_ok());

    if let Ok(result_value) = result {
        assert_eq!(result_value, "test_value");
    }
}

#[test]
fn test_headers_available_in_response() {
    let mut engine = create_script_engine("{}", "dev", "{}");

    let response = http::Response::builder()
        .version(Version::HTTP_09)
        .header("X-Auth-Token", "SomeTokenValue")
        .status(StatusCode::OK)
        .body(Some("{}".to_string()))
        .unwrap();

    inject(engine.as_mut(), &response).unwrap();

    let result = engine
        .execute_script(&Script::internal_script("response.headers['x-auth-token']"))
        .unwrap();

    assert_eq!("SomeTokenValue", result);
}
