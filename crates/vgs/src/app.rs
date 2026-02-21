use std::sync::Arc;
use crate::app_types::{UiLang, ProviderFilter, ActiveView};
use vangriten_ai_swarm::shared::models::{Priority, BalancingStrategy};

/// Root application state for the `vgs` GUI.
pub struct VgaGuiApp {
    pub runtime: tokio::runtime::Runtime,
    pub services: Arc<vangriten_ai_swarm::backend::BackendServices>,

    pub lang: UiLang,
    pub active_view: ActiveView,

    pub task_view: crate::components::task::TaskComponent,
    pub network_view: crate::components::network::NetworkComponent,
    pub ollama_view: crate::components::ollama::OllamaComponent,
    pub resources_view: crate::components::resources::ResourcesComponent,

    // Task inputs
    pub task_language: String,
    pub task_target: String,
    pub task_context: String,

    // API Manager popup state
    pub show_api_manager: bool,
    pub api_password: String,
    pub api_password_confirm: String,
    pub api_provider: String,
    pub api_key_input: String,
    pub api_revealed_key: String,
    pub api_show_plaintext: bool,
    pub api_status: String,
    pub api_list_json: String,

    pub provider_filter: ProviderFilter,
    pub provider_id: String,

    // Resource manager inputs
    pub allow_remote_access: bool,
    pub group_name: String,
    pub group_max_members: usize,
    pub group_id: String,
    pub node_id: String,
    pub pool_name: String,
    pub pool_node_ids_csv: String,
    pub allocation_id: String,
    pub req_task_type: String,
    pub req_priority: Priority,
    pub req_cpu_cores: String,
    pub req_memory_mb: String,
    pub req_gpu_required: bool,
    pub req_gpu_memory_mb: String,
    pub req_preferred_models_csv: String,
    pub balancing_strategy: BalancingStrategy,

    // Cached view state (JSON display strings)
    pub last_error: Option<String>,
    pub swarm_json: String,
    pub agents_json: String,
    pub projects_json: String,
    pub leases_json: String,
    pub tasks_json: String,
    pub peers_json: String,

    pub providers_json: String,
    pub provider_config_json: String,
    pub resource_json: String,

    pub auto_refresh: bool,
    pub refresh_interval_secs: u64,
    pub last_refresh_instant: std::time::Instant,
}

impl VgaGuiApp {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("create tokio runtime");

        let services = runtime.block_on(async {
            let api_manager = vangriten_ai_swarm::backend::ApiKeyManager::new().await;
            let agent_scheduler = vangriten_ai_swarm::backend::AgentScheduler::new().await;
            let network_discovery = vangriten_ai_swarm::backend::NetworkDiscovery::new().await;
            let compilation_scheduler = vangriten_ai_swarm::backend::CompilationScheduler::new().await;
            let ollama_manager = vangriten_ai_swarm::backend::OllamaManager::new(None).await;
            let c_compiler = match vangriten_ai_swarm::backend::CCompilationScheduler::new(2).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("CCompilationScheduler init failed: {e:?}");
                    vangriten_ai_swarm::backend::CCompilationScheduler::new(0)
                        .await
                        .expect("c compiler scheduler")
                }
            };
            let resource_manager = match vangriten_ai_swarm::backend::ResourceManager::new(false).await {
                Ok(rm) => rm,
                Err(_) => vangriten_ai_swarm::backend::ResourceManager::new(true)
                    .await
                    .expect("resource manager"),
            };

            Arc::new(vangriten_ai_swarm::backend::BackendServices {
                api_manager,
                agent_scheduler,
                network_discovery,
                compilation_scheduler,
                resource_manager,
                c_compiler,
                ollama_manager,
                projects: Arc::new(tokio::sync::RwLock::new(Vec::new())),
                leases: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            })
        });

        let mut app = Self {
            runtime,
            services,

            lang: UiLang::Zh,
            active_view: ActiveView::Task,

            task_view: crate::components::task::TaskComponent::default(),
            network_view: crate::components::network::NetworkComponent::default(),
            ollama_view: crate::components::ollama::OllamaComponent::default(),
            resources_view: crate::components::resources::ResourcesComponent::default(),

            task_language: "rust".to_string(),
            task_target: "code".to_string(),
            task_context: "Generate a simple function".to_string(),

            show_api_manager: false,
            api_password: String::new(),
            api_password_confirm: String::new(),
            api_provider: "openai".to_string(),
            api_key_input: String::new(),
            api_revealed_key: String::new(),
            api_show_plaintext: false,
            api_status: String::new(),
            api_list_json: "(not loaded)".to_string(),

            provider_filter: ProviderFilter::All,
            provider_id: "openai".to_string(),

            allow_remote_access: false,
            group_name: "demo".to_string(),
            group_max_members: 10,
            group_id: String::new(),
            node_id: String::new(),
            pool_name: "pool".to_string(),
            pool_node_ids_csv: String::new(),
            allocation_id: String::new(),
            req_task_type: "distributed".to_string(),
            req_priority: Priority::Medium,
            req_cpu_cores: "1".to_string(),
            req_memory_mb: "1024".to_string(),
            req_gpu_required: false,
            req_gpu_memory_mb: String::new(),
            req_preferred_models_csv: String::new(),
            balancing_strategy: BalancingStrategy::LeastLoaded,

            last_error: None,
            swarm_json: "(not loaded)".to_string(),
            agents_json: "(not loaded)".to_string(),
            projects_json: "(not loaded)".to_string(),
            leases_json: "(not loaded)".to_string(),
            tasks_json: "(not loaded)".to_string(),
            peers_json: "(not loaded)".to_string(),

            providers_json: "(not loaded)".to_string(),
            provider_config_json: "(not loaded)".to_string(),
            resource_json: "(not loaded)".to_string(),

            auto_refresh: true,
            refresh_interval_secs: 3,
            last_refresh_instant: std::time::Instant::now(),
        };

        app.refresh_all();
        app
    }

    /// Returns the localized string for the current language.
    pub fn tr(&self, zh: &'static str, en: &'static str) -> &'static str {
        match self.lang {
            UiLang::Zh => zh,
            UiLang::En => en,
        }
    }

    pub fn set_error(&mut self, err: impl ToString) {
        self.last_error = Some(err.to_string());
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    pub fn pretty<T: serde::Serialize>(value: &T) -> String {
        serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("(serialize error: {e})"))
    }
}
