#[macro_use]
extern crate anyhow;

use std::io;
use std::io::ErrorKind;
use std::io::Write;

use log::Level;
use serde_json::Map;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTextAreaElement;
use yew::format::Text;
use yew::prelude::*;
use yew::services::fetch::FetchTask;
use yew::services::FetchService;

use dot_http_lib::output::print::FormattedOutputter;
use dot_http_lib::output::{FormatItem, Outputter};
use dot_http_lib::parser::RequestScript;
use dot_http_lib::script_engine::ScriptEngine;
use dot_http_lib::{parser, process, Response, Result};

use crate::script_engine::BrowserScriptEngine;

mod script_engine;

const EXAMPLE_REQUEST: &str = r#"GET http://localhost:8080/example.json


> {%
    client.global.set('next', response.body.next);
%}

###

GET http://localhost:8080/{{next}}.json

"#;

#[wasm_bindgen(start)]
pub fn run_app() -> core::result::Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(Level::Debug));

    yew::start_app::<DotHttpClient>();

    Ok(())
}

pub struct TextAreaWriter {
    output_ref: NodeRef,
}

impl TextAreaWriter {
    fn append(&self, value: &str) -> Result<()> {
        let textarea = self
            .output_ref
            .cast::<HtmlTextAreaElement>()
            .ok_or_else(|| anyhow!("Couldn't find TextArea"))?;

        textarea.set_value(&format!("{}{}", textarea.value(), value));

        Ok(())
    }

    fn clear(&self) -> Result<()> {
        let textarea = self
            .output_ref
            .cast::<HtmlTextAreaElement>()
            .ok_or_else(|| anyhow!("Couldn't find TextArea"))?;

        textarea.set_value(&"");

        Ok(())
    }
}

impl Write for TextAreaWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buffer_str =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

        self.append(buffer_str)
            .map_err(|e| io::Error::new(ErrorKind::NotFound, e))?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct DotHttpClient {
    code_ref: NodeRef,
    // output_ref: NodeRef,
    writer: TextAreaWriter,
    link: ComponentLink<Self>,

    fetch_task: Option<FetchTask>,
}

pub struct RunningRequest {
    request_scripts: Vec<RequestScript>,
    engine: BrowserScriptEngine,
    // outputter: FormattedOutputter<'a, TextAreaWriter>,
}

#[allow(clippy::large_enum_variant)]
pub enum Message {
    StartRequest,
    Run(RunningRequest),
    Response(RequestScript, Response, RunningRequest),
}

impl Component for DotHttpClient {
    type Message = Message;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        DotHttpClient {
            code_ref: NodeRef::default(),
            // output_ref: NodeRef::default(),
            writer: TextAreaWriter {
                output_ref: NodeRef::default(),
            },
            link,
            fetch_task: None,
        }
    }

    fn update(&mut self, message: Message) -> ShouldRender {
        match message {
            Message::StartRequest => {
                self.writer.clear().unwrap();

                match self.build_request(0, true) {
                    Ok(request) => self.link.send_message(Message::Run(request)),
                    Err(e) => {
                        log::error!("Error creating request: {}", e);
                        self.writer
                            .append(&format!("Error creating request: {}", e))
                            .unwrap();
                    }
                }
            }
            Message::Run(running_request) => match self.run_request(running_request) {
                Ok(fetch_task) => self.fetch_task = fetch_task,
                Err(e) => {
                    log::error!("Error running request: {}", e);
                    self.writer
                        .append(&format!("Error running request: {}", e))
                        .unwrap();
                }
            },
            Message::Response(script, response, running_request) => {
                if let Err(e) = self.handle_response(script, response, running_request) {
                    log::error!("Error handling response: {}", e);
                    self.writer
                        .append(&format!("Error handling response: {}", e))
                        .unwrap();
                }
            }
        }

        false
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        false
    }

    fn view(&self) -> Html {
        let run_callback = self.link.callback(|_| Message::StartRequest);

        html! { <>
            <textarea ref=self.code_ref.clone() style="\
                width: 500px;\
                height: 600px;\
            ">
                { EXAMPLE_REQUEST }
            </textarea>
            <textarea ref=self.writer.output_ref.clone() style="\
                width: 500px;\
                height: 600px;\
            "></textarea>
            <br />
            <button onclick=run_callback>{ "Run" }</button>
        </> }
    }
}

impl DotHttpClient {
    fn build_request(&mut self, offset: usize, all: bool) -> Result<RunningRequest> {
        let code = self
            .code_ref
            .cast::<HtmlTextAreaElement>()
            .map(|t| t.value())
            .unwrap();
        let file = dot_http_lib::parser::parse("wasm-example".into(), &code)?;

        let request_scripts = file
            .request_scripts(offset, all)
            .map(Clone::clone)
            .collect();

        Ok(RunningRequest {
            request_scripts,
            engine: BrowserScriptEngine::new(serde_json::Value::Object(Map::new()), "{}")?,
        })
    }

    fn run_request(&mut self, mut running_request: RunningRequest) -> Result<Option<FetchTask>> {
        if !running_request.request_scripts.is_empty() {
            let script = running_request.request_scripts.remove(0);

            let request = process(&mut running_request.engine, &script.request)?;
            self.outputter().request(&request)?;

            let request: http::Request<Text> =
                request.map(|body| body.ok_or_else(|| anyhow!("This signals an empty body")));

            let callback =
                self.link
                    .callback_once(move |response: http::Response<Result<String>>| {
                        let response = response.map(Result::ok);

                        Message::Response(script, response, running_request)
                    });

            Ok(Some(FetchService::fetch(request, callback)?))
        } else {
            Ok(None)
        }
    }

    fn handle_response(
        &mut self,
        request_script: RequestScript,
        response: Response,
        mut running_request: RunningRequest,
    ) -> Result<()> {
        self.outputter().response(&response)?;

        if let Some(parser::Handler { script, selection }) = &request_script.handler {
            running_request.engine.handle(
                &dot_http_lib::script_engine::Script {
                    selection: selection.clone(),
                    src: script.as_str(),
                },
                &response,
            )?;
        }

        running_request.engine.reset()?;

        self.link.send_message(Message::Run(running_request));

        Ok(())
    }

    fn outputter(&mut self) -> FormattedOutputter<'_, TextAreaWriter> {
        FormattedOutputter::new(
            &mut self.writer,
            vec![FormatItem::FirstLine, FormatItem::Chars("\n\n".to_string())],
            vec![
                FormatItem::FirstLine,
                FormatItem::Chars("\n".to_string()),
                FormatItem::Headers,
                FormatItem::Chars("\n".to_string()),
                FormatItem::Body,
                FormatItem::Chars("\n".to_string()),
            ],
        )
    }
}
