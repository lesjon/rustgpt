use std::env;
use std::error::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    content: String,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Chat {
    messages: Vec<Message>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: Option<Message>,
    delta: Option<Delta>,
    finish_reason: Option<String>,
    index: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Delta {
    content: Option<String>,
    role: Option<String>,
    function_call: Option<FunctionCall>,
    index: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionCall {
    name: String,
    arguments: Vec<String>,
}

async fn print_models(openai_api_key: &str, client: &reqwest::Client) -> Result<(), Box<dyn Error>> {
    let res = client.get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .send().await?;
    println!("{:?}", res.text().await?);
    Ok(())
}

async fn get_completion(openai_api_key: &str, client: &reqwest::Client, prompt: &str) -> Result<(), Box<dyn Error>> {
    let mut messages = Vec::new();
    messages.push(Message { content: prompt.to_string(), role: "user".to_string() });
    let request = OpenAiRequest {
        model: "gpt-4".to_string(),
        messages: messages,
        temperature: 0.9,
        stream: Option::Some(false),
    };
    println!("request: {:?}", request);
    let mut response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .body(serde_json::to_string(&request)?)
        .send().await?;
    println!("response: {:?}", response);

    while let Some(chunk) = response.chunk().await? {
        println!("chunk: {:?}", chunk);
        let response: OpenAiResponse = serde_json::from_slice(&chunk)?;
        if let Some(choice) = response.choices.first() {
            println!("choice: {:?}", choice);
            if let Some(delta) = &choice.delta {
                println!("delta: {:?}", delta);
                if let Some(content) = &delta.content {
                    println!("{}", content);
                }
            }
            if let Some(message) = &choice.message {
                println!("message: {:?}", message);
                println!("message.content: {:?}", message.content);
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    let input = args[1..].join(" ");
    println!("request is '{}'", input);
    let client = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    // let models_complete = print_models(&openai_api_key, &client);

    let completion = get_completion(&openai_api_key, &client, &input);


    completion.await?;
    // models_complete.await?;
    Ok(())
}
