#[cfg(not(feature = "native-gui"))]
fn main() {
    eprintln!("Native GUI is disabled. Re-run with: cargo run --features native-gui --bin vgs");
}

#[cfg(feature = "native-gui")]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("vas")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "vas",
        native_options,
        Box::new(|cc| {
            if let Err(err) = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx) {
                eprintln!("Failed to load Chinese fonts: {err}");
            }
            Box::new(VgaGuiApp::new())
        }),
    )
}

#[cfg(feature = "native-gui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiLang {
    Zh,
    En,
}

#[cfg(feature = "native-gui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderFilter {
    All,
    China,
    USA,
    Global,
}

#[cfg(feature = "native-gui")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveView {
    Task,
    Api,
    Network,
    Ollama,
    Resources,
}

#[cfg(feature = "native-gui")]
struct VgaGuiApp {
    runtime: tokio::runtime::Runtime,
    services: std::sync::Arc<vangriten_ai_swarm::backend::BackendServices>,

    lang: UiLang,
    active_view: ActiveView,

    task_view: crate::components::task::TaskComponent,
    network_view: crate::components::network::NetworkComponent,
    ollama_view: crate::components::ollama::OllamaComponent,
    resources_view: crate::components::resources::ResourcesComponent,

    // Inputs
    task_language: String,
    task_target: String,
    task_context: String,

    // API Manager (popup window)
    show_api_manager: bool,
    api_password: String,
    api_password_confirm: String,
    api_provider: String,
    api_key_input: String,
    api_revealed_key: String,
    api_show_plaintext: bool,
    api_status: String,
    api_list_json: String,

    provider_filter: ProviderFilter,
    provider_id: String,

    // Resource manager inputs
    allow_remote_access: bool,
    group_name: String,
    group_max_members: usize,
    group_id: String,
    node_id: String,
    pool_name: String,
    pool_node_ids_csv: String,
    allocation_id: String,
    req_task_type: String,
    req_priority: vangriten_ai_swarm::shared::models::Priority,
    req_cpu_cores: String,
    req_memory_mb: String,
    req_gpu_required: bool,
    req_gpu_memory_mb: String,
    req_preferred_models_csv: String,
    balancing_strategy: vangriten_ai_swarm::shared::models::BalancingStrategy,

    // View state
    last_error: Option<String>,
    swarm_json: String,
    agents_json: String,
    projects_json: String,
    leases_json: String,
    tasks_json: String,
    peers_json: String,

    providers_json: String,
    provider_config_json: String,
    resource_json: String,

    auto_refresh: bool,
    refresh_interval_secs: u64,
    last_refresh_instant: std::time::Instant,
}

#[cfg(feature = "native-gui")]
impl VgaGuiApp {
    fn new() -> Self {
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

            std::sync::Arc::new(vangriten_ai_swarm::backend::BackendServices {
                api_manager,
                agent_scheduler,
                network_discovery,
                compilation_scheduler,
                resource_manager,
                c_compiler,
                ollama_manager,
                projects: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
                leases: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
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
            req_priority: vangriten_ai_swarm::shared::models::Priority::Medium,
            req_cpu_cores: "1".to_string(),
            req_memory_mb: "1024".to_string(),
            req_gpu_required: false,
            req_gpu_memory_mb: "".to_string(),
            req_preferred_models_csv: "".to_string(),
            balancing_strategy: vangriten_ai_swarm::shared::models::BalancingStrategy::LeastLoaded,

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

    fn tr(&self, zh: &'static str, en: &'static str) -> &'static str {
        match self.lang {
            UiLang::Zh => zh,
            UiLang::En => en,
        }
    }

    fn set_error(&mut self, err: impl ToString) {
        self.last_error = Some(err.to_string());
    }

    fn clear_error(&mut self) {
        self.last_error = None;
    }

    fn pretty<T: serde::Serialize>(value: &T) -> String {
        serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("(serialize error: {e})"))
    }

    fn render_api_manager_window(&mut self, ctx: &eframe::egui::Context) {
        if !self.show_api_manager {
            return;
        }

        let title = self.tr("API 管理", "API Manager");
        let label_initialized = self.tr("已初始化", "Initialized");
        let label_unlocked = self.tr("已解锁", "Unlocked");
        let label_password = self.tr("密码", "Password");
        let label_confirm = self.tr("确认密码", "Confirm");
        let label_init = self.tr("初始化", "Initialize");
        let label_unlock = self.tr("解锁", "Unlock");
        let label_lock = self.tr("锁定", "Lock");
        let label_provider = self.tr("Provider", "Provider");
        let label_apikey = self.tr("API Key", "API Key");
        let label_store = self.tr("保存", "Store");
        let label_list = self.tr("列表", "List");
        let label_delete = self.tr("删除", "Delete");
        let label_reveal = self.tr("查看", "Reveal");
        let label_plain = self.tr("显示明文", "Show plaintext");
        let label_revealed = self.tr("已读取的 APIKey", "Revealed API Key");
        let label_local_keys = self.tr("本地存储的 keys", "Local keys");

        let api_manager = self.services.api_manager.clone();
        let initialized = api_manager.vault_is_initialized();
        let unlocked = api_manager.vault_is_unlocked();

        let mut open = self.show_api_manager;
        eframe::egui::Window::new(title)
            .id(eframe::egui::Id::new("api_manager_window"))
            .open(&mut open)
            .resizable(true)
            .default_width(560.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{label_initialized}: {initialized}"));
                    ui.separator();
                    ui.label(format!("{label_unlocked}: {unlocked}"));
                });

                ui.separator();

                if !initialized {
                    ui.label(self.tr(
                        "首次使用需要设置一个 Vault 密码（用于本地加密 APIKey）。",
                        "First use: set a Vault password (used to locally encrypt API keys).",
                    ));
                    ui.horizontal(|ui| {
                        ui.label(label_password);
                        ui.add(eframe::egui::TextEdit::singleline(&mut self.api_password).password(true));
                    });
                    ui.horizontal(|ui| {
                        ui.label(label_confirm);
                        ui.add(
                            eframe::egui::TextEdit::singleline(&mut self.api_password_confirm)
                                .password(true),
                        );
                        if ui.button(label_init).clicked() {
                            if self.api_password != self.api_password_confirm {
                                self.api_status = self.tr("两次密码不一致", "Passwords do not match").to_string();
                            } else {
                                match api_manager.vault_initialize(&self.api_password) {
                                    Ok(()) => {
                                        self.api_status = self.tr("初始化成功", "Initialized").to_string();
                                        self.api_password.clear();
                                        self.api_password_confirm.clear();
                                    }
                                    Err(e) => self.api_status = format!("init failed: {e:?}"),
                                }
                            }
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label(label_password);
                        ui.add(eframe::egui::TextEdit::singleline(&mut self.api_password).password(true));
                        if ui.button(label_unlock).clicked() {
                            match api_manager.vault_unlock(&self.api_password) {
                                Ok(()) => {
                                    self.api_status = self.tr("已解锁", "Unlocked").to_string();
                                    self.api_password.clear();
                                }
                                Err(e) => self.api_status = format!("unlock failed: {e:?}"),
                            }
                        }
                        if ui.button(label_lock).clicked() {
                            api_manager.vault_lock();
                            self.api_revealed_key.clear();
                            self.api_status = self.tr("已锁定", "Locked").to_string();
                        }
                    });
                }

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(label_provider);
                    ui.text_edit_singleline(&mut self.api_provider);
                });

                ui.horizontal(|ui| {
                    ui.label(label_apikey);
                    ui.add(
                        eframe::egui::TextEdit::singleline(&mut self.api_key_input)
                            .password(!self.api_show_plaintext),
                    );
                    ui.checkbox(&mut self.api_show_plaintext, label_plain);

                    if ui.button(label_store).clicked() {
                        let op = vangriten_ai_swarm::shared::models::VaultOp::Store {
                            provider: self.api_provider.clone(),
                            key: self.api_key_input.clone(),
                        };
                        match api_manager.vault_operation(op) {
                            Ok(_) => {
                                self.api_status = self.tr("已保存", "Stored").to_string();
                                self.api_key_input.clear();
                                self.api_revealed_key.clear();
                            }
                            Err(e) => self.api_status = format!("store failed: {e:?}"),
                        }
                    }

                    if ui.button(label_delete).clicked() {
                        let op = vangriten_ai_swarm::shared::models::VaultOp::Delete {
                            provider: self.api_provider.clone(),
                        };
                        match api_manager.vault_operation(op) {
                            Ok(_) => {
                                self.api_status = self.tr("已删除", "Deleted").to_string();
                                self.api_revealed_key.clear();
                            }
                            Err(e) => self.api_status = format!("delete failed: {e:?}"),
                        }
                    }

                    if ui.button(label_reveal).clicked() {
                        let op = vangriten_ai_swarm::shared::models::VaultOp::Retrieve {
                            provider: self.api_provider.clone(),
                        };
                        match api_manager.vault_operation(op) {
                            Ok(vangriten_ai_swarm::shared::models::VaultResult::Key(k)) => {
                                self.api_revealed_key = k;
                                self.api_status = self.tr("已读取", "Retrieved").to_string();
                            }
                            Ok(v) => self.api_status = format!("unexpected: {v:?}"),
                            Err(e) => self.api_status = format!("retrieve failed: {e:?}"),
                        }
                    }

                    if ui.button(label_list).clicked() {
                        match api_manager.vault_operation(vangriten_ai_swarm::shared::models::VaultOp::List) {
                            Ok(v) => self.api_list_json = Self::pretty(&v),
                            Err(e) => self.api_status = format!("list failed: {e:?}"),
                        }
                    }
                });

                ui.separator();

                ui.label(label_revealed);
                ui.add(
                    eframe::egui::TextEdit::singleline(&mut self.api_revealed_key)
                        .password(!self.api_show_plaintext)
                        .desired_width(f32::INFINITY),
                );

                ui.separator();
                ui.label(label_local_keys);
                eframe::egui::ScrollArea::vertical()
                    .id_source("api_manager_list_scroll")
                    .max_height(140.0)
                    .show(ui, |ui| {
                        ui.monospace(&self.api_list_json);
                    });

                if !self.api_status.trim().is_empty() {
                    ui.separator();
                    ui.monospace(&self.api_status);
                }
            });
        self.show_api_manager = open;
    }

    fn refresh_all(&mut self) {
        self.clear_error();
        let services = self.services.clone();

        let result = self.runtime.block_on(async move {
            let swarm = services.agent_scheduler.get_swarm_status().await;
            let agents = services.agent_scheduler.list_agents().await;
            let projects = services.projects.read().await.clone();
            let leases = services.leases.read().await.clone();
            let tasks = services.agent_scheduler.list_tasks().await;
            (swarm, agents, projects, leases, tasks)
        });

        self.swarm_json = Self::pretty(&result.0);
        self.agents_json = Self::pretty(&result.1);
        self.projects_json = Self::pretty(&result.2);
        self.leases_json = Self::pretty(&result.3);
        self.tasks_json = Self::pretty(&result.4);

        self.last_refresh_instant = std::time::Instant::now();
    }

    fn deploy_sample_project(&mut self) {
        self.clear_error();
        let services = self.services.clone();

        let result = self.runtime.block_on(async move {
            let project_id = uuid::Uuid::new_v4();
            let project = vangriten_ai_swarm::shared::models::Project {
                id: project_id,
                name: format!("project-{project_id}"),
                config: vangriten_ai_swarm::shared::models::ProjectConfig {
                    tech_stack: vec!["rust".to_string(), "tauri".to_string()],
                    default_provider: "local".to_string(),
                    concurrency_strategy: "gatling".to_string(),
                },
                agents: Vec::new(),
                workflow: vangriten_ai_swarm::shared::models::WorkflowGraph::default(),
                state: vangriten_ai_swarm::shared::models::ProjectStatus::Initialized,
                stats: vangriten_ai_swarm::shared::models::ExecutionStats {
                    total_tokens: 0,
                    total_duration: std::time::Duration::from_secs(0),
                    total_cost: 0.0,
                },
                last_updated: chrono::Utc::now(),
            };

            services.projects.write().await.push(project);
            project_id
        });

        self.projects_json = format!("Deployed project: {result}\n\n{}", self.projects_json);
        self.refresh_all();
    }

    fn request_sample_compute(&mut self) {
        self.clear_error();
        let services = self.services.clone();

        self.runtime.block_on(async move {
            let lease = vangriten_ai_swarm::shared::models::ResourceLease {
                id: uuid::Uuid::new_v4().to_string(),
                gpu_memory: 0,
                duration: std::time::Duration::from_secs(600),
            };
            services.leases.write().await.push(lease);
        });

        self.refresh_all();
    }

    fn execute_task(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let spec = vangriten_ai_swarm::shared::models::TaskSpec {
            language: self.task_language.clone(),
            target: self.task_target.clone(),
            context_range: self.task_context.clone(),
        };

        let res = self.runtime.block_on(async move {
            services.agent_scheduler.execute_task_spec(spec).await
        });

        match res {
            Ok(out) => self.tasks_json = format!("Last execute_task output:\n{}\n\n{}", Self::pretty(&out), self.tasks_json),
            Err(e) => self.set_error(format!("execute_task failed: {e:?}")),
        }

        self.refresh_all();
    }

    fn submit_task(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let task = vangriten_ai_swarm::shared::models::Task::new(
            vangriten_ai_swarm::shared::models::TaskSpec {
                language: self.task_language.clone(),
                target: self.task_target.clone(),
                context_range: self.task_context.clone(),
            },
            vangriten_ai_swarm::shared::models::Priority::Medium,
            std::path::PathBuf::from("snapshots/gui.json"),
        );

        let res = self.runtime.block_on(async move { services.agent_scheduler.submit_task(task).await });
        match res {
            Ok(id) => self.tasks_json = format!("Submitted task: {id}\n\n{}", self.tasks_json),
            Err(e) => self.set_error(format!("submit_task failed: {e:?}")),
        }

        self.refresh_all();
    }

    fn load_providers(&mut self) {
        self.clear_error();

        let res = self
            .services
            .api_manager
            .vault_operation(vangriten_ai_swarm::shared::models::VaultOp::GetProviders);

        match res {
            Ok(vangriten_ai_swarm::shared::models::VaultResult::ProviderConfigs(list)) => {
                let filtered: Vec<_> = match self.provider_filter {
                    ProviderFilter::All => list,
                    ProviderFilter::China => list
                        .into_iter()
                        .filter(|p| p.region == vangriten_ai_swarm::shared::models::ProviderRegion::China)
                        .collect(),
                    ProviderFilter::USA => list
                        .into_iter()
                        .filter(|p| p.region == vangriten_ai_swarm::shared::models::ProviderRegion::USA)
                        .collect(),
                    ProviderFilter::Global => list
                        .into_iter()
                        .filter(|p| p.region == vangriten_ai_swarm::shared::models::ProviderRegion::Global)
                        .collect(),
                };
                self.providers_json = Self::pretty(&filtered);
            }
            Ok(other) => self.set_error(format!("Unexpected result: {other:?}")),
            Err(e) => self.set_error(format!("load providers failed: {e:?}")),
        }
    }

    fn get_provider_config(&mut self) {
        self.clear_error();
        let op = vangriten_ai_swarm::shared::models::VaultOp::GetProviderConfig {
            provider: self.provider_id.clone(),
        };
        match self.services.api_manager.vault_operation(op) {
            Ok(vangriten_ai_swarm::shared::models::VaultResult::ProviderConfig(cfg)) => {
                self.provider_config_json = Self::pretty(&cfg);
            }
            Ok(other) => self.set_error(format!("Unexpected result: {other:?}")),
            Err(e) => self.set_error(format!("get provider config failed: {e:?}")),
        }
    }

    fn set_default_provider(&mut self) {
        self.clear_error();
        let op = vangriten_ai_swarm::shared::models::VaultOp::SetDefaultProvider {
            provider: self.provider_id.clone(),
        };
        match self.services.api_manager.vault_operation(op) {
            Ok(vangriten_ai_swarm::shared::models::VaultResult::DefaultProvider(p)) => {
                self.provider_config_json = format!("Default provider set: {p}\n\n{}", self.provider_config_json);
            }
            Ok(other) => self.set_error(format!("Unexpected result: {other:?}")),
            Err(e) => self.set_error(format!("set default provider failed: {e:?}")),
        }
    }

    fn discover_nodes(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.discover_nodes().await });
        match res {
            Ok(nodes) => self.resource_json = Self::pretty(&nodes),
            Err(e) => self.set_error(format!("discover_nodes failed: {e:?}")),
        }
    }

    fn list_discovered_nodes(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let nodes = self.runtime.block_on(async move { services.resource_manager.list_discovered_nodes().await });
        self.resource_json = Self::pretty(&nodes);
    }

    fn set_remote_access(&mut self) {
        self.clear_error();
        let allow = self.allow_remote_access;
        let services = self.services.clone();
        self.runtime.block_on(async move {
            services.resource_manager.set_remote_access(allow).await;
        });
        self.resource_json = format!("remote_access={allow}\n\n{}", self.resource_json);
    }

    fn get_remote_access_status(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let allow = self.runtime.block_on(async move { services.resource_manager.get_remote_access_status().await });
        self.allow_remote_access = allow;
        self.resource_json = format!("remote_access={allow}\n\n{}", self.resource_json);
    }

    fn create_swarm_group(&mut self) {
        self.clear_error();
        let name = self.group_name.clone();
        let max_members = self.group_max_members;
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.create_swarm_group(name, max_members).await
        });
        match res {
            Ok(id) => {
                self.group_id = id.clone();
                self.resource_json = format!("created group: {id}\n\n{}", self.resource_json);
            }
            Err(e) => self.set_error(format!("create_swarm_group failed: {e:?}")),
        }
    }

    fn join_swarm_group(&mut self) {
        self.clear_error();
        let group_id = self.group_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.join_swarm_group(group_id).await });
        if let Err(e) = res {
            self.set_error(format!("join_swarm_group failed: {e:?}"));
        }
    }

    fn leave_swarm_group(&mut self) {
        self.clear_error();
        let group_id = self.group_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.leave_swarm_group(group_id).await });
        if let Err(e) = res {
            self.set_error(format!("leave_swarm_group failed: {e:?}"));
        }
    }

    fn list_swarm_groups(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let groups = self.runtime.block_on(async move { services.resource_manager.list_swarm_groups().await });
        self.resource_json = Self::pretty(&groups);
    }

    fn get_group_members(&mut self) {
        self.clear_error();
        let group_id = self.group_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.resource_manager.get_group_members(group_id).await });
        match res {
            Ok(members) => self.resource_json = Self::pretty(&members),
            Err(e) => self.set_error(format!("get_group_members failed: {e:?}")),
        }
    }

    fn set_balancing_strategy(&mut self) {
        self.clear_error();
        let strategy = self.balancing_strategy.clone();
        let services = self.services.clone();
        self.runtime.block_on(async move {
            services.resource_manager.set_balancing_strategy(strategy).await;
        });
    }

    fn get_balancing_strategy(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let s = self.runtime.block_on(async move { services.resource_manager.get_balancing_strategy().await });
        self.balancing_strategy = s;
        self.resource_json = format!("balancing={:?}\n\n{}", self.balancing_strategy, self.resource_json);
    }

    fn create_resource_pool(&mut self) {
        self.clear_error();
        let name = self.pool_name.clone();
        let node_ids: Vec<String> = self
            .pool_node_ids_csv
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.create_resource_pool(name, node_ids).await
        });
        match res {
            Ok(id) => self.resource_json = format!("created pool: {id}\n\n{}", self.resource_json),
            Err(e) => self.set_error(format!("create_resource_pool failed: {e:?}")),
        }
    }

    fn list_resource_pools(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let pools = self.runtime.block_on(async move { services.resource_manager.list_resource_pools().await });
        self.resource_json = Self::pretty(&pools);
    }

    fn request_resources(&mut self) {
        self.clear_error();

        let cpu_cores = self.req_cpu_cores.trim().parse::<u32>().ok();
        let memory_mb = self.req_memory_mb.trim().parse::<u64>().ok();
        let gpu_memory_mb = self.req_gpu_memory_mb.trim().parse::<u64>().ok();
        let preferred_models: Vec<String> = self
            .req_preferred_models_csv
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let requirements = vangriten_ai_swarm::shared::models::ResourceRequirements {
            cpu_cores,
            memory_mb,
            gpu_required: self.req_gpu_required,
            gpu_memory_mb,
            preferred_models,
        };

        let task_type = self.req_task_type.clone();
        let priority = self.req_priority.clone();
        let services = self.services.clone();

        let res = self.runtime.block_on(async move {
            services
                .resource_manager
                .request_resources(requirements, task_type, priority)
                .await
        });

        match res {
            Ok(allocation) => {
                self.allocation_id = allocation.allocation_id.clone();
                self.resource_json = Self::pretty(&allocation);
            }
            Err(e) => self.set_error(format!("request_resources failed: {e:?}")),
        }
    }

    fn release_allocation(&mut self) {
        self.clear_error();
        let allocation_id = self.allocation_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.release_allocation(allocation_id).await
        });
        if let Err(e) = res {
            self.set_error(format!("release_allocation failed: {e:?}"));
        }
    }

    fn perform_health_check(&mut self) {
        self.clear_error();
        let node_id = self.node_id.clone();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move {
            services.resource_manager.perform_health_check(node_id).await
        });
        match res {
            Ok(hc) => self.resource_json = Self::pretty(&hc),
            Err(e) => self.set_error(format!("health_check failed: {e:?}")),
        }
    }
}

#[cfg(feature = "native-gui")]
impl eframe::App for VgaGuiApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.auto_refresh
            && self.last_refresh_instant.elapsed().as_secs() >= self.refresh_interval_secs
        {
            self.refresh_all();
        }

        eframe::egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            eframe::egui::menu::bar(ui, |ui| {
                let menu_label = self.tr("菜单", "Menu");
                let lang_label = self.tr("语言", "Language");

                ui.menu_button(menu_label, |ui| {
                    if ui.button(self.tr("刷新", "Refresh")).clicked() {
                        self.refresh_all();
                        ui.close_menu();
                    }
                    if ui.button(self.tr("API管理", "API Manager")).clicked() {
                        self.show_api_manager = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button(self.tr("部署示例项目", "Deploy Sample Project")).clicked() {
                        self.deploy_sample_project();
                        ui.close_menu();
                    }
                    if ui.button(self.tr("申请示例算力", "Request Sample Compute")).clicked() {
                        self.request_sample_compute();
                        ui.close_menu();
                    }
                    ui.separator();
                    let auto_refresh_label = self.tr("自动刷新", "Auto refresh");
                    ui.checkbox(&mut self.auto_refresh, auto_refresh_label);
                    ui.add(
                        eframe::egui::DragValue::new(&mut self.refresh_interval_secs)
                            .clamp_range(1..=60)
                            .suffix("s"),
                    );
                });

                ui.menu_button(lang_label, |ui| {
                    ui.selectable_value(&mut self.lang, UiLang::Zh, "中文");
                    ui.selectable_value(&mut self.lang, UiLang::En, "EN");
                });

                ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                    ui.strong(self.tr("vas", "vas"));
                });
            });

            if let Some(err) = &self.last_error {
                ui.separator();
                ui.colored_label(eframe::egui::Color32::RED, err);
            }
        });

        self.render_api_manager_window(ctx);

        eframe::egui::SidePanel::left("left_nav")
            .resizable(false)
            .default_width(170.0)
            .show(ctx, |ui| {
                ui.heading(self.tr("功能", "Views"));
                ui.separator();

                let label_task = self.tr("任务", "Task");
                let label_api = self.tr("API", "API");
                let label_network = self.tr("网络", "Network");
                let label_ollama = self.tr("本地Ollama", "Ollama");
                let label_resources = self.tr("资源管理", "Resources");

                if ui.selectable_label(self.active_view == ActiveView::Task, label_task).clicked() {
                    self.active_view = ActiveView::Task;
                }
                if ui.selectable_label(self.active_view == ActiveView::Api, label_api).clicked() {
                    self.active_view = ActiveView::Api;
                }
                if ui.selectable_label(self.active_view == ActiveView::Network, label_network).clicked() {
                    self.active_view = ActiveView::Network;
                }
                if ui.selectable_label(self.active_view == ActiveView::Ollama, label_ollama).clicked() {
                    self.active_view = ActiveView::Ollama;
                }
                if ui.selectable_label(self.active_view == ActiveView::Resources, label_resources).clicked() {
                    self.active_view = ActiveView::Resources;
                }
            });

        eframe::egui::SidePanel::right("right_info")
            .resizable(true)
            .default_width(480.0)
            .show(ctx, |ui| {
                ui.heading(self.tr("信息", "Info"));
                ui.separator();

                eframe::egui::ScrollArea::vertical()
                    .id_source("right_info_scroll")
                    .show(ui, |ui| {
                        ui.columns(2, |cols| {
                            cols[0].group(|ui| {
                                ui.heading(self.tr("蜂群", "Swarm"));
                                ui.monospace(&self.swarm_json);
                            });
                            cols[0].add_space(8.0);
                            cols[0].group(|ui| {
                                ui.heading(self.tr("代理", "Agents"));
                                ui.monospace(&self.agents_json);
                            });

                            cols[1].group(|ui| {
                                ui.heading(self.tr("项目", "Projects"));
                                ui.monospace(&self.projects_json);
                            });
                            cols[1].add_space(8.0);
                            cols[1].group(|ui| {
                                ui.heading(self.tr("租约", "Leases"));
                                ui.monospace(&self.leases_json);
                            });
                            cols[1].add_space(8.0);
                            cols[1].group(|ui| {
                                ui.heading(self.tr("任务列表", "Tasks"));
                                ui.monospace(&self.tasks_json);
                            });
                        });
                    });
            });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            eframe::egui::ScrollArea::vertical()
                .id_source("center_view_scroll")
                .show(ui, |ui| {
                    match self.active_view {
                        ActiveView::Task => {
                            let mut task_view = std::mem::take(&mut self.task_view);
                            task_view.ui(ui, self);
                            self.task_view = task_view;
                        }
                        ActiveView::Api => {
                            ui.heading(self.tr("API", "API"));
                            ui.separator();

                            eframe::egui::CollapsingHeader::new(self.tr("API密钥管理", "API Keys"))
                                .id_source("api_sub_keys")
                                .default_open(true)
                                .show(ui, |ui| {
                                    ui.label(self.tr(
                                        "APIKey 的查看/保存在 API管理 弹窗中完成：需要先输入密码解锁（本地加密存储）。",
                                        "API keys are managed in the API Manager popup: unlock with a password first (encrypted at rest).",
                                    ));
                                    if ui
                                        .button(self.tr("打开 API管理 弹窗", "Open API Manager"))
                                        .clicked()
                                    {
                                        self.show_api_manager = true;
                                    }
                                });

                            ui.separator();

                            eframe::egui::CollapsingHeader::new(self.tr("API提供商", "Providers"))
                                .id_source("api_sub_providers")
                                .default_open(true)
                                .show(ui, |ui| {
                                    eframe::egui::CollapsingHeader::new(self.tr("列表", "List"))
                                        .id_source("providers_sub_list")
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            ui.horizontal_wrapped(|ui| {
                                                let label_filter = self.tr("筛选：", "Filter:");
                                                let label_all = self.tr("全部", "All");
                                                let label_china = self.tr("中国", "China");
                                                let label_usa = self.tr("美国", "USA");
                                                let label_global = self.tr("全球", "Global");

                                                ui.label(label_filter);
                                                ui.selectable_value(&mut self.provider_filter, ProviderFilter::All, label_all);
                                                ui.selectable_value(&mut self.provider_filter, ProviderFilter::China, label_china);
                                                ui.selectable_value(&mut self.provider_filter, ProviderFilter::USA, label_usa);
                                                ui.selectable_value(&mut self.provider_filter, ProviderFilter::Global, label_global);

                                                if ui.button(self.tr("加载", "Load")).clicked() {
                                                    self.load_providers();
                                                }
                                            });

                                            eframe::egui::ScrollArea::vertical()
                                                .id_source("providers_json_scroll")
                                                .max_height(220.0)
                                                .show(ui, |ui| {
                                                    ui.monospace(&self.providers_json);
                                                });
                                        });

                                    ui.separator();

                                    eframe::egui::CollapsingHeader::new(self.tr("配置", "Config"))
                                        .id_source("providers_sub_config")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            ui.horizontal_wrapped(|ui| {
                                                ui.label(self.tr("Provider ID", "Provider ID"));
                                                ui.text_edit_singleline(&mut self.provider_id);
                                                if ui.button(self.tr("获取配置", "Get Config")).clicked() {
                                                    self.get_provider_config();
                                                }
                                                if ui.button(self.tr("设为默认", "Set Default")).clicked() {
                                                    self.set_default_provider();
                                                }
                                            });

                                            eframe::egui::ScrollArea::vertical()
                                                .id_source("provider_config_json_scroll")
                                                .max_height(200.0)
                                                .show(ui, |ui| {
                                                    ui.monospace(&self.provider_config_json);
                                                });
                                        });
                                });
                        }
                        ActiveView::Network => {
                            let mut network_view = std::mem::take(&mut self.network_view);
                            network_view.ui(ui, self);
                            self.network_view = network_view;
                        }
                        ActiveView::Ollama => {
                            let mut ollama_view = std::mem::take(&mut self.ollama_view);
                            ollama_view.ui(ui, self);
                            self.ollama_view = ollama_view;
                        }
                        ActiveView::Resources => {
                            let mut resources_view = std::mem::take(&mut self.resources_view);
                            resources_view.ui(ui, self);
                            self.resources_view = resources_view;
                        }
                    }
                });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
