//! Core workflow types: projects, agents, tasks, scheduling, etc.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::collections::HashMap;

// ─── Type aliases ─────────────────────────────────────────────────────────────

pub type ProjectId = Uuid;
pub type AgentId = Uuid;
pub type TaskId = Uuid;

// ─── Project ──────────────────────────────────────────────────────────────────

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
        self.workflow = WorkflowGraph::default();
    }

    pub fn validate_and_snapshot(&self) -> Result<Snapshot, VgaError> {
        Ok(Snapshot {
            project_id: self.id,
            timestamp: Utc::now(),
            data: vec![],
        })
    }
}

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
        Self { nodes: vec![], edges: vec![] }
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
pub struct ProjectResult {
    pub project_id: ProjectId,
    pub status: String,
}

// ─── Agent ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub role: AgentType,
    pub status: AgentStatus,
    pub skills: SkillVector,
    pub current_task: Option<TaskId>,
    pub performance: PerfMetrics,
    pub heartbeat: DateTime<Utc>,
}

impl Agent {
    pub async fn execute_block(&self, _task_spec: TaskSpec) -> Result<TaskOutput, VgaError> {
        Ok(TaskOutput::default())
    }

    pub fn is_overloaded(&self) -> bool {
        false
    }
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

#[derive(Debug)]
pub struct GatlingState {
    pub available_pool: Vec<AgentId>,
    pub rotation_index: std::sync::atomic::AtomicUsize,
    pub max_concurrency: usize,
    pub waiting_queue: Vec<TaskId>,
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

// ─── Task ─────────────────────────────────────────────────────────────────────

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
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub language: String,
    pub target: String,
    pub context_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    pub metadata: HashMap<String, String>,
}

pub struct TaskHandle {
    pub task_id: TaskId,
    pub handle: tokio::task::JoinHandle<Result<TaskOutput, VgaError>>,
}

impl TaskHandle {
    pub fn new(task_id: TaskId, handle: tokio::task::JoinHandle<Result<TaskOutput, VgaError>>) -> Self {
        Self { task_id, handle }
    }
}

// ─── Misc core types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VgaError {
    AuthVaultError(String),
    AgentTimeout(AgentId),
    EnvironmentLockError,
    NetworkSplit,
    CompileFailure(String),
    ResourceLimit(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSpec {
    pub language: String,
    pub requirements: Vec<String>,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSpec {
    pub language: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvPath {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetBinary {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmPulse {
    pub total_agents: usize,
    pub active_tasks: usize,
    pub queue_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextManager {
    pub memory_slots: HashMap<String, String>,
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
pub struct ResourceLease {
    pub id: String,
    pub gpu_memory: u64,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeReq {
    pub gpu_required: bool,
    pub memory_mb: u64,
    pub duration_secs: u64,
}

/// The client operating mode (master orchestrator or slave worker).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMode {
    Master,
    Slave,
}
