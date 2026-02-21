//! Tauri commands for the local Ollama integration.

use crate::backend::BackendServices;
use crate::backend::ollama_client::{
    OllamaConnectionStatus, OllamaModel, OllamaModelInfo,
    ChatRequest, ChatResponse, GenerateRequest, GenerateResponse, OllamaUsageStats,
};
use tauri::State;
use std::sync::Arc;

#[tauri::command]
pub async fn cmd_ollama_check_connection(
    state: State<'_, Arc<BackendServices>>,
) -> Result<OllamaConnectionStatus, String> {
    Ok(state.ollama_manager.check_connection().await)
}

#[tauri::command]
pub async fn cmd_ollama_list_models(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<OllamaModel>, String> {
    state.ollama_manager.list_models().await
}

#[tauri::command]
pub async fn cmd_ollama_show_model_info(
    model_name: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<OllamaModelInfo, String> {
    state.ollama_manager.show_model_info(&model_name).await
}

#[tauri::command]
pub async fn cmd_ollama_pull_model(
    model_name: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    state.ollama_manager.pull_model(&model_name).await
}

#[tauri::command]
pub async fn cmd_ollama_delete_model(
    model_name: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    state.ollama_manager.delete_model(&model_name).await
}

#[tauri::command]
pub async fn cmd_ollama_chat(
    request: ChatRequest,
    state: State<'_, Arc<BackendServices>>,
) -> Result<ChatResponse, String> {
    state.ollama_manager.chat(request).await
}

#[tauri::command]
pub async fn cmd_ollama_chat_simple(
    model: String,
    prompt: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    state.ollama_manager.chat_simple(&model, &prompt).await
}

#[tauri::command]
pub async fn cmd_ollama_generate(
    request: GenerateRequest,
    state: State<'_, Arc<BackendServices>>,
) -> Result<GenerateResponse, String> {
    state.ollama_manager.generate(request).await
}

#[tauri::command]
pub async fn cmd_ollama_generate_simple(
    model: String,
    prompt: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    state.ollama_manager.generate_simple(&model, &prompt).await
}

#[tauri::command]
pub async fn cmd_ollama_embed(
    model: String,
    input: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<f32>, String> {
    state.ollama_manager.embed(&model, &input).await
}

#[tauri::command]
pub async fn cmd_ollama_get_version(
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    state.ollama_manager.get_version().await
}

#[tauri::command]
pub async fn cmd_ollama_get_usage_stats(
    state: State<'_, Arc<BackendServices>>,
) -> Result<OllamaUsageStats, String> {
    Ok(state.ollama_manager.get_usage_stats().await)
}

#[tauri::command]
pub async fn cmd_ollama_reset_usage_stats(
    state: State<'_, Arc<BackendServices>>,
) -> Result<(), String> {
    state.ollama_manager.reset_usage_stats().await;
    Ok(())
}
