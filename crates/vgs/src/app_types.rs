/// UI language selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLang {
    Zh,
    En,
}

/// Filter for the provider list view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFilter {
    All,
    China,
    USA,
    Global,
}

/// Top-level navigation view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Task,
    Api,
    Network,
    Ollama,
    Resources,
}

/// A user-defined custom / relay API provider.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CustomProvider {
    /// Short identifier used as the `provider` field in `AiEntity`, e.g. "my-relay"
    pub id: String,
    /// Human-readable display name, e.g. "My DeepSeek Relay"
    pub name: String,
    /// Base URL of the relay, e.g. "https://relay.example.com/v1"
    pub base_url: String,
    /// HTTP header name (leave empty for default "Authorization")
    pub key_header: String,
    /// Token prefix (leave empty for default "Bearer")
    pub key_prefix: String,
    /// Space- or comma-separated model list suggestion
    pub models_hint: String,
    /// Optional description
    pub description: String,
}

/// A named AI entity: one API key + provider + model combination that can be
/// referenced by name throughout the swarm.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AiEntity {
    /// User-chosen name, e.g. "gpt4-coder", "deepseek-main"
    pub name: String,
    /// Provider id, e.g. "openai", "deepseek"  (or any label for a custom relay)
    pub provider: String,
    /// Model id, e.g. "gpt-4o", "deepseek-chat"
    pub model: String,
    /// Optional human note
    pub note: String,
    /// Custom base URL for relay / self-hosted / proxy providers.
    /// When set, this overrides the predefined provider endpoint.
    /// e.g. "https://my-relay.example.com/v1"
    #[serde(default)]
    pub custom_base_url: Option<String>,
    /// HTTP header name used to send the key (default: "Authorization").
    /// e.g. "api-key" for Azure, or "Authorization" for most others.
    #[serde(default)]
    pub key_header: Option<String>,
    /// Prefix placed before the key in the header (default: "Bearer ").
    /// Set to "" if the provider wants a bare key.
    #[serde(default)]
    pub key_prefix: Option<String>,
}
