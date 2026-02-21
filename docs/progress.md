# 项目进度（截至 2026-02-21）

本文件记录当前开发阶段的“已完成/可验证/待办”，便于协作与继续推进。

## 当前状态

- ✅ `cargo build` / `cargo check` 可通过（Windows 环境已验证）
- ✅ Rust 原生 GUI（`egui/eframe`）已可用：`cargo run --features native-gui --bin vgs`
  - Windows 产物：`target/debug/vgs.exe`（dev）/ `target/release/vgs.exe`（release）
  - GUI 窗口标题：`vas`
  - 中文字体：已接入 `egui-chinese-font`，中文可正确显示
  - 左侧操作区已按子功能折叠（下拉/折叠菜单），避免按钮堆叠
  - 已为多个 `ScrollArea` 显式设置唯一 `id_source`，避免 ID 冲突
- ✅ Tauri 后端命令仍可用（可选路径），前端占位页可通过 `invoke` 调用并展示数据
- ✅ 基础 CI 已添加（GitHub Actions：Windows 上 `cargo check` + `cargo build`）
- ✅ 任务生命周期命令已接入（提交/查询/列表/取消）并可追踪队列与运行中数量

## 已完成（关键里程碑）

### 1) 工程可编译、可运行

- 增加 Tauri 必需的构建脚本 build.rs（`tauri-build`）
- 修复多处 Rust 编译错误（模型类型、导入、trait 签名、Tauri 宏配置等）
- 增加最小前端资源 `dist/index.html`，让 app 能跑起来并显示数据

### 2) 后端服务骨架（可被前端调用）

- **AgentScheduler**
  - 默认注册 3 个 agent（Architect / Programmer / Environment）用于演示
  - 支持列出 agents、查询 swarm 状态
  - 增加 `execute_task_spec(TaskSpec)`：按 `language/target` 路由到对应 agent 执行并返回 `TaskOutput`
  - 任务队列与状态流转（Pending/Running/Completed/Failed/Cancelled），支持提交/查询/取消

- **BackendServices 内存状态**
  - 项目列表 `projects`、算力租约 `leases` 以进程内 `RwLock<Vec<_>>` 形式持久化

### 3) Vault（API Key 管理）

- ✅ 修复 AES-GCM 固定 nonce 的安全隐患：
  - 每次加密随机生成 nonce
  - 存盘格式为 `nonce || ciphertext`
- 提供更易用的 Tauri commands：
  - `cmd_vault_store` / `cmd_vault_retrieve` / `cmd_vault_list` / `cmd_vault_delete`
  - `cmd_vault_usage`：返回每个 provider 的请求次数/最后使用时间

### 4) 前端占位 UI（可直接验证）

- 列表展示：Swarm / Agents / Projects / Leases
- 操作按钮：
  - Deploy Sample Project
  - Request Sample Compute
  - Execute Task（提交 `TaskSpec` 并显示 `TaskOutput`）
  - Vault：Store / Retrieve / List / Delete / Usage

### 5) 协作与开发体验

- 添加 VS Code tasks（check/build/run）与推荐扩展
- `.gitignore` 调整：
  - 提交 `Cargo.lock`
  - 忽略 `vault/` 机密目录、忽略安装器文件

## 如何验证（建议顺序）

1. 构建：`cargo build`
2. 运行（原生 GUI 推荐）：`cargo run --features native-gui --bin vgs`
3. 在 GUI 窗口里：
  - 左侧折叠菜单可展开/收起：Vault / Network / Providers / Resources
  - 点 `刷新(Refresh)` 看到 agents/swarm
  - 点 `部署示例项目(Deploy Sample Project)` 后 `Projects` 增加
  - 点 `申请示例算力(Request Sample Compute)` 后 `Leases` 增加
  - 在 `任务(Task)` 区填写内容后点击 `执行(Execute)`，看到 `TaskOutput`
  - Vault 区可 Store/Retrieve/List/Delete/Usage

（可选）如果你需要验证 Tauri 路径：`cargo tauri dev`

## 已知限制/技术债

- 目前大量模块仍是骨架实现：网络发现、编译调度、真实 token/cost 统计等未完成
- agents 的“智能逻辑”是占位实现（主要用于验证调用链路），未接入真实模型/工具
- 仍存在部分 `dead_code` 类 warning（结构体字段暂未读写），不影响运行
- 任务调度与队列是内存实现，尚无持久化与重启恢复
- 构建会提示 `net2 v0.2.39` future-incompat warning（当前不影响编译/运行）

## 下一步（推荐）

- 任务输出与日志：支持流式输出、失败原因与重试策略
- 将 projects/leases 从内存存储升级为可持久化（本地文件/SQLite）
- 完善编译调度：根据 `BuildPlan`/`EnvSpec` 实际创建环境与构建产物
- 网络发现与远程节点：mDNS 发现 + 远程执行协议/鉴权
- 逐步收敛 warnings，并为关键模块补单元测试/集成测试
