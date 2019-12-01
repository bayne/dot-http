use dot_http::commands::app;
use dot_http::{Error, ErrorKind};

const OK: i32 = 0;
const PARSE_ERROR: i32 = 1;

fn main() {
    match app() {
        Ok(_) => {
            std::process::exit(OK);
        }
        Err(Error {
            kind: ErrorKind::Parse,
            message,
        }) => {
            println!("{}", message);
            std::process::exit(PARSE_ERROR);
        }
        Err(e) => {
            panic!(e.message);
        }
    }
}
