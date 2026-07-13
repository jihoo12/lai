use super::{LlmBackend, Message, Role};
use serde::{Deserialize, Serialize};

pub struct LlamaCppBackend {
    url: String,
    model: String,
    agent: ureq::Agent,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

impl LlamaCppBackend {
    pub fn new(url: &str, model: &str) -> Self {
        Self {
            url: url.trim_end_matches('/').to_string(),
            model: model.to_string(),
            agent: ureq::Agent::new_with_defaults(),
        }
    }
}

impl LlmBackend for LlamaCppBackend {
    fn complete(&mut self, messages: &[Message]) -> Result<String, String> {
        let chat_messages: Vec<ChatMessage> = messages
            .iter()
            .map(|m| {
                let (role, content) = match m.role {
                    Role::System => ("system".to_string(), m.content.clone()),
                    Role::User => ("user".to_string(), m.content.clone()),
                    Role::Assistant => ("assistant".to_string(), m.content.clone()),
                    Role::Tool => (
                        "user".to_string(),
                        format!("[Tool Result]\n{}", m.content),
                    ),
                };
                ChatMessage { role, content }
            })
            .collect();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: chat_messages,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        };

        let url = format!("{}/v1/chat/completions", self.url);
        let mut resp = self
            .agent
            .post(&url)
            .header("Content-Type", "application/json")
            .send_json(&request)
            .map_err(|e| format!("request failed: {}", e))?;

        let body: ChatResponse = resp
            .body_mut()
            .read_json()
            .map_err(|e| format!("read body: {}", e))?;

        body.choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| "empty response".to_string())
    }
}
