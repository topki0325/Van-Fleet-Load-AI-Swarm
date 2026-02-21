use crate::shared::models::*;

pub mod app;
pub mod project_view;
pub mod agent_monitor;
pub mod client_gui;

// Command sub-modules grouped by domain.
pub mod vault_commands;
pub mod project_commands;
pub mod task_commands;
pub mod resource_commands;
pub mod compiler_commands;
pub mod ollama_commands;

// Re-export all commands so callers can still use `crate::frontend::cmd_*`.
pub use vault_commands::*;
pub use project_commands::*;
pub use task_commands::*;
pub use resource_commands::*;
pub use compiler_commands::*;
pub use ollama_commands::*;

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
