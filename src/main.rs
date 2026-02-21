use std::sync::Arc;

use tokio::sync::RwLock;
use vangriten_ai_swarm::backend;
use vangriten_ai_swarm::shared::models::{AgentTrait, ContextManager};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Vangriten AI Swarm");

    let backend_services = Arc::new(setup_backend_services().await);

    tauri::Builder::default()
        .manage(backend_services)
        .invoke_handler(tauri::generate_handler![
            vangriten_ai_swarm::frontend::cmd_get_billing,
            vangriten_ai_swarm::frontend::cmd_vault_op,
            vangriten_ai_swarm::frontend::cmd_vault_store,
            vangriten_ai_swarm::frontend::cmd_vault_retrieve,
            vangriten_ai_swarm::frontend::cmd_vault_list,
            vangriten_ai_swarm::frontend::cmd_vault_delete,
            vangriten_ai_swarm::frontend::cmd_vault_usage,
            vangriten_ai_swarm::frontend::cmd_deploy_project,
            vangriten_ai_swarm::frontend::cmd_node_discovery,
            vangriten_ai_swarm::frontend::cmd_get_all_agents,
            vangriten_ai_swarm::frontend::cmd_request_compute,
            vangriten_ai_swarm::frontend::cmd_force_terminate,
            vangriten_ai_swarm::frontend::cmd_list_projects,
            vangriten_ai_swarm::frontend::cmd_list_leases,
            vangriten_ai_swarm::frontend::cmd_get_swarm_status,
            vangriten_ai_swarm::frontend::cmd_execute_task,
            vangriten_ai_swarm::frontend::cmd_submit_task,
            vangriten_ai_swarm::frontend::cmd_get_task,
            vangriten_ai_swarm::frontend::cmd_list_tasks,
            vangriten_ai_swarm::frontend::cmd_cancel_task,
            vangriten_ai_swarm::frontend::cmd_get_providers,
            vangriten_ai_swarm::frontend::cmd_get_provider_config,
            vangriten_ai_swarm::frontend::cmd_set_default_provider,
            vangriten_ai_swarm::frontend::cmd_discover_nodes,
            vangriten_ai_swarm::frontend::cmd_list_discovered_nodes,
            vangriten_ai_swarm::frontend::cmd_create_swarm_group,
            vangriten_ai_swarm::frontend::cmd_join_swarm_group,
            vangriten_ai_swarm::frontend::cmd_leave_swarm_group,
            vangriten_ai_swarm::frontend::cmd_list_swarm_groups,
            vangriten_ai_swarm::frontend::cmd_get_group_members,
            vangriten_ai_swarm::frontend::cmd_request_resources,
            vangriten_ai_swarm::frontend::cmd_release_allocation,
            vangriten_ai_swarm::frontend::cmd_perform_health_check,
            vangriten_ai_swarm::frontend::cmd_set_remote_access,
            vangriten_ai_swarm::frontend::cmd_get_remote_access_status,
            vangriten_ai_swarm::frontend::cmd_set_balancing_strategy,
            vangriten_ai_swarm::frontend::cmd_get_balancing_strategy,
            vangriten_ai_swarm::frontend::cmd_create_resource_pool,
            vangriten_ai_swarm::frontend::cmd_list_resource_pools,
            vangriten_ai_swarm::frontend::cmd_list_gcc_instances,
            vangriten_ai_swarm::frontend::cmd_get_c_compiler_status,
            vangriten_ai_swarm::frontend::cmd_compile_c_round_robin,
            vangriten_ai_swarm::frontend::cmd_compile_c_parallel,
            vangriten_ai_swarm::frontend::cmd_ollama_check_connection,
            vangriten_ai_swarm::frontend::cmd_ollama_list_models,
            vangriten_ai_swarm::frontend::cmd_ollama_show_model_info,
            vangriten_ai_swarm::frontend::cmd_ollama_pull_model,
            vangriten_ai_swarm::frontend::cmd_ollama_delete_model,
            vangriten_ai_swarm::frontend::cmd_ollama_chat,
            vangriten_ai_swarm::frontend::cmd_ollama_chat_simple,
            vangriten_ai_swarm::frontend::cmd_ollama_generate,
            vangriten_ai_swarm::frontend::cmd_ollama_generate_simple,
            vangriten_ai_swarm::frontend::cmd_ollama_embed,
            vangriten_ai_swarm::frontend::cmd_ollama_get_version,
            vangriten_ai_swarm::frontend::cmd_ollama_get_usage_stats,
            vangriten_ai_swarm::frontend::cmd_ollama_reset_usage_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn setup_backend_services() -> backend::BackendServices {
    let api_manager = backend::ApiKeyManager::new().await;
    let agent_scheduler = backend::AgentScheduler::new().await;
    let network_discovery = backend::NetworkDiscovery::new().await;
    let compilation_scheduler = backend::CompilationScheduler::new().await;
    let resource_manager = backend::ResourceManager::new(true).await.unwrap();
    let c_compiler = backend::CCompilationScheduler::new(4)
        .await
        .expect("Failed to initialize C compiler scheduler");
    let ollama_manager = backend::OllamaManager::new(None).await;

    compilation_scheduler.prime_environment_cache().await;
    compilation_scheduler.prime_demo_usage().await;
    let _ = resource_manager.start_discovery().await;
    resource_manager.prime_demo_usage().await;
    let _ = network_discovery.discover_peers().await;
    network_discovery.broadcast_presence();
    api_manager.prime_demo_usage();
    agent_scheduler.prime_demo_usage().await;
    vangriten_ai_swarm::frontend::prime_frontend_stubs().await;
    vangriten_ai_swarm::shared::prime_shared_usage().await;

    let mut architect = backend::agents::ArchitectAgent::new();
    let mut programmer = backend::agents::ProgrammerAgent::new();
    let mut environment = backend::agents::EnvironmentAgent::new();
    let context = ContextManager {
        memory_slots: std::collections::HashMap::new(),
        docs: Vec::new(),
    };
    architect.update_context(&context);
    programmer.update_context(&context);
    environment.update_context(&context);

    let _ = architect.execute_instruction("event ui service".to_string()).await;
    let _ = programmer.execute_instruction("generate rust function".to_string()).await;
    let _ = environment.execute_instruction("setup rust 1.70".to_string()).await;
    let _ = architect.get_metrics();
    let _ = programmer.get_metrics();
    let _ = environment.get_metrics();

    let _ = vec![
        backend::agents::environment::EnvironmentStatus::Maintenance,
        backend::agents::environment::EnvironmentStatus::Failed,
    ];

    backend::BackendServices {
        api_manager,
        agent_scheduler,
        network_discovery,
        compilation_scheduler,
        resource_manager,
        c_compiler,
        ollama_manager,
        projects: Arc::new(RwLock::new(Vec::new())),
        leases: Arc::new(RwLock::new(Vec::new())),
    }
}
