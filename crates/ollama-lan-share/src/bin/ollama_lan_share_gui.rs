use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use uuid::Uuid;
use ollama_lan_share::backend::{NetworkDiscovery, OllamaManager, OllamaModel};
use ollama_lan_share::shared::models::{PeerStatus, VgaError};

const PROXY_PORT: u16 = 11435;
const OLLAMA_PORT: u16 = 11434;
const SHARE_KEY_HEADER: &str = "x-vas-key";

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting ollama-lan-share GUI...");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "ollama-lan-share",
        native_options,
        Box::new(|cc| {
            let _ = egui_chinese_font::setup_chinese_fonts(&cc.egui_ctx);
            Box::new(OllamaLanShareApp::new())
        }),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedGuiState {
    local_name: Option<String>,
    /// Password required to call our shared Ollama proxy (if enabled).
    /// Backward-compat: previously stored as `lan_key`.
    #[serde(default, alias = "lan_key")]
    share_key: Option<String>,
    #[serde(default)]
    require_share_key: bool,
    #[serde(default)]
    chat_key: Option<String>,
    my_groups: Vec<String>,
    group_names: BTreeMap<String, String>,
}

fn gui_state_path() -> PathBuf {
    // Keep it workspace-local (repo already has a vault/ directory).
    // This avoids needing platform-specific dirs crates.
    let mut base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    base.push("vault");
    base.push("ollama-lan-share-gui-state.json");
    base
}

struct OllamaLanShareApp {
    runtime: tokio::runtime::Runtime,
    discovery: Arc<NetworkDiscovery>,
    ollama: Arc<OllamaManager>,

    local_name: String,
    edit_name: String,

    require_share_key: bool,
    share_key: String,
    edit_share_key: String,

    chat_key: String,
    edit_chat_key: String,

    // LAN groups
    my_groups: Vec<String>,
    group_names: BTreeMap<String, String>,
    active_group: Option<String>,

    active_tab: ActiveTab,

    create_group_name: String,
    pending_create_group_id: Option<String>,
    pending_create_group_name: Option<String>,

    join_group_id: String,
    pending_join_group_id: Option<String>,

    show_create_group_ui: bool,
    show_join_group_ui: bool,

    // Discovery
    peers: Vec<PeerStatus>,
    last_peer_refresh: Instant,

    // Sharing state
    sharing_active: bool,
    models: Vec<String>,
    selected: BTreeSet<String>,

    proxy: Option<Arc<ProxyState>>,

    // Chat
    chat_group_id: Option<String>,

    // Target selection
    chat_target_auto: bool,
    chat_target_peer_id: Option<String>,

    // Model selection
    chat_model_auto: bool,
    chat_model: String,

    chat_prompt: String,
    chat_transcript: String,

    // Round-robin + per-peer queue
    chat_rr_index_by_group: BTreeMap<String, usize>,
    chat_inflight_by_peer: BTreeMap<String, bool>,
    chat_queue_by_peer: BTreeMap<String, VecDeque<ChatJob>>,
    chat_job_counter: u64,
    chat_result_tx: tokio::sync::mpsc::UnboundedSender<ChatResult>,
    chat_result_rx: tokio::sync::mpsc::UnboundedReceiver<ChatResult>,

    status: String,
    last_error: Option<String>,

    last_model_refresh: Instant,

    lang: UiLang,
}

struct ProxyState {
    cfg: tokio::sync::RwLock<ProxyConfig>,
    sem: tokio::sync::Semaphore,
    client: reqwest::Client,
}

#[derive(Clone, Debug, Default)]
struct ProxyConfig {
    enabled: bool,
    key: String,
    allowed_models: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct ChatJob {
    job_id: u64,
    peer_id: String,
    who: String,
    base_url: String,
    model: String,
    prompt: String,
    key: String,
}

#[derive(Debug, Clone)]
struct ChatResult {
    job_id: u64,
    peer_id: String,
    who: String,
    model: String,
    prompt: String,
    result: Result<String, String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiLang {
    Zh,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveTab {
    Groups,
    Chat,
}

fn tr_lang<'a>(lang: UiLang, zh: &'a str, en: &'a str) -> &'a str {
    match lang {
        UiLang::Zh => zh,
        UiLang::En => en,
    }
}

impl OllamaLanShareApp {
    fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("create tokio runtime");

        let discovery: Arc<NetworkDiscovery> = runtime.block_on(async { Arc::new(NetworkDiscovery::new().await) });
        let local_name: String = runtime.block_on(async { discovery.local_node_name().await });
        let ollama = runtime.block_on(async { Arc::new(OllamaManager::new(None).await) });

        let (chat_result_tx, chat_result_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut app = Self {
            runtime,
            discovery,
            ollama,

            local_name: local_name.clone(),
            edit_name: local_name,

            require_share_key: false,
            share_key: String::new(),
            edit_share_key: String::new(),

            chat_key: String::new(),
            edit_chat_key: String::new(),

            my_groups: Vec::new(),
            group_names: BTreeMap::new(),
            active_group: None,

            active_tab: ActiveTab::Groups,

            create_group_name: String::new(),
            pending_create_group_id: None,
            pending_create_group_name: None,

            join_group_id: String::new(),
            pending_join_group_id: None,

            show_create_group_ui: false,
            show_join_group_ui: false,

            peers: Vec::new(),
            last_peer_refresh: Instant::now() - Duration::from_secs(3600),

            sharing_active: false,
            models: Vec::new(),
            selected: BTreeSet::new(),

            proxy: None,

            chat_group_id: None,

            chat_target_auto: true,
            chat_target_peer_id: None,

            chat_model_auto: true,
            chat_model: String::new(),
            chat_prompt: String::new(),
            chat_transcript: String::new(),

            status: String::new(),
            last_error: None,

            last_model_refresh: Instant::now() - Duration::from_secs(3600),
            chat_rr_index_by_group: BTreeMap::new(),
            chat_inflight_by_peer: BTreeMap::new(),
            chat_queue_by_peer: BTreeMap::new(),
            chat_job_counter: 0,
            chat_result_tx,
            chat_result_rx,

            // Default English as requested.
            lang: UiLang::En,
        };

        app.load_persisted_state();
        // Ensure discovery broadcasts the loaded groups immediately.
        app.apply_and_announce();
        app
    }

    fn tr<'a>(&self, zh: &'a str, en: &'a str) -> &'a str {
        tr_lang(self.lang, zh, en)
    }

    fn refresh_peers(&mut self) {
        self.last_error = None;
        let discovery = self.discovery.clone();
        let res: Result<Vec<PeerStatus>, VgaError> = self
            .runtime
            .block_on(async move { discovery.discover_peers().await });

        match res {
            Ok(mut peers) => {
                peers.sort_unstable_by(|a: &PeerStatus, b: &PeerStatus| {
                    let an = a.name.clone().unwrap_or_default();
                    let bn = b.name.clone().unwrap_or_default();
                    an.cmp(&bn).then_with(|| a.address.cmp(&b.address))
                });
                self.peers = peers;
                self.last_peer_refresh = Instant::now();
            }
            Err(err) => {
                self.last_error = Some(format!(
                    "{}: {err:?}",
                    self.tr("发现节点失败", "Discovery failed")
                ));
            }
        }
    }

    fn refresh_models(&mut self) {
        self.last_error = None;
        let ollama = self.ollama.clone();
        let res: Result<Vec<OllamaModel>, String> = self
            .runtime
            .block_on(async move { ollama.list_models().await });

        match res {
            Ok(list) => {
                let mut models: Vec<String> = list.into_iter().map(|m: OllamaModel| m.name).collect();
                models.sort();
                models.dedup();
                self.models = models;

                self.selected.retain(|m| self.models.contains(m));

                self.status = format!(
                    "{} {}",
                    self.tr("已加载模型数", "Loaded models"),
                    self.models.len()
                );
                self.last_model_refresh = Instant::now();
            }
            Err(err) => {
                self.last_error = Some(format!(
                    "{}: {err}",
                    self.tr("拉取模型列表失败", "Failed to list models")
                ));
            }
        }
    }

    fn apply_and_announce(&mut self) {
        self.last_error = None;

        let name = self.edit_name.trim().to_string();
        let mut groups = self.my_groups.clone();
        groups.sort();
        groups.dedup();

        let discovery = self.discovery.clone();
        let res = self.runtime.block_on(async move {
            if !name.is_empty() {
                discovery.set_local_node_name(name).await;
            }
            discovery.set_local_groups(groups).await;
        });
        let _ = res;

        let local_name = self.runtime.block_on(async { self.discovery.local_node_name().await });
        self.local_name = local_name;

        self.discovery.broadcast_presence();
        self.status = self.tr("已广播本机状态", "Announced presence").to_string();

        self.save_persisted_state();
    }

    fn load_persisted_state(&mut self) {
        let path = gui_state_path();
        let data = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(_) => return,
        };
        let state: PersistedGuiState = match serde_json::from_str(&data) {
            Ok(s) => s,
            Err(err) => {
                tracing::warn!("Failed to parse GUI state at {:?}: {err}", path);
                return;
            }
        };

        self.require_share_key = state.require_share_key;
        if let Some(key) = state.share_key.clone() {
            self.share_key = key;
            self.edit_share_key = self.share_key.clone();
        }
        if let Some(key) = state.chat_key.clone() {
            self.chat_key = key;
            self.edit_chat_key = self.chat_key.clone();
        }

        if let Some(name) = state.local_name.clone() {
            if !name.trim().is_empty() {
                self.local_name = name.clone();
                self.edit_name = name;
            }
        }

        self.my_groups = state.my_groups;
        self.my_groups.retain(|g| !g.trim().is_empty());
        self.my_groups.sort();
        self.my_groups.dedup();

        self.group_names = state.group_names;

        // Push loaded settings into discovery so we actually broadcast them.
        let groups = self.my_groups.clone();
        let name = self.edit_name.trim().to_string();
        let discovery = self.discovery.clone();
        let _ = self.runtime.block_on(async move {
            if !name.is_empty() {
                discovery.set_local_node_name(name).await;
            }
            discovery.set_local_groups(groups).await;
        });
        self.discovery.broadcast_presence();
    }

    fn save_persisted_state(&self) {
        let path = gui_state_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let state = PersistedGuiState {
            local_name: Some(self.edit_name.trim().to_string()).filter(|s| !s.is_empty()),
            share_key: Some(self.share_key.trim().to_string()).filter(|s| !s.is_empty()),
            require_share_key: self.require_share_key,
            chat_key: Some(self.chat_key.trim().to_string()).filter(|s| !s.is_empty()),
            my_groups: self.my_groups.clone(),
            group_names: self.group_names.clone(),
        };
        match serde_json::to_string_pretty(&state) {
            Ok(json) => {
                if let Err(err) = std::fs::write(&path, json) {
                    tracing::warn!("Failed to write GUI state at {:?}: {err}", path);
                }
            }
            Err(err) => tracing::warn!("Failed to serialize GUI state: {err}"),
        }
    }

    fn group_display_name(&self, group_id: &str) -> String {
        let name = self
            .group_names
            .get(group_id)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        name.unwrap_or_else(|| short_id(group_id))
    }

    fn start_create_group(&mut self) {
        self.last_error = None;
        let name = self.create_group_name.trim().to_string();
        if name.is_empty() {
            self.last_error = Some(self.tr("请输入组名", "Enter a group name").to_string());
            return;
        }

        let id = Uuid::new_v4().to_string();
        self.pending_create_group_id = Some(id);
        self.pending_create_group_name = Some(name);
    }

    fn confirm_create_group(&mut self) {
        let Some(id) = self.pending_create_group_id.take() else {
            return;
        };

        if let Some(name) = self.pending_create_group_name.take() {
            self.group_names.insert(id.clone(), name);
        }

        if !self.my_groups.contains(&id) {
            self.my_groups.push(id.clone());
        }
        self.active_group = Some(id);
        self.active_tab = ActiveTab::Groups;

        self.create_group_name.clear();

        self.apply_and_announce();
        self.status = self.tr("已创建并加入组", "Created and joined group").to_string();

        self.save_persisted_state();
    }

    fn start_join_group(&mut self) {
        self.last_error = None;
        let id = self.join_group_id.trim().to_string();
        if id.is_empty() {
            self.last_error = Some(self.tr("请输入 Group ID", "Enter a Group ID").to_string());
            return;
        }
        self.pending_join_group_id = Some(id);
    }

    fn confirm_join_group(&mut self) {
        let Some(id) = self.pending_join_group_id.take() else {
            return;
        };

        if !self.my_groups.contains(&id) {
            self.my_groups.push(id.clone());
        }
        self.active_group = Some(id);
        self.active_tab = ActiveTab::Groups;

        self.apply_and_announce();
        self.status = self.tr("已加入组", "Joined group").to_string();

        self.save_persisted_state();
    }

    fn leave_group(&mut self, group_id: &str) {
        self.last_error = None;
        self.my_groups.retain(|g| g != group_id);
        if self.active_group.as_deref() == Some(group_id) {
            self.active_group = self.my_groups.first().cloned();
            self.chat_target_peer_id = None;
        }

        if self.my_groups.is_empty() {
            self.active_group = None;
            self.chat_group_id = None;
        }

        if self.my_groups.is_empty() && self.sharing_active {
            self.stop_share();
        }

        self.apply_and_announce();
        self.status = self.tr("已退出组", "Left group").to_string();

        self.save_persisted_state();
    }

    fn group_members(&self, group_id: &str) -> Vec<PeerStatus> {
        self.peers
            .iter()
            .filter(|p| p.groups.iter().any(|g| g == group_id))
            .cloned()
            .collect()
    }

    fn chat_candidates(&self, group_id: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for p in &self.peers {
            if !p.groups.iter().any(|g| g == group_id) {
                continue;
            }
            let ok = p.ollama.as_ref().map(|o| o.enabled).unwrap_or(false);
            if !ok {
                continue;
            }
            let name = p.name.clone().unwrap_or_else(|| p.id.chars().take(8).collect());
            out.push((p.id.clone(), name));
        }
        out.sort_by(|a, b| a.1.cmp(&b.1));
        out
    }

    fn chat_candidate_peers(&self, group_id: &str) -> Vec<PeerStatus> {
        let mut out: Vec<PeerStatus> = self
            .peers
            .iter()
            .filter(|p| p.groups.iter().any(|g| g == group_id))
            .filter(|p| p.ollama.as_ref().map(|o| o.enabled).unwrap_or(false))
            .cloned()
            .collect();

        out.sort_by(|a, b| {
            let an = a.name.clone().unwrap_or_else(|| a.id.chars().take(8).collect());
            let bn = b.name.clone().unwrap_or_else(|| b.id.chars().take(8).collect());
            an.cmp(&bn).then_with(|| a.id.cmp(&b.id))
        });
        out
    }

    fn local_ollama_base_url(&self) -> String {
        // Best-effort: compute LAN-reachable URL so peers can call us.
        let ip = self.runtime.block_on(async { get_local_ip_best_effort().await });
        match ip {
            Some(ip) => format!("http://{ip}:{OLLAMA_PORT}"),
            None => format!("http://localhost:{OLLAMA_PORT}"),
        }
    }

    fn local_proxy_base_url(&self) -> String {
        let ip = self.runtime.block_on(async { get_local_ip_best_effort().await });
        match ip {
            Some(ip) => format!("http://{ip}:{PROXY_PORT}"),
            None => format!("http://localhost:{PROXY_PORT}"),
        }
    }

    fn ensure_proxy_started(&mut self) {
        if self.proxy.is_some() {
            return;
        }

        let state = Arc::new(ProxyState {
            cfg: tokio::sync::RwLock::new(ProxyConfig::default()),
            sem: tokio::sync::Semaphore::new(4),
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(300)) // 5 minutes max for LLM generation
                .pool_idle_timeout(Duration::from_secs(90))
                .build()
                .expect("failed to create proxy client"),
        });

        let state_for_task = state.clone();
        self.runtime.spawn(async move {
            use axum::{
                extract::State,
                extract::DefaultBodyLimit,
                http::{HeaderMap, StatusCode},
                routing::post,
                Json, Router,
            };

            fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
                if a.len() != b.len() {
                    return false;
                }
                let mut out = 0u8;
                for i in 0..a.len() {
                    out |= a[i] ^ b[i];
                }
                out == 0
            }

            async fn handler(
                State(state): State<Arc<ProxyState>>,
                headers: HeaderMap,
                Json(body): Json<serde_json::Value>,
            ) -> (StatusCode, Json<serde_json::Value>) {
                // Limit concurrent work to avoid overwhelming the local Ollama.
                let _permit = match state.sem.acquire().await {
                    Ok(p) => p,
                    Err(_) => {
                        return (
                            StatusCode::SERVICE_UNAVAILABLE,
                            Json(serde_json::json!({"error": "busy"})),
                        )
                    }
                };

                let cfg = state.cfg.read().await.clone();
                if !cfg.enabled {
                    return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "not sharing"})));
                }

                // Only support non-streaming chat.
                let stream = body.get("stream").and_then(|v| v.as_bool()).unwrap_or(false);
                if stream {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "stream not supported"})),
                    );
                }

                if !cfg.key.is_empty() {
                    let given = headers
                        .get(SHARE_KEY_HEADER)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("");
                    if !constant_time_eq(given.as_bytes(), cfg.key.as_bytes()) {
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(serde_json::json!({"error": "unauthorized"})),
                        );
                    }
                }

                let model = body
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if model.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({"error": "missing model"})),
                    );
                }
                if !cfg.allowed_models.contains(model) {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({"error": "model not allowed"})),
                    );
                }

                // Forward only /api/chat (stream=false)
                let url = format!("http://127.0.0.1:{}/api/chat", OLLAMA_PORT);
                let resp = state.client.post(url).json(&body).send().await;
                match resp {
                    Ok(r) => {
                        let status = StatusCode::from_u16(r.status().as_u16())
                            .unwrap_or(StatusCode::BAD_GATEWAY);
                        match r.json::<serde_json::Value>().await {
                            Ok(v) => (status, Json(v)),
                            Err(e) => (
                                StatusCode::BAD_GATEWAY,
                                Json(serde_json::json!({"error": format!("bad upstream json: {e}")})),
                            ),
                        }
                    }
                    Err(e) => (
                        StatusCode::BAD_GATEWAY,
                        Json(serde_json::json!({"error": format!("upstream error: {e}")})),
                    ),
                }
            }

            let app = Router::new()
                .route("/api/chat", post(handler))
                .layer(DefaultBodyLimit::max(256 * 1024))
                .with_state(state_for_task);
            let addr: SocketAddr = format!("0.0.0.0:{PROXY_PORT}").parse().expect("proxy addr");
            let listener = match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::warn!("Failed to bind proxy {}: {e}", addr);
                    return;
                }
            };
            let _ = axum::serve(listener, app).await;
        });

        self.proxy = Some(state);
    }

    fn configure_proxy(&mut self, enabled: bool, key: String, allowed_models: BTreeSet<String>) {
        self.ensure_proxy_started();
        let Some(state) = self.proxy.clone() else {
            return;
        };
        self.runtime.block_on(async move {
            let mut cfg = state.cfg.write().await;
            cfg.enabled = enabled;
            cfg.key = key;
            cfg.allowed_models = allowed_models;
        });
    }

    fn safe_remote_base_url(&self, peer: &PeerStatus) -> String {
        // SSRF mitigation: accept offer.base_url only when its host matches the discovered peer IP.
        let peer_ip = peer_ip_from_status(peer);
        if let (Some(ip), Some(offer)) = (peer_ip, &peer.ollama) {
            if let Some(url) = &offer.base_url {
                if let Ok(parsed) = url::Url::parse(url) {
                    if parsed.scheme() == "http" {
                        if let Some(host) = parsed.host_str() {
                            let host_ip = host.parse::<IpAddr>().ok();
                            if host_ip == Some(ip) {
                                let port = parsed.port().unwrap_or(OLLAMA_PORT);
                                let host_fmt = if host.contains(':') { format!("[{host}]") } else { host.to_string() };
                                return format!("http://{host_fmt}:{port}");
                            }
                        }
                    }
                }
            }
        }

        match peer_ip_from_status(peer) {
            Some(IpAddr::V4(v4)) => format!("http://{v4}:{OLLAMA_PORT}"),
            Some(IpAddr::V6(v6)) => format!("http://[{v6}]:{OLLAMA_PORT}"),
            None => format!("http://localhost:{OLLAMA_PORT}"),
        }
    }

    fn confirm_share(&mut self) {
        self.last_error = None;

        let Some(active_group) = self.active_group.clone() else {
            self.last_error = Some(self.tr("请先选择一个组", "Select a group first").to_string());
            return;
        };
        if !self.my_groups.contains(&active_group) {
            self.last_error = Some(self.tr("请先加入该组", "Join the group first").to_string());
            return;
        }

        let mut models: Vec<String> = self.selected.iter().cloned().collect();
        models.sort();
        models.dedup();

        if models.is_empty() {
            self.last_error = Some(self.tr("请至少选择 1 个模型", "Select at least 1 model").to_string());
            return;
        }

        if self.require_share_key {
            let k = self.edit_share_key.trim().to_string();
            if k.is_empty() {
                self.last_error = Some(self.tr("请填写共享密码", "Enter a share key").to_string());
                return;
            }
            self.share_key = k.clone();
            self.edit_share_key = self.share_key.clone();
            self.configure_proxy(true, self.share_key.clone(), models.iter().cloned().collect());
        } else {
            self.configure_proxy(false, String::new(), BTreeSet::new());
        }

        let base_url = if self.require_share_key {
            self.local_proxy_base_url()
        } else {
            self.local_ollama_base_url()
        };
        let discovery = self.discovery.clone();
        self.runtime.block_on(async move {
            discovery.set_ollama_offer(true, models, Some(base_url)).await;
        });
        self.discovery.broadcast_presence();

        self.sharing_active = true;
        self.status = self.tr("已开始共享", "Sharing enabled").to_string();

        self.save_persisted_state();
    }

    fn stop_share(&mut self) {
        self.last_error = None;

        self.configure_proxy(false, String::new(), BTreeSet::new());

        let base_url = self.local_ollama_base_url();
        let discovery = self.discovery.clone();
        self.runtime.block_on(async move {
            discovery.set_ollama_offer(false, Vec::new(), Some(base_url)).await;
        });
        self.discovery.broadcast_presence();

        self.sharing_active = false;
        self.status = self.tr("已停止共享", "Sharing disabled").to_string();
    }

    fn drain_chat_results(&mut self) {
        loop {
            match self.chat_result_rx.try_recv() {
                Ok(msg) => {
                    self.chat_inflight_by_peer.insert(msg.peer_id.clone(), false);

                    match msg.result {
                        Ok(text) => {
                            self.chat_transcript.push_str(&format!("\n{}:\n{}\n", msg.who, text));
                        }
                        Err(err) => {
                            self.last_error = Some(format!(
                                "{}: {err}",
                                self.tr("对话失败", "Chat failed")
                            ));
                        }
                    }

                    self.start_next_chat_job(&msg.peer_id);
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
    }

    fn start_next_chat_job(&mut self, peer_id: &str) {
        if self.chat_inflight_by_peer.get(peer_id).copied().unwrap_or(false) {
            return;
        }

        let Some(queue) = self.chat_queue_by_peer.get_mut(peer_id) else {
            return;
        };
        let Some(job) = queue.pop_front() else {
            return;
        };

        self.chat_inflight_by_peer.insert(peer_id.to_string(), true);

        let tx = self.chat_result_tx.clone();
        self.runtime.spawn(async move {
            // Use direct HTTP so we can optionally attach an auth header.
            let url = format!("{}/api/chat", job.base_url);
            let req = serde_json::json!({
                "model": job.model.clone(),
                "messages": [{"role": "user", "content": job.prompt.clone()}],
                "stream": false
            });

            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            
            let mut builder = client.post(url).json(&req);
            if !job.key.is_empty() {
                builder = builder.header(SHARE_KEY_HEADER, job.key.clone());
            }

            let resp = builder.send().await;
            let result = match resp {
                Ok(r) => {
                    if !r.status().is_success() {
                        let status = r.status();
                        let text = r.text().await.unwrap_or_default();
                        Err(format!("HTTP {status}: {text}"))
                    } else {
                        match r.json::<serde_json::Value>().await {
                            Ok(v) => v
                                .get("message")
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_str())
                                .map(|s| s.to_string())
                                .ok_or_else(|| "bad response".to_string()),
                            Err(e) => Err(format!("parse error: {e}")),
                        }
                    }
                }
                Err(e) => Err(e.to_string()),
            };

            let _ = tx.send(ChatResult {
                job_id: job.job_id,
                peer_id: job.peer_id.clone(),
                who: job.who.clone(),
                model: job.model.clone(),
                prompt: job.prompt.clone(),
                result,
            });
        });
    }

    fn enqueue_chat_job(&mut self, job: ChatJob) {
        const MAX_QUEUE_PER_PEER: usize = 16;
        let q = self
            .chat_queue_by_peer
            .entry(job.peer_id.clone())
            .or_insert_with(VecDeque::new);

        if q.len() >= MAX_QUEUE_PER_PEER {
            self.last_error = Some(self.tr("队列已满，请稍后再试", "Queue is full, try again later").to_string());
            return;
        }
        q.push_back(job.clone());

        let pending = q.len();
        self.chat_transcript.push_str(&format!(
            "\n> queued → {} ({}) [{} pending]\n{}\n",
            job.who, job.model, pending, job.prompt
        ));

        self.start_next_chat_job(&job.peer_id);
    }

    fn send_chat(&mut self) {
        self.last_error = None;

        let Some(group_id) = self.chat_group_id.clone() else {
            self.last_error = Some(self.tr("请先选择一个组", "Select a group first").to_string());
            return;
        };
        if !self.my_groups.contains(&group_id) {
            self.last_error = Some(self.tr("请先加入该组", "Join the group first").to_string());
            return;
        }

        let prompt = self.chat_prompt.trim().to_string();
        if prompt.is_empty() {
            return;
        }

        let explicit_model = if self.chat_model_auto {
            None
        } else {
            let m = self.chat_model.trim().to_string();
            if m.is_empty() { None } else { Some(m) }
        };

        let mut candidates = self.chat_candidate_peers(&group_id);
        if candidates.is_empty() {
            self.last_error = Some(self.tr("该组暂无可用组员", "No available members in this group").to_string());
            return;
        }

        // If an explicit model is chosen, only keep members who offer it.
        if let Some(m) = &explicit_model {
            candidates.retain(|p| {
                p.ollama
                    .as_ref()
                    .map(|o| o.models.iter().any(|x| x == m))
                    .unwrap_or(false)
            });
            if candidates.is_empty() {
                self.last_error = Some(self.tr("该组没有组员提供该模型", "No member offers the selected model").to_string());
                return;
            }
        }

        let chosen_peer = if self.chat_target_auto {
            let idx = self
                .chat_rr_index_by_group
                .get(&group_id)
                .copied()
                .unwrap_or(0);
            let pick = candidates[idx % candidates.len()].clone();
            self.chat_rr_index_by_group
                .insert(group_id.clone(), (idx + 1) % candidates.len());
            pick
        } else {
            let Some(peer_id) = self.chat_target_peer_id.clone() else {
                self.last_error = Some(self.tr("请选择一个组员", "Select a group member").to_string());
                return;
            };
            let Some(p) = candidates.into_iter().find(|p| p.id == peer_id) else {
                self.last_error = Some(self.tr("找不到该组员", "Member not found").to_string());
                return;
            };
            p
        };

        let Some(offer) = &chosen_peer.ollama else {
            self.last_error = Some(self.tr("该组员未共享 Ollama", "Member is not sharing Ollama").to_string());
            return;
        };
        if !offer.enabled {
            self.last_error = Some(self.tr("该组员未共享 Ollama", "Member is not sharing Ollama").to_string());
            return;
        }

        let base_url = self.safe_remote_base_url(&chosen_peer);

        let needs_key = base_url.ends_with(&format!(":{PROXY_PORT}"));
        let key = if needs_key {
            let k = self.edit_chat_key.trim().to_string();
            if k.is_empty() {
                self.last_error = Some(self.tr("该组员需要密码", "This member requires a key").to_string());
                return;
            }
            self.chat_key = k.clone();
            self.edit_chat_key = k;
            self.save_persisted_state();
            self.chat_key.clone()
        } else {
            String::new()
        };

        let model = if let Some(m) = explicit_model {
            m
        } else {
            offer
                .models
                .first()
                .cloned()
                .unwrap_or_else(|| "llama3".to_string())
        };

        let who = chosen_peer
            .name
            .clone()
            .unwrap_or_else(|| chosen_peer.id.chars().take(8).collect());

        self.chat_job_counter = self.chat_job_counter.saturating_add(1);
        let job_id = self.chat_job_counter;

        self.enqueue_chat_job(ChatJob {
            job_id,
            peer_id: chosen_peer.id.clone(),
            who,
            base_url,
            model,
            prompt: prompt.clone(),
            key,
        });

        self.chat_prompt.clear();
    }
}

fn short_id(s: &str) -> String {
    s.chars().take(8).collect()
}

fn sidebar_big_button(ui: &mut egui::Ui, selected: bool, title: &str, subtitle: &str) -> egui::Response {
    let fill = if selected {
        ui.visuals().selection.bg_fill
    } else {
        ui.visuals().widgets.inactive.bg_fill
    };

    ui.add_sized(
        [76.0, 76.0],
        egui::Button::new(
            egui::RichText::new(format!("{title}\n{subtitle}"))
                .size(16.0)
                .strong(),
        )
        .fill(fill),
    )
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i >= max_chars {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

fn peer_ip_from_status(status: &PeerStatus) -> Option<IpAddr> {
    status.address.parse::<SocketAddr>().ok().map(|sa: SocketAddr| sa.ip())
}

fn ollama_base_url_for_peer(status: &PeerStatus) -> Option<String> {
    let ip = peer_ip_from_status(status)?;
    Some(match ip {
        IpAddr::V4(v4) => format!("http://{v4}:11434"),
        IpAddr::V6(v6) => format!("http://[{v6}]:11434"),
    })
}

impl eframe::App for OllamaLanShareApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let lang = self.lang;

        // Apply completed chat jobs back into the transcript and continue queued work.
        self.drain_chat_results();

        // Ensure we have a model list without requiring a manual click.
        // (The user expects to be able to select local models immediately in the selected group panel.)
        if self.models.is_empty() && self.last_model_refresh.elapsed() >= Duration::from_secs(3) {
            self.refresh_models();
        }

        if self.last_peer_refresh.elapsed() >= Duration::from_secs(2) {
            self.refresh_peers();
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button(tr_lang(lang, "语言", "Language"), |ui| {
                    ui.selectable_value(&mut self.lang, UiLang::Zh, "中文");
                    ui.selectable_value(&mut self.lang, UiLang::En, "EN");
                });

                ui.menu_button(tr_lang(lang, "节点", "Node"), |ui| {
                    ui.label(tr_lang(lang, "当前名称：", "Current name:"));
                    ui.monospace(&self.local_name);
                    ui.separator();
                    ui.label(tr_lang(lang, "设置名称：", "Set name:"));
                    ui.text_edit_singleline(&mut self.edit_name);
                    if ui.button(tr_lang(lang, "应用并广播", "Apply + Announce")).clicked() {
                        self.apply_and_announce();
                        ui.close_menu();
                    }
                });

                ui.menu_button(tr_lang(lang, "组", "Groups"), |ui| {
                    ui.label(tr_lang(lang, "新组名称", "New group name"));
                    ui.text_edit_singleline(&mut self.create_group_name);
                    if ui.button(tr_lang(lang, "建立…", "Create…")).clicked() {
                        self.start_create_group();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.label(tr_lang(lang, "加入 Group ID", "Join Group ID"));
                    ui.text_edit_singleline(&mut self.join_group_id);
                    if ui.button(tr_lang(lang, "加入…", "Join…")).clicked() {
                        self.start_join_group();
                        ui.close_menu();
                    }
                });

                if ui.button(tr_lang(lang, "刷新模型", "Refresh models")).clicked() {
                    self.refresh_models();
                }
                if ui.button(tr_lang(lang, "刷新发现", "Refresh discovery")).clicked() {
                    self.refresh_peers();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.strong("ollama-lan-share");
                });
            });

            if let Some(err) = &self.last_error {
                ui.colored_label(egui::Color32::RED, err);
            }
            if !self.status.is_empty() {
                ui.small(&self.status);
            }
        });

        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(96.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);

                    let home_selected = self.active_group.is_none();
                    if sidebar_big_button(
                        ui,
                        self.active_tab == ActiveTab::Groups && home_selected,
                        "👥",
                        tr_lang(lang, "组", "Groups"),
                    )
                    .clicked()
                    {
                        self.active_tab = ActiveTab::Groups;
                        self.active_group = None;
                    }

                    ui.add_space(6.0);

                    if sidebar_big_button(
                        ui,
                        self.active_tab == ActiveTab::Chat,
                        "💬",
                        tr_lang(lang, "对话", "Chat"),
                    )
                    .clicked()
                    {
                        self.active_tab = ActiveTab::Chat;
                        if self.chat_group_id.is_none() {
                            self.chat_group_id = self.active_group.clone().or_else(|| self.my_groups.first().cloned());
                        }
                    }

                    ui.add_space(10.0);

                    for (idx, gid) in self.my_groups.clone().into_iter().enumerate() {
                        let selected = self.active_tab == ActiveTab::Groups
                            && self.active_group.as_deref() == Some(&gid);
                        let name = self.group_display_name(&gid);
                        let subtitle = format!(
                            "{}{}",
                            tr_lang(lang, "组:", "Group:"),
                            truncate_chars(&name, 6)
                        );
                        let hover = format!(
                            "{} #{}\n{} {}\n{}",
                            tr_lang(lang, "组", "Group"),
                            idx + 1,
                            tr_lang(lang, "组名=", "name="),
                            name,
                            gid
                        );
                        let resp = sidebar_big_button(ui, selected, "👥", &subtitle).on_hover_text(hover);
                        if resp.clicked() {
                            self.active_tab = ActiveTab::Groups;
                            self.active_group = Some(gid);
                            self.chat_group_id = self.active_group.clone();
                        }
                        ui.add_space(6.0);
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.active_tab == ActiveTab::Chat {
                ui.heading(tr_lang(lang, "简单对话", "Simple chat"));

                // Pick group
                ui.horizontal(|ui| {
                    ui.label(tr_lang(lang, "组", "Group"));
                    let mut current = self.chat_group_id.clone().unwrap_or_default();
                    let selected_text = if current.trim().is_empty() {
                        tr_lang(lang, "(未选择)", "(not selected)").to_string()
                    } else {
                        self.group_display_name(&current)
                    };

                    egui::ComboBox::from_id_source("chat_group")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for gid in &self.my_groups {
                                let label = self.group_display_name(gid);
                                ui.selectable_value(&mut current, gid.clone(), label)
                                    .on_hover_text(gid);
                            }
                        });

                    self.chat_group_id = if current.trim().is_empty() {
                        None
                    } else {
                        Some(current)
                    };
                });

                // Pick member
                ui.horizontal(|ui| {
                    ui.label(tr_lang(lang, "组员", "Member"));
                    let group_id = self.chat_group_id.clone().unwrap_or_default();
                    let candidates = if group_id.trim().is_empty() {
                        Vec::new()
                    } else {
                        self.chat_candidates(&group_id)
                    };

                    let selected_text = if self.chat_target_auto {
                        tr_lang(lang, "(自动轮询)", "(auto round-robin)").to_string()
                    } else {
                        let current_id = self.chat_target_peer_id.clone().unwrap_or_default();
                        candidates
                            .iter()
                            .find(|(id, _)| id == &current_id)
                            .map(|(_, name)| name.clone())
                            .unwrap_or_else(|| tr_lang(lang, "(未选择)", "(not selected)").to_string())
                    };

                    egui::ComboBox::from_id_source("chat_member")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(self.chat_target_auto, tr_lang(lang, "自动轮询", "Auto (round-robin)"))
                                .clicked()
                            {
                                self.chat_target_auto = true;
                                self.chat_target_peer_id = None;
                            }

                            ui.separator();
                            for (id, name) in &candidates {
                                let selected = !self.chat_target_auto
                                    && self.chat_target_peer_id.as_deref() == Some(id.as_str());
                                if ui.selectable_label(selected, name).clicked() {
                                    self.chat_target_auto = false;
                                    self.chat_target_peer_id = Some(id.clone());
                                }
                            }
                        });
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(tr_lang(lang, "模型", "Model"));
                    let group_id = self.chat_group_id.clone().unwrap_or_default();
                    let mut offered: BTreeSet<String> = BTreeSet::new();
                    if !group_id.trim().is_empty() {
                        for p in self.chat_candidate_peers(&group_id) {
                            if let Some(o) = &p.ollama {
                                if o.enabled {
                                    for m in &o.models {
                                        offered.insert(m.clone());
                                    }
                                }
                            }
                        }
                    }

                    let selected_text = if self.chat_model_auto {
                        tr_lang(lang, "(自动)", "(auto)").to_string()
                    } else {
                        let m = self.chat_model.trim();
                        if m.is_empty() {
                            tr_lang(lang, "(未选择)", "(not selected)").to_string()
                        } else {
                            m.to_string()
                        }
                    };

                    egui::ComboBox::from_id_source("chat_model")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(self.chat_model_auto, tr_lang(lang, "自动", "Auto"))
                                .clicked()
                            {
                                self.chat_model_auto = true;
                                self.chat_model.clear();
                            }
                            ui.separator();
                            for m in offered {
                                let selected = !self.chat_model_auto && self.chat_model.trim() == m;
                                if ui.selectable_label(selected, &m).clicked() {
                                    self.chat_model_auto = false;
                                    self.chat_model = m;
                                }
                            }
                        });
                });

                ui.label(tr_lang(lang, "输入", "Prompt"));
                ui.add(
                    egui::TextEdit::multiline(&mut self.chat_prompt)
                        .desired_rows(4)
                        .hint_text(tr_lang(lang, "输入要发送的内容…", "Type your message…")),
                );

                ui.horizontal(|ui| {
                    let enabled = self.chat_group_id.is_some()
                        && (self.chat_target_auto || self.chat_target_peer_id.is_some());
                    if ui
                        .add_enabled(enabled, egui::Button::new(tr_lang(lang, "发送", "Send")))
                        .clicked()
                    {
                        self.send_chat();
                    }
                    if ui.button(tr_lang(lang, "清空", "Clear")).clicked() {
                        self.chat_transcript.clear();
                    }
                });

                egui::ScrollArea::vertical()
                    .id_source("chat_scroll_full")
                    .show(ui, |ui| {
                        ui.monospace(&self.chat_transcript);
                    });

                return;
            }

            // Always show the two-column layout on the Groups page.
            // Simplified Groups page: show only the selected group details, or the Ollama groups list.
            // Node name + create/join actions live in the top menu.
            let selected_group = self.active_group.clone();

            if let Some(gid) = selected_group {
                let gname = self.group_display_name(&gid);
                ui.heading(tr_lang(lang, "Ollama组（当前）", "Ollama group (selected)"));
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(tr_lang(lang, "组名:", "Name:"));
                    ui.monospace(&gname);
                });

                ui.horizontal(|ui| {
                    ui.label(tr_lang(lang, "Group ID:", "Group ID:"));
                    ui.monospace(&gid);
                    if ui.button(tr_lang(lang, "退出组", "Leave")).clicked() {
                        self.leave_group(&gid);
                    }
                });

                ui.add_space(6.0);
                ui.heading(tr_lang(lang, "组员", "Members"));
                let members = self.group_members(&gid);
                egui::ScrollArea::vertical()
                    .id_source("members_scroll")
                    .max_height(150.0)
                    .show(ui, |ui| {
                        if members.is_empty() {
                            ui.small(tr_lang(lang, "（暂无发现的组员）", "(no members discovered yet)"));
                            return;
                        }
                        for p in &members {
                            let name = p
                                .name
                                .clone()
                                .unwrap_or_else(|| p.id.chars().take(8).collect());
                            ui.horizontal(|ui| {
                                    let can_pick = p.ollama.as_ref().map(|o| o.enabled).unwrap_or(false);
                                    let pick = self.chat_target_peer_id.as_deref() == Some(&p.id);
                                    if ui
                                        .add_enabled(can_pick, egui::SelectableLabel::new(pick, &name))
                                        .clicked()
                                    {
                                        self.chat_target_peer_id = Some(p.id.clone());
                                        if let Some(o) = &p.ollama {
                                            if self.chat_model.trim().is_empty() {
                                                if let Some(m) = o.models.first() {
                                                    self.chat_model = m.clone();
                                                }
                                            }
                                        }
                                    }
                                    if let Some(o) = &p.ollama {
                                        if o.enabled {
                                            ui.small(tr_lang(lang, "ollama: 已共享", "ollama: shared"));
                                        }
                                    }
                            });
                            ui.small(&p.address);
                        }
                    });

                ui.add_space(8.0);
                ui.label(tr_lang(
                        lang,
                        "Ollama 共享（需加入该组）",
                        "Ollama sharing (requires joining this group)",
                ));

                // Optional access control (only applies when using the proxy share).
                ui.horizontal(|ui| {
                    let enabled = !self.sharing_active;
                    ui.add_enabled_ui(enabled, |ui| {
                        ui.checkbox(
                            &mut self.require_share_key,
                            tr_lang(lang, "需要密码", "Require key"),
                        );
                        if self.require_share_key {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.edit_share_key)
                                    .password(true)
                                    .hint_text(tr_lang(lang, "共享密码", "share key")),
                            );
                        }
                    });
                });
                ui.horizontal(|ui| {
                    if !self.sharing_active {
                        if ui.button(tr_lang(lang, "确认共享", "Confirm share")).clicked() {
                            self.confirm_share();
                        }
                    } else if ui.button(tr_lang(lang, "停止共享", "Stop sharing")).clicked() {
                        self.stop_share();
                    }

                    ui.small(if self.sharing_active {
                        tr_lang(lang, "状态：共享中", "status: sharing")
                    } else {
                        tr_lang(lang, "状态：未共享", "status: not sharing")
                    });
                });

                ui.add_space(6.0);
                ui.label(tr_lang(lang, "选择要共享的模型：", "Models to share:"));
                ui.horizontal(|ui| {
                    let enabled = !self.sharing_active;
                    ui.add_enabled_ui(enabled, |ui| {
                        if ui.button(tr_lang(lang, "全选", "Select all")).clicked() {
                            self.selected = self.models.iter().cloned().collect();
                        }
                        if ui.button(tr_lang(lang, "全不选", "Select none")).clicked() {
                            self.selected.clear();
                        }
                    });
                    ui.small(format!(
                        "{}: {} / {}",
                        tr_lang(lang, "已选", "selected"),
                        self.selected.len(),
                        self.models.len()
                    ));
                });
                egui::ScrollArea::vertical()
                    .id_source("models_scroll")
                    .max_height(160.0)
                    .show(ui, |ui| {
                        if self.models.is_empty() {
                            ui.label(tr_lang(
                                lang,
                                "（还没加载模型，点顶部“刷新模型”）",
                                "(no models loaded yet — click Refresh models)",
                            ));
                            if ui.button(tr_lang(lang, "刷新模型", "Refresh models")).clicked() {
                                self.refresh_models();
                            }
                            return;
                        }
                        let enabled = !self.sharing_active;
                        ui.add_enabled_ui(enabled, |ui| {
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
                    });

                ui.add_space(8.0);
                ui.small(tr_lang(
                        lang,
                        "（对话请点击左侧💬）",
                        "(Open chat from the left 💬)",
                ));
            } else {
                // Create/join controls live here (collapsed by default).
                ui.horizontal(|ui| {
                    if ui
                        .button(tr_lang(lang, "新建组", "New group"))
                        .clicked()
                    {
                        self.show_create_group_ui = !self.show_create_group_ui;
                        if self.show_create_group_ui {
                            self.show_join_group_ui = false;
                        }
                    }

                    if ui
                        .button(tr_lang(lang, "加入组", "Join group"))
                        .clicked()
                    {
                        self.show_join_group_ui = !self.show_join_group_ui;
                        if self.show_join_group_ui {
                            self.show_create_group_ui = false;
                        }
                    }
                });

                if self.show_create_group_ui {
                    ui.group(|ui| {
                        ui.label(tr_lang(lang, "新组名称", "New group name"));
                        ui.text_edit_singleline(&mut self.create_group_name);
                        ui.horizontal(|ui| {
                            if ui.button(tr_lang(lang, "建立…", "Create…")).clicked() {
                                self.start_create_group();
                            }
                            if ui.button(tr_lang(lang, "收起", "Collapse")).clicked() {
                                self.show_create_group_ui = false;
                            }
                        });

                        if let Some(id) = self.pending_create_group_id.clone() {
                            let name = self
                                .pending_create_group_name
                                .clone()
                                .unwrap_or_else(|| "".to_string());
                            ui.separator();
                            ui.label(tr_lang(lang, "待确认建立", "Pending create"));
                            if !name.trim().is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(tr_lang(lang, "组名:", "Name:"));
                                    ui.monospace(&name);
                                });
                            }
                            ui.horizontal(|ui| {
                                ui.label(tr_lang(lang, "Group ID:", "Group ID:"));
                                ui.monospace(&id);
                            });
                            ui.horizontal(|ui| {
                                if ui.button(tr_lang(lang, "确认建立", "Confirm create")).clicked() {
                                    self.confirm_create_group();
                                }
                                if ui.button(tr_lang(lang, "取消", "Cancel")).clicked() {
                                    self.pending_create_group_id = None;
                                    self.pending_create_group_name = None;
                                }
                            });
                        }
                    });
                    ui.add_space(8.0);
                }

                if self.show_join_group_ui {
                    ui.group(|ui| {
                        ui.label(tr_lang(lang, "加入 Group ID", "Join Group ID"));
                        ui.text_edit_singleline(&mut self.join_group_id);
                        ui.horizontal(|ui| {
                            if ui.button(tr_lang(lang, "加入…", "Join…")).clicked() {
                                self.start_join_group();
                            }
                            if ui.button(tr_lang(lang, "收起", "Collapse")).clicked() {
                                self.show_join_group_ui = false;
                            }
                        });

                        if let Some(id) = self.pending_join_group_id.clone() {
                            ui.separator();
                            ui.label(tr_lang(lang, "待确认加入", "Pending join"));
                            ui.horizontal(|ui| {
                                ui.label(tr_lang(lang, "Group ID:", "Group ID:"));
                                ui.monospace(&id);
                            });
                            ui.horizontal(|ui| {
                                if ui.button(tr_lang(lang, "确认加入", "Confirm join")).clicked() {
                                    self.confirm_join_group();
                                }
                                if ui.button(tr_lang(lang, "取消", "Cancel")).clicked() {
                                    self.pending_join_group_id = None;
                                }
                            });
                        }
                    });
                    ui.add_space(8.0);
                }

                ui.heading(tr_lang(lang, "本地组", "My groups"));

                if self.my_groups.is_empty() {
                    ui.small(tr_lang(
                        lang,
                        "（本机还没有组：从顶部菜单“组/Groups”里建立或加入）",
                        "(no local groups yet — use the top menu Groups to create/join)",
                    ));
                } else {
                    egui::ScrollArea::vertical()
                        .id_source("my_groups_scroll")
                        .max_height(160.0)
                        .show(ui, |ui| {
                            for gid in self.my_groups.clone() {
                                let mut open = false;
                                let mut leave = false;
                                let mut save_now = false;

                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(tr_lang(lang, "组名", "Name"));
                                        let name_entry = self.group_names.entry(gid.clone()).or_insert_with(String::new);
                                        let resp = ui.text_edit_singleline(name_entry);
                                        if (resp.lost_focus()
                                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                                            || (resp.changed() && !resp.has_focus())
                                        {
                                            save_now = true;
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button(tr_lang(lang, "退出", "Leave")).clicked() {
                                                    leave = true;
                                                }
                                                if ui.button(tr_lang(lang, "打开", "Open")).clicked() {
                                                    open = true;
                                                }
                                            },
                                        );
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(tr_lang(lang, "Group ID", "Group ID"));
                                        ui.monospace(&gid);
                                    });
                                });

                                if save_now {
                                    self.save_persisted_state();
                                }
                                if open {
                                    self.active_group = Some(gid.clone());
                                }
                                if leave {
                                    self.leave_group(&gid);
                                }

                                ui.add_space(6.0);
                            }
                        });
                }

                ui.add_space(10.0);
                ui.heading(tr_lang(lang, "Ollama组列表", "Ollama groups"));
                ui.separator();

                // Always show groups you created/joined, even if no one is sharing yet.
                let mut groups: BTreeMap<String, Vec<(String, String, Vec<String>)>> = BTreeMap::new();
                for gid in &self.my_groups {
                    groups.entry(gid.clone()).or_default();
                }
                for p in &self.peers {
                    let Some(o) = &p.ollama else { continue; };
                    if !o.enabled {
                        continue;
                    }
                    let name = p.name.clone().unwrap_or_else(|| p.id.chars().take(8).collect());
                    let addr = p.address.clone();
                    let models = o.models.clone();
                    for group_id in &p.groups {
                        groups
                            .entry(group_id.clone())
                            .or_default()
                            .push((name.clone(), addr.clone(), models.clone()));
                    }
                }

                if groups.is_empty() {
                    ui.small(tr_lang(
                        lang,
                        "（暂无组：从顶部菜单“组/Groups”里建立或加入）",
                        "(no groups yet — use the top menu Groups to create/join)",
                    ));
                } else {
                    egui::ScrollArea::vertical()
                        .id_source("ollama_groups_scroll")
                        .show(ui, |ui| {
                            for (gid, members) in groups {
                                let gid_for_open = gid.clone();
                                let gname = self.group_display_name(&gid);
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new("👥")
                                                .size(18.0)
                                                .strong(),
                                        );
                                        ui.vertical(|ui| {
                                            ui.small(tr_lang(lang, "Ollama组", "Ollama group"));
                                            ui.monospace(format!(
                                                "{} {}",
                                                tr_lang(lang, "组名=", "name="),
                                                gname
                                            ));
                                            ui.monospace(&gid);
                                        });
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button(tr_lang(lang, "打开", "Open")).clicked() {
                                                    self.active_group = Some(gid_for_open.clone());
                                                }
                                            },
                                        );
                                    });
                                    ui.add_space(6.0);

                                    if members.is_empty() {
                                        ui.small(tr_lang(
                                            lang,
                                            "（该组暂时没有成员在共享 Ollama）",
                                            "(no shared members yet)",
                                        ));
                                    } else {
                                        for (name, addr, models) in members {
                                            ui.monospace(&name);
                                            ui.small(&addr);
                                            if !models.is_empty() {
                                                ui.small(format!("models: {}", models.join(", ")));
                                            }
                                            ui.add_space(6.0);
                                        }
                                    }
                                });
                                ui.add_space(8.0);
                            }
                        });
                }
            }
        });

        ctx.request_repaint_after(Duration::from_millis(200));
    }
}

async fn get_local_ip_best_effort() -> Option<String> {
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await.ok()?;
    socket.connect("8.8.8.8:80").await.ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}
