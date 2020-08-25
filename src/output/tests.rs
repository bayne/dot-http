use crate::output::{parse_format, prettify_response_body, FormatItem};

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
    )
}
