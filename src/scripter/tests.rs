use crate::parser::tests::test_file;
use crate::scripter::{parser_expr, Processable};
use boa::exec::{Executor, Interpreter};
use boa::realm::Realm;

#[cfg(test)]
fn setup(src: &'static str) -> Interpreter {
    let realm = Realm::create();
    let mut engine: Interpreter = Executor::new(realm);

    let expr = parser_expr(src).unwrap();
    engine.run(&expr).unwrap();
    return engine;
}

#[test]
fn test_process_file() {
    let (init, _, mut file, expected) = test_file();
    let mut engine = setup(init);
    if let Err(e) = file.process(&mut engine) {
        println!("{}", e.message);
    }
    assert_eq!(format!("{:#?}", file), format!("{:#?}", expected));
}
