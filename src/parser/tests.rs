use super::*;
use crate::model::Method::{Get, Post};
use crate::Unprocessed::WithInline;
use crate::Unprocessed::WithoutInline;
use std::path::Path;

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
fn test_min_file() {
    let test = "POST http://example.com HTTP/1.1\n";

    let file = ScriptParser::parse(Rule::file, test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());
}

#[test]
fn test_new_line_in_request_body_file() {
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

    let file = ScriptParser::parse(Rule::file, test);
    if let Err(e) = &file {
        println!("{:?}", e);
    }

    assert!(file.is_ok());
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
fn test_request_handler() {
    let test = "\
> {%
 console.log('hi');
%}
";
    let handler = ScriptParser::parse(Rule::request_handler, test);
    if let Err(e) = &handler {
        println!("{:?}", e);
    }

    assert!(handler.is_ok());
}

#[test]
fn test_parse() {
    let (_, test, expected, _) = test_file();
    let file = parse(Path::new("").to_path_buf(), test);
    if let Err(e) = &file {
        println!("{}", e.message);
    }
    assert!(file.is_ok(), file);

    let file = file.unwrap();

    assert_eq!(format!("{:#?}", file), format!("{:#?}", expected));
}

#[cfg(test)]
pub(crate) fn test_file() -> (
    &'static str,
    &'static str,
    File,
    Vec<RequestScript<Processed>>,
) {
    (
        "\
{
    \"host\": \"example\",
    \"content_type\": \"application/json\",
    \"url_param\": \"?query=id\"
}
        ",
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
                        method: Post(Selection {
                            filename: Path::new("").to_path_buf(),
                            start: Position { line: 4, col: 1 },
                            end: Position { line: 4, col: 5 },
                        }),
                        target: Value {
                            state: WithInline {
                                value: "http://{{host}}.com".to_string(),
                                inline_scripts: vec![InlineScript {
                                    script: "host".to_string(),
                                    placeholder: "{{host}}".to_string(),
                                    selection: Selection {
                                        filename: Path::new("").to_path_buf(),
                                        start: Position { line: 4, col: 13 },
                                        end: Position { line: 4, col: 21 },
                                    },
                                }],
                                selection: Selection {
                                    filename: Path::new("").to_path_buf(),
                                    start: Position { line: 4, col: 6 },
                                    end: Position { line: 4, col: 25 },
                                },
                            },
                        },
                        headers: vec![
                            Header {
                                field_name: "Accept".to_string(),
                                field_value: Value {
                                    state: WithoutInline(
                                        "*#/*".to_string(),
                                        Selection {
                                            filename: Path::new("").to_path_buf(),
                                            start: Position { line: 5, col: 9 },
                                            end: Position { line: 5, col: 13 },
                                        },
                                    ),
                                },
                                selection: Selection {
                                    filename: Path::new("").to_path_buf(),
                                    start: Position { line: 5, col: 1 },
                                    end: Position { line: 5, col: 13 },
                                },
                            },
                            Header {
                                field_name: "Content-Type".to_string(),
                                field_value: Value {
                                    state: WithInline {
                                        value: "{{ content_type }}".to_string(),
                                        inline_scripts: vec![InlineScript {
                                            script: "content_type".to_string(),
                                            placeholder: "{{ content_type }}".to_string(),
                                            selection: Selection {
                                                filename: Path::new("").to_path_buf(),
                                                start: Position { line: 7, col: 15 },
                                                end: Position { line: 7, col: 33 },
                                            },
                                        }],
                                        selection: Selection {
                                            filename: Path::new("").to_path_buf(),
                                            start: Position { line: 7, col: 15 },
                                            end: Position { line: 7, col: 33 },
                                        },
                                    },
                                },
                                selection: Selection {
                                    filename: Path::new("").to_path_buf(),
                                    start: Position { line: 7, col: 1 },
                                    end: Position { line: 7, col: 33 },
                                },
                            },
                        ],
                        body: Some(Value {
                            state: WithoutInline(
                                "{\n    \"fieldA\": \"value1\"\n}".to_string(),
                                Selection {
                                    filename: Path::new("").to_path_buf(),
                                    start: Position { line: 9, col: 1 },
                                    end: Position { line: 11, col: 2 },
                                },
                            ),
                        }),
                        selection: Selection {
                            filename: Path::new("").to_path_buf(),
                            start: Position { line: 4, col: 1 },
                            end: Position { line: 17, col: 3 },
                        },
                    },
                    handler: Some(Handler {
                        script: "console.log(\'Success!\');\n\n    var a = \"what\"".to_string(),
                        selection: Selection {
                            filename: Path::new("").to_path_buf(),
                            start: Position { line: 13, col: 1 },
                            end: Position { line: 17, col: 3 },
                        },
                    }),
                    selection: Selection {
                        filename: Path::new("").to_path_buf(),
                        start: Position { line: 4, col: 1 },
                        end: Position { line: 17, col: 3 },
                    },
                },
                RequestScript {
                    request: Request {
                        method: Get(Selection {
                            filename: Path::new("").to_path_buf(),
                            start: Position { line: 22, col: 1 },
                            end: Position { line: 22, col: 4 },
                        }),
                        target: Value {
                            state: WithInline {
                                value: "http://example.com/{{url_param}}".to_string(),
                                inline_scripts: vec![InlineScript {
                                    script: "url_param".to_string(),
                                    placeholder: "{{url_param}}".to_string(),
                                    selection: Selection {
                                        filename: Path::new("").to_path_buf(),
                                        start: Position { line: 22, col: 24 },
                                        end: Position { line: 22, col: 37 },
                                    },
                                }],
                                selection: Selection {
                                    filename: Path::new("").to_path_buf(),
                                    start: Position { line: 22, col: 5 },
                                    end: Position { line: 22, col: 37 },
                                },
                            },
                        },
                        headers: vec![Header {
                            field_name: "Accept".to_string(),
                            field_value: Value {
                                state: WithoutInline(
                                    "*/*".to_string(),
                                    Selection {
                                        filename: Path::new("").to_path_buf(),
                                        start: Position { line: 23, col: 9 },
                                        end: Position { line: 23, col: 12 },
                                    },
                                ),
                            },
                            selection: Selection {
                                filename: Path::new("").to_path_buf(),
                                start: Position { line: 23, col: 1 },
                                end: Position { line: 23, col: 12 },
                            },
                        }],
                        body: None,
                        selection: Selection {
                            filename: Path::new("").to_path_buf(),
                            start: Position { line: 22, col: 1 },
                            end: Position { line: 24, col: 1 },
                        },
                    },
                    handler: None,
                    selection: Selection {
                        filename: Path::new("").to_path_buf(),
                        start: Position { line: 22, col: 1 },
                        end: Position { line: 24, col: 1 },
                    },
                },
            ],
        },
        vec![
            RequestScript {
                request: Request {
                    method: Post(Selection {
                        filename: Path::new("").to_path_buf(),
                        start: Position { line: 4, col: 1 },
                        end: Position { line: 4, col: 5 },
                    }),
                    target: Value {
                        state: Processed {
                            value: "http://example.com".to_string(),
                        },
                    },
                    headers: vec![
                        Header {
                            field_name: "Accept".to_string(),
                            field_value: Value {
                                state: Processed {
                                    value: "*#/*".to_string(),
                                },
                            },
                            selection: Selection {
                                filename: Path::new("").to_path_buf(),
                                start: Position { line: 5, col: 1 },
                                end: Position { line: 5, col: 13 },
                            },
                        },
                        Header {
                            field_name: "Content-Type".to_string(),
                            field_value: Value {
                                state: Processed {
                                    value: "application/json".to_string(),
                                },
                            },
                            selection: Selection {
                                filename: Path::new("").to_path_buf(),
                                start: Position { line: 7, col: 1 },
                                end: Position { line: 7, col: 33 },
                            },
                        },
                    ],
                    body: Some(Value {
                        state: Processed {
                            value: "{\n    \"fieldA\": \"value1\"\n}".to_string(),
                        },
                    }),
                    selection: Selection {
                        filename: Path::new("").to_path_buf(),
                        start: Position { line: 4, col: 1 },
                        end: Position { line: 17, col: 3 },
                    },
                },
                handler: Some(Handler {
                    script: "console.log(\'Success!\');\n\n    var a = \"what\"".to_string(),
                    selection: Selection {
                        filename: Path::new("").to_path_buf(),
                        start: Position { line: 13, col: 1 },
                        end: Position { line: 17, col: 3 },
                    },
                }),
                selection: Selection {
                    filename: Path::new("").to_path_buf(),
                    start: Position { line: 4, col: 1 },
                    end: Position { line: 17, col: 3 },
                },
            },
            RequestScript {
                request: Request {
                    method: Get(Selection {
                        filename: Path::new("").to_path_buf(),
                        start: Position { line: 22, col: 1 },
                        end: Position { line: 22, col: 4 },
                    }),
                    target: Value {
                        state: Processed {
                            value: "http://example.com/?query=id".to_string(),
                        },
                    },
                    headers: vec![Header {
                        field_name: "Accept".to_string(),
                        field_value: Value {
                            state: Processed {
                                value: "*/*".to_string(),
                            },
                        },
                        selection: Selection {
                            filename: Path::new("").to_path_buf(),
                            start: Position { line: 23, col: 1 },
                            end: Position { line: 23, col: 12 },
                        },
                    }],
                    body: None,
                    selection: Selection {
                        filename: Path::new("").to_path_buf(),
                        start: Position { line: 22, col: 1 },
                        end: Position { line: 24, col: 1 },
                    },
                },
                handler: None,
                selection: Selection {
                    filename: Path::new("").to_path_buf(),
                    start: Position { line: 22, col: 1 },
                    end: Position { line: 24, col: 1 },
                },
            },
        ],
    )
}
