//! High-level manager wrapping OllamaClient with usage tracking.

use std::sync::Arc;
use tokio::sync::RwLock;
use super::types::*;
use super::client::OllamaClient;

#[derive(Clone)]
pub struct OllamaManager {
    client: Arc<RwLock<OllamaClient>>,
    usage_stats: Arc<RwLock<OllamaUsageStats>>,
}

impl OllamaManager {
    pub async fn new(base_url: Option<String>) -> Self {
        Self {
            client: Arc::new(RwLock::new(OllamaClient::new(base_url))),
            usage_stats: Arc::new(RwLock::new(OllamaUsageStats::default())),
        }
    }

    pub async fn check_connection(&self) -> OllamaConnectionStatus {
        self.client.read().await.check_connection().await
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        self.client.read().await.list_models().await
    }

    pub async fn show_model_info(&self, model_name: &str) -> Result<OllamaModelInfo, String> {
        self.client.read().await.show_model_info(model_name).await
    }

    pub async fn pull_model(&self, model_name: &str) -> Result<String, String> {
        let result = self.client.read().await.pull_model(model_name).await;
        if result.is_ok() {
            self.usage_stats.write().await.total_requests += 1;
        }
        result
    }

    pub async fn delete_model(&self, model_name: &str) -> Result<String, String> {
        self.client.read().await.delete_model(model_name).await
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        let model = request.model.clone();
        let response = self.client.read().await.chat(request).await?;

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
        self.client.read().await.chat_simple(model, prompt).await
    }

    pub async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse, String> {
        let model = request.model.clone();
        let response = self.client.read().await.generate(request).await?;

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
        self.client.read().await.generate_simple(model, prompt).await
    }

    pub async fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>, String> {
        self.client.read().await.embed(model, input).await
    }

    pub async fn get_version(&self) -> Result<String, String> {
        self.client.read().await.get_version().await
    }

    pub async fn get_usage_stats(&self) -> OllamaUsageStats {
        self.usage_stats.read().await.clone()
    }

    pub async fn reset_usage_stats(&self) {
        *self.usage_stats.write().await = OllamaUsageStats::default();
    }
}
