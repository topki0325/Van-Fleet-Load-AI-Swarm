//! Distributed resource management: nodes, pools, groups, allocations, balancing.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use super::core::{ClientMode, Priority, TaskId, TaskSpec, TaskOutput};

// ─── Node information ─────────────────────────────────────────────────────────

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

// ─── Resource requests & allocations ─────────────────────────────────────────

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

// ─── Health check ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub node_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: NodeStatus,
    pub resources: NodeResources,
    pub active_allocations: usize,
    pub response_time_ms: u64,
}

// ─── Load balancing ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingStrategy {
    pub strategy_type: BalancingStrategy,
    pub weights: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BalancingStrategy {
    RoundRobin,
    LeastLoaded,
    Weighted,
    Geographic,
    Random,
}

// ─── Distributed tasks ────────────────────────────────────────────────────────

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

// ─── Swarm groups & resource pools ───────────────────────────────────────────

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
