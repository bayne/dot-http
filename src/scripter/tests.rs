use crate::parser::tests::test_file;
use crate::scripter::boa::BoaScriptEngine;
use crate::scripter::{Processable, ScriptEngine};
use crate::*;
use boa::exec::{Executor, Interpreter};
use boa::realm::Realm;

#[cfg(test)]
fn setup(src: &'static str) -> BoaScriptEngine {
    let mut engine = BoaScriptEngine::new();
    let expr = engine.parse(String::from(src)).unwrap();
    engine.execute(expr).unwrap();
    return engine;
}

#[test]
fn test_process_file() {
    let (init, _, mut file, expected) = test_file();
    let mut engine = setup(init);
    if let Err(e) = file.process(&mut engine) {
        //        println!("{}", e.message);
    }
    assert_eq!(format!("{:#?}", file), format!("{:#?}", expected));
}
