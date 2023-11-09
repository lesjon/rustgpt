mod models;

pub mod config;

use std::collections::HashMap;
pub use models::*;

use std::error::Error;
use std::io;
use log;

const MODEL: &str = "gpt-3.5-turbo";

struct OpenAiResponseIter {
    buffer: Vec<u8>,
}

impl Iterator for OpenAiResponseIter {
    type Item = OpenAiResponse;

    fn next(&mut self) -> Option<Self::Item> {
        log::debug!("Getting next part from buffer:'{:?}'", String::from_utf8_lossy(self.buffer.as_ref()));
        if self.buffer.is_empty() {
            log::debug!("buffer is empty returning None");
            return None;
        }
        let index = if let Some(newline_index) = self.buffer.iter().position(|&x| x == b'\n') {
            newline_index
        } else {
            log::debug!("No more newlines in buffer, returning using full length of buffer");
            self.buffer.len()-1
        };
        let line = self.buffer.drain(..=index).collect::<Vec<u8>>();
        log::debug!("Clearing buffer '{:?}' as it's stored in line: {:?}", String::from_utf8_lossy(self.buffer.as_ref()), String::from_utf8_lossy(line.as_ref()));
        if !self.buffer.is_empty(){
            self.buffer.remove(0);
        }
        if line.is_empty() {
            log::debug!("No data in line, returning None");
            return None;
        }
        if line.starts_with(b"data: [DONE]") {
            log::debug!("End of data message");
            return None;
        }
        log::debug!("serde_json parsing '{}'", String::from_utf8_lossy(line.as_ref()));
        if let Ok(openai_resp) = serde_json::from_slice(&line[6..]) {
            Some(openai_resp)
        } else {
            log::error!("could not parse: {:?}", String::from_utf8(line.to_vec()));
            None
        }
    }
}

fn get_chat_request(messages: Messages) -> OpenAiRequest {
    OpenAiRequest {
        model: MODEL.to_string(),
        messages,
        temperature: 0.5,
        stream: Some(true),
        functions: None,
    }
}

fn get_request_with_powershell_functions(messages: Messages) -> OpenAiRequest {
    OpenAiRequest {
        model: MODEL.to_string(),
        messages,
        temperature: 0.1,
        stream: Some(true),
        functions: Some(vec![
            OpenaiFunction {
                name: "powershell".to_string(),
                description: "Call a powershell command".to_string(),
                parameters: FunctionParameters {
                    r#type: "object".to_string(),
                    properties: HashMap::from([("command".into(),
                                                FunctionProperty {
                                                    r#type: "string".into(),
                                                    description: Some("the powershell command".into()),
                                                    r#enum: vec![],
                                                })]),
                    required: vec!["command".into()],
                },
            },
            OpenaiFunction {
                name: "theme".to_string(),
                description: "Call a powershell command to change the windows theme".to_string(),
                parameters: FunctionParameters {
                    r#type: "object".to_string(),
                    properties: HashMap::from([("command".into(),
                                                FunctionProperty {
                                                    r#type: "string".into(),
                                                    description: Some("themeA for dark mode and C for light mode".into()),
                                                    r#enum: vec!["& \"C:\\Windows\\Resources\\Themes\\themeA.theme\"".into(),
                                                                 "& \"C:\\Windows\\Resources\\Themes\\themeC.theme\"".into()],
                                                })]),
                    required: vec!["command".into()],
                },
            },
        ]),
    }
}

pub async fn print_models(openai_api_key: &str, client: &reqwest::Client) -> Result<String, Box<dyn Error>> {
    let res = client.get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .send().await?;
    Ok(res.text().await?)
}

async fn get_next_from_request(openai_api_key: &str, client: &reqwest::Client, request: OpenAiRequest) -> Result<Message, Box<dyn Error>> {
    let body_str = serde_json::to_string(&request)?;
    log::debug!("GET https://api.openai.com/v1/chat/completions response with message: {body_str}");
    let mut response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .body(body_str)
        .send().await?;

    let mut new_msg = Message {
        content: String::new(),
        role: String::new(),
        function_call: None,
        name: None,
    };
    let mut rec_role = String::new();
    let mut rec_content = String::new();
    let mut function_name = String::new();
    let mut function_arguments = String::new();

    if response.status() != 200 {
        log::error!("Received Error '{}' from openai api: {:?}", response.status(), response.text().await?);
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "Error from api")))?;
    }
    while let Some(chunk) = response.chunk().await.expect("Did not get new chunk after awaiting") {
        log::debug!("Parsing chunk:'{}'", String::from_utf8_lossy(chunk.as_ref()));
        let openai_response_iter = OpenAiResponseIter {
            buffer: chunk.to_vec(),
        };
        for partial_response in openai_response_iter {
            for message in partial_response.choices {
                if let Some(delta) = message.delta {
                    if let Some(role) = delta.role {
                        rec_role = role.into();
                        print!("{}: ", rec_role);
                    }
                    if let Some(content) = delta.content {
                        rec_content.push_str(&content);
                        print!("{}", content);
                    }
                    if let Some(function_call) = delta.function_call {
                        if let Some(name) = function_call.name {
                            print!("{}", name);
                            function_name.push_str(&name);
                        }
                        if let Some(arguments) = function_call.arguments {
                            print!("{arguments}");
                            function_arguments.push_str(&arguments);
                        }
                    }
                }
            }
        }
    }
    new_msg.role = rec_role;
    new_msg.content = rec_content;
    if !function_name.is_empty() {
        let f_call = FunctionCall {
            name: function_name,
            arguments: function_arguments,
        };
        new_msg.function_call = Some(f_call);
    }
    Ok(new_msg)
}

pub async fn get_next(openai_api_key: &str, client: &reqwest::Client, mut history: Messages) -> Result<Messages, Box<dyn Error>> {
    let request = get_chat_request(history.clone());
    let new_msg = get_next_from_request(openai_api_key, client, request).await?;
    history.push(new_msg);
    Ok(history)
}

pub async fn get_next_powershell_command(openai_api_key: &str, client: &reqwest::Client, mut history: Messages) -> Result<Messages, Box<dyn Error>> {
    let request = get_request_with_powershell_functions(history.clone());
    let new_msg = get_next_from_request(openai_api_key, client, request).await?;
    history.push(new_msg);
    Ok(history)
}
