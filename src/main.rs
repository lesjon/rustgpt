use std::{env, io};
use std::error::Error;
use std::io::Read;
use rustgpt::openai;
use rustgpt::openai::ChatHistory;
use rustgpt::openai::config::Settings;
use rustgpt::powershell;

async fn models() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let models_response = openai::print_models(&openai_api_key, &client);
    println!("{:?}", models_response.await);
    Ok(())
}

async fn chat<'a>(settings: &Settings, args: &[&str]) -> Result<(), Box<dyn Error>> {
    let input = args.join(" ");

    let mut conversation = settings.get_history()?;
    conversation.set_system_message("");
    conversation.add_user_message(&input);
    print!("{}", conversation);

    let httpclient = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let completion = openai::get_next_powershell_command(&openai_api_key, &httpclient, conversation);
    conversation = completion.await?;
    settings.write_history(conversation)
}

async fn pwsh<'a>(settings: &Settings, args: &[&str]) -> Result<(), Box<dyn Error>> {
    let input = args.join(" ");

    let mut conversation = settings.get_history()?;
    conversation.set_system_message(" You are a machine translating human commands to powershell commands.\
        These powershell commands can be returned as function calls.\
        You can also ask the user for more information.\
        If the function could do something dangerous always ask the user if the command should be run."
    );
    conversation.add_user_message(&input);
    println!("{}", conversation);

    let httpclient = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();
    let completion = openai::get_next_powershell_command(&openai_api_key, &httpclient, conversation);
    conversation = completion.await?;
    if let Some(msg) = conversation.last() {
        if let Some(function_call) = &msg.function_call {
            if let Some(cmd) = function_call.arguments.get("command") {
                let output = powershell::run_command(cmd);
                conversation.add_powershell_message(&output);
            }
        }
    }
    let pwsh_response = conversation.last().unwrap();
    print!("{}", pwsh_response);
    settings.write_history(conversation)
}

async fn print_conversation<'a>(settings: &Settings, args: &[&str]) -> Result<(), Box<dyn Error>> {
    let conversation = settings.get_history()?;
    if args.contains(&"--system") {
        for msg in conversation.get_system_messages() {
            println!("{}", msg);
        }
    } else {
        print!("{}", conversation);
    }
    Ok(())
}

async fn add_file_from_stdin(file_name: &str, settings: &Settings) -> Result<(), Box<dyn Error>> {
    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents)?;
    let mut file_name_line = ":\n".to_string();
    file_name_line.insert_str(0, file_name);
    contents.insert_str(0, &file_name_line);
    let mut conversation = settings.get_history()?;
    conversation.add_user_message(&contents);
    Ok(settings.write_history(conversation)?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = env::args().collect::<Vec<String>>();
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let settings = Settings::new();
    return match args.as_slice() {
        &[] => { panic!("can not call program without any args!") }
        &[_] => { Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "Invalid number of arguments")))? }
        &[_, command, ..] => {
            return match command {
                "models" => {
                    models().await
                }
                "pwsh" => {
                    pwsh(&settings, &args[2..]).await
                }
                "chat" => {
                    chat(&settings, &args[2..]).await
                }
                "print" => {
                    print_conversation(&settings, &args[2..]).await
                }
                "clear" => {
                    settings.clear_history()
                }
                "file" => {
                    let filename = match args.get(2) {
                        Some(filename) => filename,
                        None => "file"
                    };
                    add_file_from_stdin(filename, &settings).await
                }
                _ => {
                    Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, format!("Unknown command {}", args[1]))))?
                }
            };
        }
    };
}


