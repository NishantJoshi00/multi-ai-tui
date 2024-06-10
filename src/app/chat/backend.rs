use serde::{Deserialize, Serialize};
use std::sync::mpsc;

use crate::logging::footstones::*;

#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}
// {"model":"llama3","created_at":"2024-06-09T15:54:01.34414989Z","message":{"role":"assistant","content":"?"},"done":false}
// {"model":"llama3","created_at":"2024-06-09T15:54:01.426999551Z","message":{"role":"assistant","content":""},"done_reason":"stop","done":true,"total_duration":4600646509,"load_duration":2069650368,"prompt_eval_count":10,"prompt_eval_duration":326180000,"eval_count":26,"eval_duration":2072971000}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub message: Message,
    pub done: bool,
}

pub async fn handle_streaming_request((tx, req): (mpsc::Sender<ChatResponse>, ChatRequest)) {
    info!("Sending chat request: {:?}", req);
    let mut response = reqwest::Client::new()
        .post("http://localhost:11434/api/chat")
        .json(&req)
        .send()
        .await
        .unwrap();

    while let Some(chunk) = response.chunk().await.unwrap() {
        let chat_response: ChatResponse = serde_json::from_slice(&chunk).unwrap();
        match tx.send(chat_response) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to send chat response: {:?}", e);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_request() {
        let chat_request = ChatRequest {
            model: "llama3".to_string(),
            messages: vec![Message {
                role: Role::User,
                content: "Hello".to_string(),
            }],
        };

        let mut response = reqwest::Client::new()
            .post("http://localhost:11434/api/chat")
            .json(&chat_request)
            .send()
            .await
            .unwrap();

        while let Some(chunk) = response.chunk().await.unwrap() {
            let chat_response: ChatResponse = serde_json::from_slice(&chunk).unwrap();
        }
    }
}
