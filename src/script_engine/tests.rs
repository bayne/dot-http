use crate::model::{InlineScript, Position, Selection, Unprocessed, Value};
use crate::parser::tests::test_file;
use crate::script_engine::boa::BoaScriptEngine;
use crate::script_engine::{Processable, Script, ScriptEngine};
use crate::{Processed, RequestScript};

#[cfg(test)]
fn setup(src: &'static str) -> BoaScriptEngine {
    let mut engine = BoaScriptEngine::new();
    engine
        .initialize(String::from("{}"), String::from("dev"), String::from(src))
        .unwrap();
    let expr = engine
        .parse(Script::internal_script(String::from(src)))
        .unwrap();
    engine.execute(expr).unwrap();
    return engine;
}

#[test]
fn test_process_file() {
    let (init, _, file, expected) = test_file();
    let mut engine = setup(init);
    let request_scripts: Vec<RequestScript<Processed>> = file
        .request_scripts
        .iter()
        .map(|script| script.process(&mut engine).unwrap())
        .collect();
    assert_eq!(
        format!("{:#?}", request_scripts),
        format!("{:#?}", expected)
    );
}

#[test]
fn test_lex_error() {
    let mut engine = setup("{}");
    let value = Value {
        state: Unprocessed::WithInline {
            value: "{{..test}}".to_string(),
            inline_scripts: vec![InlineScript {
                script: "..test".to_string(),
                placeholder: "{{..test}}".to_string(),
                selection: Selection {
                    filename: Default::default(),
                    start: Position { line: 10, col: 3 },
                    end: Position { line: 11, col: 4 },
                },
            }],
            selection: Selection::none(),
        },
    };
    let error = value.process(&mut engine).unwrap_err();
    assert_eq!(error.to_string(), ":10:3: Expecting Token .".to_string());
}

#[test]
fn test_parse_error() {
    let mut engine = setup("{}");
    let value = Value {
        state: Unprocessed::WithInline {
            value: "{{.test}}".to_string(),
            inline_scripts: vec![InlineScript {
                script: ".test".to_string(),
                placeholder: "{{.test}}".to_string(),
                selection: Selection {
                    filename: Default::default(),
                    start: Position { line: 10, col: 3 },
                    end: Position { line: 11, col: 4 },
                },
            }],
            selection: Selection::none(),
        },
    };
    let error = value.process(&mut engine).unwrap_err();
    assert_eq!(error.to_string(), ":10:3: Error while parsing".to_string());
}

#[test]
fn test_initialize_error() {
    let mut engine = BoaScriptEngine::new();
    let error = engine
        .initialize(
            String::from("invalid"),
            String::from("dev"),
            String::from("bad"),
        )
        .unwrap_err();

    assert_eq!(
        error.to_string(),
        ":0:0:, Could not parse initialize object, _env_file".to_string()
    );
}
