use std::sync::Arc;
use tokio::sync::RwLock;

mod frontend;
mod backend;
mod shared;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting VGA (Vangriten Gatling AI)swarm");

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

    backend::BackendServices {
        api_manager,
        agent_scheduler,
        network_discovery,
        compilation_scheduler,
        resource_manager,
        projects: Arc::new(RwLock::new(Vec::new())),
        leases: Arc::new(RwLock::new(Vec::new())),
    }
}