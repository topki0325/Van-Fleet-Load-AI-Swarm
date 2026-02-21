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
struct VgaGuiApp {
    runtime: tokio::runtime::Runtime,
    services: std::sync::Arc<vangriten_ai_swarm::backend::BackendServices>,

    // Inputs
    task_language: String,
    task_target: String,
    task_context: String,

    vault_provider: String,
    vault_key: String,

    // View state
    last_error: Option<String>,
    swarm_json: String,
    agents_json: String,
    projects_json: String,
    leases_json: String,
    tasks_json: String,
    vault_json: String,
    peers_json: String,

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

            task_language: "rust".to_string(),
            task_target: "code".to_string(),
            task_context: "Generate a simple function".to_string(),

            vault_provider: "openai".to_string(),
            vault_key: String::new(),

            last_error: None,
            swarm_json: "(not loaded)".to_string(),
            agents_json: "(not loaded)".to_string(),
            projects_json: "(not loaded)".to_string(),
            leases_json: "(not loaded)".to_string(),
            tasks_json: "(not loaded)".to_string(),
            vault_json: "(not loaded)".to_string(),
            peers_json: "(not loaded)".to_string(),

            auto_refresh: true,
            refresh_interval_secs: 3,
            last_refresh_instant: std::time::Instant::now(),
        };

        app.refresh_all();
        app
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
                ui.heading("VGA Swarm (Rust GUI)");
                ui.separator();

                if ui.button("Refresh").clicked() {
                    self.refresh_all();
                }

                if ui.button("Deploy Sample Project").clicked() {
                    self.deploy_sample_project();
                }

                if ui.button("Request Sample Compute").clicked() {
                    self.request_sample_compute();
                }

                ui.separator();
                ui.checkbox(&mut self.auto_refresh, "Auto");
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
                    ui.heading("Task");
                    ui.horizontal(|ui| {
                        ui.label("Language");
                        ui.text_edit_singleline(&mut self.task_language);
                        ui.label("Target");
                        ui.text_edit_singleline(&mut self.task_target);
                    });
                    ui.label("Context");
                    ui.text_edit_multiline(&mut self.task_context);

                    ui.horizontal(|ui| {
                        if ui.button("Execute").clicked() {
                            self.execute_task();
                        }
                        if ui.button("Submit").clicked() {
                            self.submit_task();
                        }
                    });
                });

                cols[0].add_space(8.0);

                cols[0].group(|ui| {
                    ui.heading("Vault");
                    ui.horizontal(|ui| {
                        ui.label("Provider");
                        ui.text_edit_singleline(&mut self.vault_provider);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Key");
                        ui.add(eframe::egui::TextEdit::singleline(&mut self.vault_key).password(true));
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Store").clicked() {
                            self.vault_store();
                        }
                        if ui.button("Retrieve").clicked() {
                            self.vault_retrieve();
                        }
                        if ui.button("List").clicked() {
                            self.vault_list();
                        }
                        if ui.button("Delete").clicked() {
                            self.vault_delete();
                        }
                        if ui.button("Usage").clicked() {
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
                    ui.heading("Network");
                    if ui.button("Discover Peers").clicked() {
                        self.discover_peers();
                    }
                    eframe::egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.monospace(&self.peers_json);
                    });
                });

                // Right: data
                cols[1].group(|ui| {
                    ui.heading("Swarm");
                    ui.monospace(&self.swarm_json);
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading("Agents");
                    eframe::egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.monospace(&self.agents_json);
                    });
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading("Projects");
                    eframe::egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                        ui.monospace(&self.projects_json);
                    });
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading("Leases");
                    eframe::egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                        ui.monospace(&self.leases_json);
                    });
                });
                cols[1].add_space(8.0);
                cols[1].group(|ui| {
                    ui.heading("Tasks");
                    eframe::egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.monospace(&self.tasks_json);
                    });
                });
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}
