mod models;

pub use models::*;

use std::error::Error;

pub async fn print_models(openai_api_key: &str, client: &reqwest::Client) -> Result<(), Box<dyn Error>> {
    let res = client.get("https://api.openai.com/v1/models")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .send().await?;
    println!("{:?}", res.text().await?);
    Ok(())
}

pub async fn get_completion(openai_api_key: &str, client: &reqwest::Client, prompt: &str) -> Result<(), Box<dyn Error>> {
    let mut messages = Vec::new();
    messages.push(Message { content: prompt.to_string(), role: "user".to_string() });
    let request = OpenAiRequest {
        model: "gpt-4".to_string(),
        messages,
        temperature: 0.9,
        stream: Some(true),
    };
    println!("request: {:?}", request);
    let mut response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {openai_api_key}"))
        .body(serde_json::to_string(&request)?)
        .send().await?;

    while let Some(chunk) = response.chunk().await? {
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
            let openai_resp: OpenAiResponse = serde_json::from_str(json_object)?;
            for message in openai_resp.choices {
                if let Some(delta) = message.delta {
                    if let Some(role) = delta.role {
                        print!("{}: ", role);
                    }
                    if let Some(content) = delta.content {
                        print!("{}", content);
                    }
                }
            }
        }
    }
    Ok(())
}
