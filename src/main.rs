//! # dot-http
//!
//! ![Verify build](https://github.com/bayne/dot-http/workflows/Verify/badge.svg)
//! ![gitmoji](https://img.shields.io/badge/gitmoji-%20%F0%9F%98%9C%20%F0%9F%98%8D-FFDD67.svg?style=flat-square)
//! ![Powered by Rust](https://img.shields.io/badge/Powered%20By-Rust-orange?style=flat-square)
//!
//! dot-http is a text-based scriptable HTTP client. It is a simple language that resembles the actual HTTP protocol but with just a smidgen of magic to make it more practical for someone who builds and tests APIs.
//!
//! ![demo](https://user-images.githubusercontent.com/712014/72685883-36b2f700-3aa3-11ea-8a89-0e454391579f.gif)
//!
//! ## Installation
//!
//! ### Script
//!
//! Enter the following in a command prompt:
//!
//! ```text,no_run
//! curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git bayne/dot-http
//! ```
//!
//! ### Binary releases
//!
//! The easiest way for most users is simply to download the prebuilt binaries.
//! You can find binaries for various platforms on the
//! [release](https://github.com/bayne/dot-http/releases) page.
//!
//! ### Cargo
//!
//! First, install [cargo](https://rustup.rs/). Then:
//!
//! ```bash,no_run
//! $ cargo install dot-http
//! ```
//!
//! You will need to use the stable release for this to work; if in doubt run
//!
//! ```bash,no_run
//! rustup run stable cargo install dot-http
//! ```
//!
//! ## Usage
//!
//! See `dot-http --help` for usage.
//!
//! ### Vim
//!
//! See this [plugin](https://github.com/bayne/vim-dot-http) to use dot-http within vim.
//!
//! ### The request
//!
//! The request format is intended to resemble HTTP as close as possible. HTTP was initially designed to be human-readable and simple, so why not use that?
//!
//! **simple.http**
//! ```text,no_run
//! GET http://httpbin.org
//! Accept: */*
//! ```
//! Executing that script just prints the response to stdout:
//! ```text,no_run
//! $ dot-http simple.http
//! GET http://httpbin.org/get
//!
//! HTTP/1.1 200 OK
//! access-control-allow-credentials: true
//! access-control-allow-origin: *
//! content-type: application/json
//! date: Sat, 18 Jan 2020 20:48:50 GMT
//! referrer-policy: no-referrer-when-downgrade
//! server: nginx
//! x-content-type-options: nosniff
//! x-frame-options: DENY
//! x-xss-protection: 1; mode=block
//! content-length: 170
//! connection: keep-alive
//!
//! {
//!   "args": {},
//!   "headers": {
//!     "Accept": "*/*",
//!     "Host": "httpbin.org"
//!   },
//!   "url": "https://httpbin.org/get"
//! }
//! ```
//!
//! ### Variables
//!
//! Use variables to build the scripts dynamically, either pulling data from your environment file or from a previous request's response handler.
//!
//! **simple_with_variables.http**
//! ```text,no_run
//! POST http://httpbin.org/post
//! Accept: */*
//! X-Auth-Token: {{token}}
//!
//! {
//!     "id": {{env_id}}
//! }
//! ```
//!
//! **http-client.env.json**
//! ```text,no_run
//! {
//!     "dev": {
//!         "env_id": 42,
//!         "token": "SuperSecretToken"
//!     }
//! }
//! ```
//!
//! Note that the variables are replaced by their values
//! ```text,no_run
//! $ dot-http simple_with_variables.http
//! POST http://httpbin.org/post
//!
//! HTTP/1.1 200 OK
//! access-control-allow-credentials: true
//! access-control-allow-origin: *
//! content-type: application/json
//! date: Sat, 18 Jan 2020 20:55:24 GMT
//! referrer-policy: no-referrer-when-downgrade
//! server: nginx
//! x-content-type-options: nosniff
//! x-frame-options: DENY
//! x-xss-protection: 1; mode=block
//! content-length: 342
//! connection: keep-alive
//!
//! {
//!   "args": {},
//!   "data": "{\r\n    \"id\": 42\r\n}",
//!   "files": {},
//!   "form": {},
//!   "headers": {
//!     "Accept": "*/*",
//!     "Content-Length": "18",
//!     "Host": "httpbin.org",
//!     "X-Auth-Token": "SuperSecretToken"
//!   },
//!   "json": {
//!     "id": 42
//!   },
//!   "url": "https://httpbin.org/post"
//! }
//! ```
//!
//! ### Environment file
//!
//! Use an environment file to control what initial values variables have
//!
//! **http-client.env.json**
//! ```text,no_run
//! {
//!     "dev": {
//!         "host": localhost,
//!         "token": "SuperSecretToken"
//!     },
//!     "prod": {
//!         "host": example.com,
//!         "token": "ProductionToken"
//!     }
//! }
//! ```
//!
//! **env_demo.http**
//! ```text,no_run
//! GET http://{{host}}
//! X-Auth-Token: {{token}}
//! ```
//!
//! Specifying different environments when invoking the command results in different values
//! for the variables in the script
//!
//! ```text,no_run
//! $ dot-http -e dev env_demo.http
//! GET http://localhost
//! X-Auth-Token: SuperSecretToken
//!
//! $ dot-http -e prod env_demo.htp
//! GET http://example.com
//! X-Auth-Token: ProductionToken
//! ```
//!
//! ### Response handler
//!
//! Use previous requests to populate some of the data in future requests
//!
//! **response_handler.http**
//! ```text,no_run
//! POST http://httpbin.org/post
//! Content-Type: application/json
//!
//! {
//!     "token": "sometoken",
//!     "id": 237
//! }
//!
//! > {%
//!    client.global.set('auth_token', response.body.json.token);
//!    client.global.set('some_id', response.body.json.id);
//! %}
//!
//! ###
//!
//! PUT http://httpbin.org/put
//! X-Auth-Token: {{auth_token}}
//!
//! {
//!     "id": {{some_id}}
//! }
//! ```
//!
//! Data from a previous request
//!
//! ```text,no_run
//! $ dot-http test.http
//! POST http://httpbin.org/post
//!
//! HTTP/1.1 200 OK
//! access-control-allow-credentials: true
//! access-control-allow-origin: *
//! content-type: application/json
//! date: Sat, 18 Jan 2020 21:01:59 GMT
//! referrer-policy: no-referrer-when-downgrade
//! server: nginx
//! x-content-type-options: nosniff
//! x-frame-options: DENY
//! x-xss-protection: 1; mode=block
//! content-length: 404
//! connection: keep-alive
//!
//! {
//!   "args": {},
//!   "data": "{\r\n    \"token\": \"sometoken\",\r\n    \"id\": 237\r\n}",
//!   "files": {},
//!   "form": {},
//!   "headers": {
//!     "Accept": "*/*",
//!     "Content-Length": "46",
//!     "Content-Type": "application/json",
//!     "Host": "httpbin.org"
//!   },
//!   "json": {
//!     "id": 237,
//!     "token": "sometoken"
//!   },
//!   "url": "https://httpbin.org/post"
//! }
//! ```
//!
//! Can populate data in a future request
//!
//! ```text,no_run
//! $ dot-http -l 16 test.http
//! PUT http://httpbin.org/put
//!
//! HTTP/1.1 200 OK
//! access-control-allow-credentials: true
//! access-control-allow-origin: *
//! content-type: application/json
//! date: Sat, 18 Jan 2020 21:02:28 GMT
//! referrer-policy: no-referrer-when-downgrade
//! server: nginx
//! x-content-type-options: nosniff
//! x-frame-options: DENY
//! x-xss-protection: 1; mode=block
//! content-length: 336
//! connection: keep-alive
//!
//! {
//!   "args": {},
//!   "data": "{\r\n    \"id\": 237\r\n}",
//!   "files": {},
//!   "form": {},
//!   "headers": {
//!     "Accept": "*/*",
//!     "Content-Length": "19",
//!     "Host": "httpbin.org",
//!     "X-Auth-Token": "sometoken"
//!   },
//!   "json": {
//!     "id": 237
//!   },
//!   "url": "https://httpbin.org/put"
//! }
//! ```
//!
//! ## Contributing
//!
//! Contributions and suggestions are very welcome!
//!
//! Please create an issue before submitting a PR, PRs will only be accepted if they reference an existing issue. If you have a suggested change please create an issue first so that we can discuss it.
//!
//! ## License
//! [Apache License 2.0](https://github.com/bayne/dot-http/blob/master/LICENSE)

use anyhow::Result;
use clap::{App, Arg};
use dot_http::output::{parse_format, print::FormattedOutputter};
use dot_http::{ClientConfig, Runtime};
use std::borrow::BorrowMut;
use std::io::stdout;
use std::path::Path;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let matches = App::new("dot-http")
        .version(VERSION)
        .about("Executes HTTP scripts")
        .author("Brian Payne")
        .arg(
            Arg::with_name("ENV_FILE")
                .short("n")
                .long("environment-file")
                .help("A file containing a JSON object that describes the initial values for variables")
                .default_value("http-client.env.json"),
        )
        .arg(
            Arg::with_name("SNAPSHOT_FILE")
                .short("p")
                .long("snapshot-file")
                .help("A file containing a JSON object that persists variables between each invocation")
                .default_value(".snapshot.json"),
        )
        .arg(
            Arg::with_name("ENVIRONMENT")
                .short("e")
                .required(true)
                .help("The key value to use on the environment file")
                .default_value("dev"),
        )
        .arg(Arg::with_name("FILE").required(true).index(1))
        .arg(
            Arg::with_name("LINE")
                .short("l")
                .default_value("1")
                .help("A line number that belongs to the request")
                .validator(is_valid_line_number)
                .required(true),
        )
        .arg(
            Arg::with_name("ALL")
                .short("a")
                .help("Sequentially run all the requests in the file"),
        )
        .arg(
            Arg::with_name("ACCEPT_INVALID_CERT")
                .short("k")
                .long("danger-accept-invalid-certs")
                .help("Controls the use of certificate validation."),
        )
        .arg(
            Arg::with_name("RESPONSE_OUTPUT_FORMAT")
                .long("response-output-format")
                .short("s")
                .default_value("%R\n%H\n%B\n")
                .hide_default_value(true)
                .help("Define the format for print the response, possible options %R response line, %H headers, %B body \n[default: %R\\n%H\\n%N\\n]")
        )
        .arg(
            Arg::with_name("REQUEST_OUTPUT_FORMAT")
                .long("request-output-format")
                .short("q")
                .default_value("%R\n\n")
                .hide_default_value(true)
                .help("Define the format for print the request, possible options %R request line, %H headers, %B body \n[default: %R\\n\\n]")
        )
        .usage("dot-http [OPTIONS] <FILE>")
        .get_matches();

    let script_file = matches.value_of("FILE").unwrap();
    let offset: usize = matches.value_of("LINE").unwrap().parse().unwrap();
    let all: bool = matches.is_present("ALL");
    let env = matches.value_of("ENVIRONMENT").unwrap();
    let env_file = matches.value_of("ENV_FILE").unwrap();
    let snapshot_file = matches.value_of("SNAPSHOT_FILE").unwrap();
    let ignore_certificates: bool = matches.is_present("IGNORE_CERT");
    let response_format = matches.value_of("RESPONSE_OUTPUT_FORMAT").unwrap();
    let request_format = matches.value_of("REQUEST_OUTPUT_FORMAT").unwrap();

    let client_config = ClientConfig::new(!ignore_certificates);

    let mut stdout = stdout();
    let mut outputter = FormattedOutputter::new(
        stdout.borrow_mut(),
        parse_format(request_format)?,
        parse_format(response_format)?,
    );

    let mut runtime = Runtime::new(
        env,
        Path::new(snapshot_file),
        Path::new(env_file),
        outputter.borrow_mut(),
        client_config,
    )
    .unwrap();

    runtime.execute(Path::new(script_file), offset, all)
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
