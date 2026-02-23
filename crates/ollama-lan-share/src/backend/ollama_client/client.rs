//! Low-level HTTP client for the Ollama REST API.

use reqwest::Client;
use std::time::Duration;
use super::types::*;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    pub(super) base_url: String,
    pub(super) client: Client,
    pub(super) connection_timeout: Duration,
}

impl OllamaClient {
    pub fn new(base_url: Option<String>) -> Self {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url,
            client,
            connection_timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    pub async fn check_connection(&self) -> OllamaConnectionStatus {
        let url = format!("{}/api/version", self.base_url);

        match tokio::time::timeout(self.connection_timeout, self.client.get(&url).send()).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    match response.json::<VersionResponse>().await {
                        Ok(version) => OllamaConnectionStatus {
                            is_connected: true,
                            version: Some(version.version),
                            error: None,
                        },
                        Err(e) => OllamaConnectionStatus {
                            is_connected: true,
                            version: None,
                            error: Some(format!("Failed to parse version: {}", e)),
                        },
                    }
                } else {
                    OllamaConnectionStatus {
                        is_connected: false,
                        version: None,
                        error: Some(format!("HTTP error: {}", response.status())),
                    }
                }
            }
            Ok(Err(e)) => OllamaConnectionStatus {
                is_connected: false,
                version: None,
                error: Some(format!("Connection error: {}", e)),
            },
            Err(_) => OllamaConnectionStatus {
                is_connected: false,
                version: None,
                error: Some("Connection timeout".to_string()),
            },
        }
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to list models: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(models_response.models)
    }

    pub async fn show_model_info(&self, model_name: &str) -> Result<OllamaModelInfo, String> {
        let url = format!("{}/api/show", self.base_url);
        let body = serde_json::json!({ "name": model_name });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to show model info: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let model_info: OllamaModelInfo = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(model_info)
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<String, String> {
        let url = format!("{}/api/pull", self.base_url);
        let request = PullRequest { name: model_name.to_string(), stream: Some(false) };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to pull model: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        Ok(format!("Model '{}' pulled successfully", model_name))
    }

    pub async fn delete_model(&self, model_name: &str) -> Result<String, String> {
        let url = format!("{}/api/delete", self.base_url);
        let request = DeleteRequest { name: model_name.to_string() };

        let response = self
            .client
            .delete(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to delete model: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        Ok(format!("Model '{}' deleted successfully", model_name))
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        let url = format!("{}/api/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send chat request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP error {}: {}", status, error_text));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(chat_response)
    }

    pub async fn chat_simple(&self, model: &str, prompt: &str) -> Result<String, String> {
        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                images: None,
            }],
            stream: Some(false),
            format: None,
            options: None,
        };
        let response = self.chat(request).await?;
        Ok(response.message.content)
    }

    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, String> {
        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send generate request: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("HTTP error {}: {}", status, error_text));
        }

        let generate_response: GenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(generate_response)
    }

    pub async fn generate_simple(&self, model: &str, prompt: &str) -> Result<String, String> {
        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: Some(false),
            format: None,
            options: None,
        };
        let response = self.generate(request).await?;
        Ok(response.response)
    }

    pub async fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, String> {
        let url = format!("{}/api/embed", self.base_url);
        let request = EmbedRequest { model: model.to_string(), input: input.to_string() };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send embed request: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let embed_response: EmbedResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(embed_response.embedding)
    }

    pub async fn get_version(&self) -> Result<String, String> {
        let url = format!("{}/api/version", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to get version: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let version_response: VersionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(version_response.version)
    }
}
