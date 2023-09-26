use std::env;
use std::error::Error;
use crate::openai::{ChatHistory, Messages};

mod openai;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    let input = args[1..].join(" ");
    println!("request is '{}'", input);
    let client = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();

    let mut conversation = Messages::new();
    conversation.add_system_message("
        You are a machine translating human commands to powershell commands.\
        These powershell commands should be returned as function calls.\
        If the function could do something dangerous ask the user if the command should be run."
    );
    conversation.add_user_message(&input);
    let completion = openai::get_next(&openai_api_key, &client, conversation);
    conversation = completion.await?;
    println!("resulting conversation:\n{conversation:?}");
    Ok(())
}