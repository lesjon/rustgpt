mod models;

use std::collections::HashMap;
pub use models::*;

use std::error::Error;

fn get_request_with_pwsh_functions(messages: Messages) -> OpenAiRequest {
    OpenAiRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages,
        temperature: 0.9,
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

pub async fn print_models(openai_api_key: &str, client: &reqwest::Client) -> Result<(), Box<dyn Error>> {
    let res = client.get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .send().await?;
    println!("{:?}", res.text().await?);
    Ok(())
}

struct OpenAiResponseIter {
    buffer: Vec<u8>,
}

impl Iterator for OpenAiResponseIter {
    type Item = OpenAiResponse;


    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            return None;
        }
        let index = if let Some(newline_index) = self.buffer.iter().position(|&x| x == b'\n') {
            newline_index
        } else {
            self.buffer.len()
        };
        let line = self.buffer.drain(..=index).collect::<Vec<u8>>();
        self.buffer.remove(0);
        println!("line: {:?}", String::from_utf8(line.to_vec()));
        if line.is_empty() {
            return None;
        }
        if line.starts_with(b"data: [DONE]") {
            return None;
        }
        if let Ok(openai_resp) = serde_json::from_slice(&line[6..]) {
            Some(openai_resp)
        } else {
            eprintln!("could not parse: {:?}", String::from_utf8(line.to_vec()));
            None
        }
    }
}

pub async fn get_next(openai_api_key: &str, client: &reqwest::Client, mut history: Messages) -> Result<Messages, Box<dyn Error>> {
    let request = get_request_with_pwsh_functions(history.clone());
    let body_str = serde_json::to_string(&request)?;
    let mut response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .body(body_str)
        .send().await?;

    let mut new_msg = Message {
        content: String::new(),
        role: String::new(),
        function_call: None,
    };
    let mut rec_role = String::new();
    let mut rec_content = String::new();
    let mut function_name = String::new();
    let mut function_arguments = String::new();

    while let Some(chunk) = response.chunk().await.expect("Did not get new chunk after awaiting") {
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
        let args = serde_json::from_str::<HashMap<String, String>>(&function_arguments);
        let f_call = FunctionCall {
            name: function_name,
            arguments: args.expect(&format!("could not parse: {function_arguments:?}")),
        };
        new_msg.function_call = Some(f_call);
    }

    println!("resulting message: {new_msg:?}");

    history.push(new_msg);
    Ok(history)
}
