//! Core types needed for ollama-lan-share

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ─── Client Mode ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMode {
    Master,
    Worker,
}

// ─── Error ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, thiserror::Error)]
pub enum VgaError {
    #[error("Resource limit: {0}")]
    ResourceLimit(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Compile failure: {0}")]
    CompileFailure(String),
    #[error("IO error: {0}")]
    Io(String),
}

// ─── Context Manager (simplified) ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ContextManager {
    pub memory_slots: std::collections::HashMap<String, String>,
    pub docs: Vec<String>,
}

// ─── Agent Trait (simplified) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub language: String,
    pub target: String,
    pub context_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskOutput {
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[async_trait::async_trait]
pub trait AgentTrait {
    async fn execute_instruction(&self, instr: String) -> Result<TaskOutput, VgaError>;
    fn update_context(&mut self, context: &ContextManager);
    fn get_metrics(&self) -> PerfMetrics;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub avg_response_time: std::time::Duration,
}
