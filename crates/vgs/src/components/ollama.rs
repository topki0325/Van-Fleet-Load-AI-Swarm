use eframe::egui;
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub struct OllamaComponent {
    status_line: String,
    version_line: String,
    models: Vec<String>,
    selected: BTreeSet<String>,
    share_enabled: bool,
    last_error: Option<String>,
}

impl OllamaComponent {
    pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::app::VgaGuiApp) {
        ui.heading(app.tr("本地 Ollama 模型", "Local Ollama Models"));
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            if ui.button(app.tr("检查连接", "Check connection")).clicked() {
                self.check_connection(app);
            }
            if ui.button(app.tr("加载模型列表", "Load models")).clicked() {
                self.load_models(app);
            }
            if ui.button(app.tr("清空选择", "Clear selection")).clicked() {
                self.selected.clear();
            }
            if ui.button(app.tr("全选", "Select all")).clicked() {
                for m in &self.models {
                    self.selected.insert(m.clone());
                }
            }
        });

        if !self.status_line.is_empty() {
            ui.label(&self.status_line);
        }
        if !self.version_line.is_empty() {
            ui.label(&self.version_line);
        }
        if let Some(err) = &self.last_error {
            ui.colored_label(egui::Color32::RED, err);
        }

        ui.add_space(8.0);
        ui.checkbox(
            &mut self.share_enabled,
            app.tr("允许被别人调用（共享）", "Allow remote calls (share)"),
        );

        ui.horizontal_wrapped(|ui| {
            if ui
                .button(app.tr("应用共享", "Apply sharing"))
                .clicked()
            {
                self.apply_sharing(app);
            }
            if ui
                .button(app.tr("广播一次", "Announce once"))
                .clicked()
            {
                app.services.network_discovery.broadcast_presence();
            }
        });

        ui.add_space(8.0);
        ui.label(app.tr("可调用模型（多选）", "Callable models (multi-select)"));

        egui::ScrollArea::vertical()
            .id_source("ollama_models_scroll")
            .max_height(360.0)
            .show(ui, |ui| {
                if self.models.is_empty() {
                    ui.label(app.tr("(未加载)", "(not loaded)"));
                    return;
                }

                for model in &self.models {
                    let mut checked = self.selected.contains(model);
                    if ui.checkbox(&mut checked, model).changed() {
                        if checked {
                            self.selected.insert(model.clone());
                        } else {
                            self.selected.remove(model);
                        }
                    }
                }
            });
    }

    fn check_connection(&mut self, app: &mut crate::app::VgaGuiApp) {
        self.last_error = None;
        let services = app.services.clone();
        let res = app.runtime.block_on(async move { services.ollama_manager.check_connection().await });

        if res.is_connected {
            self.status_line = app.tr("状态：已连接", "Status: connected").to_string();
            self.version_line = match res.version {
                Some(v) => format!("{}{}", app.tr("版本：", "Version: "), v),
                None => app.tr("版本：未知", "Version: unknown").to_string(),
            };
        } else {
            self.status_line = app.tr("状态：未连接", "Status: not connected").to_string();
            self.version_line.clear();
            if let Some(e) = res.error {
                self.last_error = Some(e);
            }
        }
    }

    fn load_models(&mut self, app: &mut crate::app::VgaGuiApp) {
        self.last_error = None;
        let services = app.services.clone();
        let res = app.runtime.block_on(async move { services.ollama_manager.list_models().await });

        match res {
            Ok(list) => {
                let mut names: Vec<String> = list.into_iter().map(|m| m.name).collect();
                names.sort();
                names.dedup();
                self.models = names;

                // Keep selection that still exists.
                self.selected.retain(|m| self.models.iter().any(|x| x == m));

                self.status_line = format!(
                    "{}{}",
                    app.tr("已加载模型数：", "Models loaded: "),
                    self.models.len()
                );
            }
            Err(e) => {
                self.last_error = Some(e);
            }
        }
    }

    fn apply_sharing(&mut self, app: &mut crate::app::VgaGuiApp) {
        self.last_error = None;
        let enabled = self.share_enabled;
        let models: Vec<String> = self.selected.iter().cloned().collect();

        let services = app.services.clone();
        let res = app.runtime.block_on(async move {
            services
                .network_discovery
                .set_ollama_offer(enabled, models, Some("http://localhost:11434".to_string()))
                .await;
            Ok::<(), vangriten_ai_swarm::shared::models::VgaError>(())
        });

        if let Err(e) = res {
            self.last_error = Some(format!("apply sharing failed: {e:?}"));
            return;
        }

        app.services.network_discovery.broadcast_presence();
        self.status_line = app.tr("已应用共享设置", "Sharing settings applied").to_string();
    }
}
