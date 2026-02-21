use crate::app::VgaGuiApp;
use crate::app_types::{UiLang, ProviderFilter, ActiveView};
use vangriten_ai_swarm::shared::models::{VaultOp, VaultResult};

impl VgaGuiApp {
    pub fn render_api_manager_window(&mut self, ctx: &eframe::egui::Context) {
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
                        let op = VaultOp::Store {
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
                        let op = VaultOp::Delete {
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
                        let op = VaultOp::Retrieve {
                            provider: self.api_provider.clone(),
                        };
                        match api_manager.vault_operation(op) {
                            Ok(VaultResult::Key(k)) => {
                                self.api_revealed_key = k;
                                self.api_status = self.tr("已读取", "Retrieved").to_string();
                            }
                            Ok(v) => self.api_status = format!("unexpected: {v:?}"),
                            Err(e) => self.api_status = format!("retrieve failed: {e:?}"),
                        }
                    }

                    if ui.button(label_list).clicked() {
                        match api_manager.vault_operation(VaultOp::List) {
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
}

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
