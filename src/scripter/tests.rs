use crate::parser::tests::test_file;
use crate::scripter::pre_process;

#[test]
fn test_pre_process() {
    let (_, mut file) = test_file();
    if let Err(e) = pre_process(&mut file) {
        println!("{}", e.message);
    }
}
