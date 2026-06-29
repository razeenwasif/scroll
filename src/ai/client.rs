//! Ollama API HTTP client.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::config::AiConfig;

/// A client for communicating with a local Ollama service.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    model: String,
    client: Client,
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    stream: bool,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

impl OllamaClient {
    /// Creates a new Ollama client using the provided AI config.
    pub fn new(config: &AiConfig) -> Self {
        Self {
            base_url: config.ollama_url.trim_end_matches('/').to_string(),
            model: config.model.clone(),
            client: Client::new(),
        }
    }

    /// Sends a prompt to the Ollama model and returns the completed text.
    pub async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);
        let request_body = GenerateRequest {
            model: &self.model,
            prompt,
            system,
            stream: false,
        };

        let response = self.client.post(&url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let err_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Ollama returned error code {}: {}",
                status,
                err_text
            ));
        }

        let resp_json: GenerateResponse = response.json()
            .await
            .context("Failed to parse Ollama JSON response")?;

        Ok(resp_json.response)
    }

    /// Checks if the Ollama service is running and accessible.
    #[allow(dead_code)]
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}
