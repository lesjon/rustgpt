use std::{env, process};
use std::error::Error;
use rustgpt::openai;
use rustgpt::openai::{ChatHistory, Messages};
use rustgpt::powershell;

async fn model() {
    let client = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let models_response = openai::print_models(&openai_api_key, &client);
    println!("{:?}", models_response.await);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let args = env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Not enough arguments in command {:?}", args);
        process::exit(1)
    }
    if args[1] == "models" {
        model().await;
        process::exit(0)
    }
    let input = args[1..].join(" ");

    let mut conversation = Messages::new();
    conversation.add_system_message("
        You are a machine translating human commands to powershell commands.\
        These powershell commands can be returned as function calls.\
        You can also ask the user for more information.\
        If the function could do something dangerous always ask the user if the command should be run."
    );
    conversation.add_user_message(&input);
    println!("{}", conversation);

    let httpclient = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let completion = openai::get_next(&openai_api_key, &httpclient, conversation);
    conversation = completion.await?;
    if let Some(msg) = conversation.last() {
        if let Some(function_call) = &msg.function_call {
            if let Some(cmd) = function_call.arguments.get("command") {
                let output = powershell::run_command(cmd);
                conversation.add_powershell_message(&output);
            }
        }
    }
    Ok(())
}