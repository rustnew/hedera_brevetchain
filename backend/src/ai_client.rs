use reqwest;
use serde::{Serialize};
use std::time::Duration;
use crate::models::AiResponse;

#[derive(Serialize)]
struct AiRequest {
    raw_idea: String,
}

pub async fn call_ai_service(raw_idea: String) -> Result<AiResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut attempts = 0;
    loop {
        attempts += 1;
        match client
            .post("http://localhost:8000/ai/structure")
            .json(&AiRequest { raw_idea: raw_idea.clone() })
            .timeout(Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    return Ok(response.json::<AiResponse>().await?);
                } else {
                    eprintln!("AI response error: {}", response.status());
                }
            }
            Err(e) => eprintln!("AI request failed (attempt {}): {}", attempts, e),
        }
        if attempts >= 3 {
            return Err("AI service unreachable after 3 attempts".into());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}