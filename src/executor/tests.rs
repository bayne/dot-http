use crate::executor::Executor;
use crate::*;
use futures::executor::block_on;

#[test]
fn test_execute() {
    block_on(async {
        let script = RequestScript {
            request: Request {
                method: Method::Get(Selection::none()),
                target: Value {
                    state: Processed {
                        value: "http://httpbin.org/get".to_string(),
                    },
                },
                headers: vec![],
                body: None,
                selection: Selection::none(),
            },
            handler: None,
            selection: Selection::none(),
        };
        let executor = Executor::new();
        let res = executor.execute(&script).await;
        dbg!(res).unwrap();
    });
}
