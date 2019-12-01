use crate::executor::Executor;
use crate::*;
use futures::executor::block_on;

#[test]
fn test_execute() {
    block_on(async {
        let file = File {
            request_scripts: vec![RequestScript {
                request: Request {
                    method: Method::Get,
                    target: Value::WithoutInline("http://httpbin.org/get".to_string()),
                    headers: vec![],
                    body: None,
                },
                handler: None,
            }],
        };
        let script = file.request_scripts.get(0).unwrap();
        let executor = Executor::new();
        let res = executor.execute(script).await;
        dbg!(res).unwrap();
    });
}
