use eframe::egui;
use std::collections::{BTreeMap, BTreeSet};
use vas_core::backend::network_discovery::DiscoveryDebugStats;
use vangriten_ai_swarm::shared::models::OllamaOfferStatus;

#[derive(Debug)]
pub struct NetworkComponent {
    join_group_id: String,
    share_ollama: bool,
    share_models_csv: String,
    local_groups_json: String,

    // Discovery display
    peers: Vec<PeerSummary>,
    groups: BTreeMap<String, Vec<PeerSummary>>,
    debug: DiscoveryDebugStats,

    // Local Ollama models
    ollama_models: Vec<String>,
    ollama_selected: BTreeSet<String>,
    ollama_status: String,
}

impl Default for NetworkComponent {
    fn default() -> Self {
        Self {
            join_group_id: String::new(),
            share_ollama: false,
            share_models_csv: String::new(),
            local_groups_json: "(not loaded)".to_string(),

            peers: Vec::new(),
            groups: BTreeMap::new(),
            debug: DiscoveryDebugStats::default(),

            ollama_models: Vec::new(),
            ollama_selected: BTreeSet::new(),
            ollama_status: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct PeerSummary {
    id: String,
    name: String,
    address: String,
    mode: vangriten_ai_swarm::shared::models::ClientMode,
    latency: Option<u64>,
    groups: Vec<String>,
    ollama: Option<OllamaOfferStatus>,
}

impl NetworkComponent {
    pub fn ui(&mut self, ui: &mut egui::Ui, app: &mut crate::app::VgaGuiApp) {
        ui.heading(app.tr("网络", "Network"));
        ui.separator();

        // First-time initialization for this view.
        if self.local_groups_json == "(not loaded)" {
            self.refresh_local_groups(app);
            app.services.network_discovery.broadcast_presence();
            self.refresh_discovery(app);
        }

        // Local profile
        let local_name = app
            .runtime
            .block_on(async { app.services.network_discovery.local_node_name().await });

        ui.horizontal(|ui| {
            draw_avatar(ui, &local_name);
            ui.vertical(|ui| {
                ui.label(app.tr("本机", "Local"));
                ui.monospace(&local_name);
                ui.small(format!(
                    "{}: {}",
                    app.tr("Peers", "Peers"),
                    self.peers.len()
                ));
                ui.small(format!(
                    "{}: {}",
                    app.tr("Groups", "Groups"),
                    self.groups.len()
                ));
            });
        });

        ui.add_space(6.0);
        ui.small(format!(
            "discovery: bound={} bind={} tx(a/q)={}/{} rx(a/q)={}/{} last_rx={} {} age={}ms",
            self.debug.socket_bound,
            self.debug.bind.clone().unwrap_or_else(|| "(none)".to_string()),
            self.debug.sent_announces,
            self.debug.sent_queries,
            self.debug.received_announces,
            self.debug.received_queries,
            self.debug
                .last_received_kind
                .clone()
                .unwrap_or_else(|| "(none)".to_string()),
            self.debug
                .last_received_from
                .clone()
                .unwrap_or_else(|| "".to_string()),
            self.debug
                .last_received_age_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string())
        ));

        ui.add_space(8.0);
        ui.label(app.tr("本地组", "Local groups"));
        ui.horizontal_wrapped(|ui| {
            if ui.button(app.tr("刷新本地组", "Refresh local groups")).clicked() {
                self.refresh_local_groups(app);
            }

            if ui.button(app.tr("刷新发现", "Refresh discovery")).clicked() {
                self.refresh_discovery(app);
            }
            if ui.button(app.tr("广播一次", "Announce once")).clicked() {
                app.services.network_discovery.broadcast_presence();
            }

            ui.label(app.tr("加入 Group ID", "Join Group ID"));
            ui.text_edit_singleline(&mut self.join_group_id);

            ui.checkbox(
                &mut self.share_ollama,
                app.tr("共享 Ollama", "Share Ollama"),
            );
            ui.label(app.tr("模型(csv)", "Models (csv)"));
            ui.text_edit_singleline(&mut self.share_models_csv);

            if ui.button(app.tr("加入", "Join")).clicked() {
                self.join_group_with_offer(app);
                self.refresh_local_groups(app);
                self.refresh_discovery(app);
            }
        });

        egui::ScrollArea::vertical()
            .id_source("network_local_groups_scroll")
            .max_height(160.0)
            .show(ui, |ui| {
                ui.monospace(&self.local_groups_json);
            });

        ui.separator();

        // Local Ollama sharing (subset selection)
        egui::CollapsingHeader::new(app.tr("本机 Ollama 共享", "Local Ollama Share"))
            .id_source("network_sub_ollama_share")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(app.tr("检查连接", "Check connection")).clicked() {
                        self.check_ollama(app);
                    }
                    if ui.button(app.tr("加载模型", "Load models")).clicked() {
                        self.load_ollama_models(app);
                    }
                    if ui.button(app.tr("清空选择", "Clear selection")).clicked() {
                        self.ollama_selected.clear();
                    }
                    if ui.button(app.tr("全选", "Select all")).clicked() {
                        for m in &self.ollama_models {
                            self.ollama_selected.insert(m.clone());
                        }
                    }
                    if ui.button(app.tr("应用共享", "Apply sharing")).clicked() {
                        self.apply_ollama_share(app);
                        self.refresh_discovery(app);
                    }
                });

                if !self.ollama_status.is_empty() {
                    ui.label(&self.ollama_status);
                }

                egui::ScrollArea::vertical()
                    .id_source("network_ollama_models_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if self.ollama_models.is_empty() {
                            ui.label(app.tr("(未加载)", "(not loaded)"));
                            return;
                        }
                        for model in &self.ollama_models {
                            let mut checked = self.ollama_selected.contains(model);
                            if ui.checkbox(&mut checked, model).changed() {
                                if checked {
                                    self.ollama_selected.insert(model.clone());
                                } else {
                                    self.ollama_selected.remove(model);
                                }
                            }
                        }
                    });
            });

        ui.separator();

        // Peers & groups view (like vgs-discovery)
        ui.columns(2, |cols| {
            cols[0].heading(app.tr("同款软件列表", "Peers"));
            cols[0].separator();
            egui::ScrollArea::vertical()
                .id_source("network_peers_cards_scroll")
                .max_height(320.0)
                .show(&mut cols[0], |ui| {
                    if self.peers.is_empty() {
                        ui.label(app.tr("(暂无)", "(none)"));
                        return;
                    }
                    for p in &self.peers {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                draw_avatar(ui, &p.name);
                                ui.vertical(|ui| {
                                    ui.monospace(&p.name);
                                    ui.small(format!("{} ({:?})", p.address, p.mode));
                                    if let Some(lat) = p.latency {
                                        ui.small(format!("latency: {}ms", lat));
                                    }
                                    if let Some(ollama) = &p.ollama {
                                        if ollama.enabled {
                                            let models = if ollama.models.is_empty() {
                                                "(models not specified)".to_string()
                                            } else {
                                                ollama.models.join(", ")
                                            };
                                            ui.small(format!("ollama: enabled | {}", models));
                                        }
                                    }
                                });
                            });
                            if !p.groups.is_empty() {
                                ui.small(format!("groups: {}", p.groups.join(", ")));
                            }
                        });
                        ui.add_space(6.0);
                    }
                });

            cols[1].heading(app.tr("群", "Groups"));
            cols[1].separator();
            egui::ScrollArea::vertical()
                .id_source("network_groups_cards_scroll")
                .max_height(320.0)
                .show(&mut cols[1], |ui| {
                    if self.groups.is_empty() {
                        ui.label(app.tr("(暂无)", "(none)"));
                        return;
                    }
                    for (group_id, members) in &self.groups {
                        ui.group(|ui| {
                            ui.monospace(group_id);
                            ui.add_space(4.0);
                            for m in members {
                                ui.horizontal(|ui| {
                                    draw_avatar(ui, &m.name);
                                    ui.vertical(|ui| {
                                        ui.monospace(&m.name);
                                        ui.small(&m.address);
                                        if let Some(ollama) = &m.ollama {
                                            if ollama.enabled {
                                                ui.small(app.tr("ollama: 已共享", "ollama: shared"));
                                            }
                                        }
                                    });
                                });
                            }
                        });
                        ui.add_space(8.0);
                    }
                });
        });
    }

    fn refresh_local_groups(&mut self, app: &mut crate::app::VgaGuiApp) {
        let services = app.services.clone();
        let res = app.runtime.block_on(async move {
            let local_id = services.resource_manager.local_node_id().await;
            let all = services.resource_manager.list_swarm_groups().await;
            let local_groups: Vec<_> = all
                .into_iter()
                .filter(|g| g.members.iter().any(|m| m == &local_id))
                .collect();

            let advertised_ids: Vec<String> = local_groups.iter().map(|g| g.group_id.clone()).collect();
            services.network_discovery.set_local_groups(advertised_ids).await;
            Ok::<_, vangriten_ai_swarm::shared::models::VgaError>(local_groups)
        });

        match res {
            Ok(groups) => self.local_groups_json = crate::app::VgaGuiApp::pretty(&groups),
            Err(e) => app.set_error(format!("list local groups failed: {e:?}")),
        }
    }

    fn join_group_with_offer(&mut self, app: &mut crate::app::VgaGuiApp) {
        let group_id = self.join_group_id.trim().to_string();
        if group_id.is_empty() {
            app.set_error("Group ID is empty");
            return;
        }

        let enabled = self.share_ollama;
        let models: Vec<String> = self
            .share_models_csv
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let services = app.services.clone();
        let res = app.runtime.block_on(async move {
            services.resource_manager.join_swarm_group(group_id).await?;
            services
                .network_discovery
                .set_ollama_offer(enabled, models, Some("http://localhost:11434".to_string()))
                .await;
            Ok::<_, vangriten_ai_swarm::shared::models::VgaError>(())
        });

        if let Err(e) = res {
            app.set_error(format!("join group failed: {e:?}"));
        }
    }

    fn refresh_discovery(&mut self, app: &mut crate::app::VgaGuiApp) {
        self.debug = app.runtime.block_on(async { app.services.network_discovery.debug_stats().await });
        let services = app.services.clone();
        let res = app.runtime.block_on(async move { services.network_discovery.discover_peers().await });
        match res {
            Ok(mut list) => {
                list.sort_by(|a, b| {
                    let an = a.name.clone().unwrap_or_default();
                    let bn = b.name.clone().unwrap_or_default();
                    an.cmp(&bn).then_with(|| a.address.cmp(&b.address))
                });
                let summaries: Vec<PeerSummary> = list
                    .iter()
                    .map(|p| PeerSummary {
                        id: p.id.clone(),
                        name: p
                            .name
                            .clone()
                            .unwrap_or_else(|| p.id.chars().take(8).collect()),
                        address: p.address.clone(),
                        mode: p.mode.clone(),
                        latency: p.latency,
                        groups: p.groups.clone(),
                        ollama: p.ollama.clone(),
                    })
                    .collect();

                self.peers = summaries;
                self.groups = build_groups(&self.peers);

                // Keep legacy json string updated too.
                app.peers_json = crate::app::VgaGuiApp::pretty(&list);
            }
            Err(e) => app.set_error(format!("discover_peers failed: {e:?}")),
        }
    }

    fn check_ollama(&mut self, app: &mut crate::app::VgaGuiApp) {
        let services = app.services.clone();
        let status = app.runtime.block_on(async move { services.ollama_manager.check_connection().await });
        if status.is_connected {
            let ver = status.version.unwrap_or_else(|| "?".to_string());
            self.ollama_status = format!("{}{}", app.tr("已连接 version=", "connected version="), ver);
        } else {
            self.ollama_status = format!(
                "{}{}",
                app.tr("未连接：", "not connected: "),
                status.error.unwrap_or_else(|| "unknown".to_string())
            );
        }
    }

    fn load_ollama_models(&mut self, app: &mut crate::app::VgaGuiApp) {
        let services = app.services.clone();
        let res = app.runtime.block_on(async move { services.ollama_manager.list_models().await });
        match res {
            Ok(list) => {
                let mut names: Vec<String> = list.into_iter().map(|m| m.name).collect();
                names.sort();
                names.dedup();
                self.ollama_models = names;
                self.ollama_selected.retain(|m| self.ollama_models.iter().any(|x| x == m));
                self.ollama_status = format!(
                    "{}{}",
                    app.tr("模型数：", "models: "),
                    self.ollama_models.len()
                );
            }
            Err(e) => {
                self.ollama_status = format!("{}{}", app.tr("加载失败：", "load failed: "), e);
            }
        }
    }

    fn apply_ollama_share(&mut self, app: &mut crate::app::VgaGuiApp) {
        let enabled = true;
        let models: Vec<String> = self.ollama_selected.iter().cloned().collect();
        let services = app.services.clone();
        app.runtime.block_on(async move {
            services
                .network_discovery
                .set_ollama_offer(enabled, models, Some("http://localhost:11434".to_string()))
                .await;
        });
        app.services.network_discovery.broadcast_presence();
        self.ollama_status = app.tr("已广播共享设置", "sharing announced").to_string();
    }
}

fn build_groups(peers: &[PeerSummary]) -> BTreeMap<String, Vec<PeerSummary>> {
    let mut out: BTreeMap<String, Vec<PeerSummary>> = BTreeMap::new();
    for peer in peers {
        if peer.groups.is_empty() {
            continue;
        }
        for g in &peer.groups {
            out.entry(g.clone()).or_default().push(peer.clone());
        }
    }

    for members in out.values_mut() {
        members.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.address.cmp(&b.address)));
        members.dedup_by(|a, b| a.id == b.id);
    }

    out
}

fn draw_avatar(ui: &mut egui::Ui, name: &str) {
    let initials = initials_from_name(name);
    let size = egui::vec2(34.0, 34.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let visuals = ui.visuals();
    let bg = visuals.widgets.inactive.bg_fill;
    let fg = visuals.text_color();

    ui.painter().circle_filled(rect.center(), 16.0, bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        initials,
        egui::FontId::proportional(14.0),
        fg,
    );
}

fn initials_from_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "?".to_string();
    }

    // Prefer first 2 chars (works for CJK as well as latin).
    let mut chars = trimmed.chars();
    let first = chars.next().unwrap_or('?');
    let second = chars.next();
    match second {
        Some(c2) => format!("{}{}", first, c2),
        None => first.to_string(),
    }
}
