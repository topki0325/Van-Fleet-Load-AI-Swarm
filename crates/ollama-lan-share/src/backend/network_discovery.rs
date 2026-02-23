use crate::shared::models::{ClientMode, OllamaOfferStatus, PeerStatus, VgaError};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use uuid;

type HmacSha256 = Hmac<sha2::Sha256>;

const DISCOVERY_PORT: u16 = 45555;
const BROADCAST_ADDR: &str = "255.255.255.255:45555";
const ANNOUNCE_INTERVAL_SECS: u64 = 10;
const PEER_STALE_SECS: u64 = 5 * 60;
const EMPTY_LOG_EVERY_SECS: u64 = 30;
const MAX_PEERS: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryDebugStats {
    pub socket_bound: bool,
    pub bind: Option<String>,

    pub sent_announces: u64,
    pub sent_queries: u64,
    pub received_announces: u64,
    pub received_queries: u64,

    pub last_received_from: Option<String>,
    pub last_received_kind: Option<String>,
    pub last_received_age_ms: Option<u64>,
}

#[derive(Clone)]
pub struct NetworkDiscovery {
    node_id: String,
    mode: ClientMode,
    rt: Option<tokio::runtime::Handle>,
    local_name: Arc<RwLock<String>>,
    local_groups: Arc<RwLock<Vec<String>>>,
    local_ollama_offer: Arc<RwLock<OllamaOfferStatus>>,
    auth_key: Arc<RwLock<Option<Vec<u8>>>>,
    socket: Option<Arc<UdpSocket>>,
    discovered_peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    debug: Arc<RwLock<DiscoveryDebugStats>>,
    last_received_at: Arc<RwLock<Option<Instant>>>,
    last_empty_log: Arc<RwLock<Instant>>,
}

#[derive(Clone, Debug)]
struct PeerInfo {
    status: PeerStatus,
    last_seen: Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryPacket {
    kind: DiscoveryPacketKind,
    status: PeerStatus,
    #[serde(default)]
    mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DiscoveryPacketKind {
    Announce,
    Query,
}

impl NetworkDiscovery {
    pub async fn new() -> Self {
        let rt = tokio::runtime::Handle::try_current().ok();
        let node_id = uuid::Uuid::new_v4().to_string();
        let local_name = Arc::new(RwLock::new(get_machine_name()));
        let local_groups = Arc::new(RwLock::new(Vec::new()));
        let local_ollama_offer = Arc::new(RwLock::new(OllamaOfferStatus {
            enabled: false,
            base_url: Some("http://localhost:11434".to_string()),
            models: Vec::new(),
            auth_required: false,
            proxy_port: None,
        }));

        let auth_key: Arc<RwLock<Option<Vec<u8>>>> = Arc::new(RwLock::new(None));

        let debug = Arc::new(RwLock::new(DiscoveryDebugStats::default()));
        let socket = match UdpSocket::bind(format!("0.0.0.0:{DISCOVERY_PORT}")).await {
            Ok(sock) => {
                if let Err(e) = sock.set_broadcast(true) {
                    tracing::warn!("Failed to enable UDP broadcast: {e}");
                }
                {
                    let mut d = debug.write().await;
                    d.socket_bound = true;
                    d.bind = sock.local_addr().ok().map(|a| a.to_string());
                }
                Some(Arc::new(sock))
            }
            Err(e) => {
                tracing::warn!("Network discovery disabled (bind 0.0.0.0:{DISCOVERY_PORT} failed): {e}");
                {
                    let mut d = debug.write().await;
                    d.socket_bound = false;
                }
                None
            }
        };

        let this = Self {
            node_id,
            mode: ClientMode::Master,
            rt,
            local_name,
            local_groups,
            local_ollama_offer,
            auth_key,
            socket: socket.clone(),
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
            debug,
            last_received_at: Arc::new(RwLock::new(None)),
            last_empty_log: Arc::new(RwLock::new(Instant::now() - Duration::from_secs(EMPTY_LOG_EVERY_SECS))),
        };

        if let Some(sock) = socket {
            this.start_background(sock);
        }

        this
    }

    fn spawn(&self, fut: impl Future<Output = ()> + Send + 'static) {
        if let Some(rt) = &self.rt {
            rt.spawn(fut);
        } else {
            tracing::warn!("NetworkDiscovery: no Tokio runtime handle available; skipping background task");
        }
    }

    pub async fn debug_stats(&self) -> DiscoveryDebugStats {
        let mut d = self.debug.read().await.clone();
        let last = self.last_received_at.read().await;
        d.last_received_age_ms = last.map(|t| t.elapsed().as_millis() as u64);
        d
    }

    pub fn local_node_id(&self) -> &str {
        &self.node_id
    }

    pub async fn local_node_name(&self) -> String {
        self.local_name.read().await.clone()
    }

    pub async fn set_local_node_name(&self, name: String) {
        *self.local_name.write().await = name;
    }

    pub async fn set_local_groups(&self, groups: Vec<String>) {
        *self.local_groups.write().await = groups;
    }

    /// Set an optional shared key used to authenticate discovery packets.
    /// When set, incoming packets without a valid MAC are ignored.
    pub async fn set_auth_key(&self, key: Option<String>) {
        let v = key
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| s.into_bytes());
        *self.auth_key.write().await = v;
    }

    pub async fn clear_discovered_peers(&self) {
        self.discovered_peers.write().await.clear();
    }

    pub async fn set_ollama_offer(&self, enabled: bool, models: Vec<String>, base_url: Option<String>) {
        let mut offer = self.local_ollama_offer.write().await;
        offer.enabled = enabled;
        offer.models = models;
        if let Some(url) = base_url {
            offer.base_url = Some(url);
        }
    }

    pub fn broadcast_presence(&self) {
        let Some(sock) = self.socket.clone() else {
            return;
        };
        let node_id = self.node_id.clone();
        let mode = self.mode.clone();
        let local_name = self.local_name.clone();
        let local_groups = self.local_groups.clone();
        let local_offer = self.local_ollama_offer.clone();
        let auth_key = self.auth_key.clone();
        let debug = self.debug.clone();
        self.spawn(async move {
            let status = build_local_status(&node_id, mode, &local_name, &local_groups, &local_offer, None).await;
            let mut packet = DiscoveryPacket {
                kind: DiscoveryPacketKind::Announce,
                status,
                mac: None,
            };

            if let Some(key) = auth_key.read().await.clone() {
                packet.mac = compute_packet_mac(&key, &packet.kind, &packet.status);
            }
            if let Ok(data) = serde_json::to_vec(&packet) {
                let _ = sock.send_to(&data, BROADCAST_ADDR).await;
                let mut d = debug.write().await;
                d.sent_announces = d.sent_announces.saturating_add(1);
            }
        });
    }

    pub async fn discover_peers(&self) -> Result<Vec<PeerStatus>, VgaError> {
        let peers = self.discovered_peers.read().await;
        let out: Vec<PeerStatus> = peers
            .values()
            .map(|peer| {
                let mut s = peer.status.clone();
                s.latency = Some(peer.last_seen.elapsed().as_millis() as u64);
                s
            })
            .collect();

        if out.is_empty() {
            let mut last = self.last_empty_log.write().await;
            if last.elapsed().as_secs() >= EMPTY_LOG_EVERY_SECS {
                tracing::info!("No peers discovered for {}", self.node_id);
                *last = Instant::now();
            }
        }

        Ok(out)
    }

    fn start_background(&self, socket: Arc<UdpSocket>) {
        let discovered = self.discovered_peers.clone();
        let node_id = self.node_id.clone();
        let mode = self.mode.clone();
        let local_name = self.local_name.clone();
        let local_groups = self.local_groups.clone();
        let local_offer = self.local_ollama_offer.clone();
        let auth_key_recv = self.auth_key.clone();
        let debug_recv = self.debug.clone();
        let last_received_at = self.last_received_at.clone();

        // Receiver
        let socket_recv = socket.clone();
        self.spawn(async move {
            let mut buf = [0u8; 64 * 1024];
            loop {
                let Ok((len, addr)) = socket_recv.recv_from(&mut buf).await else {
                    continue;
                };

                let Ok(packet) = serde_json::from_slice::<DiscoveryPacket>(&buf[..len]) else {
                    continue;
                };

                // If auth is enabled, require a valid MAC.
                if let Some(key) = auth_key_recv.read().await.clone() {
                    if !verify_packet_mac(&key, &packet) {
                        continue;
                    }
                }

                if packet.status.id == node_id {
                    continue;
                }

                match packet.kind {
                    DiscoveryPacketKind::Announce => {
                        *last_received_at.write().await = Some(Instant::now());
                        {
                            let mut d = debug_recv.write().await;
                            d.received_announces = d.received_announces.saturating_add(1);
                            d.last_received_from = Some(addr.to_string());
                            d.last_received_kind = Some("announce".to_string());
                            d.last_received_age_ms = None;
                        }
                        let mut status = packet.status;
                        status.address = normalize_addr(status.address, addr);
                        let mut peers = discovered.write().await;
                        if !peers.contains_key(&status.id) && peers.len() >= MAX_PEERS {
                            // Avoid unbounded growth if someone floods spoofed node ids on the LAN.
                            continue;
                        }
                        peers.insert(
                            status.id.clone(),
                            PeerInfo {
                                status,
                                last_seen: Instant::now(),
                            },
                        );
                    }
                    DiscoveryPacketKind::Query => {
                        *last_received_at.write().await = Some(Instant::now());
                        {
                            let mut d = debug_recv.write().await;
                            d.received_queries = d.received_queries.saturating_add(1);
                            d.last_received_from = Some(addr.to_string());
                            d.last_received_kind = Some("query".to_string());
                            d.last_received_age_ms = None;
                        }
                        let status = build_local_status(&node_id, mode.clone(), &local_name, &local_groups, &local_offer, Some(addr)).await;
                        let mut response = DiscoveryPacket {
                            kind: DiscoveryPacketKind::Announce,
                            status,
                            mac: None,
                        };
                        if let Some(key) = auth_key_recv.read().await.clone() {
                            response.mac = compute_packet_mac(&key, &response.kind, &response.status);
                        }
                        if let Ok(data) = serde_json::to_vec(&response) {
                            let _ = socket_recv.send_to(&data, addr).await;
                        }
                    }
                }
            }
        });

        // Broadcaster
        let socket_send = socket.clone();
        let node_id_send = self.node_id.clone();
        let mode_send = self.mode.clone();
        let local_name_send = self.local_name.clone();
        let local_groups_send = self.local_groups.clone();
        let local_offer_send = self.local_ollama_offer.clone();
        let auth_key_send = self.auth_key.clone();
        let debug_send = self.debug.clone();
        self.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(ANNOUNCE_INTERVAL_SECS));
            loop {
                interval.tick().await;
                let status = build_local_status(
                    &node_id_send,
                    mode_send.clone(),
                    &local_name_send,
                    &local_groups_send,
                    &local_offer_send,
                    None,
                )
                .await;
                let mut packet = DiscoveryPacket {
                    kind: DiscoveryPacketKind::Announce,
                    status,
                    mac: None,
                };
                if let Some(key) = auth_key_send.read().await.clone() {
                    packet.mac = compute_packet_mac(&key, &packet.kind, &packet.status);
                }
                if let Ok(data) = serde_json::to_vec(&packet) {
                    let _ = socket_send.send_to(&data, BROADCAST_ADDR).await;
                    let mut d = debug_send.write().await;
                    d.sent_announces = d.sent_announces.saturating_add(1);
                }
            }
        });

        // Cleanup
        let discovered_cleanup = self.discovered_peers.clone();
        self.spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let mut peers = discovered_cleanup.write().await;
                peers.retain(|_, v| v.last_seen.elapsed().as_secs() <= PEER_STALE_SECS);
            }
        });

        // Initial query (kickstart)
        let socket_query = socket;
        let node_id_q = self.node_id.clone();
        let mode_q = self.mode.clone();
        let local_name_q = self.local_name.clone();
        let local_groups_q = self.local_groups.clone();
        let local_offer_q = self.local_ollama_offer.clone();
        let auth_key_q = self.auth_key.clone();
        let debug_query = self.debug.clone();
        self.spawn(async move {
            let status = build_local_status(&node_id_q, mode_q, &local_name_q, &local_groups_q, &local_offer_q, None).await;
            let mut packet = DiscoveryPacket {
                kind: DiscoveryPacketKind::Query,
                status,
                mac: None,
            };
            if let Some(key) = auth_key_q.read().await.clone() {
                packet.mac = compute_packet_mac(&key, &packet.kind, &packet.status);
            }
            if let Ok(data) = serde_json::to_vec(&packet) {
                let _ = socket_query.send_to(&data, BROADCAST_ADDR).await;
                let mut d = debug_query.write().await;
                d.sent_queries = d.sent_queries.saturating_add(1);
            }
        });
    }
}

fn kind_byte(kind: &DiscoveryPacketKind) -> u8 {
    match kind {
        DiscoveryPacketKind::Announce => 1,
        DiscoveryPacketKind::Query => 2,
    }
}

fn compute_packet_mac(key: &[u8], kind: &DiscoveryPacketKind, status: &PeerStatus) -> Option<String> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(&[kind_byte(kind)]);
    let status_bytes = serde_json::to_vec(status).ok()?;
    mac.update(&status_bytes);
    Some(hex::encode(mac.finalize().into_bytes()))
}

fn verify_packet_mac(key: &[u8], packet: &DiscoveryPacket) -> bool {
    let Some(given) = &packet.mac else {
        return false;
    };
    let Some(expected) = compute_packet_mac(key, &packet.kind, &packet.status) else {
        return false;
    };
    constant_time_eq(given.as_bytes(), expected.as_bytes())
}

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

async fn build_local_status(
    node_id: &str,
    mode: ClientMode,
    local_name: &Arc<RwLock<String>>,
    local_groups: &Arc<RwLock<Vec<String>>>,
    local_offer: &Arc<RwLock<OllamaOfferStatus>>,
    reply_to: Option<SocketAddr>,
) -> PeerStatus {
    let name = local_name.read().await.clone();
    let groups = local_groups.read().await.clone();
    let offer: OllamaOfferStatus = local_offer.read().await.clone();

    PeerStatus {
        id: node_id.to_string(),
        address: reply_to
            .map(|a| a.to_string())
            .unwrap_or_else(|| format!("0.0.0.0:{DISCOVERY_PORT}")),
        mode,
        latency: None,
        name: Some(name),
        groups,
        ollama: Some(offer),
    }
}

fn normalize_addr(payload_addr: String, recv_addr: SocketAddr) -> String {
    // Treat the UDP sender IP as the source of truth, otherwise a malicious host can
    // broadcast a packet claiming to be at a different LAN address.
    let recv_ip = recv_addr.ip();
    let recv_norm = format!("{}:{}", recv_ip, DISCOVERY_PORT);

    let p = payload_addr.trim();
    if p.is_empty() || p.starts_with("0.0.0.0") {
        return recv_norm;
    }

    // If the payload address parses and matches the sender IP, keep it; otherwise normalize.
    match p.parse::<SocketAddr>() {
        Ok(sa) if sa.ip() == recv_ip => payload_addr,
        _ => recv_norm,
    }
}

fn get_machine_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "vas-node".to_string())
}