use std::env;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::coach::CoachAdvisor;
use crate::content::models::Step;

pub struct LlmCoach {
    client: Client,
    api_key: String,
    endpoint: String,
    model: String,
}

impl LlmCoach {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("CKA_COACH_LLM_API_KEY").context("missing CKA_COACH_LLM_API_KEY")?;
        let endpoint = env::var("CKA_COACH_LLM_ENDPOINT")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1/chat/completions".to_string());
        let model =
            env::var("CKA_COACH_LLM_MODEL").unwrap_or_else(|_| "openai/gpt-4.1-mini".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            endpoint,
            model,
        })
    }
}

impl CoachAdvisor for LlmCoach {
    fn hint(&self, step: &Step, recent_output: &[String]) -> String {
        let recent = recent_output
            .iter()
            .rev()
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are a concise CKA pair-coach. Keep response under 45 words.\nStep: {}\nObjective: {}\nCommands: {}\nRecent output:\n{}\nGive one short hint and one next command.",
            step.title,
            step.objective,
            step.run_commands.join("; "),
            recent
        );

        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            temperature: 0.2,
        };

        let resp = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send();

        let Ok(resp) = resp else {
            return "Hint unavailable: LLM request failed. Use deterministic verify flow."
                .to_string();
        };

        let parsed = resp.json::<ChatResponse>();
        let Ok(parsed) = parsed else {
            return "Hint unavailable: LLM response parse failed. Use deterministic verify flow."
                .to_string();
        };

        parsed
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Hint unavailable: empty LLM response.".to_string())
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}
