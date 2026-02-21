# 项目文件架构 (System File Architecture)

VGA (Vangriten Gatling AI)swarm 遵循高内聚、低耦合的模块化设计。以下是系统的核心源码文件布局及其承担的特定函数职责。

## 📁 源码目录详细解析

### 1. `src/main.rs` (引导程序)
- **`fn main()`**: 应用程序入口，初始化并发运行时（Tokio Runtime）及系统托盘。
- **`async fn setup_backend_services()`**: 串行初始化 `ApiKeyManager`、`NetworkDiscovery` 等单例服务。

### 2. `src/frontend/` (指令层)
- **`mod.rs`**: 暴露命令路由，包含 `#[tauri::command]` 宏包装的所有对外接口。
- **`app.rs`**: **`fn handle_global_state()`** - 管理前端 Redux-like 的状态同步逻辑。
- **`client_gui.rs`**: **`fn switch_layout(mode: ClientMode)`** - 根据 Master/Slave 角色动态注入不同的 UI 组件树。

### 3. `src/backend/` (逻辑层核心)
- **`api_manager.rs`**: **`fn vault_operation()`** - 密钥金库的原子级加解密与 CRUD 操作。
- **`agent_scheduler.rs`**: **`fn gatling_rotate_next()`** - 集群负载均衡调度算法的核心实现。
- **`compilation_scheduler.rs`**: **`async fn dispatch_build_segments()`** - 编译任务的分片化分发与进度流监控。
- **`network_discovery.rs`**: **`fn broadcast_presence()`** - 周期性探测局域网并同步 Master/Slave 角色标识。

### 4. `src/backend/agents/` (蜂群代理具体实现)
所有代理类均通过实现统一的 **`AgentTrait`** 进行解耦：
- **`ArchitectAgent`**: **`async fn execute_instruction()`** - 接收指令并产出系统蓝图或任务图谱。
- **`ProgrammerAgent`**: **`async fn execute_block()`** - 执行具体的代码块生成、修改与重构任务。
- **`EnvironmentAgent`**: **`async fn setup_sandboxed_environment()`** - 环境准备与自动化部署脚本生成。

### 5. `src/shared/` (合约层)
- **`models.rs`**: 定义全系列跨进程传输的 `struct` 与 `enum`。
- **`utils.rs`**: **`fn compute_hash(data: &[u8]) -> String`** 等通用工具函数。
