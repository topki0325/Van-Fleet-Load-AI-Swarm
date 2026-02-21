//! Tauri commands for API key vault and provider configuration.

use crate::backend::BackendServices;
use crate::shared::models::*;
use tauri::State;
use std::sync::Arc;

#[tauri::command]
pub async fn cmd_get_billing(
    provider: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<BillingReport, String> {
    let _ = &state;
    Ok(BillingReport {
        provider,
        total_tokens: 0,
        total_cost: 0.0,
        period: "month".to_string(),
    })
}

#[tauri::command]
pub async fn cmd_vault_op(
    op: VaultOp,
    state: State<'_, Arc<BackendServices>>,
) -> Result<VaultResult, String> {
    match state.api_manager.vault_operation(op) {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Vault operation failed: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_vault_store(
    provider: String,
    key: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    match state.api_manager.vault_operation(VaultOp::Store { provider, key }) {
        Ok(VaultResult::Success) => Ok(true),
        Ok(other) => Err(format!("Unexpected vault result: {:?}", other)),
        Err(e) => Err(format!("Vault store failed: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_vault_retrieve(
    provider: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    match state.api_manager.vault_operation(VaultOp::Retrieve { provider }) {
        Ok(VaultResult::Key(key)) => Ok(key),
        Ok(other) => Err(format!("Unexpected vault result: {:?}", other)),
        Err(e) => Err(format!("Vault retrieve failed: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_vault_list(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<String>, String> {
    match state.api_manager.vault_operation(VaultOp::List) {
        Ok(VaultResult::Providers(p)) => Ok(p),
        Ok(other) => Err(format!("Unexpected vault result: {:?}", other)),
        Err(e) => Err(format!("Vault list failed: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_vault_delete(
    provider: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    match state.api_manager.vault_operation(VaultOp::Delete { provider }) {
        Ok(VaultResult::Success) => Ok(true),
        Ok(other) => Err(format!("Unexpected vault result: {:?}", other)),
        Err(e) => Err(format!("Vault delete failed: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_vault_usage(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<VaultUsageEntry>, String> {
    Ok(state.api_manager.get_usage_entries().await)
}

#[tauri::command]
pub async fn cmd_get_providers(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<ProviderConfig>, String> {
    match state.api_manager.vault_operation(VaultOp::GetProviders) {
        Ok(VaultResult::ProviderConfigs(providers)) => Ok(providers),
        Ok(other) => Err(format!("Unexpected vault result: {:?}", other)),
        Err(e) => Err(format!("Failed to get providers: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_get_provider_config(
    provider: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<ProviderConfig, String> {
    match state.api_manager.vault_operation(VaultOp::GetProviderConfig { provider }) {
        Ok(VaultResult::ProviderConfig(config)) => Ok(config),
        Ok(other) => Err(format!("Unexpected vault result: {:?}", other)),
        Err(e) => Err(format!("Failed to get provider config: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_set_default_provider(
    provider: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    match state.api_manager.vault_operation(VaultOp::SetDefaultProvider { provider }) {
        Ok(VaultResult::DefaultProvider(provider)) => Ok(provider),
        Ok(other) => Err(format!("Unexpected vault result: {:?}", other)),
        Err(e) => Err(format!("Failed to set default provider: {:?}", e)),
    }
}
