use clap::{App, Arg};
use dot_http::DotHttp;
use std::path::Path;

fn main() {
    let matches = App::new("dot_http")
        .version("0.1.0")
        .about("Executes HTTP scripts")
        .author("Brian Payne")
        .arg(
            Arg::with_name("env file")
                .long("environment-file")
                .hidden(true)
                .default_value("http-client.env.json"),
        )
        .arg(
            Arg::with_name("snapshot file")
                .long("snapshot-file")
                .hidden(true)
                .default_value(".snapshot.json"),
        )
        .arg(
            Arg::with_name("environment")
                .short("e")
                .required(false)
                .default_value("dev"),
        )
        .arg(Arg::with_name("script file").required(true).index(1))
        .arg(
            Arg::with_name("line number")
                .short("l")
                .default_value("1")
                .validator(is_valid_line_number)
                .required(false),
        )
        .get_matches();

    let script_file = matches.value_of("script file").unwrap().to_string();
    let offset: usize = matches.value_of("line number").unwrap().parse().unwrap();
    let env = matches.value_of("environment").unwrap().to_string();
    let env_file = matches.value_of("env file").unwrap().to_string();
    let snapshot_file = matches.value_of("snapshot file").unwrap().to_string();

    let mut dot_http = DotHttp::default();
    match dot_http.execute(
        offset,
        env,
        Path::new(&script_file),
        Path::new(&snapshot_file),
        Path::new(&env_file),
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

fn is_valid_line_number(val: String) -> Result<(), String> {
    match val.parse::<i32>() {
        Ok(line_number) if line_number <= 0 => {
            Err(String::from("Line number is not a valid integer"))
        }
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Line number is not a valid integer")),
    }
}
