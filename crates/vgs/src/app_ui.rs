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

    fn render_new_project_wizard(&mut self, ctx: &eframe::egui::Context) {
        if !self.show_new_project_wizard {
            return;
        }

        let title = self.tr("新建项目", "New Project");
        let label_kind = self.tr("项目类型", "Project Type");
        let label_root = self.tr("工作区目录", "Workspace Folder");
        let label_name = self.tr("项目名称", "Project Name");
        let label_topic = match self.new_project_kind.as_str() {
            "文章快速写" => self.tr("文章主题", "Article Topic"),
            "网站原型快速搭建" => self.tr("网站功能/风格描述", "Website Features/Style"),
            "软件原型快速搭建" => self.tr("软件核心逻辑/需求", "Core Logic/Requirements"),
            "游戏原型快速搭建" => self.tr("游戏玩法/核心概念", "Gameplay/Core Concept"),
            _ => self.tr("项目目标", "Project Goal"),
        };
        let label_entities = self.tr("参与模型实体", "Participating Entities");
        let label_outline = match self.new_project_kind.as_str() {
            "文章快速写" => self.tr("目录实体", "Outline Entity"),
            _ => self.tr("架构/设计实体", "Architect/Designer Entity"),
        };
        let label_master = match self.new_project_kind.as_str() {
            "文章快速写" => self.tr("主拼合实体", "Master Merge Entity"),
            _ => self.tr("代码整合实体", "Master Integrator Entity"),
        };
        let label_groups = self.tr("小组数量", "Groups");

        let hint_root = self
            .tr("如 D:/work/articles", "e.g. D:/work/articles")
            .to_string();
        let hint_name = self
            .tr("如 my-new-project", "e.g. my-new-project")
            .to_string();
        let hint_topic = self
            .tr("描述一下要实现什么...", "Describe what to build...")
            .to_string();

        let kind_quick_write = self
            .tr("文章快速写", "Article Quick-Write")
            .to_string();
        let kind_web_proto = self
            .tr("网站原型快速搭建", "Website Prototype Quick-Build")
            .to_string();
        let kind_soft_proto = self
            .tr("软件原型快速搭建", "Software Prototype Quick-Build")
            .to_string();
        let kind_game_proto = self
            .tr("游戏原型快速搭建", "Game Prototype Quick-Build")
            .to_string();

        let mut open = self.show_new_project_wizard;

        eframe::egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(eframe::egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .default_width(720.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(label_kind);
                        eframe::egui::ComboBox::from_id_source("new_project_kind")
                            .selected_text(&self.new_project_kind)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.new_project_kind,
                                    "文章快速写".to_string(),
                                    &kind_quick_write,
                                );
                                ui.selectable_value(
                                    &mut self.new_project_kind,
                                    "网站原型快速搭建".to_string(),
                                    &kind_web_proto,
                                );
                                ui.selectable_value(
                                    &mut self.new_project_kind,
                                    "软件原型快速搭建".to_string(),
                                    &kind_soft_proto,
                                );
                                ui.selectable_value(
                                    &mut self.new_project_kind,
                                    "游戏原型快速搭建".to_string(),
                                    &kind_game_proto,
                                );
                            });
                    });

                    ui.add_space(6.0);

                    eframe::egui::Grid::new("new_project_grid")
                        .num_columns(2)
                        .spacing([10.0, 6.0])
                        .show(ui, |ui| {
                            ui.label(label_root);
                            ui.horizontal(|ui| {
                                ui.add(
                                    eframe::egui::TextEdit::singleline(&mut self.new_project_root_dir)
                                        .desired_width(380.0)
                                        .hint_text(&hint_root),
                                );
                                if ui.button("📂").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.new_project_root_dir = path.display().to_string();
                                    }
                                }
                            });
                            ui.end_row();

                            ui.label(label_name);
                            ui.add(
                                eframe::egui::TextEdit::singleline(&mut self.new_project_name)
                                    .desired_width(460.0)
                                    .hint_text(&hint_name),
                            );
                            ui.end_row();

                            ui.label(label_topic);
                            ui.add(
                                eframe::egui::TextEdit::singleline(&mut self.article_topic)
                                    .desired_width(560.0)
                                    .hint_text(&hint_topic),
                            );
                            ui.end_row();
                        });

                    ui.separator();
                    ui.strong(label_entities);

                    if self.ai_entities.is_empty() {
                        ui.label(self.tr(
                            "未发现任何 AI 个体。请先到 API 视图创建/加载。",
                            "No AI entities found. Create/load them in the API view first.",
                        ));
                    } else {
                        let mut changed = false;
                        eframe::egui::ScrollArea::vertical()
                            .id_source("np_entities_scroll")
                            .max_height(150.0)
                            .show(ui, |ui| {
                                for e in &self.ai_entities {
                                    let mut checked = self.article_selected_entities.contains(&e.name);
                                    let label = format!("{}  ({}/{})", e.name, e.provider, e.model);
                                    if ui.checkbox(&mut checked, label).clicked() {
                                        changed = true;
                                        if checked {
                                            self.article_selected_entities.push(e.name.clone());
                                        } else {
                                            self.article_selected_entities.retain(|n| n != &e.name);
                                        }
                                    }
                                }
                            });

                        if changed {
                            if !self.article_selected_entities.contains(&self.article_outline_entity) {
                                self.article_outline_entity = self
                                    .article_selected_entities
                                    .first()
                                    .cloned()
                                    .unwrap_or_default();
                            }
                            if !self.article_selected_entities.contains(&self.article_master_entity) {
                                self.article_master_entity = self
                                    .article_selected_entities
                                    .first()
                                    .cloned()
                                    .unwrap_or_default();
                            }
                            self.article_group_assignments.clear();
                        }

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(label_outline);
                            eframe::egui::ComboBox::from_id_source("np_outline_entity")
                                .selected_text(if self.article_outline_entity.is_empty() {
                                    self.tr("(未选择)", "(none)")
                                } else {
                                    &self.article_outline_entity
                                })
                                .show_ui(ui, |ui| {
                                    for n in &self.article_selected_entities {
                                        ui.selectable_value(&mut self.article_outline_entity, n.clone(), n);
                                    }
                                });

                            ui.add_space(12.0);

                            ui.label(label_master);
                            eframe::egui::ComboBox::from_id_source("np_master_entity")
                                .selected_text(if self.article_master_entity.is_empty() {
                                    self.tr("(未选择)", "(none)")
                                } else {
                                    &self.article_master_entity
                                })
                                .show_ui(ui, |ui| {
                                    for n in &self.article_selected_entities {
                                        ui.selectable_value(&mut self.article_master_entity, n.clone(), n);
                                    }
                                });
                        });

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(label_groups);
                            ui.add(
                                eframe::egui::DragValue::new(&mut self.article_groups_count)
                                    .clamp_range(1u8..=10u8)
                                    .speed(0.1),
                            );
                        });

                        let groups_count = self.article_groups_count.clamp(1, 10) as usize;
                        if self.article_group_assignments.len() != groups_count {
                            let mut pool: Vec<String> = self
                                .article_selected_entities
                                .iter()
                                .filter(|n| *n != &self.article_outline_entity && *n != &self.article_master_entity)
                                .cloned()
                                .collect();
                            if pool.is_empty() {
                                pool.push(self.article_master_entity.clone());
                            }
                            self.article_group_assignments = (0..groups_count)
                                .map(|i| pool[i % pool.len()].clone())
                                .collect();
                        }

                        let options = self.article_selected_entities.clone();
                        eframe::egui::CollapsingHeader::new(self.tr("小组分配", "Group Assignment"))
                            .id_source("np_group_assign")
                            .default_open(true)
                            .show(ui, |ui| {
                                for i in 0..groups_count {
                                    let current = self
                                        .article_group_assignments
                                        .get(i)
                                        .cloned()
                                        .unwrap_or_default();
                                    ui.horizontal(|ui| {
                                        ui.label(format!("{} {}/{}", self.tr("小组", "Group"), i + 1, groups_count));
                                        eframe::egui::ComboBox::from_id_source(format!("np_group_{i}"))
                                            .selected_text(current)
                                            .show_ui(ui, |ui| {
                                                for n in &options {
                                                    ui.selectable_value(
                                                        &mut self.article_group_assignments[i],
                                                        n.clone(),
                                                        n,
                                                    );
                                                }
                                            });
                                    });
                                }
                            });

                        if self.article_outline_entity == self.article_master_entity
                            && !self.article_outline_entity.is_empty()
                        {
                            ui.colored_label(
                                eframe::egui::Color32::YELLOW,
                                self.tr(
                                    "⚠ 目录实体与主拼合实体相同也可以，但通常建议不同。",
                                    "⚠ Outline and master are the same; allowed but usually better to separate.",
                                ),
                            );
                        }
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(self.tr("🚀 创建并提交任务", "🚀 Create & Submit")).clicked() {
                            match self.new_project_kind.as_str() {
                                "文章快速写" => self.create_article_quick_project(),
                                _ => self.create_prototype_quick_project(),
                            }
                        }
                        if ui.button(self.tr("关闭", "Close")).clicked() {
                            self.show_new_project_wizard = false;
                        }
                    });
                });
            });

        self.show_new_project_wizard = open;
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
        self.render_new_project_wizard(ctx);

        eframe::egui::SidePanel::left("left_nav")
            .resizable(false)
            .default_width(170.0)
            .show(ctx, |ui| {
                if ui.button(self.tr("➕ 新建项目", "➕ New Project")).clicked() {
                    self.show_new_project_wizard = true;
                }
                ui.add_space(6.0);
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
                            ui.heading(self.tr("AI 个体管理", "AI Entity Manager"));
                            ui.separator();

                            let api_manager = self.services.api_manager.clone();
                            let initialized = api_manager.vault_is_initialized();
                            let unlocked = api_manager.vault_is_unlocked();

                            // ── 1. 密钥库状态条 ───────────────────────────────────────────
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    let (color, vault_label) = if !initialized {
                                        (eframe::egui::Color32::GRAY,
                                         self.tr("⬜ 密钥库未初始化", "⬜ Vault Not Initialized"))
                                    } else if unlocked {
                                        (eframe::egui::Color32::from_rgb(0, 180, 60),
                                         self.tr("🔓 密钥库已解锁", "🔓 Vault Unlocked"))
                                    } else {
                                        (eframe::egui::Color32::from_rgb(220, 80, 0),
                                         self.tr("🔒 密钥库已锁定", "🔒 Vault Locked"))
                                    };
                                    ui.colored_label(color, vault_label);
                                    ui.with_layout(
                                        eframe::egui::Layout::right_to_left(eframe::egui::Align::Center),
                                        |ui| {
                                            if ui.small_button(self.tr("⚙ 高级", "⚙ Advanced")).clicked() {
                                                self.show_api_manager = true;
                                            }
                                        },
                                    );
                                });
                                ui.add_space(2.0);
                                if !initialized {
                                    ui.label(self.tr(
                                        "首次使用：初始化密钥库后才能添加 AI 个体。",
                                        "First time: initialize the Vault before adding AI entities.",
                                    ));
                                    if ui.button(self.tr("🚀 初始化密钥库", "🚀 Initialize Vault")).clicked() {
                                        self.show_api_manager = true;
                                    }
                                } else if !unlocked {
                                    ui.horizontal(|ui| {
                                        ui.label(self.tr("密码:", "Password:"));
                                        ui.add(
                                            eframe::egui::TextEdit::singleline(&mut self.api_password)
                                                .password(true)
                                                .desired_width(160.0),
                                        );
                                        if ui.button(self.tr("🔓 解锁", "🔓 Unlock")).clicked() {
                                            match api_manager.vault_unlock(&self.api_password) {
                                                Ok(()) => {
                                                    self.api_password.clear();
                                                    self.load_stored_keys();
                                                    self.load_entities();
                                                    self.load_custom_providers();
                                                    self.load_providers();
                                                    self.api_quick_status =
                                                        self.tr("✅ 已解锁", "✅ Unlocked").to_string();
                                                }
                                                Err(e) => self.api_quick_status = format!("❌ {e:?}"),
                                            }
                                        }
                                    });
                                } else {
                                    ui.horizontal(|ui| {
                                        ui.label(self.tr(
                                            "密钥库已解锁。可管理所有 AI 个体及其密钥。",
                                            "Vault unlocked. Manage all AI entities and keys.",
                                        ));
                                        if ui.small_button(self.tr("🔒 锁定", "🔒 Lock")).clicked() {
                                            api_manager.vault_lock();
                                            self.api_revealed_key.clear();
                                            self.api_stored_providers.clear();
                                            self.entity_selected = None;
                                            self.api_quick_status =
                                                self.tr("🔒 已锁定", "🔒 Locked").to_string();
                                        }
                                    });
                                }
                            });

                            ui.add_space(6.0);

                            if unlocked {
                                // ── 2. AI 个体列表 ────────────────────────────────────────
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(self.tr("AI 个体列表", "AI Entity List"));
                                        if ui.small_button(self.tr("🔄 刷新", "🔄 Refresh")).clicked() {
                                            self.load_entities();
                                            self.load_stored_keys();
                                        }
                                    });
                                    ui.separator();

                                    if self.ai_entities.is_empty() {
                                        ui.label(self.tr(
                                            "暂无 AI 个体。在下方表单中添加第一个。",
                                            "No AI entities yet. Add one using the form below.",
                                        ));
                                    } else {
                                        let stored = self.api_stored_providers.clone();
                                        let entities = self.ai_entities.clone();
                                        let selected = self.entity_selected;

                                        eframe::egui::ScrollArea::vertical()
                                            .id_source("entity_list_scroll")
                                            .max_height(200.0)
                                            .show(ui, |ui| {
                                                eframe::egui::Grid::new("entity_list_grid")
                                                    .num_columns(6)
                                                    .striped(true)
                                                    .spacing([10.0, 4.0])
                                                    .show(ui, |ui| {
                                                        ui.strong(self.tr("名称", "Name"));
                                                        ui.strong(self.tr("供应商", "Provider"));
                                                        ui.strong(self.tr("模型", "Model"));
                                                        ui.strong(self.tr("备注", "Note"));
                                                        ui.strong("🔗").on_hover_text(self.tr("自定义转发地址", "Custom relay URL"));
                                                        ui.strong(self.tr("操作", "Actions"));
                                                        ui.end_row();

                                                        for (idx, entity) in entities.iter().enumerate() {
                                                            let is_selected = selected == Some(idx);
                                                            let has_key = stored.contains(&entity.name);

                                                            // 名称（高亮选中行）
                                                            let name_label = if has_key {
                                                                format!("✔ {}", entity.name)
                                                            } else {
                                                                format!("— {}", entity.name)
                                                            };
                                                            let resp = ui.selectable_label(
                                                                is_selected,
                                                                &name_label,
                                                            );
                                                            if resp.clicked() {
                                                                if is_selected {
                                                                    self.entity_selected = None;
                                                                } else {
                                                                    self.entity_selected = Some(idx);
                                                                    self.api_provider =
                                                                        entity.provider.clone();
                                                                    self.provider_id =
                                                                        entity.provider.clone();
                                                                    self.entity_name_input =
                                                                        entity.name.clone();
                                                                    self.entity_model_input =
                                                                        entity.model.clone();
                                                                    self.entity_note_input =
                                                                        entity.note.clone();
                                                                    self.entity_custom_url_input =
                                                                        entity.custom_base_url.clone().unwrap_or_default();
                                                                    self.entity_key_header_input =
                                                                        entity.key_header.clone().unwrap_or_default();
                                                                    self.entity_key_prefix_input =
                                                                        entity.key_prefix.clone().unwrap_or_default();
                                                                    self.api_key_input.clear();
                                                                    self.api_revealed_key.clear();
                                                                }
                                                            }

                                                            ui.monospace(&entity.provider);
                                                            ui.monospace(&entity.model);
                                                            ui.label(if entity.note.is_empty() {
                                                                "—"
                                                            } else {
                                                                &entity.note
                                                            });
                                                            // Relay URL indicator
                                                            let relay_text = if let Some(url) = &entity.custom_base_url {
                                                                let short = if url.len() > 28 { format!("{}...", &url[..28]) } else { url.clone() };
                                                                ui.label("🔗").on_hover_text(url.as_str());
                                                                let _ = short;
                                                            } else {
                                                                ui.label("—");
                                                            };
                                                            let _ = relay_text;

                                                            ui.horizontal(|ui| {
                                                                let am = self.services.api_manager.clone();
                                                                if has_key {
                                                                    if ui
                                                                        .small_button("👁")
                                                                        .on_hover_text(self.tr("查看密钥", "Reveal key"))
                                                                        .clicked()
                                                                    {
                                                                        match am.vault_operation(VaultOp::Retrieve {
                                                                            provider: entity.name.clone(),
                                                                        }) {
                                                                            Ok(VaultResult::Key(k)) => {
                                                                                self.entity_selected = Some(idx);
                                                                                self.api_provider = entity.provider.clone();
                                                                                self.entity_name_input = entity.name.clone();
                                                                                self.entity_model_input = entity.model.clone();
                                                                                self.entity_note_input = entity.note.clone();
                                                                                self.api_revealed_key = k;
                                                                                self.api_quick_status = format!(
                                                                                    "{}: {}",
                                                                                    entity.name,
                                                                                    self.tr("密钥已读取", "key revealed")
                                                                                );
                                                                            }
                                                                            Err(e) => {
                                                                                self.api_quick_status =
                                                                                    format!("❌ {e:?}")
                                                                            }
                                                                            _ => {}
                                                                        }
                                                                    }
                                                                }
                                                                if ui
                                                                    .small_button("🗑")
                                                                    .on_hover_text(self.tr("删除此个体", "Delete entity"))
                                                                    .clicked()
                                                                {
                                                                    let name = entity.name.clone();
                                                                    self.delete_entity(&name);
                                                                    if self.entity_selected == Some(idx) {
                                                                        self.entity_selected = None;
                                                                    }
                                                                }
                                                            });
                                                            ui.end_row();
                                                        }
                                                    });
                                            });

                                    }
                                });

                                ui.add_space(6.0);

                                // ── 3. 添加 / 编辑 AI 个体表单 ───────────────────────────
                                let form_title = if self.entity_selected.is_some() {
                                    self.tr("✏ 编辑 AI 个体", "✏ Edit AI Entity")
                                } else {
                                    self.tr("➕ 添加 AI 个体", "➕ Add AI Entity")
                                };
                                ui.group(|ui| {
                                    ui.strong(form_title);
                                    ui.separator();

                                    eframe::egui::Grid::new("entity_form_grid")
                                        .num_columns(2)
                                        .spacing([8.0, 6.0])
                                        .show(ui, |ui| {
                                            let hint_name    = self.tr("唯一名称, 如 gpt4-coder", "unique name, e.g. gpt4-coder");
                                            let hint_note    = self.tr("可选说明", "optional note");
                                            let hint_key     = self.tr("编辑时留空则不更改密钥", "leave blank when editing to keep existing key");
                                            let show_label   = self.tr("显示明文", "Show");
                                            ui.label(self.tr("名称 *", "Name *"));
                                            ui.add(
                                                eframe::egui::TextEdit::singleline(
                                                    &mut self.entity_name_input,
                                                )
                                                .desired_width(220.0)
                                                .hint_text(hint_name),
                                            );
                                            ui.end_row();

                                            ui.label(self.tr("供应商 *", "Provider *"));
                                            ui.horizontal(|ui| {
                                                ui.add(
                                                    eframe::egui::TextEdit::singleline(
                                                        &mut self.api_provider,
                                                    )
                                                    .desired_width(160.0)
                                                    .hint_text("openai / deepseek / ..."),
                                                );
                                                self.provider_id = self.api_provider.clone();
                                                let picker_label = if self.show_provider_picker {
                                                    self.tr("▲ 收起", "▲ Close")
                                                } else {
                                                    self.tr("📋 选择供应商", "📋 Browse")
                                                };
                                                if ui.small_button(picker_label).clicked() {
                                                    self.show_provider_picker = !self.show_provider_picker;
                                                }
                                            });
                                            ui.end_row();

                                            // ── 供应商选择器（展开时显示） ──────────────────────
                                            if self.show_provider_picker {
                                                ui.label(""); // left column placeholder
                                                ui.vertical(|ui| {
                                                    // Parse built-in providers from providers_json
                                                    let builtin: Vec<serde_json::Value> = {
                                                        let v: serde_json::Value =
                                                            serde_json::from_str(&self.providers_json)
                                                            .unwrap_or(serde_json::Value::Array(vec![]));
                                                        match v { serde_json::Value::Array(a) => a, _ => vec![] }
                                                    };
                                                    if !builtin.is_empty() {
                                                        ui.label(self.tr("🌐 内置官方供应商", "🌐 Built-in Providers"));
                                                        eframe::egui::ScrollArea::vertical()
                                                            .id_source("picker_builtin_scroll")
                                                            .max_height(140.0)
                                                            .show(ui, |ui| {
                                                                eframe::egui::Grid::new("picker_builtin_grid")
                                                                    .num_columns(3)
                                                                    .striped(true)
                                                                    .spacing([8.0, 2.0])
                                                                    .show(ui, |ui| {
                                                                        ui.strong(self.tr("名称", "Name"));
                                                                        ui.strong(self.tr("地址", "Endpoint"));
                                                                        ui.strong("");
                                                                        ui.end_row();
                                                                        for item in &builtin {
                                                                            let id = item["id"].as_str().unwrap_or("");
                                                                            let name = item["name"].as_str().unwrap_or(id);
                                                                            let ep = item["api_endpoint"].as_str().unwrap_or("");
                                                                            ui.label(name);
                                                                            ui.label(
                                                                                eframe::egui::RichText::new(
                                                                                    if ep.len() > 38 { format!("{}...", &ep[..38]) } else { ep.to_string() }
                                                                                ).monospace().small(),
                                                                            ).on_hover_text(ep);
                                                                            if ui.small_button(self.tr("选择", "Select")).clicked() {
                                                                                self.api_provider = id.to_string();
                                                                                self.provider_id  = id.to_string();
                                                                                // Built-in: clear custom URL so system uses its own endpoint
                                                                                self.entity_custom_url_input.clear();
                                                                                self.entity_key_header_input.clear();
                                                                                self.entity_key_prefix_input.clear();
                                                                                // Suggest first model if model not yet set
                                                                                if self.entity_model_input.is_empty() {
                                                                                    if let Some(m) = item["models"].as_array().and_then(|a| a.first()).and_then(|v| v.as_str()) {
                                                                                        self.entity_model_input = m.to_string();
                                                                                    }
                                                                                }
                                                                                // Auto-suggest entity name if blank
                                                                                if self.entity_name_input.is_empty() {
                                                                                    let m = self.entity_model_input.clone();
                                                                                    if !m.is_empty() {
                                                                                        self.entity_name_input = format!("{m}-{id}");
                                                                                    }
                                                                                }
                                                                                self.show_provider_picker = false;
                                                                            }
                                                                            ui.end_row();
                                                                        }
                                                                    });
                                                            });
                                                    }
                                                    let custom_pvds = self.custom_providers.clone();
                                                    if !custom_pvds.is_empty() {
                                                        ui.add_space(4.0);
                                                        ui.label(self.tr("🔧 自建转发商", "🔧 Custom Relay Providers"));
                                                        eframe::egui::ScrollArea::vertical()
                                                            .id_source("picker_custom_scroll")
                                                            .max_height(100.0)
                                                            .show(ui, |ui| {
                                                                eframe::egui::Grid::new("picker_custom_grid")
                                                                    .num_columns(3)
                                                                    .striped(true)
                                                                    .spacing([8.0, 2.0])
                                                                    .show(ui, |ui| {
                                                                        ui.strong(self.tr("名称", "Name"));
                                                                        ui.strong(self.tr("地址", "Base URL"));
                                                                        ui.strong("");
                                                                        ui.end_row();
                                                                        for cp in &custom_pvds {
                                                                            ui.label(&cp.name);
                                                                            ui.label(
                                                                                eframe::egui::RichText::new(
                                                                                    if cp.base_url.len() > 38 { format!("{}...", &cp.base_url[..38]) } else { cp.base_url.clone() }
                                                                                ).monospace().small(),
                                                                            ).on_hover_text(&cp.base_url);
                                                                            if ui.small_button(self.tr("选择", "Select")).clicked() {
                                                                                self.api_provider = cp.id.clone();
                                                                                self.provider_id  = cp.id.clone();
                                                                                self.entity_custom_url_input = cp.base_url.clone();
                                                                                self.entity_key_header_input = cp.key_header.clone();
                                                                                self.entity_key_prefix_input = cp.key_prefix.clone();
                                                                                if self.entity_model_input.is_empty() && !cp.models_hint.is_empty() {
                                                                                    self.entity_model_input = cp.models_hint
                                                                                        .split([',', ' '])
                                                                                        .find(|s| !s.is_empty())
                                                                                        .unwrap_or("").to_string();
                                                                                }
                                                                                // Auto-suggest entity name if blank
                                                                                if self.entity_name_input.is_empty() {
                                                                                    let m = self.entity_model_input.clone();
                                                                                    let p = cp.id.clone();
                                                                                    if !m.is_empty() {
                                                                                        self.entity_name_input = format!("{m}-{p}");
                                                                                    } else if !p.is_empty() {
                                                                                        self.entity_name_input = p;
                                                                                    }
                                                                                }
                                                                                self.show_provider_picker = false;
                                                                            }
                                                                            ui.end_row();
                                                                        }
                                                                    });
                                                            });
                                                    }
                                                    if builtin.is_empty() && self.custom_providers.is_empty() {
                                                        ui.label(self.tr(
                                                            "尚未加载。请先解锁密鑰库或点击「加载」。",
                                                            "Not loaded. Unlock vault or click Load Provider List.",
                                                        ));
                                                    }
                                                });
                                                ui.end_row();
                                            }

                                            ui.label(self.tr("模型", "Model"));
                                            ui.add(
                                                eframe::egui::TextEdit::singleline(
                                                    &mut self.entity_model_input,
                                                )
                                                .desired_width(220.0)
                                                .hint_text("gpt-4o / deepseek-chat / ..."),
                                            );
                                            ui.end_row();

                                            ui.label(self.tr("备注", "Note"));
                                            ui.add(
                                                eframe::egui::TextEdit::singleline(
                                                    &mut self.entity_note_input,
                                                )
                                                .desired_width(220.0)
                                                .hint_text(hint_note),
                                            );
                                            ui.end_row();

                                            ui.label(self.tr("API Key *", "API Key *"));
                                            ui.horizontal(|ui| {
                                                ui.add(
                                                    eframe::egui::TextEdit::singleline(
                                                        &mut self.api_key_input,
                                                    )
                                                    .password(!self.api_show_plaintext)
                                                    .desired_width(280.0)
                                                    .hint_text(hint_key),
                                                );
                                                ui.checkbox(&mut self.api_show_plaintext, show_label);
                                            });
                                            ui.end_row();

                                            // ── 转发商自定义字段（可选） ─────────────────────────
                                            ui.label(self.tr("🔗 转发地址", "🔗 Custom URL"))
                                                .on_hover_text(self.tr(
                                                    "自建转发商的 API 基址，留空表示使用官方地址",
                                                    "Base URL for custom relay; leave blank to use official endpoint",
                                                ));
                                            ui.add(
                                                eframe::egui::TextEdit::singleline(
                                                    &mut self.entity_custom_url_input,
                                                )
                                                .desired_width(340.0)
                                                .hint_text("https://relay.example.com/v1"),
                                            );
                                            ui.end_row();

                                            ui.label(self.tr("Key 请求头", "Key Header"))
                                                .on_hover_text(self.tr(
                                                    "HTTP 请求头字段名，默认 Authorization",
                                                    "HTTP header name for the key; default: Authorization",
                                                ));
                                            ui.add(
                                                eframe::egui::TextEdit::singleline(
                                                    &mut self.entity_key_header_input,
                                                )
                                                .desired_width(220.0)
                                                .hint_text("Authorization (默认/default)"),
                                            );
                                            ui.end_row();

                                            ui.label(self.tr("Key 前缀", "Key Prefix"))
                                                .on_hover_text(self.tr(
                                                    "Key 值前缀，默认 Bearer，留空表示裸 key",
                                                    "Prefix before the key value; default Bearer; leave blank for raw key",
                                                ));
                                            ui.add(
                                                eframe::egui::TextEdit::singleline(
                                                    &mut self.entity_key_prefix_input,
                                                )
                                                .desired_width(160.0)
                                                .hint_text("Bearer (默认/default)"),
                                            );
                                            ui.end_row();
                                        });

                                    if !self.api_revealed_key.is_empty() {
                                        ui.horizontal(|ui| {
                                            ui.label(self.tr("当前密钥:", "Current Key:"));
                                            ui.add(
                                                eframe::egui::TextEdit::singleline(
                                                    &mut self.api_revealed_key,
                                                )
                                                .password(!self.api_show_plaintext)
                                                .desired_width(f32::INFINITY),
                                            );
                                        });
                                    }

                                    ui.separator();
                                    ui.horizontal(|ui| {
                                        if ui.button(self.tr("💾 保存", "💾 Save")).clicked() {
                                            // if editing and key blank, keep existing
                                            if self.api_key_input.is_empty() && self.entity_selected.is_some() {
                                                // just update metadata, no key change
                                                let name = self.entity_name_input.trim().to_string();
                                                let c_url = self.entity_custom_url_input.trim();
                                                let k_hdr = self.entity_key_header_input.trim();
                                                let k_pfx = self.entity_key_prefix_input.trim();
                                                let entity = crate::app_types::AiEntity {
                                                    name: name.clone(),
                                                    provider: self.api_provider.trim().to_string(),
                                                    model: self.entity_model_input.trim().to_string(),
                                                    note: self.entity_note_input.trim().to_string(),
                                                    custom_base_url: if c_url.is_empty() { None } else { Some(c_url.to_string()) },
                                                    key_header: if k_hdr.is_empty() { None } else { Some(k_hdr.to_string()) },
                                                    key_prefix: if k_pfx.is_empty() { None } else { Some(k_pfx.to_string()) },
                                                };
                                                if let Some(pos) = self.ai_entities.iter().position(|e| e.name == name) {
                                                    self.ai_entities[pos] = entity;
                                                } else {
                                                    self.ai_entities.push(entity);
                                                }
                                                self.save_entities();
                                                self.entity_selected = None;
                                                self.entity_name_input.clear();
                                                self.entity_model_input.clear();
                                                self.entity_note_input.clear();
                                                self.entity_custom_url_input.clear();
                                                self.entity_key_header_input.clear();
                                                self.entity_key_prefix_input.clear();
                                                self.api_quick_status = format!("✅ 已更新: {name}");
                                            } else {
                                                self.add_entity();
                                                self.entity_selected = None;
                                            }
                                        }
                                        if ui.button(self.tr("✖ 清空", "✖ Clear")).clicked() {
                                            self.entity_selected = None;
                                            self.entity_name_input.clear();
                                            self.entity_model_input.clear();
                                            self.entity_note_input.clear();
                                            self.entity_custom_url_input.clear();
                                            self.entity_key_header_input.clear();
                                            self.entity_key_prefix_input.clear();
                                            self.api_key_input.clear();
                                            self.api_revealed_key.clear();
                                            self.api_quick_status.clear();
                                        }
                                    });

                                    // ── 裂变模式 ────────────────────────────────────
                                    ui.separator();
                                    let burst_tip = self.tr(
                                        "创建 N 个编号副本，共享同一 Key。适合单个 API Key 最高 N 路并发",
                                        "Create N numbered clones sharing the same key for max concurrent calls",
                                    ).to_string();
                                    let burst_sfx = self.tr(" 个", " clones").to_string();
                                    ui.horizontal(|ui| {
                                        ui.label(self.tr("🔀 裂变并发:", "🔀 Burst:"));
                                        ui.add(
                                            eframe::egui::DragValue::new(&mut self.entity_burst_count)
                                                .clamp_range(1u8..=10u8)
                                                .suffix(&burst_sfx)
                                                .speed(0.1),
                                        ).on_hover_text(&burst_tip);
                                        let burst_btn_label = format!(
                                            "{}-1 … {}-{}",
                                            self.entity_name_input.trim(),
                                            self.entity_name_input.trim(),
                                            self.entity_burst_count,
                                        );
                                        if ui.button(
                                            self.tr("🔀 裂变创建", "🔀 Burst Create")
                                        ).on_hover_text(format!("{burst_tip}\n{burst_btn_label}"))
                                        .clicked() {
                                            self.burst_add_entities();
                                            self.entity_selected = None;
                                        }
                                    });
                                });

                                ui.add_space(4.0);

                                // ── 4. 已知供应商快速参考（折叠）────────────────────────
                                eframe::egui::CollapsingHeader::new(
                                    self.tr("📖 供应商参考列表", "📖 Provider Reference"),
                                )
                                .id_source("provider_ref_header")
                                .default_open(false)
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        let label_all = self.tr("全部", "All");
                                        let label_cn  = self.tr("中国", "China");
                                        let label_us  = self.tr("美国", "USA");
                                        let label_gl  = self.tr("全球", "Global");
                                        ui.label(self.tr("筛选:", "Filter:"));
                                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::All,    label_all);
                                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::China,  label_cn);
                                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::USA,    label_us);
                                        ui.selectable_value(&mut self.provider_filter, ProviderFilter::Global, label_gl);
                                        if ui.small_button(self.tr("加载", "Load")).clicked() {
                                            self.load_providers();
                                            self.load_custom_providers();
                                        }
                                    });
                                    if self.providers_json != "(not loaded)" {
                                        // parse and show as clickable buttons
                                        let val: serde_json::Value =
                                            serde_json::from_str(&self.providers_json)
                                                .unwrap_or(serde_json::Value::Array(vec![]));
                                        if let serde_json::Value::Array(arr) = val {
                                            ui.horizontal_wrapped(|ui| {
                                                for item in &arr {
                                                    if let (Some(id), Some(name)) = (
                                                        item["id"].as_str(),
                                                        item["name"].as_str(),
                                                    ) {
                                                        if ui
                                                            .small_button(format!("{name} ({id})"))
                                                            .on_hover_text(self.tr(
                                                                "点击填入供应商",
                                                                "Click to fill provider",
                                                            ))
                                                            .clicked()
                                                        {
                                                            self.api_provider = id.to_string();
                                                            self.provider_id  = id.to_string();
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    if self.provider_config_json != "(not loaded)" && !self.provider_config_json.is_empty() {
                                        eframe::egui::CollapsingHeader::new(
                                            self.tr("配置详情", "Config Detail"),
                                        )
                                        .id_source("provider_cfg_inner")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            ui.horizontal_wrapped(|ui| {
                                                if ui.small_button(self.tr("获取配置", "Get Config")).clicked() {
                                                    self.get_provider_config();
                                                }
                                                if ui.small_button(self.tr("设为默认", "Set Default")).clicked() {
                                                    self.set_default_provider();
                                                }
                                            });
                                            eframe::egui::ScrollArea::vertical()
                                                .id_source("prov_cfg_scroll")
                                                .max_height(160.0)
                                                .show(ui, |ui| {
                                                    ui.monospace(&self.provider_config_json);
                                                });
                                        });
                                    }

                                    // ── 新建/管理自建转发供应商 ──────────────────────────
                                    eframe::egui::CollapsingHeader::new(
                                        self.tr("➕ 新建转发供应商", "➕ New Custom Provider"),
                                    )
                                    .id_source("new_custom_provider_hdr")
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        let hint_cp_name  = self.tr("可选", "optional").to_string();
                                        let hint_cp_note  = self.tr("可选说明", "optional").to_string();
                                        eframe::egui::Grid::new("cp_form_grid")
                                            .num_columns(2)
                                            .spacing([8.0, 4.0])
                                            .show(ui, |ui| {
                                                ui.label(self.tr("ID *", "ID *"))
                                                    .on_hover_text(self.tr("供应商标识，如 my-relay", "e.g. my-relay"));
                                                ui.add(eframe::egui::TextEdit::singleline(&mut self.cp_id_input)
                                                    .desired_width(200.0).hint_text("my-relay"));
                                                ui.end_row();

                                                ui.label(self.tr("显示名", "Display Name"));
                                                ui.add(eframe::egui::TextEdit::singleline(&mut self.cp_name_input)
                                                    .desired_width(200.0).hint_text(hint_cp_name));
                                                ui.end_row();

                                                ui.label(self.tr("基础地址 *", "Base URL *"));
                                                ui.add(eframe::egui::TextEdit::singleline(&mut self.cp_url_input)
                                                    .desired_width(320.0).hint_text("https://relay.example.com/v1"));
                                                ui.end_row();

                                                ui.label(self.tr("Key 请求头", "Key Header"))
                                                    .on_hover_text(self.tr("默认 Authorization", "Default: Authorization"));
                                                ui.add(eframe::egui::TextEdit::singleline(&mut self.cp_key_header_input)
                                                    .desired_width(180.0).hint_text("Authorization"));
                                                ui.end_row();

                                                ui.label(self.tr("Key 前缀", "Key Prefix"))
                                                    .on_hover_text(self.tr("默认 Bearer，留空=裸key", "Default Bearer; blank=raw key"));
                                                ui.add(eframe::egui::TextEdit::singleline(&mut self.cp_key_prefix_input)
                                                    .desired_width(120.0).hint_text("Bearer"));
                                                ui.end_row();

                                                ui.label(self.tr("模型提示", "Models Hint"))
                                                    .on_hover_text(self.tr("逗号/空格分隔", "comma or space separated"));
                                                ui.add(eframe::egui::TextEdit::singleline(&mut self.cp_models_input)
                                                    .desired_width(280.0).hint_text("gpt-4o, gpt-4-turbo, ..."));
                                                ui.end_row();

                                                ui.label(self.tr("备注", "Note"));
                                                ui.add(eframe::egui::TextEdit::singleline(&mut self.cp_description_input)
                                                    .desired_width(280.0).hint_text(hint_cp_note));
                                                ui.end_row();
                                            });
                                        ui.separator();
                                        ui.horizontal(|ui| {
                                            if ui.button(self.tr("💾 保存供应商", "💾 Save Provider")).clicked() {
                                                self.add_custom_provider();
                                            }
                                            if ui.small_button(self.tr("✖ 清空", "✖ Clear")).clicked() {
                                                self.cp_id_input.clear(); self.cp_name_input.clear();
                                                self.cp_url_input.clear(); self.cp_key_header_input.clear();
                                                self.cp_key_prefix_input.clear(); self.cp_models_input.clear();
                                                self.cp_description_input.clear();
                                            }
                                        });
                                        // List existing custom providers with edit/delete
                                        let custom_pvds = self.custom_providers.clone();
                                        if !custom_pvds.is_empty() {
                                            ui.add_space(4.0);
                                            ui.separator();
                                            ui.strong(self.tr("已保存的转发供应商", "Saved Custom Providers"));
                                            for cp in &custom_pvds {
                                                ui.horizontal(|ui| {
                                                    ui.label(eframe::egui::RichText::new(&cp.name).strong());
                                                    ui.label(
                                                        eframe::egui::RichText::new(&cp.base_url)
                                                            .monospace().small()
                                                    );
                                                    if ui.small_button("🗑").on_hover_text(
                                                        self.tr("删除此供应商", "Delete")
                                                    ).clicked() {
                                                        let id = cp.id.clone();
                                                        self.delete_custom_provider(&id);
                                                    }
                                                    if ui.small_button(self.tr("编辑", "Edit")).clicked() {
                                                        self.cp_id_input = cp.id.clone();
                                                        self.cp_name_input = cp.name.clone();
                                                        self.cp_url_input = cp.base_url.clone();
                                                        self.cp_key_header_input = cp.key_header.clone();
                                                        self.cp_key_prefix_input = cp.key_prefix.clone();
                                                        self.cp_models_input = cp.models_hint.clone();
                                                        self.cp_description_input = cp.description.clone();
                                                    }
                                                });
                                            }
                                        }
                                    });
                                });

                                // ── 5. 使用统计（折叠）───────────────────────────────────
                                eframe::egui::CollapsingHeader::new(
                                    self.tr("📊 使用统计", "📊 Usage Stats"),
                                )
                                .id_source("api_usage_inline")
                                .default_open(false)
                                .show(ui, |ui| {
                                    if ui.small_button(self.tr("加载统计", "Load Stats")).clicked() {
                                        self.load_usage_stats();
                                    }
                                    eframe::egui::ScrollArea::vertical()
                                        .id_source("usage_scroll")
                                        .max_height(120.0)
                                        .show(ui, |ui| {
                                            ui.monospace(&self.api_usage_json);
                                        });
                                });
                            }

                            // ── 状态消息 ──────────────────────────────────────────────────
                            if !self.api_quick_status.is_empty() {
                                ui.separator();
                                ui.monospace(&self.api_quick_status);
                            }
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
