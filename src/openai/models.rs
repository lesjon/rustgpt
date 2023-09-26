use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub type Messages = Vec<Message>;

pub trait ChatHistory {
    fn add_user_message(&mut self, msg: &str);
    fn add_system_message(&mut self, msg: &str);
    fn add_message(&mut self, role: &str, msg: &str);

    fn from(openai_message: Message) -> Messages {
        vec![openai_message]
    }

    fn new() -> Messages {
        vec![]
    }
}

impl ChatHistory for Messages {
    fn add_user_message(&mut self, msg: &str) {
        self.add_message("user", msg)
    }

    fn add_system_message(&mut self, msg: &str) {
        self.add_message("system", msg)
    }

    fn add_message(&mut self, role: &str, msg: &str) {
        let openai_msg = Message {
            role: role.to_string(),
            content: msg.to_string(),
            function_call: None,
        };
        self.push(openai_msg);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub content: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenaiFunction {
    pub name: String,
    pub description: String,
    pub parameters: FunctionParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionParameters {
    pub r#type: String,
    pub properties: HashMap<String, FunctionProperty>,
    pub required: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionProperty {
    pub r#type: String,
    pub description: Option<String>,
    pub r#enum: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Messages,
    pub temperature: f32,
    pub stream: Option<bool>,
    pub functions: Option<Vec<OpenaiFunction>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub(crate) message: Option<Message>,
    pub delta: Option<Delta>,
    pub(crate) finish_reason: Option<String>,
    pub(crate) index: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
    pub function_call: Option<StreamingFunctionCall>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: HashMap<String, String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StreamingFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>
}
