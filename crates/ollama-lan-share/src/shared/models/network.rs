//! Peer discovery and Ollama offer types.

use serde::{Deserialize, Serialize};
use super::core::ClientMode;

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
