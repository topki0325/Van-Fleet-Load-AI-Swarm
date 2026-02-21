# 资源管理代理 (Resource Management Agent)

## 概述

资源管理代理是 Vangriten AI Swarm 的核心组件，负责发现、管理和分配局域网内的 AI 模型资源和 GPU 计算资源。它实现了分布式计算资源的自动发现、负载均衡和跨节点任务调度。

## 核心功能

### 1. 局域网资源发现

- **自动发现**：通过 UDP 广播自动发现局域网内的其他节点
- **节点信息**：收集每个节点的资源信息（CPU、内存、GPU、支持的模型）
- **实时更新**：定期广播和接收节点状态更新
- **智能过滤**：只显示允许远程访问的节点

### 2. 群组管理 (Swarm Groups)

- **创建群组**：创建一个 AI 蜂群群组，邀请其他节点加入
- **加入群组**：加入现有的群组，共享资源
- **离开群组**：退出群组，释放资源
- **群组领导**：自动选举群组领导者，协调资源分配

### 3. 资源分配

- **智能调度**：根据负载均衡策略选择最佳节点
- **资源预留**：为任务预留 CPU、内存和 GPU 资源
- **优先级处理**：支持任务优先级，高优先级任务优先分配
- **超时管理**：自动处理超时的资源请求

### 4. 负载均衡策略

- **轮询 (Round Robin)**：依次分配给每个节点
- **最少负载 (Least Loaded)**：选择当前负载最低的节点
- **加权 (Weighted)**：根据节点性能加权分配
- **地理位置 (Geographic)**：根据地理位置选择最近的节点
- **随机 (Random)**：随机选择可用节点

### 5. 健康检查

- **节点监控**：定期检查节点的健康状态
- **资源监控**：实时监控节点的资源使用情况
- **响应时间**：测量节点的响应时间
- **自动恢复**：自动检测和恢复离线节点

### 6. 权限控制

- **远程访问开关**：控制是否允许其他节点调用本机的 AI 资源
- **节点过滤**：只与信任的节点交互
- **访问控制**：支持白名单和黑名单机制

## 数据模型

### NodeInfo

```rust
pub struct NodeInfo {
    pub id: String,                    // 节点唯一标识
    pub address: String,                // IP 地址
    pub port: u16,                     // 端口号
    pub mode: ClientMode,              // 模式（Master/Slave）
    pub resources: NodeResources,       // 资源信息
    pub allow_remote_access: bool,      // 是否允许远程访问
    pub last_seen: DateTime<Utc>,      // 最后活跃时间
    pub status: NodeStatus,            // 节点状态
}
```

### NodeResources

```rust
pub struct NodeResources {
    pub cpu_cores: u32,               // CPU 核心数
    pub total_memory_mb: u64,          // 总内存 (MB)
    pub available_memory_mb: u64,       // 可用内存 (MB)
    pub gpus: Vec<GpuInfo>,           // GPU 列表
    pub supported_models: Vec<String>,  // 支持的 AI 模型
    pub current_load: f64,             // 当前负载 (0.0 - 1.0)
}
```

### ResourceRequest

```rust
pub struct ResourceRequest {
    pub request_id: String,            // 请求 ID
    pub requester_id: String,           // 请求者 ID
    pub required_resources: ResourceRequirements, // 资源需求
    pub task_type: String,             // 任务类型
    pub priority: Priority,            // 优先级
    pub timeout_secs: u64,             // 超时时间（秒）
}
```

### ResourceAllocation

```rust
pub struct ResourceAllocation {
    pub allocation_id: String,          // 分配 ID
    pub node_id: String,               // 分配的节点 ID
    pub request_id: String,            // 关联的请求 ID
    pub allocated_resources: AllocatedResources, // 分配的资源
    pub status: AllocationStatus,       // 分配状态
    pub created_at: DateTime<Utc>,     // 创建时间
    pub expires_at: DateTime<Utc>,     // 过期时间
}
```

## 使用示例

### 初始化资源管理器

```rust
use crate::backend::resource_manager::ResourceManager;

// 创建资源管理器，允许远程访问
let resource_manager = ResourceManager::new(true).await.unwrap();

// 启动网络发现
resource_manager.start_discovery().await.unwrap();
```

### 发现节点

```rust
// 主动发现局域网内的节点
let discovered_nodes = resource_manager.discover_nodes().await.unwrap();

for node in discovered_nodes {
    println!("Found node: {} at {}", node.id, node.address);
    println!("  CPU cores: {}", node.resources.cpu_cores);
    println!("  Memory: {} MB", node.resources.available_memory_mb);
    println!("  GPUs: {}", node.resources.gpus.len());
}
```

### 创建和加入群组

```rust
// 创建新的群组
let group_id = resource_manager.create_swarm_group(
    "AI Research Group".to_string(),
    10, // 最大成员数
).await.unwrap();

// 加入现有群组
resource_manager.join_swarm_group(group_id.clone()).await.unwrap();

// 查看群组成员
let members = resource_manager.get_group_members(group_id).await.unwrap();
println!("Group members: {}", members.len());
```

### 请求资源

```rust
use crate::shared::models::{ResourceRequirements, Priority};

// 定义资源需求
let requirements = ResourceRequirements {
    cpu_cores: Some(4),
    memory_mb: Some(8192),
    gpu_required: true,
    gpu_memory_mb: Some(8192),
    preferred_models: vec!["gpt-4".to_string(), "claude-3".to_string()],
};

// 请求资源分配
let allocation = resource_manager.request_resources(
    requirements,
    "inference".to_string(),
    Priority::High,
).await.unwrap();

println!("Allocated on node: {}", allocation.node_id);
println!("CPU cores: {}", allocation.allocated_resources.cpu_cores);
println!("Memory: {} MB", allocation.allocated_resources.memory_mb);
```

### 分布式任务调度

```rust
use crate::shared::models::TaskSpec;

// 创建任务规范
let task_spec = TaskSpec {
    language: "python".to_string(),
    target: "inference".to_string(),
    context_range: "full".to_string(),
};

// 定义资源需求
let requirements = ResourceRequirements {
    cpu_cores: Some(2),
    memory_mb: Some(4096),
    gpu_required: false,
    gpu_memory_mb: None,
    preferred_models: vec![],
};

// 分发任务到最佳节点
let task_id = resource_manager.dispatch_distributed_task(
    task_spec,
    requirements,
).await.unwrap();

// 查询任务状态
if let Some(task) = resource_manager.get_distributed_task(task_id).await {
    println!("Task status: {:?}", task.status);
    println!("Assigned to: {:?}", task.assigned_node);
}
```

### 健康检查

```rust
// 执行健康检查
let health = resource_manager.perform_health_check(node_id).await.unwrap();

println!("Node status: {:?}", health.status);
println!("Response time: {} ms", health.response_time_ms);
println!("Active allocations: {}", health.active_allocations);
println!("CPU load: {:.2}%", health.resources.current_load * 100.0);
```

### 负载均衡策略

```rust
use crate::shared::models::BalancingStrategy;

// 设置负载均衡策略
resource_manager.set_balancing_strategy(
    BalancingStrategy::LeastLoaded
).await;

// 查看当前策略
let strategy = resource_manager.get_balancing_strategy().await;
println!("Current strategy: {:?}", strategy);
```

### 权限控制

```rust
// 允许远程访问
resource_manager.set_remote_access(true).await;

// 禁用远程访问
resource_manager.set_remote_access(false).await;

// 查看当前状态
let allow = resource_manager.get_remote_access_status().await;
println!("Remote access: {}", allow);
```

### 创建资源池

```rust
// 创建资源池，将多个节点的资源聚合
let pool_id = resource_manager.create_resource_pool(
    "High Performance Pool".to_string(),
    vec![
        "node-1-id".to_string(),
        "node-2-id".to_string(),
        "node-3-id".to_string(),
    ],
).await.unwrap();

// 查看所有资源池
let pools = resource_manager.list_resource_pools().await;
for pool in pools {
    println!("Pool: {}", pool.name);
    println!("  Total CPU: {} cores", pool.total_resources.cpu_cores);
    println!("  Total Memory: {} MB", pool.total_resources.total_memory_mb);
    println!("  GPUs: {}", pool.total_resources.gpus.len());
}
```

## 网络发现协议

资源管理器使用 UDP 广播进行节点发现：

1. **查询消息 (Query)**：节点广播查询消息，请求其他节点响应
2. **公告消息 (Announce)**：节点定期广播自己的信息
3. **响应消息**：节点收到查询后，回复自己的信息

消息格式（JSON）：

```json
{
  "message_type": "Announce",
  "node_info": {
    "id": "uuid",
    "address": "192.168.1.100",
    "port": 8080,
    "mode": "Master",
    "resources": { ... },
    "allow_remote_access": true,
    "last_seen": "2024-01-01T00:00:00Z",
    "status": "Online"
  }
}
```

## 最佳实践

1. **合理设置超时**：根据任务类型设置合适的超时时间
2. **监控资源使用**：定期检查节点的资源使用情况
3. **选择合适的负载均衡策略**：根据应用场景选择最佳策略
4. **管理群组大小**：控制群组成员数量，避免过度集中
5. **定期健康检查**：定期执行健康检查，及时发现和解决问题
6. **释放资源**：任务完成后及时释放资源，避免资源浪费

## 故障处理

资源管理器会自动处理以下故障情况：

- **节点离线**：自动从可用节点列表中移除
- **资源不足**：返回错误，建议等待或降低需求
- **网络分区**：保持本地状态，网络恢复后自动同步
- **任务超时**：自动标记任务为超时状态
- **分配失败**：尝试重新分配到其他节点

## 性能优化

- **批量操作**：尽量批量请求资源，减少网络开销
- **缓存节点信息**：缓存节点信息，减少重复查询
- **异步处理**：所有操作都是异步的，不会阻塞主线程
- **连接池**：使用连接池管理网络连接
- **压缩传输**：对大数据包进行压缩传输

## 安全考虑

- **访问控制**：只允许信任的节点访问资源
- **加密传输**：敏感数据使用加密传输
- **身份验证**：节点之间进行身份验证
- **审计日志**：记录所有资源分配和释放操作
- **权限隔离**：不同任务之间资源隔离

## 未来扩展

- 支持 GPU 集群调度
- 支持跨地域资源调度
- 支持动态资源扩缩容
- 支持资源竞价和拍卖
- 支持多租户资源隔离
- 支持资源使用计费