#[cfg(not(feature = "native-gui"))]
fn main() {
    eprintln!("Native GUI is disabled. Re-run with: cargo run --features native-gui --bin vga-gui");
}

#[cfg(feature = "native-gui")]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("VGA Swarm (Rust GUI)")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "VGA Swarm (Rust GUI)",
        native_options,
        Box::new(|_cc| Box::new(VgaGuiApp::new())),
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
struct VgaGuiApp {
    runtime: tokio::runtime::Runtime,
    services: std::sync::Arc<vangriten_ai_swarm::backend::BackendServices>,

    lang: UiLang,

    // Inputs
    task_language: String,
    task_target: String,
    task_context: String,

    vault_provider: String,
    vault_key: String,

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
    vault_json: String,
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
                projects: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
                leases: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            })
        });

        let mut app = Self {
            runtime,
            services,

            lang: UiLang::Zh,

            task_language: "rust".to_string(),
            task_target: "code".to_string(),
            task_context: "Generate a simple function".to_string(),

            vault_provider: "openai".to_string(),
            vault_key: String::new(),

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
            vault_json: "(not loaded)".to_string(),
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

    fn vault_store(&mut self) {
        self.clear_error();
        let op = vangriten_ai_swarm::shared::models::VaultOp::Store {
            provider: self.vault_provider.clone(),
            key: self.vault_key.clone(),
        };

        match self.services.api_manager.vault_operation(op) {
            Ok(v) => self.vault_json = Self::pretty(&v),
            Err(e) => self.set_error(format!("vault_store failed: {e:?}")),
        }
    }

    fn vault_retrieve(&mut self) {
        self.clear_error();
        let op = vangriten_ai_swarm::shared::models::VaultOp::Retrieve {
            provider: self.vault_provider.clone(),
        };

        match self.services.api_manager.vault_operation(op) {
            Ok(v) => self.vault_json = Self::pretty(&v),
            Err(e) => self.set_error(format!("vault_retrieve failed: {e:?}")),
        }
    }

    fn vault_list(&mut self) {
        self.clear_error();
        match self.services.api_manager.vault_operation(vangriten_ai_swarm::shared::models::VaultOp::List) {
            Ok(v) => self.vault_json = Self::pretty(&v),
            Err(e) => self.set_error(format!("vault_list failed: {e:?}")),
        }
    }

    fn vault_delete(&mut self) {
        self.clear_error();
        let op = vangriten_ai_swarm::shared::models::VaultOp::Delete {
            provider: self.vault_provider.clone(),
        };

        match self.services.api_manager.vault_operation(op) {
            Ok(v) => self.vault_json = Self::pretty(&v),
            Err(e) => self.set_error(format!("vault_delete failed: {e:?}")),
        }
    }

    fn vault_usage(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let usage = self.runtime.block_on(async move { services.api_manager.get_usage_entries().await });
        self.vault_json = Self::pretty(&usage);
    }

    fn discover_peers(&mut self) {
        self.clear_error();
        let services = self.services.clone();
        let res = self.runtime.block_on(async move { services.network_discovery.discover_peers().await });
        match res {
            Ok(v) => self.peers_json = Self::pretty(&v),
            Err(e) => self.set_error(format!("discover_peers failed: {e:?}")),
        }
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

        eframe::egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.tr("VGA 蜂群（Rust GUI）", "VGA Swarm (Rust GUI)"));
                ui.separator();

                ui.label(self.tr("语言：", "Lang:"));
                ui.selectable_value(&mut self.lang, UiLang::Zh, "中文");
                ui.selectable_value(&mut self.lang, UiLang::En, "EN");
                ui.separator();

                if ui.button(self.tr("刷新", "Refresh")).clicked() {
                    self.refresh_all();
                }

                if ui.button(self.tr("部署示例项目", "Deploy Sample Project")).clicked() {
                    self.deploy_sample_project();
                }

                if ui.button(self.tr("申请示例算力", "Request Sample Compute")).clicked() {
                    self.request_sample_compute();
                }

                ui.separator();
                let auto_label = self.tr("自动", "Auto");
                ui.checkbox(&mut self.auto_refresh, auto_label);
                ui.add(
                    eframe::egui::DragValue::new(&mut self.refresh_interval_secs)
                        .clamp_range(1..=60)
                        .suffix("s"),
                );

                if let Some(err) = &self.last_error {
                    ui.separator();
                    ui.colored_label(eframe::egui::Color32::RED, err);
                }
            });
        });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                // Left: ops
                cols[0].group(|ui| {
                    ui.heading(self.tr("任务", "Task"));
                    ui.horizontal(|ui| {
                        ui.label(self.tr("语言", "Language"));
                        ui.text_edit_singleline(&mut self.task_language);
                        ui.label(self.tr("目标", "Target"));
                        ui.text_edit_singleline(&mut self.task_target);
                    });
                    ui.label(self.tr("上下文", "Context"));
                    ui.text_edit_multiline(&mut self.task_context);

                    ui.horizontal(|ui| {
                        if ui.button(self.tr("执行", "Execute")).clicked() {
                            self.execute_task();
                        }
                        if ui.button(self.tr("提交", "Submit")).clicked() {
                            self.submit_task();
                        }
                    });
                });

                cols[0].add_space(8.0);

                cols[0].group(|ui| {
                    ui.heading(self.tr("金库（Vault）", "Vault"));
                    ui.horizontal(|ui| {
                        ui.label(self.tr("提供商", "Provider"));
                        ui.text_edit_singleline(&mut self.vault_provider);
                    });
                    ui.horizontal(|ui| {
                        ui.label(self.tr("密钥", "Key"));
                        ui.add(eframe::egui::TextEdit::singleline(&mut self.vault_key).password(true));
                    });
                    ui.horizontal(|ui| {
                        if ui.button(self.tr("保存", "Store")).clicked() {
                            self.vault_store();
                        }
                        if ui.button(self.tr("读取", "Retrieve")).clicked() {
                            self.vault_retrieve();
                        }
                        if ui.button(self.tr("列表", "List")).clicked() {
                            self.vault_list();
                        }
                        if ui.button(self.tr("删除", "Delete")).clicked() {
                            self.vault_delete();
                        }
                        if ui.button(self.tr("用量", "Usage")).clicked() {
                            self.vault_usage();
                        }
                    });
                    ui.separator();
                    eframe::egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.monospace(&self.vault_json);
                    });
                });

                cols[0].add_space(8.0);

                cols[0].group(|ui| {
                    ui.heading(self.tr("网络", "Network"));
                    if ui.button(self.tr("发现节点", "Discover Peers")).clicked() {
                        self.discover_peers();
                    }
                    eframe::egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.monospace(&self.peers_json);
                    });
                });

                cols[0].add_space(8.0);

                cols[0].group(|ui| {
                    ui.heading(self.tr("API 提供商", "API Providers"));

                    ui.horizontal(|ui| {
                        ui.label(self.tr("筛选：", "Filter:"));
                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::All, self.tr("全部", "All"));
                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::China, self.tr("中国", "China"));
                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::USA, self.tr("美国", "USA"));
                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::Global, self.tr("全球", "Global"));

                        if ui.button(self.tr("加载", "Load")).clicked() {
                            self.load_providers();
                        }
                    });

                    eframe::egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.monospace(&self.providers_json);
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(self.tr("Provider ID", "Provider ID"));
                        ui.text_edit_singleline(&mut self.provider_id);
                        if ui.button(self.tr("获取配置", "Get Config")).clicked() {
                            self.get_provider_config();
                        }
                        if ui.button(self.tr("设为默认", "Set Default")).clicked() {
                            self.set_default_provider();
                        }
                    });

                    eframe::egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.monospace(&self.provider_config_json);
                    });
                });

                cols[0].add_space(8.0);

                cols[0].group(|ui| {
                    ui.heading(self.tr("资源管理", "Resources"));

                    ui.horizontal(|ui| {
                        if ui.button(self.tr("发现节点", "Discover Nodes")).clicked() {
                            self.discover_nodes();
                        }
                        if ui.button(self.tr("列出节点", "List Nodes")).clicked() {
                            self.list_discovered_nodes();
                        }
                        if ui.button(self.tr("读取远程开关", "Get Remote Status")).clicked() {
                            self.get_remote_access_status();
                        }
                        if ui.button(self.tr("应用远程开关", "Set Remote Status")).clicked() {
                            self.set_remote_access();
                        }
                        ui.checkbox(&mut self.allow_remote_access, self.tr("允许远程", "Allow remote"));
                    });

                    ui.horizontal(|ui| {
                        ui.label(self.tr("策略", "Strategy"));
                        ui.selectable_value(
                            &mut self.balancing_strategy,
                            vangriten_ai_swarm::shared::models::BalancingStrategy::LeastLoaded,
                            self.tr("最小负载", "LeastLoaded"),
                        );
                        ui.selectable_value(
                            &mut self.balancing_strategy,
                            vangriten_ai_swarm::shared::models::BalancingStrategy::RoundRobin,
                            self.tr("轮询", "RoundRobin"),
                        );
                        ui.selectable_value(
                            &mut self.balancing_strategy,
                            vangriten_ai_swarm::shared::models::BalancingStrategy::Random,
                            self.tr("随机", "Random"),
                        );

                        if ui.button(self.tr("设置", "Set")).clicked() {
                            self.set_balancing_strategy();
                        }
                        if ui.button(self.tr("读取", "Get")).clicked() {
                            self.get_balancing_strategy();
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(self.tr("Group 名称", "Group name"));
                        ui.text_edit_singleline(&mut self.group_name);
                        ui.label(self.tr("最大成员", "Max"));
                        ui.add(eframe::egui::DragValue::new(&mut self.group_max_members).clamp_range(1..=10_000));
                        if ui.button(self.tr("创建组", "Create")).clicked() {
                            self.create_swarm_group();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(self.tr("Group ID", "Group ID"));
                        ui.text_edit_singleline(&mut self.group_id);
                        if ui.button(self.tr("加入", "Join")).clicked() {
                            self.join_swarm_group();
                        }
                        if ui.button(self.tr("离开", "Leave")).clicked() {
                            self.leave_swarm_group();
                        }
                        if ui.button(self.tr("列出组", "List")).clicked() {
                            self.list_swarm_groups();
                        }
                        if ui.button(self.tr("成员", "Members")).clicked() {
                            self.get_group_members();
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(self.tr("Pool 名称", "Pool name"));
                        ui.text_edit_singleline(&mut self.pool_name);
                        ui.label(self.tr("节点IDs (逗号)", "Node IDs (csv)"));
                        ui.text_edit_singleline(&mut self.pool_node_ids_csv);
                    });
                    ui.horizontal(|ui| {
                        if ui.button(self.tr("创建 Pool", "Create Pool")).clicked() {
                            self.create_resource_pool();
                        }
                        if ui.button(self.tr("列出 Pools", "List Pools")).clicked() {
                            self.list_resource_pools();
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(self.tr("任务类型", "Task type"));
                        ui.text_edit_singleline(&mut self.req_task_type);
                        ui.label(self.tr("优先级", "Priority"));
                        ui.selectable_value(&mut self.req_priority, vangriten_ai_swarm::shared::models::Priority::Low, self.tr("低", "Low"));
                        ui.selectable_value(&mut self.req_priority, vangriten_ai_swarm::shared::models::Priority::Medium, self.tr("中", "Medium"));
                        ui.selectable_value(&mut self.req_priority, vangriten_ai_swarm::shared::models::Priority::High, self.tr("高", "High"));
                        ui.selectable_value(&mut self.req_priority, vangriten_ai_swarm::shared::models::Priority::Critical, self.tr("紧急", "Critical"));
                    });
                    ui.horizontal(|ui| {
                        ui.label(self.tr("CPU cores", "CPU cores"));
                        ui.text_edit_singleline(&mut self.req_cpu_cores);
                        ui.label(self.tr("内存MB", "Memory MB"));
                        ui.text_edit_singleline(&mut self.req_memory_mb);
                        ui.checkbox(&mut self.req_gpu_required, self.tr("需要GPU", "GPU"));
                        ui.label(self.tr("GPU MB", "GPU MB"));
                        ui.text_edit_singleline(&mut self.req_gpu_memory_mb);
                    });
                    ui.horizontal(|ui| {
                        ui.label(self.tr("偏好模型(csv)", "Models (csv)"));
                        ui.text_edit_singleline(&mut self.req_preferred_models_csv);
                        if ui.button(self.tr("申请资源", "Request")).clicked() {
                            self.request_resources();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(self.tr("Allocation ID", "Allocation ID"));
                        ui.text_edit_singleline(&mut self.allocation_id);
                        if ui.button(self.tr("释放", "Release")).clicked() {
                            self.release_allocation();
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(self.tr("Node ID", "Node ID"));
                        ui.text_edit_singleline(&mut self.node_id);
                        if ui.button(self.tr("健康检查", "Health Check")).clicked() {
                            self.perform_health_check();
                        }
                    });

                    eframe::egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.monospace(&self.resource_json);
                    });
                });

                // Right: data
                cols[1].group(|ui| {
                    ui.heading(self.tr("蜂群", "Swarm"));
                    ui.monospace(&self.swarm_json);
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading(self.tr("代理", "Agents"));
                    eframe::egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.monospace(&self.agents_json);
                    });
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading(self.tr("项目", "Projects"));
                    eframe::egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                        ui.monospace(&self.projects_json);
                    });
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading(self.tr("租约", "Leases"));
                    eframe::egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                        ui.monospace(&self.leases_json);
                    });
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading(self.tr("任务列表", "Tasks"));
                    eframe::egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.monospace(&self.tasks_json);
                    });
                });
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
