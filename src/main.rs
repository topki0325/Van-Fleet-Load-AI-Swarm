use std::sync::Arc;
use tokio::sync::RwLock;
use crate::shared::models::{AgentTrait, ContextManager};

mod frontend;
mod backend;
mod shared;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Vangriten AI Swarm");

    // Initialize backend services
    let backend_services = Arc::new(setup_backend_services().await);

    tauri::Builder::default()
        .manage(backend_services)
        .invoke_handler(tauri::generate_handler![
            frontend::cmd_get_billing,
            frontend::cmd_vault_op,
            frontend::cmd_vault_store,
            frontend::cmd_vault_retrieve,
            frontend::cmd_vault_list,
            frontend::cmd_vault_delete,
            frontend::cmd_vault_usage,
            frontend::cmd_deploy_project,
            frontend::cmd_node_discovery,
            frontend::cmd_get_all_agents,
            frontend::cmd_request_compute,
            frontend::cmd_force_terminate,
            frontend::cmd_list_projects,
            frontend::cmd_list_leases,
            frontend::cmd_get_swarm_status,
            frontend::cmd_execute_task,
            frontend::cmd_submit_task,
            frontend::cmd_get_task,
            frontend::cmd_list_tasks,
            frontend::cmd_cancel_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn setup_backend_services() -> backend::BackendServices {
    // Initialize core services
    let api_manager = backend::ApiKeyManager::new().await;
    let agent_scheduler = backend::AgentScheduler::new().await;
    let network_discovery = backend::NetworkDiscovery::new().await;
    let compilation_scheduler = backend::CompilationScheduler::new().await;
    let resource_manager = backend::ResourceManager::new(true).await.unwrap();

    compilation_scheduler.prime_environment_cache().await;
    compilation_scheduler.prime_demo_usage().await;
    let _ = resource_manager.start_discovery().await;
    resource_manager.prime_demo_usage().await;
    let _ = network_discovery.discover_peers().await;
    network_discovery.broadcast_presence();
    api_manager.prime_demo_usage();
    agent_scheduler.prime_demo_usage().await;
    frontend::prime_frontend_stubs().await;
    shared::prime_shared_usage().await;

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

    let services = backend::BackendServices {
        api_manager,
        agent_scheduler,
        network_discovery,
        compilation_scheduler,
        resource_manager,
        projects: Arc::new(RwLock::new(Vec::new())),
        leases: Arc::new(RwLock::new(Vec::new())),
    };

    let _ = &services.compilation_scheduler;
    let _ = &services.resource_manager;
    services
}