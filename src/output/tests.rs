use crate::output::prettify_response_body;

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
