//! Tauri commands for distributed resource management.

use crate::backend::BackendServices;
use crate::shared::models::*;
use tauri::State;
use std::sync::Arc;

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
pub async fn cmd_discover_nodes(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<NodeInfo>, String> {
    state.resource_manager.discover_nodes().await
        .map_err(|e| format!("Failed to discover nodes: {:?}", e))
}

#[tauri::command]
pub async fn cmd_list_discovered_nodes(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<NodeInfo>, String> {
    Ok(state.resource_manager.list_discovered_nodes().await)
}

#[tauri::command]
pub async fn cmd_create_swarm_group(
    name: String,
    max_members: usize,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    state.resource_manager.create_swarm_group(name, max_members).await
        .map_err(|e| format!("Failed to create swarm group: {:?}", e))
}

#[tauri::command]
pub async fn cmd_join_swarm_group(
    group_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    state.resource_manager.join_swarm_group(group_id).await
        .map(|_| true)
        .map_err(|e| format!("Failed to join swarm group: {:?}", e))
}

#[tauri::command]
pub async fn cmd_leave_swarm_group(
    group_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    state.resource_manager.leave_swarm_group(group_id).await
        .map(|_| true)
        .map_err(|e| format!("Failed to leave swarm group: {:?}", e))
}

#[tauri::command]
pub async fn cmd_list_swarm_groups(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<SwarmGroup>, String> {
    Ok(state.resource_manager.list_swarm_groups().await)
}

#[tauri::command]
pub async fn cmd_get_group_members(
    group_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<NodeInfo>, String> {
    state.resource_manager.get_group_members(group_id).await
        .map_err(|e| format!("Failed to get group members: {:?}", e))
}

#[tauri::command]
pub async fn cmd_request_resources(
    requirements: ResourceRequirements,
    task_type: String,
    priority: Priority,
    state: State<'_, Arc<BackendServices>>,
) -> Result<ResourceAllocation, String> {
    state.resource_manager.request_resources(requirements, task_type, priority).await
        .map_err(|e| format!("Failed to request resources: {:?}", e))
}

#[tauri::command]
pub async fn cmd_release_allocation(
    allocation_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    state.resource_manager.release_allocation(allocation_id).await
        .map(|_| true)
        .map_err(|e| format!("Failed to release allocation: {:?}", e))
}

#[tauri::command]
pub async fn cmd_perform_health_check(
    node_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<HealthCheck, String> {
    state.resource_manager.perform_health_check(node_id).await
        .map_err(|e| format!("Failed to perform health check: {:?}", e))
}

#[tauri::command]
pub async fn cmd_set_remote_access(
    allow: bool,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    state.resource_manager.set_remote_access(allow).await;
    Ok(true)
}

#[tauri::command]
pub async fn cmd_get_remote_access_status(
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    Ok(state.resource_manager.get_remote_access_status().await)
}

#[tauri::command]
pub async fn cmd_set_balancing_strategy(
    strategy: BalancingStrategy,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    state.resource_manager.set_balancing_strategy(strategy).await;
    Ok(true)
}

#[tauri::command]
pub async fn cmd_get_balancing_strategy(
    state: State<'_, Arc<BackendServices>>,
) -> Result<BalancingStrategy, String> {
    Ok(state.resource_manager.get_balancing_strategy().await)
}

#[tauri::command]
pub async fn cmd_create_resource_pool(
    name: String,
    node_ids: Vec<String>,
    state: State<'_, Arc<BackendServices>>,
) -> Result<String, String> {
    state.resource_manager.create_resource_pool(name, node_ids).await
        .map_err(|e| format!("Failed to create resource pool: {:?}", e))
}

#[tauri::command]
pub async fn cmd_list_resource_pools(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<ResourcePool>, String> {
    Ok(state.resource_manager.list_resource_pools().await)
}
