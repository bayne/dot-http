//! # dot_http
//!
//! ![Verify build](https://github.com/bayne/dot_http/workflows/Verify/badge.svg)
//! ![gitmoji](https://img.shields.io/badge/gitmoji-%20%F0%9F%98%9C%20%F0%9F%98%8D-FFDD67.svg?style=flat-square)
//! ![Powered by Rust](https://img.shields.io/badge/Powered%20By-Rust-orange?style=flat-square)
//!
//! dot_http is a text-based scriptable HTTP client. It is a simple language that resembles the actual HTTP protocol but with just a smidgen of magic to make more it practical for someone who builds and tests APIs.
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
//! curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git bayne/dot_http
//! ```
//!
//! ### Binary releases
//!
//! The easiest way for most users is simply to download the prebuilt binaries.
//! You can find binaries for various platforms on the
//! [release](https://github.com/bayne/dot_http/releases) page.
//!
//! ### Cargo
//!
//! First, install [cargo](https://rustup.rs/). Then:
//!
//! ```bash,no_run
//! $ cargo install dot_http
//! ```
//!
//! You will need to use the nightly release for this to work; if in doubt run
//!
//! ```bash,no_run
//! rustup run nightly cargo install dot_http
//! ```
//!
//! ## Usage
//!
//! See `dot_http --help` for usage.
//!
//! ### Vim
//!
//! See this [plugin](https://github.com/bayne/vim_dot_http) to use dot_http within vim.
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
//! $ dot_http simple.http
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
//! $ dot_http simple_with_variables.http
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
//! $ dot_http test.http
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
//! $ dot_http -l 16 test.http
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
//! [Apache License 2.0](https://github.com/bayne/dot_http/blob/master/LICENSE)

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate pest;

pub mod controller;
mod http_client;
mod model;
mod parser;
mod response_handler;
mod script_engine;

use crate::controller::Controller;
use clap::{App, Arg};
use std::path::Path;

fn main() {
    let matches = App::new("controller")
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

    let mut controller = Controller::default();
    match controller.execute(
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
