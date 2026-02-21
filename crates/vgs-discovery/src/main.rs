use eframe::egui;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use vas_core::backend::NetworkDiscovery;
use vas_core::backend::network_discovery::DiscoveryDebugStats;
use vas_core::shared::models::{OllamaOfferStatus, PeerStatus};

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "vas",
        native_options,
        Box::new(|cc| {
            let _ = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx);
            Box::new(DiscoveryApp::new())
        }),
    )
}

struct DiscoveryApp {
    runtime: tokio::runtime::Runtime,
    discovery: std::sync::Arc<NetworkDiscovery>,

    local_name: String,

    peers: Vec<PeerStatus>,
    groups: BTreeMap<String, Vec<PeerSummary>>,
    debug: DiscoveryDebugStats,

    last_refresh: Instant,
    refresh_every: Duration,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
struct PeerSummary {
    id: String,
    name: String,
    address: String,
    ollama: Option<OllamaOfferStatus>,
}

impl DiscoveryApp {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("create tokio runtime");

        let discovery = runtime.block_on(async { std::sync::Arc::new(NetworkDiscovery::new().await) });
        let local_name = runtime.block_on(async { discovery.local_node_name().await });

        // Kick one announce right away.
        discovery.broadcast_presence();

        Self {
            runtime,
            discovery,
            local_name,
            peers: Vec::new(),
            groups: BTreeMap::new(),
            debug: DiscoveryDebugStats::default(),
            last_refresh: Instant::now() - Duration::from_secs(60),
            refresh_every: Duration::from_secs(2),
            last_error: None,
        }
    }

    fn refresh(&mut self) {
        self.last_error = None;
        self.debug = self.runtime.block_on(async { self.discovery.debug_stats().await });
        let discovery = self.discovery.clone();
        let res = self.runtime.block_on(async move { discovery.discover_peers().await });

        match res {
            Ok(mut peers) => {
                // Stable sort: name then address.
                peers.sort_by(|a, b| {
                    let an = a.name.clone().unwrap_or_default();
                    let bn = b.name.clone().unwrap_or_default();
                    an.cmp(&bn).then_with(|| a.address.cmp(&b.address))
                });

                self.peers = peers;
                self.groups = build_groups(&self.peers);
            }
            Err(e) => {
                self.last_error = Some(format!("discover_peers failed: {e:?}"));
            }
        }

        self.last_refresh = Instant::now();
    }
}

impl eframe::App for DiscoveryApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_refresh.elapsed() >= self.refresh_every {
            self.refresh();
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Refresh").clicked() {
                    self.refresh();
                }
                if ui.button("Announce").clicked() {
                    self.discovery.broadcast_presence();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.strong("vas");
                });
            });

            if let Some(err) = &self.last_error {
                ui.colored_label(egui::Color32::RED, err);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                draw_avatar(ui, &self.local_name);
                ui.vertical(|ui| {
                    ui.label("Local node");
                    ui.monospace(&self.local_name);
                    ui.add_space(2.0);
                    ui.small(format!("Peers: {} | Groups: {}", self.peers.len(), self.groups.len()));
                    ui.add_space(2.0);
                    ui.small(format!(
                        "discovery: bound={} bind={}",
                        self.debug.socket_bound,
                        self.debug.bind.clone().unwrap_or_else(|| "(none)".to_string())
                    ));
                    ui.small(format!(
                        "tx: announce={} query={} | rx: announce={} query={}",
                        self.debug.sent_announces,
                        self.debug.sent_queries,
                        self.debug.received_announces,
                        self.debug.received_queries
                    ));
                    ui.small(format!(
                        "last rx: {} {} age={}ms",
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
                });
            });

            ui.separator();

            ui.columns(2, |cols| {
                cols[0].heading("Peers");
                cols[0].separator();
                egui::ScrollArea::vertical()
                    .id_source("peers_scroll")
                    .show(&mut cols[0], |ui| {
                        for peer in &self.peers {
                            let name = peer
                                .name
                                .clone()
                                .unwrap_or_else(|| peer.id.chars().take(8).collect());
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    draw_avatar(ui, &name);
                                    ui.vertical(|ui| {
                                        ui.monospace(&name);
                                        ui.small(format!("{}  ({:?})", peer.address, peer.mode));
                                        if let Some(lat) = peer.latency {
                                            ui.small(format!("latency: {}ms", lat));
                                        }
                                        if let Some(ollama) = &peer.ollama {
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

                                if !peer.groups.is_empty() {
                                    ui.small(format!("groups: {}", peer.groups.join(", ")));
                                }
                            });
                            ui.add_space(6.0);
                        }
                    });

                cols[1].heading("Groups");
                cols[1].separator();
                egui::ScrollArea::vertical()
                    .id_source("groups_scroll")
                    .show(&mut cols[1], |ui| {
                        if self.groups.is_empty() {
                            ui.label("(no groups advertised yet)");
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
                                                    ui.small("ollama: enabled");
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
        });

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

fn build_groups(peers: &[PeerStatus]) -> BTreeMap<String, Vec<PeerSummary>> {
    let mut out: BTreeMap<String, Vec<PeerSummary>> = BTreeMap::new();
    for peer in peers {
        if peer.groups.is_empty() {
            continue;
        }
        let name = peer
            .name
            .clone()
            .unwrap_or_else(|| peer.id.chars().take(8).collect());
        let summary = PeerSummary {
            id: peer.id.clone(),
            name,
            address: peer.address.clone(),
            ollama: peer.ollama.clone(),
        };

        for g in &peer.groups {
            out.entry(g.clone()).or_default().push(summary.clone());
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

    let bg = color_from_string(name, ui.visuals().widgets.inactive.bg_fill);
    let fg = ui.visuals().text_color();

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

    let mut chars = trimmed.chars();
    let first = chars.next().unwrap_or('?');
    let second = chars.next();
    match second {
        Some(c2) => format!("{}{}", first, c2),
        None => first.to_string(),
    }
}

fn color_from_string(s: &str, fallback: egui::Color32) -> egui::Color32 {
    // Generate a stable "random" accent color from the string.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut hasher);
    let h = hasher.finish();

    // Map hash -> HSV-ish, then to RGB. Keep it subtle by blending with fallback.
    let r = ((h >> 0) & 0xFF) as u8;
    let g = ((h >> 8) & 0xFF) as u8;
    let b = ((h >> 16) & 0xFF) as u8;

    let base = egui::Color32::from_rgb(r, g, b);
    blend(base, fallback, 0.55)
}

fn blend(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    egui::Color32::from_rgb(
        (a.r() as f32 * inv + b.r() as f32 * t) as u8,
        (a.g() as f32 * inv + b.g() as f32 * t) as u8,
        (a.b() as f32 * inv + b.b() as f32 * t) as u8,
    )
}
