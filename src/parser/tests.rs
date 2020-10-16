use super::*;
use crate::parser;

#[test]
fn script_parser_parse() {
    let test = "\
# Comment 1
# Comment 2
# Comment 3
GET http://{{host}}.com HTTP/1.1
Accept: *#/*
# Commented Header
Content-Type: {{ content_type }}

{
    \"fieldA\": \"value1\"
}

> {%
    console.log('Success!');
%}

###

# Request Comment 2
#
GET http://example.com/{{url_param}}
Accept: */*

###

";
    let files = ScriptParser::parse(Rule::file, test);
    if let Err(e) = &files {
        println!("{}", e.to_string());
    }
    assert!(files.is_ok());

    let file = files.unwrap().next();
    let mut request_scripts = file.unwrap().into_inner();

    let request_script = request_scripts.next().unwrap();

    assert_eq!(
        request_script.as_str(),
        "\
GET http://{{host}}.com HTTP/1.1
Accept: *#/*
# Commented Header
Content-Type: {{ content_type }}

{
    \"fieldA\": \"value1\"
}

> {%
    console.log('Success!');
%}"
    );

    let mut request_script_parts = request_script.into_inner();

    let method = request_script_parts.next().unwrap();

    assert_eq!(method.as_str(), "GET");

    let request_target = request_script_parts.next().unwrap();

    assert_eq!(request_target.as_str(), "http://{{host}}.com");

    let header_field = request_script_parts.next().unwrap();
    assert_eq!(header_field.as_str(), "Accept: *#/*");
    let other_header_field = request_script_parts.next().unwrap();
    assert_eq!(
        other_header_field.as_str(),
        "Content-Type: {{ content_type }}"
    );

    let request_script = request_scripts.next().unwrap();

    assert_eq!(
        request_script.as_str(),
        "\
GET http://example.com/{{url_param}}
Accept: */*

"
    );

    let mut request_script_parts = request_script.into_inner();
    let _method = request_script_parts.next().unwrap();
    let _request_target = request_script_parts.next().unwrap();
    let _header_field = request_script_parts.next().unwrap();
    let body = request_script_parts.next();
    assert_eq!(body, None);
}

#[test]
fn min_file() {
    let test = "POST http://example.com HTTP/1.1\n";

    let file = ScriptParser::parse(Rule::file, test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());
}

#[test]
fn weird_file() {
    let test = "\
POST http://example.com HTTP/1.1

{}

> {% console.log('no'); %}";

    let file = parser::parse(PathBuf::default(), test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());
}

#[test]
fn empty_body_with_handler() {
    let test = "\
POST http://example.com HTTP/1.1
Accept: */*

> {%
    console.log('cool');
%}
###
";

    let file = parser::parse(PathBuf::default(), test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());
}

#[test]
fn new_line_in_request_body_file() {
    let test = "\
POST http://example.com HTTP/1.1
Accept: */*

{
    \"test\": \"a\",
    \"what\": [

    ]
}


> {%
    console.log('cool');
%}

###
";

    let file = parser::parse(PathBuf::default(), test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());
}

#[test]
fn request_script() {
    let test = "\
GET http://{{host}}.com HTTP/1.1
Accept: *#/*
# Commented Header
Content-Type: {{ content_type }}

{
    \"fieldA\": {{
    content_type
    }}
}

> {%
    console.log('Success!');
%}";
    let request_script = ScriptParser::parse(Rule::request_script, test);
    if let Err(e) = &request_script {
        println!("{}", e.to_string());
    }

    assert!(request_script.is_ok());
}

#[test]
fn request() {
    let test = "\
GET http://{{host}}.com HTTP/1.1
Accept: */*
Content-Type: {{ content_type }}
Content-Type2: {{ content_type2 }}
";
    let request = ScriptParser::parse(Rule::request, test);
    if let Err(e) = &request {
        println!("{:?}", e);
    }

    assert!(request.is_ok());
}

#[test]
fn response_handler() {
    let test = "\
> {%
 console.log('hi');
%}
";
    let handler = ScriptParser::parse(Rule::response_handler, test);
    if let Err(e) = &handler {
        println!("{:?}", e);
    }

    assert!(handler.is_ok());
}

#[test]
fn response_handler_with_comment() {
    let test = "\
POST http://httpbin.org/post

{}

# should be fine > {% %}
> {%
  console.log('hi');
%}
";
    let file = ScriptParser::parse(Rule::file, test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());
}

#[test]
fn mixing_body_and_headers() {
    let test = "\
GET http://example.com HTTP/1.1
header: some-value";

    let file = parser::parse(PathBuf::default(), test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());

    let request = &file.unwrap().request_scripts[0].request;

    assert!(&request.headers[0].field_name == "header");
    assert!(&request.body.is_none());
}
