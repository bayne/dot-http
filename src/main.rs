use clap::{App, Arg};
use dot_http::{Config, DotHttp, Parameters};

fn main() {
    let matches = App::new("dot_http")
        .version("0.1.0")
        .about("Executes HTTP stuff")
        .author("Brian Payne")
        .arg(
            Arg::with_name("env_file")
                .long("environment-file")
                .default_value("http-client.env.json"),
        )
        .arg(
            Arg::with_name("snapshot_file")
                .long("snapshot-file")
                .default_value(".snapshot.json"),
        )
        .arg(
            Arg::with_name("env")
                .short("e")
                .long("environment")
                .default_value("dev"),
        )
        .arg(Arg::with_name("INPUT").required(true).index(1))
        .arg(
            Arg::with_name("OFFSET")
                .default_value("1")
                .required(true)
                .index(2),
        )
        .get_matches();

    let script_file = matches.value_of("INPUT").unwrap().to_string();
    let offset: usize = matches.value_of("OFFSET").unwrap().parse().unwrap();
    let env = matches.value_of("env").unwrap().to_string();
    let env_file = matches.value_of("env_file").unwrap().to_string();
    let snapshot_file = matches.value_of("snapshot_file").unwrap().to_string();
    let mut dot_http = DotHttp::new(Config {
        env_file,
        snapshot_file,
    });
    dot_http
        .execute(Parameters {
            script_file,
            offset,
            env,
        })
        .unwrap();
}
