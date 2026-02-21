# 前端组件与 API 接口 (Frontend & UI API)

Vangriten AI Swarm 前端通过 Tauri 提供的强类型跨语言接口实现。所有 UI 组件均通过响应式框架构建，并对接 Rust 后端的指令流。

## 🖥️ 核心前端组件说明 (Frontend Components)

### 1. `App` (应用全局管理器)

- **`pub async fn initialize() -> AppContext`**: 应用初始化，完成日志等级设定与后端握手。
- **`pub fn on_route_change(new_route: RoutePath)`**: 处理单页应用的路由切换逻辑（如项目管理、资源看板等）。
- **`pub fn toggle_mode(m: ClientMode)`**: 响应用户切换 Master/Slave 模式，并触发界面的重渲染。

### 2. `ProjectView` (项目可视化器)

- **`pub fn render_workflow_tree(p: &Project)`**: 渲染多代理协作的 DAG 树状工作流界面。
- **`pub async fn sync_agent_output(t: TaskId) -> OutputEntry`**: 订阅后端推送的代理任务输出文本流。
- **`pub fn handle_manual_intervention(conflict: ConflictInfo)`**: 在多代理合并代码产生冲突时弹窗提示并捕获人工决策结果。

### 3. `AgentMonitor` (实时看板)

- **`pub fn update_swarm_pulse(p: PulseStatus)`**: 持续消耗后端推送的蜂群存活、负载指标数据。
- **`pub fn render_provider_metrics(p: Provider)`**: 显示特定 AI 提供商（如 OpenAI, Anthropic）的实时 QPS、Token 消耗率曲线图。
- **`pub fn show_gpu_utilization(node_id: &str)`**: 可视化特定节点的 GPU 热力度。

---

## 📡 后端指令集接口 (Unified Tauri Commands)

这些指令在前端通过 `invoke("command_id", { args })` 进行异步调用。前端 TypeScript 侧需通过 `ts-rs` 或同等工具生成的类型与后端对齐。

| 指令 ID (Command) | 输入参数 (Arguments) | 返回类型 (Return Type) | 核心功能 |
| :--- | :--- | :--- | :--- |
| `cmd_get_billing` | `provider: string` | `BillingReport` | 基于 `ApiKeyManager::update_usage_stats` 获取财务数据。 |
| `cmd_vault_op` | `op: VaultOp` | `VaultResult` | 通用的密钥金库管理，前端映射为配置页面的 CRUD 操作。 |
| `cmd_deploy_project` | `config: ProjectConfig` | `ProjectResult` | 调用 `AgentScheduler::dispatch_task` 初始化整蜂群任务流。 |
| `cmd_node_discovery` | 无 | `Vec<PeerStatus>` | 触发 `NetworkDiscovery::broadcast_presence` 并返回扫描列表。 |
| `cmd_get_all_agents` | 无 | `Vec<Agent>` | 获取集群内所有 `Agent` 实体的生存指标与当前任务快照。 |
| `cmd_request_compute` | `req: ComputeReq` | `ResourceLease` | 调用 `ResourceManager::acquire_cluster_gpu` 预订算力资源。 |
| `cmd_force_terminate` | `task_id: string` | `Result<bool, VgaError>` | 强制中断特定的 `Task` 及其关联的子任务链。 |

---

## 📅 实时消息协议 (Backend Events Push)

后端通过 `emit` 命令向 UI 推送强类型事件，前端使用统一的 `listen` 总线侦听：

- **`EVT_AGENT_LOG`**: `payload: { agent_id: string, log_line: string, level: LogLevel }` - 实时日志流。
- **`EVT_BUILD_UPDATE`**: `payload: BuildUpdate` - 映射 `CompilationScheduler::dispatch_build_segments` 的进度反馈。
- **`EVT_SWARM_PULSE`**: `payload: SwarmPulse` - 每秒推送一次的集群热度与负载地图。
- **`EVT_ERROR_CRITICAL`**: `payload: VgaError` - 当发生不可恢复错误（如 `AuthVaultError`）时触发全局 UI 中断。
- **`EVT_TASK_EVENT`**: `payload: { task_id: string, status: TaskStatus }` - 驱动项目看板 DAG 节点状态实时变色。
