//! Tauri commands for project and compute lease management.

use crate::backend::BackendServices;
use crate::shared::models::*;
use tauri::State;
use std::sync::Arc;

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
    Ok(ProjectResult { project_id, status: "deployed".to_string() })
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
