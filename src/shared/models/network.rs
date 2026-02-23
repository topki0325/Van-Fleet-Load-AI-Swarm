//! Peer discovery, Ollama offer, and build pipeline types.

use serde::{Deserialize, Serialize};
use super::core::{ClientMode, Project, TaskId};

// ─── Peer discovery ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatus {
    pub id: String,
    pub address: String,
    pub mode: ClientMode,
    pub latency: Option<u64>,

    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(default)]
    pub ollama: Option<OllamaOfferStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OllamaOfferStatus {
    pub enabled: bool,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,

    // Optional application-layer auth for calling the shared Ollama endpoint.
    // Discovery itself remains unauthenticated.
    #[serde(default)]
    pub auth_required: bool,
    #[serde(default)]
    pub proxy_port: Option<u16>,
}

// ─── Build pipeline ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BuildPlan {
    pub project: Project,
    pub segments: Vec<BuildSegment>,
}

#[derive(Debug, Clone)]
pub struct BuildSegment {
    pub language: String,
    pub files: Vec<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildUpdate {
    pub segment_id: String,
    pub status: BuildStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOutput {
    pub segment_id: String,
    pub binary_path: std::path::PathBuf,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Pending,
    InProgress,
    Success,
    Failed,
}

// ─── Conflict resolution ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub task_id: TaskId,
    pub conflicts: Vec<String>,
    pub resolution_options: Vec<String>,
}
