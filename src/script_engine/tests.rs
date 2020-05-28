use crate::model::{
    InlineScript, Position, Processed, RequestScript, Selection, Unprocessed, Value,
};
use crate::parser::tests::test_file;
use crate::script_engine::{create_script_engine, ErrorKind, Processable, ScriptEngine};

#[cfg(test)]
fn setup(src: &'static str) -> Box<dyn ScriptEngine> {
    let mut engine = create_script_engine();
    engine.initialize(&"{}", &"dev").unwrap();
    engine.reset(src).unwrap();
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
    let error = value.process(&mut *engine).unwrap_err();
    assert_eq!(error.selection.to_string(), ":10:3".to_string());
    let right_kind = if let ErrorKind::Execute(_) = error.kind {
        true
    } else {
        false
    };
    assert!(right_kind);
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
    let error = value.process(&mut *engine).unwrap_err();
    assert_eq!(error.selection.to_string(), ":10:3".to_string());
    let right_kind = if let ErrorKind::Execute(_) = error.kind {
        true
    } else {
        false
    };
    assert!(right_kind);
}

#[test]
fn test_initialize_error() {
    let mut engine = create_script_engine();
    let error = engine.initialize(&"invalid", &"dev").unwrap_err();

    assert_eq!(error.selection.to_string(), ":0:0".to_string());
    let right_kind = if let ErrorKind::ParseInitializeObject(_) = error.kind {
        true
    } else {
        false
    };
    assert!(right_kind);
}
