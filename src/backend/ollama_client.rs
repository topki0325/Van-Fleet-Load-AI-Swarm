//! Ollama integration split into focused sub-modules.
//!
//! - `types`: API request/response types
//! - `client`: Low-level HTTP client (OllamaClient)
//! - `manager`: High-level manager with usage stats (OllamaManager)

pub mod types;
pub mod client;
pub mod manager;

pub use types::*;
pub use client::OllamaClient;
pub use manager::OllamaManager;
