# 共享数据模型 (Shared Entities & Unified Interfaces)

VGA (Vangriten Gatling AI)swarm 建立了一套严谨、跨语言的对象模型（Entities），确保 Master 节点与 Slave 节点间在分布式推理、任务交换与工件聚合时的一致性。

## 📦 核心业务实体模型 (Core Models)

### 1. `Project` (核心项目实体)

定义了项目的生命周期、配置以及蜂群的协作图谱。

- **数据结构**：

```rust
struct Project {
    id: ProjectId,             // UUID
    name: String,              // “VGA (Vangriten Gatling AI)swarm”
    config: ProjectConfig,     // 技术栈、默认提供商、并发策略
    agents: Vec<AgentId>,      // 参与本项目的代理个体列表
    workflow: WorkflowGraph,   // DAG 描述的任务序列与依赖关系
    state: ProjectStatus,      // 初始化、运行、挂起、成功、失败
    stats: ExecutionStats,     // 统计信息：总 Token 数、总时长、总费用
    last_updated: DateTime,    // 最后一次状态同步时间
}
```

- **关键接口**：
  - **`pub fn initialize_workflow(&mut self)`**: 根据项目配置生成初始的任务依赖图。
  - **`pub fn validate_and_snapshot(&self) -> Result<Snapshot, Error>`**: 获取当前项目状态的快照，用于持久化恢复。

### 2. `Agent` (蜂群代理个体)

定义了蜂群中的工作节点，每个代理具备特定的角色属性和技能向量。

- **数据结构**：

```rust
struct Agent {
    id: AgentId,               // 分布式唯一标识符
    role: AgentType,           // 枚举：Architect, Programmer, etc.
    status: AgentStatus,       // 存活状态：Idle, Busy, Offline, Error
    skills: SkillVector,       // 技能特征（如 Python 开发、Rust 性能调优）
    current_task: Option<TaskId>, // 当前执行中的任务锁定
    performance: PerfMetrics,  // CPU/MEM 负载及其推理平均时延
    heartbeat: Instant,        // 代理主机的最后活跃时间
}
```

- **关键接口**：
  - **`pub async fn execute_block(&self, code_spec: CodeSpec) -> TaskOutput`**: 代理执行一个具体的代码生成/重构原子块。
  - **`pub fn is_overloaded(&self) -> bool`**: 基于自身的容量与当前负载判断是否接收新任务。

### 3. `Task` (任务原子单位)

描述了蜂群中的最小执行单元，支持嵌套的任务决策树逻辑。

- **数据结构**：

```rust
struct Task {
    id: TaskId,                // 任务 ID
    parent_id: Option<TaskId>, // 指向父级任务（任务树）
    spec: TaskSpec,            // 任务元数据（语言、目标、上下文范围）
    priority: Priority,        // 紧急程度枚举
    assigned_to: Option<AgentId>, // 锁定的代理 ID
    input_snapshot: PathBuf,   // 关联的源码/配置快照路径
    output: TaskResult,        // 代理解析后的结果（Success/Fail/Conflict）
    retry_count: u32,          // 失败尝试重试次数
}
```

- **关键接口**：
  - **`pub fn finalize_with_result(&mut self, res: TaskResult)`**: 标记任务完成并更新产物。
  - **`pub fn check_dependencies(&self, context: &WorkflowGraph) -> bool`**: 检查其依赖的任务是否全部就绪（Ready）。

---

## 🛠️ 调度与错误模型 (Orchestration & Error Handling)

### `GatlingState` (加特林负载均衡器状态)

用于高并发环境下的代理快速分配与防死锁。

```rust
struct GatlingState {
    available_pool: Arc<RwLock<Vec<AgentId>>>,
    rotation_index: AtomicUsize, // 关键的 Gatling 轮转计数器
    max_concurrency: usize,
    waiting_queue: MpscQueue<TaskId>,
}
```

### `VgaError` (统一错误模型)

确保跨模块、跨前后端展示的一致性错误提示。

```rust
enum VgaError {
    AuthVaultError(String),    // 获取/解密 API 密钥失败
    AgentTimeout(AgentId),     // 代理响应超时
    EnvironmentLockError,      // 编译环境被其他任务锁定
    NetworkSplit,              // 分布式集群网络隔离
    CompileFailure(String),    // 自动化编译流程报错
    ResourceLimit(String),     // 磁盘或显存配额不足
}
```

---

## 🏛️ 行为合约 (Behavioral Contracts)

### `AgentTrait` (代理核心能力定义)

所有蜂群角色必须实现的 Rust Trait。

```rust
#[async_trait]
trait AgentTrait {
    /// 指令执行的主入口
    async fn execute_instruction(&self, instr: String) -> Result<TaskOutput, VgaError>;
    
    /// 执行具体的代码生成/重构原子块 (主要由 ProgrammerAgent 实现)
    async fn execute_block(&self, code_spec: CodeSpec) -> Result<TaskOutput, VgaError> {
        Err(VgaError::CompileFailure("Not implemented for this agent type".into()))
    }

    /// 更新代理的局部记忆上下文
    fn update_context(&mut self, context: &ContextManager);

    /// 获取代理当下的运行负载与性能指标
    fn get_metrics(&self) -> PerfMetrics;
}
```

---

## 📅 核心枚举与统一常量 (Common Constants)

### `AgentType` (精细角色定义)

```rust
enum AgentType {
    ArchitectNode,   // 方案专家
    ProgrammerNode,  // 实现专家
    SecurityNode,    // 审计专家
    DocManager,      // 技术作家
    EnvManagerNode,  // 自动化基建专家
    ClusterResourceManager // 网络资源调度中心
}
```

### `LanguagePlatform` (多语言生态栈支持)

驱动 `EnvironmentManager` 的底层工具链搜索：

- `RustStack`: Cargo, Clippy, Rustfmt.
- `PythonStack`: Conda, Pip, PyEnv.
- `CBasedStack`: GCC, Makefile, CMake.
- `TauriStack`: Node.js, PNPM, Rust.

---

*注意：所有模型均实现 `Serialize` 与 `Deserialize` 特性，保证跨进程传输（RPC / Websocket / JSON over mDNS）的语义完整性。*
