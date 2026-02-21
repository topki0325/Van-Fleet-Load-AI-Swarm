use crate::shared::models::{VgaError, ClientMode, PeerStatus};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid;

#[derive(Clone)]
pub struct NetworkDiscovery {
    node_id: String,
    mode: ClientMode,
    discovered_peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
}

#[derive(Clone, Debug)]
struct PeerInfo {
    id: String,
    address: String,
    mode: ClientMode,
    last_seen: std::time::Instant,
}

impl NetworkDiscovery {
    pub async fn new() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            mode: ClientMode::Master, // Default to master
            discovered_peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn broadcast_presence(&self) {
        // TODO: Implement mDNS broadcasting
        tracing::info!("Broadcasting presence for {} as {:?}", self.node_id, self.mode);
    }

    pub async fn discover_peers(&self) -> Result<Vec<PeerStatus>, VgaError> {
        let peers = self.discovered_peers.read().await;
        let out: Vec<PeerStatus> = peers
            .values()
            .map(|peer| PeerStatus {
                id: peer.id.clone(),
                address: peer.address.clone(),
                mode: peer.mode.clone(),
                latency: Some(peer.last_seen.elapsed().as_millis() as u64),
            })
            .collect();

        if out.is_empty() {
            tracing::info!("No peers discovered for {}", self.node_id);
        }

        Ok(out)
    }
}