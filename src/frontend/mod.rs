use crate::backend::BackendServices;
use crate::shared::models::*;
use tauri::State;
use std::sync::Arc;
use uuid;

pub mod app;
pub mod project_view;
pub mod agent_monitor;
pub mod client_gui;

#[tauri::command]
pub async fn cmd_get_billing(
    provider: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<BillingReport, String> {
    // TODO: Implement billing retrieval
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
    match state
        .api_manager
        .vault_operation(VaultOp::Store { provider, key })
    {
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
    match state
        .api_manager
        .vault_operation(VaultOp::Retrieve { provider })
    {
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
pub async fn cmd_deploy_project(
    config: ProjectConfig,
    state: State<'_, Arc<BackendServices>>,
) -> Result<ProjectResult, String> {
    let project_id = uuid::Uuid::new_v4();
    let project = Project {
        id: project_id,
        name: format!("project-{}", project_id),
        config,
        agents: Vec::new(),
        workflow: WorkflowGraph::default(),
        state: ProjectStatus::Initialized,
        stats: ExecutionStats {
            total_tokens: 0,
            total_duration: std::time::Duration::from_secs(0),
            total_cost: 0.0,
        },
        last_updated: chrono::Utc::now(),
    };

    state.projects.write().await.push(project);

    Ok(ProjectResult {
        project_id,
        status: "deployed".to_string(),
    })
}

#[tauri::command]
pub async fn cmd_node_discovery(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<PeerStatus>, String> {
    match state.network_discovery.discover_peers().await {
        Ok(peers) => Ok(peers),
        Err(e) => Err(format!("Discovery failed: {:?}", e)),
    }
}

#[tauri::command]
pub async fn cmd_get_all_agents(
    _state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<Agent>, String> {
    Ok(_state.agent_scheduler.list_agents().await)
}

#[tauri::command]
pub async fn cmd_request_compute(
    req: ComputeReq,
    state: State<'_, Arc<BackendServices>>,
) -> Result<ResourceLease, String> {
    let lease = ResourceLease {
        id: uuid::Uuid::new_v4().to_string(),
        gpu_memory: if req.gpu_required { req.memory_mb } else { 0 },
        duration: std::time::Duration::from_secs(req.duration_secs),
    };

    state.leases.write().await.push(lease.clone());
    Ok(lease)
}

#[tauri::command]
pub async fn cmd_force_terminate(
    task_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    // TODO: Implement task termination
    let _ = (&task_id, &state);
    Ok(true)
}

#[tauri::command]
pub async fn cmd_list_projects(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<Project>, String> {
    Ok(state.projects.read().await.clone())
}

#[tauri::command]
pub async fn cmd_list_leases(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<ResourceLease>, String> {
    Ok(state.leases.read().await.clone())
}

#[tauri::command]
pub async fn cmd_get_swarm_status(
    state: State<'_, Arc<BackendServices>>,
) -> Result<SwarmPulse, String> {
    Ok(state.agent_scheduler.get_swarm_status().await)
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
