//! API key vault and provider configuration types.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// ─── Vault operations ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultOp {
    Store { provider: String, key: String },
    Retrieve { provider: String },
    Delete { provider: String },
    List,
    GetProviders,
    GetProviderConfig { provider: String },
    SetDefaultProvider { provider: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultResult {
    Success,
    Key(String),
    Providers(Vec<String>),
    ProviderConfigs(Vec<ProviderConfig>),
    ProviderConfig(ProviderConfig),
    DefaultProvider(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultUsageEntry {
    pub provider: String,
    pub requests_made: u64,
    pub last_used: DateTime<Utc>,
}

// ─── Provider configuration ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub region: ProviderRegion,
    pub api_endpoint: String,
    pub models: Vec<String>,
    pub pricing: PricingInfo,
    pub requires_api_key: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderRegion {
    China,
    USA,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub currency: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub free_tier_limit: Option<u64>,
}

// ─── Billing ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingReport {
    pub provider: String,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub period: String,
}
