# Resource Manager Agent

## Overview

The Resource Manager Agent is a distributed resource management system that enables discovery and utilization of AI model resources within a local area network (LAN). It provides comprehensive capabilities for resource allocation, load balancing, health monitoring, and task scheduling across multiple nodes.

## Features

### 1. LAN Resource Discovery
- **Automatic Discovery**: Automatically discover AI model resources in the LAN using mDNS and UDP broadcast
- **Node Registration**: Register and manage discovered nodes
- **Node Information**: Track node capabilities including CPU, memory, GPU, and supported models

### 2. Distributed GPU Resource Allocation
- **GPU Pooling**: Aggregate GPU resources from multiple nodes
- **Smart Allocation**: Allocate GPU resources based on task requirements
- **Resource Tracking**: Monitor GPU memory usage and availability

### 3. Network Resource Load Balancing
- **Multiple Strategies**: Support for Round Robin, Least Loaded, Weighted, Geographic, and Random
- **Dynamic Adjustment**: Automatically adjust load based on node performance
- **Custom Weights**: Configure custom weights for weighted balancing

### 4. Remote Resource Monitoring
- **Health Checks**: Periodic health checks for all nodes
- **Performance Metrics**: Track response time, resource usage, and active allocations
- **Status Updates**: Real-time status updates for all nodes

### 5. Cross-Node Task Scheduling
- **Task Distribution**: Distribute tasks across multiple nodes
- **Result Aggregation**: Collect and aggregate results from distributed tasks
- **Priority Queue**: Support for task prioritization

### 6. Permission Control
- **Remote Access Control**: Allow or deny remote access to local resources
- **Group Management**: Create and manage swarm groups
- **Access Policies**: Configure access policies for different groups

## Architecture

### Components

#### Node Discovery
- Uses mDNS for service discovery
- UDP broadcast for node announcements
- Automatic node registration and cleanup

#### Resource Manager
- Centralized resource tracking
- GPU memory management
- CPU and memory allocation

#### Load Balancer
- Multiple balancing strategies
- Real-time load monitoring
- Adaptive routing

#### Health Monitor
- Periodic health checks
- Performance metrics collection
- Failure detection and recovery

#### Task Scheduler
- Task queue management
- Priority-based scheduling
- Distributed task execution

## Web Interface Usage

### Node Discovery
1. Click "Discover Nodes" to start discovery
2. Click "List Nodes" to view discovered nodes
3. Click "Refresh" to update node list

### Remote Access Control
1. Toggle "Allow Remote Access" checkbox
2. Click "Get Status" to check current status

### Swarm Groups
1. Enter group name and max members
2. Click "Create Group" to create a new group
3. Enter group ID and click "Join" to join a group
4. Click "Leave" to leave a group
5. Click "Get Members" to view group members

### Resource Allocation
1. Enter required resources (CPU, memory, GPU)
2. Select task type and priority
3. Click "Request Resources" to allocate resources
4. View allocation results

### Health Monitoring
1. Click "Perform Health Check" to check all nodes
2. View health status and performance metrics

### Load Balancing
1. Select balancing strategy
2. Click "Set Strategy" to apply
3. Click "Get Strategy" to view current strategy

### Resource Pools
1. Enter pool name and select nodes
2. Click "Create Pool" to create a resource pool
3. Click "List Pools" to view all pools

## Code Usage Examples

### Initialize Resource Manager
```rust
use vangriten_ai_swarm::backend::ResourceManager;

// Create resource manager with local node
let resource_manager = ResourceManager::new(true).await?;

// Start discovery
resource_manager.start_discovery().await?;
```

### Discover Nodes
```rust
let nodes = resource_manager.discover_nodes().await?;
for node in nodes {
    println!("Node: {} ({})", node.id, node.address);
}
```

### Request Resources
```rust
use vangriten_ai_swarm::shared::models::{ResourceRequest, ResourceRequirements, Priority};

let request = ResourceRequest {
    request_id: uuid::Uuid::new_v4().to_string(),
    requester_id: "client-1".to_string(),
    required_resources: ResourceRequirements {
        cpu_cores: Some(4),
        memory_mb: Some(8192),
        gpu_required: true,
        gpu_memory_mb: Some(4096),
        preferred_models: vec!["llama3".to_string()],
    },
    task_type: "inference".to_string(),
    priority: Priority::High,
    timeout_secs: 300,
};

let allocation = resource_manager.request_resources(request).await?;
println!("Allocated to node: {}", allocation.node_id);
```

### Create Swarm Group
```rust
let group = resource_manager.create_swarm_group(
    "my-swarm".to_string(),
    10,
).await?;
println!("Group created: {}", group.group_id);
```

### Join Swarm Group
```rust
resource_manager.join_swarm_group("group-uuid".to_string()).await?;
```

### Perform Health Check
```rust
let health_checks = resource_manager.perform_health_checks().await?;
for check in health_checks {
    println!("Node {}: {:?}", check.node_id, check.status);
}
```

### Set Load Balancing Strategy
```rust
use vangriten_ai_swarm::shared::models::{LoadBalancingStrategy, BalancingStrategy};

let strategy = LoadBalancingStrategy {
    strategy_type: BalancingStrategy::LeastLoaded,
    weights: std::collections::HashMap::new(),
};

resource_manager.set_balancing_strategy(strategy).await?;
```

### Set Remote Access
```rust
resource_manager.set_remote_access(true).await?;
```

## Load Balancing Strategies

### Round Robin
- **Description**: Distributes requests evenly across all nodes
- **Use Case**: When all nodes have similar capabilities
- **Advantage**: Simple and predictable

### Least Loaded
- **Description**: Routes requests to the node with the lowest current load
- **Use Case**: When nodes have varying capacities
- **Advantage**: Optimizes resource utilization

### Weighted
- **Description**: Distributes requests based on configured weights
- **Use Case**: When some nodes should receive more traffic
- **Advantage**: Flexible traffic distribution

### Geographic
- **Description**: Routes requests to the geographically closest node
- **Use Case**: When nodes are distributed across locations
- **Advantage**: Reduces latency

### Random
- **Description**: Randomly selects a node for each request
- **Use Case**: When no specific strategy is needed
- **Advantage**: Simple and unpredictable

## Data Models

### NodeInfo
```rust
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
```

### ResourceRequest
```rust
pub struct ResourceRequest {
    pub request_id: String,
    pub requester_id: String,
    pub required_resources: ResourceRequirements,
    pub task_type: String,
    pub priority: Priority,
    pub timeout_secs: u64,
}
```

### ResourceAllocation
```rust
pub struct ResourceAllocation {
    pub allocation_id: String,
    pub node_id: String,
    pub request_id: String,
    pub allocated_resources: AllocatedResources,
    pub status: AllocationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
```

### HealthCheck
```rust
pub struct HealthCheck {
    pub node_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: NodeStatus,
    pub resources: NodeResources,
    pub active_allocations: usize,
    pub response_time_ms: u64,
}
```

## Best Practices

### 1. Resource Planning
- Assess your resource requirements before deployment
- Plan for peak loads
- Monitor resource usage regularly

### 2. Load Balancing
- Choose the right strategy for your use case
- Monitor load distribution
- Adjust weights as needed

### 3. Health Monitoring
- Set appropriate health check intervals
- Configure alert thresholds
- Monitor response times

### 4. Security
- Enable remote access only when needed
- Use secure communication channels
- Implement access control policies

### 5. Performance
- Optimize network configuration
- Use appropriate timeout values
- Implement retry logic for failed requests

## Troubleshooting

### Nodes Not Discovered
**Problem**: No nodes are discovered

**Solutions**:
1. Check if mDNS is enabled on your network
2. Verify firewall settings allow UDP traffic
3. Ensure nodes are on the same network
4. Check if Ollama services are running

### Resource Allocation Failed
**Problem**: Unable to allocate resources

**Solutions**:
1. Check if nodes have sufficient resources
2. Verify GPU memory availability
3. Review allocation timeout settings
4. Check node health status

### Health Check Timeout
**Problem**: Health checks timing out

**Solutions**:
1. Increase health check timeout
2. Check network connectivity
3. Verify node is running
4. Review firewall settings

### Load Balancing Not Working
**Problem**: Requests not distributed evenly

**Solutions**:
1. Verify load balancing strategy is set correctly
2. Check node status and availability
3. Review weight configurations
4. Monitor load distribution metrics

## API Reference

### ResourceManager
```rust
pub struct ResourceManager {
    // Internal implementation
}
```

#### Methods
- `new(create_local_node: bool) -> Result<Self, VgaError>` - Create resource manager
- `start_discovery(&self) -> Result<(), VgaError>` - Start node discovery
- `discover_nodes(&self) -> Result<Vec<NodeInfo>, VgaError>` - Discover nodes
- `list_discovered_nodes(&self) -> Vec<NodeInfo>` - List discovered nodes
- `request_resources(&self, request: ResourceRequest) -> Result<ResourceAllocation, VgaError>` - Request resources
- `release_allocation(&self, allocation_id: String) -> Result<(), VgaError>` - Release allocation
- `create_swarm_group(&self, name: String, max_members: usize) -> Result<SwarmGroup, VgaError>` - Create group
- `join_swarm_group(&self, group_id: String) -> Result<(), VgaError>` - Join group
- `leave_swarm_group(&self, group_id: String) -> Result<(), VgaError>` - Leave group
- `list_swarm_groups(&self) -> Vec<SwarmGroup>` - List groups
- `get_group_members(&self, group_id: String) -> Vec<String>` - Get group members
- `perform_health_checks(&self) -> Vec<HealthCheck>` - Perform health checks
- `set_remote_access(&self, allow: bool) -> Result<(), VgaError>` - Set remote access
- `get_remote_access_status(&self) -> bool` - Get remote access status
- `set_balancing_strategy(&self, strategy: LoadBalancingStrategy) -> Result<(), VgaError>` - Set strategy
- `get_balancing_strategy(&self) -> LoadBalancingStrategy` - Get strategy
- `create_resource_pool(&self, name: String, nodes: Vec<String>) -> Result<ResourcePool, VgaError>` - Create pool
- `list_resource_pools(&self) -> Vec<ResourcePool>` - List pools

## Related Links

- [Ollama Integration](ollama-en.md) - Local AI model support
- [API Key Management](api-key-management-en.md) - API key management
- [Network Discovery](../src/backend/network_discovery.rs) - Network discovery implementation

## License

Resource Manager features follow to Van Fleet Load AI Swarm license.
