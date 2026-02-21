use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

pub type ProjectId = Uuid;
pub type AgentId = Uuid;
pub type TaskId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub config: ProjectConfig,
    pub agents: Vec<AgentId>,
    pub workflow: WorkflowGraph,
    pub state: ProjectStatus,
    pub stats: ExecutionStats,
    pub last_updated: DateTime<Utc>,
}

impl Project {
    pub fn initialize_workflow(&mut self) {
        // TODO: Implement workflow initialization
        self.workflow = WorkflowGraph::default();
    }

    pub fn validate_and_snapshot(&self) -> Result<Snapshot, VgaError> {
        // TODO: Implement snapshot creation
        Ok(Snapshot {
            project_id: self.id,
            timestamp: Utc::now(),
            data: vec![], // Placeholder
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub role: AgentType,
    pub status: AgentStatus,
    pub skills: SkillVector,
    pub current_task: Option<TaskId>,
    pub performance: PerfMetrics,
    pub heartbeat: DateTime<Utc>, // Changed from Instant to DateTime for serialization
}

impl Agent {
    pub async fn execute_block(&self, _task_spec: TaskSpec) -> Result<TaskOutput, VgaError> {
        // TODO: Implement agent execution
        Ok(TaskOutput::default())
    }

    pub fn is_overloaded(&self) -> bool {
        // TODO: Implement overload check
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub parent_id: Option<TaskId>,
    pub spec: TaskSpec,
    pub priority: Priority,
    pub assigned_to: Option<AgentId>,
    pub status: TaskStatus,
    pub input_snapshot: PathBuf,
    pub output: TaskResult,
    pub retry_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(spec: TaskSpec, priority: Priority, input_snapshot: PathBuf) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new_v4(),
            parent_id: None,
            spec,
            priority,
            assigned_to: None,
            status: TaskStatus::Pending,
            input_snapshot,
            output: TaskResult::Failure("Not executed yet".into()),
            retry_count: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn finalize_with_result(&mut self, res: TaskResult) {
        self.output = res;
        self.updated_at = Utc::now();
    }

    pub fn check_dependencies(&self, _context: &WorkflowGraph) -> bool {
        // TODO: Implement dependency check
        true
    }
}

#[derive(Debug)]
pub struct GatlingState {
    pub available_pool: Vec<AgentId>,
    pub rotation_index: std::sync::atomic::AtomicUsize,
    pub max_concurrency: usize,
    pub waiting_queue: Vec<TaskId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VgaError {
    AuthVaultError(String),
    AgentTimeout(AgentId),
    EnvironmentLockError,
    NetworkSplit,
    CompileFailure(String),
    ResourceLimit(String),
}

#[async_trait::async_trait]
pub trait AgentTrait {
    async fn execute_instruction(&self, instr: String) -> Result<TaskOutput, VgaError>;
    async fn execute_block(&self, _task_spec: TaskSpec) -> Result<TaskOutput, VgaError> {
        Err(VgaError::CompileFailure("Not implemented for this agent type".into()))
    }
    fn update_context(&mut self, context: &ContextManager);
    fn get_metrics(&self) -> PerfMetrics;
}

// Additional structs and enums as per documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub tech_stack: Vec<String>,
    pub default_provider: String,
    pub concurrency_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGraph {
    pub nodes: Vec<TaskId>,
    pub edges: Vec<(TaskId, TaskId)>,
}

impl Default for WorkflowGraph {
    fn default() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Initialized,
    Running,
    Suspended,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub total_tokens: u64,
    pub total_duration: std::time::Duration,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub project_id: ProjectId,
    pub timestamp: DateTime<Utc>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    ArchitectNode,
    ProgrammerNode,
    SecurityNode,
    DocManager,
    EnvManagerNode,
    ClusterResourceManager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Idle,
    Busy,
    Offline,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVector {
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub avg_response_time: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub language: String,
    pub target: String,
    pub context_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    Success(TaskOutput),
    Failure(String),
    Conflict(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskOutput {
    pub content: String,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSpec {
    pub language: String,
    pub requirements: Vec<String>,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingReport {
    pub provider: String,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultUsageEntry {
    pub provider: String,
    pub requests_made: u64,
    pub last_used: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultOp {
    Store { provider: String, key: String },
    Retrieve { provider: String },
    Delete { provider: String },
    List,
    GetProviders,
    GetProviderConfig { provider: String },
    SetDefaultProvider { provider: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultResult {
    Success,
    Key(String),
    Providers(Vec<String>),
    ProviderConfigs(Vec<ProviderConfig>),
    ProviderConfig(ProviderConfig),
    DefaultProvider(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub region: ProviderRegion,
    pub api_endpoint: String,
    pub models: Vec<String>,
    pub pricing: PricingInfo,
    pub requires_api_key: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderRegion {
    China,
    USA,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub currency: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub free_tier_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResult {
    pub project_id: ProjectId,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeReq {
    pub gpu_required: bool,
    pub memory_mb: u64,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLease {
    pub id: String,
    pub gpu_memory: u64,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSpec {
    pub language: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvPath {
    pub path: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetBinary {
    pub path: std::path::PathBuf,
}

#[derive(Debug)]
pub struct TaskHandle {
    pub task_id: TaskId,
    pub handle: tokio::task::JoinHandle<Result<TaskOutput, VgaError>>,
}

impl TaskHandle {
    pub fn new(task_id: TaskId, handle: tokio::task::JoinHandle<Result<TaskOutput, VgaError>>) -> Self {
        Self { task_id, handle }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmPulse {
    pub total_agents: usize,
    pub active_tasks: usize,
    pub queue_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    pub memory_slots: std::collections::HashMap<String, String>,
    pub docs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEntry {
    pub task_id: TaskId,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMode {
    Master,
    Slave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStatus {
    pub id: String,
    pub address: String,
    pub mode: ClientMode,
    pub latency: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct BuildPlan {
    pub project: Project,
    pub segments: Vec<BuildSegment>,
}

#[derive(Debug, Clone)]
pub struct BuildSegment {
    pub language: String,
    pub files: Vec<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildUpdate {
    pub segment_id: String,
    pub status: BuildStatus,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOutput {
    pub segment_id: String,
    pub binary_path: std::path::PathBuf,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildStatus {
    Pending,
    InProgress,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub task_id: TaskId,
    pub conflicts: Vec<String>,
    pub resolution_options: Vec<String>,
}

// Resource Management Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub address: String,
    pub port: u16,
    pub mode: ClientMode,
    pub resources: NodeResources,
    pub allow_remote_access: bool,
    pub last_seen: DateTime<Utc>,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu_cores: u32,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub gpus: Vec<GpuInfo>,
    pub supported_models: Vec<String>,
    pub current_load: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub id: String,
    pub name: String,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub utilization: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Online,
    Busy,
    Offline,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub request_id: String,
    pub requester_id: String,
    pub required_resources: ResourceRequirements,
    pub task_type: String,
    pub priority: Priority,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<u32>,
    pub memory_mb: Option<u64>,
    pub gpu_required: bool,
    pub gpu_memory_mb: Option<u64>,
    pub preferred_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub node_id: String,
    pub request_id: String,
    pub allocated_resources: AllocatedResources,
    pub status: AllocationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResources {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub gpu: Option<AllocatedGpu>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedGpu {
    pub gpu_id: String,
    pub memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationStatus {
    Pending,
    Active,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub node_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: NodeStatus,
    pub resources: NodeResources,
    pub active_allocations: usize,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStrategy {
    pub strategy_type: BalancingStrategy,
    pub weights: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalancingStrategy {
    RoundRobin,
    LeastLoaded,
    Weighted,
    Geographic,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTask {
    pub task_id: TaskId,
    pub spec: TaskSpec,
    pub assigned_node: Option<String>,
    pub status: DistributedTaskStatus,
    pub result: Option<TaskOutput>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributedTaskStatus {
    Pending,
    Dispatched,
    Running,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmGroup {
    pub group_id: String,
    pub name: String,
    pub members: Vec<String>,
    pub leader_id: String,
    pub created_at: DateTime<Utc>,
    pub max_members: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    pub pool_id: String,
    pub name: String,
    pub nodes: Vec<String>,
    pub total_resources: NodeResources,
    pub available_resources: NodeResources,
}

use std::collections::HashMap;
