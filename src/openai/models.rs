use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub(crate) content: String,
    pub(crate) role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chat {
    messages: Vec<Message>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<Message>,
    pub(crate) temperature: f32,
    pub(crate) stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    pub(crate) choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    message: Option<Message>,
    pub(crate) delta: Option<Delta>,
    finish_reason: Option<String>,
    index: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub(crate) content: Option<String>,
    pub(crate) role: Option<String>,
    function_call: Option<FunctionCall>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCall {
    name: String,
    arguments: Vec<String>,
}
