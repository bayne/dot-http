use super::*;

#[test]
fn test() {
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
%}

###"
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

###"
    );

    let mut request_script_parts = request_script.into_inner();
    let method = request_script_parts.next().unwrap();
    let request_target = request_script_parts.next().unwrap();
    let header_field = request_script_parts.next().unwrap();
    let body = request_script_parts.next();
    assert_eq!(body, None);
}

#[test]
fn test_request_script() {
    let test = "\
GET http://{{host}}.com HTTP/1.1
Accept: *#/*
# Commented Header
Content-Type: {{ content_type }}

{
    \"fieldA\": \"value1\"
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
fn test_request() {
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
fn test_parse() {
    let (test, expected) = test_file();
    let file = parse(test);
    if let Err(e) = &file {
        println!("{}", e.message);
    }
    assert!(file.is_ok(), file);

    let file = file.unwrap();

    let expected = assert_eq!(format!("{:#?}", file), format!("{:#?}", expected));
}

#[cfg(test)]
pub(crate) fn test_file() -> (&'static str, File) {
    (
        "\
# Comment 1
# Comment 2
# Comment 3
POST http://{{host}}.com HTTP/1.1
Accept: *#/*
# Commented Header
Content-Type: {{ content_type }}

{
    \"fieldA\": \"value1\"
}

> {%
    console.log('Success!');

    var a = \"what\"
%}

###
# Request Comment 2
#
GET http://example.com/{{url_param}}
Accept: */*

",
        File {
            request_scripts: vec![
                RequestScript {
                    request: Request {
                        method: Post,
                        target: Value::WithInline {
                            value: "http://{{true}}.com".to_string(),
                            inline_scripts: vec![InlineScript {
                                script: "@true".to_string(),
                                placeholder: "{{@true}}".to_string(),
                            }],
                        },
                        headers: vec![
                            Header {
                                field_name: "Accept".to_string(),
                                field_value: Value::WithoutInline("*#/*".to_string()),
                            },
                            Header {
                                field_name: "Content-Type".to_string(),
                                field_value: Value::WithInline {
                                    value: "{{ content_type }}".to_string(),
                                    inline_scripts: vec![InlineScript {
                                        script: "content_type".to_string(),
                                        placeholder: "{{ content_type }}".to_string(),
                                    }],
                                },
                            },
                        ],
                        body: Some(Value::WithoutInline(
                            "\
{
    \"fieldA\": \"value1\"
}\
                    "
                            .to_string(),
                        )),
                        handler: Some(Handler {
                            script: "\
console.log('Success!');

    var a = \"what\"\
                    "
                            .to_string(),
                        }),
                    },
                },
                RequestScript {
                    request: Request {
                        method: Get,
                        target: Value::WithInline {
                            value: "http://example.com/{{url_param}}".to_string(),
                            inline_scripts: vec![InlineScript {
                                script: "url_param".to_string(),
                                placeholder: "{{url_param}}".to_string(),
                            }],
                        },
                        headers: vec![Header {
                            field_name: "Accept".to_string(),
                            field_value: Value::WithoutInline("*/*".to_string()),
                        }],
                        body: None,
                        handler: None,
                    },
                },
            ],
        },
    )
}
