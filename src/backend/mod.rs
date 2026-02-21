pub mod api_manager;
pub mod agent_scheduler;
pub mod compilation_scheduler;
pub mod network_discovery;
pub mod resource_manager;
pub mod agents;
pub mod provider_config;
pub mod c_compiler;
pub mod ollama_client;
pub mod skills;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::shared::models::{Project, ResourceLease};

pub struct BackendServices {
    pub api_manager: ApiKeyManager,
    pub agent_scheduler: AgentScheduler,
    pub network_discovery: NetworkDiscovery,
    pub compilation_scheduler: CompilationScheduler,
    pub resource_manager: ResourceManager,
    pub c_compiler: CCompilationScheduler,
    pub ollama_manager: ollama_client::OllamaManager,

    pub projects: Arc<RwLock<Vec<Project>>>,
    pub leases: Arc<RwLock<Vec<ResourceLease>>>,
}

pub use api_manager::ApiKeyManager;
pub use agent_scheduler::AgentScheduler;
pub use compilation_scheduler::CompilationScheduler;
pub use network_discovery::NetworkDiscovery;
pub use resource_manager::ResourceManager;
pub use c_compiler::CCompilationScheduler;
pub use ollama_client::OllamaManager;
pub use skills::{SkillEntry, SkillRepository};