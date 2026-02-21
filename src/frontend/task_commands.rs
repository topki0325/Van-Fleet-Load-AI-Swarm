//! Tauri commands for task scheduling and agent management.

use crate::backend::BackendServices;
use crate::shared::models::*;
use tauri::State;
use std::sync::Arc;

#[tauri::command]
pub async fn cmd_get_swarm_status(
    state: State<'_, Arc<BackendServices>>,
) -> Result<SwarmPulse, String> {
    Ok(state.agent_scheduler.get_swarm_status().await)
}

#[tauri::command]
pub async fn cmd_get_all_agents(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<Agent>, String> {
    Ok(state.agent_scheduler.list_agents().await)
}

#[tauri::command]
pub async fn cmd_execute_task(
    task_spec: TaskSpec,
    state: State<'_, Arc<BackendServices>>,
) -> Result<TaskOutput, String> {
    state
        .agent_scheduler
        .execute_task_spec(task_spec)
        .await
        .map_err(|e| format!("Task execution failed: {:?}", e))
}

#[tauri::command]
pub async fn cmd_submit_task(
    spec: TaskSpec,
    priority: Priority,
    input_snapshot: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    let task = Task::new(spec, priority, std::path::PathBuf::from(input_snapshot));
    state
        .agent_scheduler
        .submit_task(task)
        .await
        .map(|task_id| task_id.to_string())
        .map_err(|e| format!("Task submission failed: {:?}", e))
}

#[tauri::command]
pub async fn cmd_get_task(
    task_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<Option<Task>, String> {
    let task_id = uuid::Uuid::parse_str(&task_id)
        .map_err(|e| format!("Invalid task ID: {}", e))?;
    Ok(state.agent_scheduler.get_task(task_id).await)
}

#[tauri::command]
pub async fn cmd_list_tasks(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<Task>, String> {
    Ok(state.agent_scheduler.list_tasks().await)
}

#[tauri::command]
pub async fn cmd_cancel_task(
    task_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    let task_id = uuid::Uuid::parse_str(&task_id)
        .map_err(|e| format!("Invalid task ID: {}", e))?;
    state
        .agent_scheduler
        .cancel_task(task_id)
        .await
        .map(|_| true)
        .map_err(|e| format!("Task cancellation failed: {:?}", e))
}
