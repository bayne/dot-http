use crate::model::{Processed, RequestScript};
use crate::parser::tests::test_file;
use crate::script_engine::{create_script_engine, Processable, Script, ScriptEngine};

#[cfg(test)]
fn setup(src: &'static str) -> Box<dyn ScriptEngine> {
    let engine = create_script_engine("{}", "dev", src);
    return engine;
}

#[test]
fn test_process_file() {
    let (init, _, file, expected) = test_file();
    let mut engine = setup(init);
    let request_scripts: Vec<RequestScript<Processed>> = file
        .request_scripts
        .iter()
        .map(|script| script.process(&mut *engine).unwrap())
        .collect();
    assert_eq!(
        format!("{:#?}", request_scripts),
        format!("{:#?}", expected)
    );
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
            error.to_string().contains("SyntaxError"),
            "Should've been a syntax error, but instead got:\n {:#?}",
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
        assert!(
            error.to_string().contains("ParsingError"),
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
        assert!(result_value == "1");
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
