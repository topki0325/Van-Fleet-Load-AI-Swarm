use crate::shared::models::{
    VgaError, ClientMode, NodeInfo, NodeResources, GpuInfo, NodeStatus,
    ResourceRequest, ResourceRequirements, ResourceAllocation, AllocatedResources,
    AllocatedGpu, AllocationStatus, HealthCheck, LoadBalancingStrategy,
    BalancingStrategy, DistributedTask, DistributedTaskStatus, SwarmGroup,
    ResourcePool, TaskId, TaskSpec, Priority,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use std::time::Duration;
use tokio::net::UdpSocket;
use serde_json;
use serde::{Deserialize, Serialize};

pub struct ResourceManager {
    node_id: String,
    node_info: Arc<RwLock<NodeInfo>>,
    discovered_nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    swarm_groups: Arc<RwLock<HashMap<String, SwarmGroup>>>,
    resource_pools: Arc<RwLock<HashMap<String, ResourcePool>>>,
    allocations: Arc<RwLock<HashMap<String, ResourceAllocation>>>,
    distributed_tasks: Arc<RwLock<HashMap<TaskId, DistributedTask>>>,
    balancing_strategy: Arc<RwLock<LoadBalancingStrategy>>,
    allow_remote_access: Arc<RwLock<bool>>,
    discovery_socket: Option<Arc<UdpSocket>>,
    broadcast_port: u16,
}

impl ResourceManager {
    pub async fn new(allow_remote: bool) -> Result<Self, VgaError> {
        let node_id = Uuid::new_v4().to_string();
        let broadcast_port = Self::find_available_port(8080, 8100).await?;
        
        let node_info = NodeInfo {
            id: node_id.clone(),
            address: Self::get_local_ip().await?,
            port: broadcast_port,
            mode: ClientMode::Master,
            resources: Self::detect_local_resources().await,
            allow_remote_access: allow_remote,
            last_seen: Utc::now(),
            status: NodeStatus::Online,
        };

        let socket = UdpSocket::bind(format!("0.0.0.0:{}", broadcast_port))
            .await
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to bind socket: {}", e)))?;

        Ok(Self {
            node_id,
            node_info: Arc::new(RwLock::new(node_info)),
            discovered_nodes: Arc::new(RwLock::new(HashMap::new())),
            swarm_groups: Arc::new(RwLock::new(HashMap::new())),
            resource_pools: Arc::new(RwLock::new(HashMap::new())),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            distributed_tasks: Arc::new(RwLock::new(HashMap::new())),
            balancing_strategy: Arc::new(RwLock::new(LoadBalancingStrategy {
                strategy_type: BalancingStrategy::LeastLoaded,
                weights: HashMap::new(),
            })),
            allow_remote_access: Arc::new(RwLock::new(allow_remote)),
            discovery_socket: Some(Arc::new(socket)),
            broadcast_port,
        })
    }

    async fn get_local_ip() -> Result<String, VgaError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to bind: {}", e)))?;
        
        socket.connect("8.8.8.8:80")
            .await
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to connect: {}", e)))?;
        
        let local_addr = socket.local_addr()
            .map_err(|e| VgaError::ResourceLimit(format!("Failed to get local addr: {}", e)))?;
        
        Ok(local_addr.ip().to_string())
    }

    async fn detect_local_resources() -> NodeResources {
        let cpu_cores = num_cpus::get() as u32;
        let total_memory = Self::get_total_memory_mb();
        
        NodeResources {
            cpu_cores,
            total_memory_mb: total_memory,
            available_memory_mb: total_memory,
            gpus: Self::detect_gpus().await,
            supported_models: vec![
                "gpt-4".to_string(),
                "gpt-3.5-turbo".to_string(),
                "claude-3".to_string(),
            ],
            current_load: 0.0,
        }
    }

    fn get_total_memory_mb() -> u64 {
        sys_info::mem_info()
            .map(|info| info.total / 1024)
            .unwrap_or(8192)
    }

    async fn detect_gpus() -> Vec<GpuInfo> {
        vec![]
    }

    async fn find_available_port(start: u16, end: u16) -> Result<u16, VgaError> {
        for port in start..=end {
            if let Ok(_socket) = UdpSocket::bind(format!("0.0.0.0:{}", port)).await {
                drop(_socket);
                return Ok(port);
            }
        }
        Err(VgaError::ResourceLimit(format!("No available port in range {}-{}", start, end)))
    }

    pub async fn start_discovery(&self) -> Result<(), VgaError> {
        tracing::info!(
            "Starting resource discovery for {} on port {}",
            self.node_id,
            self.broadcast_port
        );
        let socket = self.discovery_socket.clone()
            .ok_or_else(|| VgaError::ResourceLimit("Discovery socket not initialized".into()))?;

        let node_info = self.node_info.clone();
        let discovered_nodes = self.discovered_nodes.clone();
        let allow_remote = self.allow_remote_access.clone();
        let socket_recv = socket.clone();
        let node_info_recv = node_info.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 65536];
            
            loop {
                match socket_recv.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        if let Ok(message) = serde_json::from_slice::<DiscoveryMessage>(&buf[..len]) {
                            match message.message_type {
                                DiscoveryMessageType::Announce => {
                                    let allow = allow_remote.read().await;
                                    if *allow && message.node_info.allow_remote_access {
                                        let node_id = message.node_info.id.clone();
                                        let mut nodes = discovered_nodes.write().await;
                                        nodes.insert(node_id.clone(), message.node_info);
                                        tracing::info!("Discovered node: {} from {}", node_id, addr);
                                    }
                                }
                                DiscoveryMessageType::Query => {
                                    let info = node_info_recv.read().await;
                                    if info.allow_remote_access {
                                        let response = DiscoveryMessage {
                                            message_type: DiscoveryMessageType::Announce,
                                            node_info: info.clone(),
                                        };
                                        if let Ok(data) = serde_json::to_vec(&response) {
                                            let _ = socket_recv.send_to(&data, addr).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Discovery error: {}", e);
                    }
                }
            }
        });

        let socket_broadcast = socket.clone();
        let node_info_broadcast = node_info.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                let info = node_info_broadcast.read().await;
                if info.allow_remote_access {
                    let message = DiscoveryMessage {
                        message_type: DiscoveryMessageType::Announce,
                        node_info: info.clone(),
                    };
                    if let Ok(data) = serde_json::to_vec(&message) {
                        let _ = socket_broadcast.send_to(&data, "255.255.255.255:8080").await;
                    }
                }
            }
        });

        let discovered_nodes_cleanup = self.discovered_nodes.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                let now = Utc::now();
                let mut nodes = discovered_nodes_cleanup.write().await;
                let mut to_remove = Vec::new();
                
                for (node_id, node) in nodes.iter() {
                    let timeout = chrono::Duration::minutes(5);
                    if now.signed_duration_since(node.last_seen) > timeout {
                        to_remove.push(node_id.clone());
                        tracing::warn!("Removing stale node: {}", node_id);
                    }
                }
                
                for node_id in to_remove {
                    nodes.remove(&node_id);
                }
            }
        });

        Ok(())
    }

    pub async fn create_swarm_group(&self, name: String, max_members: usize) -> Result<String, VgaError> {
        let group_id = Uuid::new_v4().to_string();
        let node_info = self.node_info.read().await;
        
        let group = SwarmGroup {
            group_id: group_id.clone(),
            name,
            members: vec![node_info.id.clone()],
            leader_id: node_info.id.clone(),
            created_at: Utc::now(),
            max_members,
        };

        let mut groups = self.swarm_groups.write().await;
        groups.insert(group_id.clone(), group);

        Ok(group_id)
    }

    pub async fn join_swarm_group(&self, group_id: String) -> Result<(), VgaError> {
        let node_info = self.node_info.read().await;
        let mut groups = self.swarm_groups.write().await;

        if let Some(group) = groups.get_mut(&group_id) {
            if group.members.len() >= group.max_members {
                return Err(VgaError::ResourceLimit("Group is full".into()));
            }
            if !group.members.contains(&node_info.id) {
                group.members.push(node_info.id.clone());
            }
            Ok(())
        } else {
            Err(VgaError::ResourceLimit("Group not found".into()))
        }
    }

    pub async fn leave_swarm_group(&self, group_id: String) -> Result<(), VgaError> {
        let node_info = self.node_info.read().await;
        let mut groups = self.swarm_groups.write().await;

        if let Some(group) = groups.get_mut(&group_id) {
            group.members.retain(|id| id != &node_info.id);
            if group.leader_id == node_info.id && !group.members.is_empty() {
                group.leader_id = group.members[0].clone();
            }
            Ok(())
        } else {
            Err(VgaError::ResourceLimit("Group not found".into()))
        }
    }

    pub async fn list_swarm_groups(&self) -> Vec<SwarmGroup> {
        self.swarm_groups.read().await.values().cloned().collect()
    }

    pub async fn get_group_members(&self, group_id: String) -> Result<Vec<NodeInfo>, VgaError> {
        let groups = self.swarm_groups.read().await;
        let group = groups.get(&group_id)
            .ok_or_else(|| VgaError::ResourceLimit("Group not found".into()))?;

        let nodes = self.discovered_nodes.read().await;
        let mut members = Vec::new();

        for member_id in &group.members {
            if let Some(node) = nodes.get(member_id) {
                members.push(node.clone());
            }
        }

        Ok(members)
    }

    pub async fn set_remote_access(&self, allow: bool) {
        let mut allow_remote = self.allow_remote_access.write().await;
        *allow_remote = allow;
        
        let mut node_info = self.node_info.write().await;
        node_info.allow_remote_access = allow;
    }

    pub async fn get_remote_access_status(&self) -> bool {
        *self.allow_remote_access.read().await
    }

    pub async fn discover_nodes(&self) -> Result<Vec<NodeInfo>, VgaError> {
        let socket = self.discovery_socket.clone()
            .ok_or_else(|| VgaError::ResourceLimit("Discovery socket not initialized".into()))?;

        let message = DiscoveryMessage {
            message_type: DiscoveryMessageType::Query,
            node_info: self.node_info.read().await.clone(),
        };

        let data = serde_json::to_vec(&message)
            .map_err(|e| VgaError::ResourceLimit(format!("Serialization error: {}", e)))?;

        socket.send_to(&data, "255.255.255.255:8080")
            .await
            .map_err(|e| VgaError::ResourceLimit(format!("Broadcast error: {}", e)))?;

        tokio::time::sleep(Duration::from_secs(2)).await;

        let nodes = self.discovered_nodes.read().await;
        Ok(nodes.values().cloned().collect())
    }

    pub async fn list_discovered_nodes(&self) -> Vec<NodeInfo> {
        self.discovered_nodes.read().await.values().cloned().collect()
    }

    pub async fn request_resources(&self, requirements: ResourceRequirements, task_type: String, priority: Priority) -> Result<ResourceAllocation, VgaError> {
        let request_id = Uuid::new_v4().to_string();
        let node_info = self.node_info.read().await;

        let request = ResourceRequest {
            request_id: request_id.clone(),
            requester_id: node_info.id.clone(),
            required_resources: requirements.clone(),
            task_type,
            priority,
            timeout_secs: 60,
        };

        let selected_node = self.select_node_for_allocation(&request).await?;

        let allocation_id = Uuid::new_v4().to_string();
        let allocation = ResourceAllocation {
            allocation_id: allocation_id.clone(),
            node_id: selected_node.id.clone(),
            request_id: request_id.clone(),
            allocated_resources: self.allocate_resources_on_node(&selected_node, &requirements).await?,
            status: AllocationStatus::Active,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        let mut allocations = self.allocations.write().await;
        allocations.insert(allocation_id.clone(), allocation.clone());

        Ok(allocation)
    }

    async fn select_node_for_allocation(&self, request: &ResourceRequest) -> Result<NodeInfo, VgaError> {
        let strategy = self.balancing_strategy.read().await;
        let nodes = self.discovered_nodes.read().await;

        let available_nodes: Vec<NodeInfo> = nodes.values()
            .filter(|node| node.allow_remote_access && node.status == NodeStatus::Online)
            .cloned()
            .collect();

        if available_nodes.is_empty() {
            return Err(VgaError::ResourceLimit("No available nodes".into()));
        }

        match strategy.strategy_type {
            BalancingStrategy::LeastLoaded => {
                Ok(available_nodes.iter()
                    .min_by(|a, b| a.resources.current_load.partial_cmp(&b.resources.current_load).unwrap())
                    .cloned()
                    .unwrap())
            }
            BalancingStrategy::RoundRobin => {
                let index = (request.request_id.as_bytes()[0] as usize) % available_nodes.len();
                Ok(available_nodes[index].clone())
            }
            _ => Ok(available_nodes[0].clone()),
        }
    }

    async fn allocate_resources_on_node(&self, node: &NodeInfo, requirements: &ResourceRequirements) -> Result<AllocatedResources, VgaError> {
        let cpu_cores = requirements.cpu_cores.unwrap_or(1);
        let memory_mb = requirements.memory_mb.unwrap_or(1024);

        if cpu_cores > node.resources.cpu_cores {
            return Err(VgaError::ResourceLimit(format!(
                "Insufficient CPU: requested {}, available {}", 
                cpu_cores, node.resources.cpu_cores
            )));
        }

        if memory_mb > node.resources.available_memory_mb {
            return Err(VgaError::ResourceLimit(format!(
                "Insufficient memory: requested {} MB, available {} MB", 
                memory_mb, node.resources.available_memory_mb
            )));
        }

        let gpu = if requirements.gpu_required {
            if let Some(gpu) = node.resources.gpus.first() {
                let gpu_memory = requirements.gpu_memory_mb.unwrap_or(gpu.available_memory_mb);
                if gpu_memory > gpu.available_memory_mb {
                    return Err(VgaError::ResourceLimit(format!(
                        "Insufficient GPU memory: requested {} MB, available {} MB", 
                        gpu_memory, gpu.available_memory_mb
                    )));
                }
                Some(AllocatedGpu {
                    gpu_id: gpu.id.clone(),
                    memory_mb: gpu_memory,
                })
            } else {
                return Err(VgaError::ResourceLimit("GPU required but not available".into()));
            }
        } else {
            None
        };

        Ok(AllocatedResources {
            cpu_cores,
            memory_mb,
            gpu,
        })
    }

    pub async fn release_allocation(&self, allocation_id: String) -> Result<(), VgaError> {
        let mut allocations = self.allocations.write().await;
        if let Some(allocation) = allocations.get_mut(&allocation_id) {
            allocation.status = AllocationStatus::Completed;
        }
        Ok(())
    }

    pub async fn dispatch_distributed_task(&self, task_spec: TaskSpec, requirements: ResourceRequirements) -> Result<TaskId, VgaError> {
        let task_id = TaskId::new_v4();
        let allocation = self.request_resources(requirements, "distributed".to_string(), Priority::Medium).await?;

        let task = DistributedTask {
            task_id,
            spec: task_spec,
            assigned_node: Some(allocation.node_id.clone()),
            status: DistributedTaskStatus::Dispatched,
            result: None,
            created_at: Utc::now(),
            started_at: Some(Utc::now()),
            completed_at: None,
        };

        let mut tasks = self.distributed_tasks.write().await;
        tasks.insert(task_id, task);

        Ok(task_id)
    }

    pub async fn get_distributed_task(&self, task_id: TaskId) -> Option<DistributedTask> {
        self.distributed_tasks.read().await.get(&task_id).cloned()
    }

    pub async fn perform_health_check(&self, node_id: String) -> Result<HealthCheck, VgaError> {
        let start = std::time::Instant::now();
        
        let nodes = self.discovered_nodes.read().await;
        let node = nodes.get(&node_id)
            .ok_or_else(|| VgaError::ResourceLimit("Node not found".into()))?;

        let allocations = self.allocations.read().await;
        let active_count = allocations.values()
            .filter(|a| a.node_id == node_id && matches!(a.status, AllocationStatus::Active))
            .count();

        Ok(HealthCheck {
            node_id: node_id.clone(),
            timestamp: Utc::now(),
            status: node.status.clone(),
            resources: node.resources.clone(),
            active_allocations: active_count,
            response_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    pub async fn update_node_resources(&self, resources: NodeResources) {
        let mut node_info = self.node_info.write().await;
        node_info.resources = resources;
        node_info.last_seen = Utc::now();
    }

    pub async fn set_balancing_strategy(&self, strategy: BalancingStrategy) {
        let mut balancing = self.balancing_strategy.write().await;
        balancing.strategy_type = strategy;
    }

    pub async fn get_balancing_strategy(&self) -> BalancingStrategy {
        self.balancing_strategy.read().await.strategy_type.clone()
    }

    pub async fn create_resource_pool(&self, name: String, node_ids: Vec<String>) -> Result<String, VgaError> {
        let pool_id = Uuid::new_v4().to_string();
        let nodes = self.discovered_nodes.read().await;

        let mut total_resources = NodeResources {
            cpu_cores: 0,
            total_memory_mb: 0,
            available_memory_mb: 0,
            gpus: vec![],
            supported_models: vec![],
            current_load: 0.0,
        };

        for node_id in &node_ids {
            if let Some(node) = nodes.get(node_id) {
                total_resources.cpu_cores += node.resources.cpu_cores;
                total_resources.total_memory_mb += node.resources.total_memory_mb;
                total_resources.available_memory_mb += node.resources.available_memory_mb;
                total_resources.gpus.extend(node.resources.gpus.clone());
            }
        }

        let pool = ResourcePool {
            pool_id: pool_id.clone(),
            name,
            nodes: node_ids,
            total_resources: total_resources.clone(),
            available_resources: total_resources,
        };

        let mut pools = self.resource_pools.write().await;
        pools.insert(pool_id.clone(), pool);

        Ok(pool_id)
    }

    pub async fn list_resource_pools(&self) -> Vec<ResourcePool> {
        self.resource_pools.read().await.values().cloned().collect()
    }

    pub async fn prime_demo_usage(&self) {
        let _ = self.swarm_groups.read().await.len();
        let _ = self.resource_pools.read().await.len();
        let _ = self.allocations.read().await.len();
        let _ = self.distributed_tasks.read().await.len();
        let _ = self.balancing_strategy.read().await.strategy_type.clone();

        let group_id = self.create_swarm_group("demo".to_string(), 1).await.unwrap_or_default();
        let _ = self.join_swarm_group(group_id.clone()).await;
        let _ = self.get_group_members(group_id.clone()).await;
        let _ = self.leave_swarm_group(group_id.clone()).await;
        let _ = self.list_swarm_groups().await;

        self.set_remote_access(true).await;
        let _ = self.get_remote_access_status().await;

        let _ = self.discover_nodes().await;
        let _ = self.list_discovered_nodes().await;

        let requirements = ResourceRequirements {
            cpu_cores: Some(1),
            memory_mb: Some(512),
            gpu_required: false,
            gpu_memory_mb: None,
            preferred_models: Vec::new(),
        };

        let _ = self.request_resources(requirements.clone(), "demo".to_string(), Priority::Low).await;
        let _ = self.release_allocation("demo".to_string()).await;

        let _ = self.dispatch_distributed_task(
            TaskSpec {
                language: "rust".to_string(),
                target: "code".to_string(),
                context_range: "demo".to_string(),
            },
            requirements.clone(),
        ).await;
        let _ = self.get_distributed_task(TaskId::new_v4()).await;
        let _ = self.perform_health_check("unknown".to_string()).await;

        self.update_node_resources(NodeResources {
            cpu_cores: 1,
            total_memory_mb: 1024,
            available_memory_mb: 512,
            gpus: Vec::new(),
            supported_models: Vec::new(),
            current_load: 0.0,
        }).await;

        self.set_balancing_strategy(BalancingStrategy::LeastLoaded).await;
        let _ = self.get_balancing_strategy().await;

        let _ = self.create_resource_pool("demo".to_string(), Vec::new()).await;
        let _ = self.list_resource_pools().await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryMessage {
    message_type: DiscoveryMessageType,
    node_info: NodeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum DiscoveryMessageType {
    Announce,
    Query,
}