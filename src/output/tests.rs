use crate::{
    output::{
        parse_format, prettify_response_body, print::FormattedOutputter, FormatItem, Outputter,
    },
    Method,
};
use http::{StatusCode, Version};

#[test]
fn output_is_prettified() {
    let pretty_body = prettify_response_body("simple");

    assert_eq!("simple", pretty_body);

    let pretty_body = prettify_response_body(r#"{"stuff": "andThings"}"#);

    assert_eq!(
        r#"
{
  "stuff": "andThings"
}
        "#
        .trim(),
        pretty_body
    );
}

#[test]
fn format_parsing_test() {
    let result = parse_format("a%R,%H,%B");
    assert_eq!(
        result.expect("parse correctly"),
        vec![
            FormatItem::Chars("a".into()),
            FormatItem::FirstLine,
            FormatItem::Chars(",".into()),
            FormatItem::Headers,
            FormatItem::Chars(",".into()),
            FormatItem::Body
        ]
    );

    let result = parse_format("a%X");
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid formatting character 'X'"
    );
    let result = parse_format("%R");
    assert_eq!(
        result.expect("parse correctly"),
        vec![FormatItem::FirstLine,]
    );
}
#[test]
fn test_format_request() {
    let request = http::Request::builder()
        .method(Method::GET)
        .uri("localhost:8080")
        .header("Content-Type","text/json" )
        .body(Some("{\"req\":\"great\"}".to_string()))
        .unwrap();
    let response = http::Response::builder()
        .status(StatusCode::OK)
        .version(Version::HTTP_11)
        .header("Content-Type","text/json" )
        .body(Some("{\"resp\":\"great-resp\"}".to_string()))
        .unwrap();
    let empty_format = parse_format("").expect("valid format");

    let mut buffer = Vec::new();
    let mut empty_output = FormattedOutputter::new(&mut buffer, empty_format.clone(), empty_format);
    empty_output
        .request(&request)
        .expect("print works correctly");
    empty_output
        .response(&response)
        .expect("print works correctly");
    assert_eq!(String::from_utf8(buffer).expect("is a string"), "");

    let full_format = parse_format("%R\n%H\n%B\n").expect("valid format");
    let mut buffer = Vec::new();
    let mut outputter = FormattedOutputter::new(&mut buffer, full_format.clone(), full_format);
    outputter.request(&request).expect("print works correctly");
    outputter
        .response(&response)
        .expect("print works correctly");
    assert_eq!(
        String::from_utf8(buffer).expect("is a string"),
        r#"GET localhost:8080
content-type: text/json

{
  "req": "great"
}
HTTP/1.1 200 OK
content-type: text/json

{
  "resp": "great-resp"
}
"#
    );

    let first_line_headers = parse_format("%R\n%H\n").expect("valid format");
    let mut buffer = Vec::new();
    let mut outputter =
        FormattedOutputter::new(&mut buffer, first_line_headers.clone(), first_line_headers);
    outputter.request(&request).expect("print works correctly");
    outputter
        .response(&response)
        .expect("print works correctly");
    assert_eq!(
        String::from_utf8(buffer).expect("is a string"),
        r#"GET localhost:8080
content-type: text/json

HTTP/1.1 200 OK
content-type: text/json

"#
    );

    let only_first_line = parse_format("%R\n").expect("valid format");
    let mut buffer = Vec::new();
    let mut outputter =
        FormattedOutputter::new(&mut buffer, only_first_line.clone(), only_first_line);
    outputter.request(&request).expect("print works correctly");
    outputter
        .response(&response)
        .expect("print works correctly");
    assert_eq!(
        String::from_utf8(buffer).expect("is a string"),
        "GET localhost:8080\nHTTP/1.1 200 OK\n"
    );
}
