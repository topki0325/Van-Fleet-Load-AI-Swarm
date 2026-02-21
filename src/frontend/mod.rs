use crate::backend::BackendServices;
use crate::shared::models::*;
use tauri::State;
use std::sync::Arc;
use uuid;

pub mod app;
pub mod project_view;
pub mod agent_monitor;
pub mod client_gui;

pub async fn prime_frontend_stubs() {
    let _ = app::initialize();
    app::on_route_change("bootstrap".to_string());
    app::toggle_mode(ClientMode::Master);

    let project_id = uuid::Uuid::new_v4();
    let demo_project = Project {
        id: project_id,
        name: "demo".to_string(),
        config: ProjectConfig {
            tech_stack: vec!["rust".to_string()],
            default_provider: "local".to_string(),
            concurrency_strategy: "gatling".to_string(),
        },
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

    project_view::render_workflow_tree(&demo_project);

    let demo_task_id = uuid::Uuid::new_v4();
    let _ = project_view::sync_agent_output(demo_task_id).await;
    project_view::handle_manual_intervention(ConflictInfo {
        task_id: demo_task_id,
        conflicts: Vec::new(),
        resolution_options: Vec::new(),
    });

    agent_monitor::update_swarm_pulse(SwarmPulse {
        total_agents: 0,
        active_tasks: 0,
        queue_length: 0,
    });
    agent_monitor::render_provider_metrics("local".to_string());
    agent_monitor::show_gpu_utilization("local");

    client_gui::switch_layout(ClientMode::Master);
}

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

#[tauri::command]
pub async fn cmd_discover_nodes(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<crate::shared::models::NodeInfo>, String> {
    state.resource_manager.discover_nodes().await
        .map_err(|e| format!("Failed to discover nodes: {:?}", e))
}

#[tauri::command]
pub async fn cmd_list_discovered_nodes(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<crate::shared::models::NodeInfo>, String> {
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
) -> Result<Vec<crate::shared::models::SwarmGroup>, String> {
    Ok(state.resource_manager.list_swarm_groups().await)
}

#[tauri::command]
pub async fn cmd_get_group_members(
    group_id: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<crate::shared::models::NodeInfo>, String> {
    state.resource_manager.get_group_members(group_id).await
        .map_err(|e| format!("Failed to get group members: {:?}", e))
}

#[tauri::command]
pub async fn cmd_request_resources(
    requirements: crate::shared::models::ResourceRequirements,
    task_type: String,
    priority: crate::shared::models::Priority,
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::shared::models::ResourceAllocation, String> {
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
) -> Result<crate::shared::models::HealthCheck, String> {
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
    strategy: crate::shared::models::BalancingStrategy,
    state: State<'_, Arc<BackendServices>>,
) -> Result<bool, String> {
    state.resource_manager.set_balancing_strategy(strategy).await;
    Ok(true)
}

#[tauri::command]
pub async fn cmd_get_balancing_strategy(
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::shared::models::BalancingStrategy, String> {
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
) -> Result<Vec<crate::shared::models::ResourcePool>, String> {
    Ok(state.resource_manager.list_resource_pools().await)
}

#[tauri::command]
pub async fn cmd_list_gcc_instances(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<crate::backend::c_compiler::CGccInstance>, String> {
    Ok(state.c_compiler.list_gcc_instances().await)
}

#[tauri::command]
pub async fn cmd_get_c_compiler_status(
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::backend::c_compiler::CCompilationStatus, String> {
    Ok(state.c_compiler.get_status().await)
}

#[tauri::command]
pub async fn cmd_compile_c_round_robin(
    source_files: Vec<String>,
    output_path: String,
    optimization_level: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::backend::c_compiler::CCompilationResult, String> {
    use std::path::PathBuf;
    
    let task = crate::backend::c_compiler::CCompilationTask {
        task_id: uuid::Uuid::new_v4().to_string(),
        source_files: source_files.into_iter().map(PathBuf::from).collect(),
        output_path: PathBuf::from(output_path),
        compiler_flags: vec!["-Wall".to_string(), "-Wextra".to_string()],
        include_paths: vec![],
        optimization_level,
    };
    
    state.c_compiler.compile_round_robin(task).await
        .map_err(|e| format!("Compilation failed: {:?}", e))
}

#[tauri::command]
pub async fn cmd_compile_c_parallel(
    tasks: Vec<crate::backend::c_compiler::CCompilationTask>,
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<Result<crate::backend::c_compiler::CCompilationResult, String>>, String> {
    let results = state.c_compiler.compile_parallel(tasks).await;
    Ok(results.into_iter().map(|r| r.map_err(|e| format!("{:?}", e))).collect())
}

#[tauri::command]
pub async fn cmd_ollama_check_connection(
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::backend::ollama_client::OllamaConnectionStatus, String> {
    Ok(state.ollama_manager.check_connection().await)
}

#[tauri::command]
pub async fn cmd_ollama_list_models(
    state: State<'_, Arc<BackendServices>>,
) -> Result<Vec<crate::backend::ollama_client::OllamaModel>, String> {
    state.ollama_manager.list_models().await
}

#[tauri::command]
pub async fn cmd_ollama_show_model_info(
    model_name: String,
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::backend::ollama_client::OllamaModelInfo, String> {
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
    request: crate::backend::ollama_client::ChatRequest,
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::backend::ollama_client::ChatResponse, String> {
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
    request: crate::backend::ollama_client::GenerateRequest,
    state: State<'_, Arc<BackendServices>>,
) -> Result<crate::backend::ollama_client::GenerateResponse, String> {
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
) -> Result<crate::backend::ollama_client::OllamaUsageStats, String> {
    Ok(state.ollama_manager.get_usage_stats().await)
}

#[tauri::command]
pub async fn cmd_ollama_reset_usage_stats(
    state: State<'_, Arc<BackendServices>>,
) -> Result<(), String> {
    state.ollama_manager.reset_usage_stats().await;
    Ok(())
}
