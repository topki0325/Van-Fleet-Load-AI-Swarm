use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    client: Client,
    connection_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: Option<ModelDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Option<Vec<String>>,
    pub parameter_size: String,
    pub quantization_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelInfo {
    pub license: String,
    pub modelfile: String,
    pub parameters: String,
    pub template: String,
    pub details: ModelDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct OllamaConnectionStatus {
    pub is_connected: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OllamaUsageStats {
    pub total_requests: u64,
    pub total_tokens: u64,
    pub total_duration_ms: u64,
    pub model_stats: HashMap<String, ModelUsageStats>,
}

#[derive(Debug, Clone, Default)]
pub struct ModelUsageStats {
    pub requests: u64,
    pub tokens: u64,
    pub duration_ms: u64,
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
        
        let request = PullRequest {
            name: model_name.to_string(),
            stream: Some(false),
        };

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
        
        let request = DeleteRequest {
            name: model_name.to_string(),
        };

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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
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
        
        let request = EmbedRequest {
            model: model.to_string(),
            input: input.to_string(),
        };

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

#[derive(Clone)]
pub struct OllamaManager {
    client: Arc<RwLock<OllamaClient>>,
    usage_stats: Arc<RwLock<OllamaUsageStats>>,
}

impl OllamaManager {
    pub async fn new(base_url: Option<String>) -> Self {
        let client = OllamaClient::new(base_url);
        
        Self {
            client: Arc::new(RwLock::new(client)),
            usage_stats: Arc::new(RwLock::new(OllamaUsageStats::default())),
        }
    }

    pub async fn check_connection(&self) -> OllamaConnectionStatus {
        let client = self.client.read().await;
        client.check_connection().await
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        let client = self.client.read().await;
        client.list_models().await
    }

    pub async fn show_model_info(&self, model_name: &str) -> Result<OllamaModelInfo, String> {
        let client = self.client.read().await;
        client.show_model_info(model_name).await
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<String, String> {
        let client = self.client.read().await;
        let result = client.pull_model(model_name).await;
        
        if result.is_ok() {
            let mut stats = self.usage_stats.write().await;
            stats.total_requests += 1;
        }
        
        result
    }

    pub async fn delete_model(&self, model_name: &str) -> Result<String, String> {
        let client = self.client.read().await;
        client.delete_model(model_name).await
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        let client = self.client.read().await;
        let model = request.model.clone();
        let response = client.chat(request).await?;
        
        let mut stats = self.usage_stats.write().await;
        stats.total_requests += 1;
        
        if let Some(eval_count) = response.eval_count {
            stats.total_tokens += eval_count as u64;
            
            let model_stats = stats.model_stats.entry(model).or_default();
            model_stats.requests += 1;
            model_stats.tokens += eval_count as u64;
        }
        
        if let Some(duration) = response.eval_duration {
            stats.total_duration_ms += duration / 1_000_000;
        }
        
        Ok(response)
    }

    pub async fn chat_simple(&self, model: &str, prompt: &str) -> Result<String, String> {
        let client = self.client.read().await;
        client.chat_simple(model, prompt).await
    }

    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, String> {
        let client = self.client.read().await;
        let model = request.model.clone();
        let response = client.generate(request).await?;
        
        let mut stats = self.usage_stats.write().await;
        stats.total_requests += 1;
        
        if let Some(eval_count) = response.eval_count {
            stats.total_tokens += eval_count as u64;
            
            let model_stats = stats.model_stats.entry(model).or_default();
            model_stats.requests += 1;
            model_stats.tokens += eval_count as u64;
        }
        
        if let Some(duration) = response.eval_duration {
            stats.total_duration_ms += duration / 1_000_000;
        }
        
        Ok(response)
    }

    pub async fn generate_simple(&self, model: &str, prompt: &str) -> Result<String, String> {
        let client = self.client.read().await;
        client.generate_simple(model, prompt).await
    }

    pub async fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, String> {
        let client = self.client.read().await;
        client.embed(model, input).await
    }

    pub async fn get_version(&self) -> Result<String, String> {
        let client = self.client.read().await;
        client.get_version().await
    }

    pub async fn get_usage_stats(&self) -> OllamaUsageStats {
        self.usage_stats.read().await.clone()
    }

    pub async fn reset_usage_stats(&self) {
        let mut stats = self.usage_stats.write().await;
        *stats = OllamaUsageStats::default();
    }
}
