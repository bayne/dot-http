use crate::parser::tests::test_file;
use crate::scripter::boa::BoaScriptEngine;
use crate::scripter::{Processable, ScriptEngine};
use crate::{Processed, RequestScript};

#[cfg(test)]
fn setup(src: &'static str) -> BoaScriptEngine {
    let mut engine = BoaScriptEngine::new();
    let expr = engine.parse(String::from(src)).unwrap();
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
