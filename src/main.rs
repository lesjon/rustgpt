use std::env;
use std::error::Error;
use crate::openai::{ChatHistory, Messages};

mod pwsh;
mod openai;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    let input = args[1..].join(" ");
    println!("request is '{}'", input);
    let client = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let mut powershell_instance = pwsh::get_instance()?;

    let mut conversation = Messages::new();
    conversation.add_system_message("
        You are a machine translating human commands to powershell commands.\
        These powershell commands should be returned as function calls.\
        If the function could do something dangerous ask the user if the command should be run."
    );
    conversation.add_user_message(&input);
    let completion = openai::get_next(&openai_api_key, &client, conversation);
    conversation = completion.await?;
    if let Some(msg) = conversation.last() {
        if let Some(function_call) = &msg.function_call {
            if let Some(cmd) = function_call.arguments.get("command") {
                let output = pwsh::run_command(&mut powershell_instance, cmd);
                let x = conversation.add_pwsh_message(&output);
            }
        }
    }
    println!("resulting conversation:\n{conversation:?}");
    Ok(())
}