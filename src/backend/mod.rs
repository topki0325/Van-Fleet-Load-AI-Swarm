pub mod api_manager;
pub mod agent_scheduler;
pub mod compilation_scheduler;
pub mod network_discovery;
pub mod agents;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::shared::models::{Project, ResourceLease};

pub struct BackendServices {
    pub api_manager: ApiKeyManager,
    pub agent_scheduler: AgentScheduler,
    pub network_discovery: NetworkDiscovery,
    pub compilation_scheduler: CompilationScheduler,

    pub projects: Arc<RwLock<Vec<Project>>>,
    pub leases: Arc<RwLock<Vec<ResourceLease>>>,
}

pub use api_manager::ApiKeyManager;
pub use agent_scheduler::AgentScheduler;
pub use compilation_scheduler::CompilationScheduler;
pub use network_discovery::NetworkDiscovery;