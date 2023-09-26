mod models;

use std::collections::HashMap;
pub use models::*;

use std::error::Error;

pub async fn print_models(openai_api_key: &str, client: &reqwest::Client) -> Result<(), Box<dyn Error>> {
    let res = client.get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .send().await?;
    println!("{:?}", res.text().await?);
    Ok(())
}

pub async fn get_next(openai_api_key: &str, client: &reqwest::Client, mut history: Messages) -> Result<Messages, Box<dyn Error>> {
    let request = OpenAiRequest {
        model: "gpt-4".to_string(),
        messages: history.clone(),
        temperature: 0.9,
        stream: Some(true),
        functions: Some(vec![
            OpenaiFunction {
                name: "pwsh".to_string(),
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
                                                    description: Some("themeA for light mode and C for dark mode".into()),
                                                    r#enum: vec!["& \"C:\\Windows\\Resources\\Themes\\themeA.theme\"".into(),
                                                                 "& \"C:\\Windows\\Resources\\Themes\\themeC.theme\"".into()],
                                                })]),
                    required: vec!["command".into()],
                },
            },

        ]),
    };
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
        let lines = chunk.split(|c| *c == b'\n');
        for line in lines {
            let line_str = std::str::from_utf8(line).expect("invalid utf8");
            if line_str.is_empty() {
                continue;
            }
            let json_object;
            if line_str.starts_with("data:") {
                json_object = &line_str[6..];
            } else {
                json_object = &line_str;
            }
            if json_object.eq("[DONE]") {
                break;
            }
            let openai_resp: OpenAiResponse = serde_json::from_str(json_object).expect(&format!("could not parse: {json_object:?}"));

            for message in openai_resp.choices {
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
