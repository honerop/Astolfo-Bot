use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn request_ollama(prompt: String) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    println!("started ollama ");

    println!("send to ollama {}",prompt);
    let request_body = OllamaRequest {
        model: "deepseek-r1:latest".into(),
        prompt,
        stream: false,
    };

    let res = client
        .post("http://localhost:11434/api/generate")
        .json(&request_body)
        .send()
        .await?;

    println!("got from  ollama ");
    let json: OllamaResponse = res.json().await?;
    println!("got from  ollama {}",json.response);

    Ok(json.response)
}
