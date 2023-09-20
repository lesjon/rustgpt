use std::env;
use std::error::Error;

mod openai;

use crate::openai::get_completion;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    let input = args[1..].join(" ");
    println!("request is '{}'", input);
    let client = reqwest::Client::new();
    let openai_api_key = env::var("OPENAI_API_KEY").unwrap();

    let completion = get_completion(&openai_api_key, &client, &input);

    completion.await?;
    Ok(())
}