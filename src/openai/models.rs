use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Messages(pub Vec<Message>);

pub trait ChatHistory {
    fn last(&self) -> Option<&Message>;
    fn push(&mut self, msg: Message);
    fn add_user_message(&mut self, msg: &str);
    fn set_system_message(&mut self, msg: &str);
    fn add_message(&mut self, role: &str, msg: &str);

    fn add_powershell_message(&mut self, msg: &str);

    fn from(openai_message: Message) -> Messages {
        Messages(vec![openai_message])
    }

    fn new() -> Messages {
        Messages(vec![])
    }

    fn clear_system_messages(&mut self);
    fn get_system_messages(&self) -> Vec<&Message>;
}

impl Display for Messages {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for msg in &self.0 {
            if msg.role == "system" {
                continue;
            }
            write!(f, "{}\n", *msg)?;
        }
        Ok(())
    }
}


impl ChatHistory for Messages {
    fn last(&self) -> Option<&Message> {
        self.0.last()
    }

    fn push(&mut self, msg: Message) {
        self.0.push(msg)
    }
    fn add_user_message(&mut self, msg: &str) {
        self.add_message("user", msg)
    }

    fn set_system_message(&mut self, msg: &str) {
        self.clear_system_messages();
        let openai_msg = Message {
            role: "system".to_string(),
            content: msg.to_string(),
            function_call: None,
        };
        self.0.insert(0, openai_msg)
    }

    fn add_message(&mut self, role: &str, msg: &str) {
        let openai_msg = Message {
            role: role.to_string(),
            content: msg.to_string(),
            function_call: None,
        };
        self.0.push(openai_msg);
    }

    fn add_powershell_message(&mut self, msg: &str) {
        self.add_message("function", msg)
    }

    fn clear_system_messages(&mut self) {
        self.0.retain(|msg| msg.role != "system");
    }

    fn get_system_messages(&self) -> Vec<&Message>{
         self.0.iter().filter(|msg| msg.role == "system").collect()
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub content: String,
    pub role: String,
    #[serde(skip_serializing)]
    pub function_call: Option<FunctionCall>,
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.role, self.content)?;
        if let Some(call) = &self.function_call {
            write!(f, "{}(", call.name)?;
            for entry in &call.arguments {
                write!(f, "{}:{}", entry.0, entry.1)?;
            }
            write!(f, ")")?;
        }
        write!(f, "\n")?;
        Ok(())
    }
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
    pub required: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionProperty {
    pub r#type: String,
    pub description: Option<String>,
    pub r#enum: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiRequest {
    pub model: String,
    pub messages: Messages,
    pub temperature: f32,
    pub stream: Option<bool>,
    pub functions: Option<Vec<OpenaiFunction>>,
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
    #[serde(flatten)]
    pub arguments: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamingFunctionCall {
    pub name: Option<String>,
    pub arguments: Option<String>,
}
