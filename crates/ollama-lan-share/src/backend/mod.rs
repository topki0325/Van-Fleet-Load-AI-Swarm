//! Backend services for ollama-lan-share.

pub mod network_discovery;
pub mod ollama_client;

pub use network_discovery::NetworkDiscovery;
pub use ollama_client::manager::OllamaManager;
pub use ollama_client::types::OllamaModel;