# 项目进度（截至 2026-02-21）

本文件记录当前开发阶段的“已完成/可验证/待办”，便于协作与继续推进。

## 当前状态

- ✅ `cargo build` / `cargo check` 可通过（Windows 环境已验证）
- ✅ Rust 原生 GUI（`egui/eframe`）已可用：`cargo run -p vgs`
  - Windows 产物：`target/debug/vgs.exe`（dev）/ `target/release/vgs.exe`（release）
  - GUI 窗口标题：`vas`
  - 中文字体：已接入 `egui-chinese-font`，中文可正确显示
  - 界面布局（VS Code 风格）：顶部菜单 + 左侧导航 + 中间主视图 + 右侧两列信息流
  - 资源管理/Providers 等长操作行已支持自动换行（避免遮挡右侧信息流面板）
  - 已为多个 `ScrollArea` 显式设置唯一 `id_source`，避免 ID 冲突
- ✅ 工程已拆分为 Cargo workspace：`crates/vas-core`（后端/模型）+ `crates/vgs`（原生 GUI）
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

- ✅ Vault 功能已合并进 GUI 的“API管理”弹窗：
  - 本地加密存储 APIKey
  - 查看/保存前必须输入密码解锁（避免明文展示/误操作）
  - 加密方案：密码派生密钥（Argon2）+ AES-GCM（随机 nonce，存盘 `nonce || ciphertext`）

### 4) 前端占位 UI（可直接验证）

- 列表展示：Swarm / Agents / Projects / Leases
- 操作入口：
  - Deploy Sample Project
  - Request Sample Compute
  - Execute Task（提交 `TaskSpec` 并显示 `TaskOutput`）
  - API管理（弹窗）：初始化/解锁/锁定/列表/保存/删除/查看（查看需解锁）

### 5) GUI 组件化（第一阶段）

- ✅ Task / Network / Resources 已抽为独立组件文件（位于 `crates/vgs/src/components/*`）
- ✅ 中间主视图通过 `ActiveView` 加载对应组件，便于后续继续把 Providers/API 管理等拆分出去

### 6) 协作与开发体验

- 添加 VS Code tasks（check/build/run）与推荐扩展
- `.gitignore` 调整：
  - 提交 `Cargo.lock`
  - 忽略 `vault/` 机密目录、忽略安装器文件

## 如何验证（建议顺序）

1. 构建：`cargo build`
2. 运行（原生 GUI 推荐）：`cargo run -p vgs`
3. 在 GUI 窗口里：
  - 左侧导航切换：Task / API Keys / Network / Providers / Resources
  - 点 `刷新(Refresh)` 看到 agents/swarm
  - 点 `部署示例项目(Deploy Sample Project)` 后 `Projects` 增加
  - 点 `申请示例算力(Request Sample Compute)` 后 `Leases` 增加
  - 在 `任务(Task)` 区填写内容后点击 `执行(Execute)`，看到 `TaskOutput`
  - 点击 `API管理(API Manager)` 打开弹窗：设置密码初始化后解锁，保存/查看 APIKey

（可选）如果你需要验证 Tauri 路径：`cargo tauri dev`

## 已知限制/技术债

- 目前大量模块仍是骨架实现：网络发现、编译调度、真实 token/cost 统计等未完成
- agents 的“智能逻辑”是占位实现（主要用于验证调用链路），未接入真实模型/工具
- 仍存在部分 `dead_code` 类 warning（结构体字段暂未读写），不影响运行
- 任务调度与队列是内存实现，尚无持久化与重启恢复
- 构建会提示 `net2 v0.2.39` future-incompat warning（当前不影响编译/运行）

## 大文件检查（拆分/清理建议）

本仓库按“排除 target/.git/dist/icons”扫描后，较大的文件主要集中在：

- `src/bin/vga_gui.rs`（约 44KB / 1000+ 行）：建议继续按组件拆分（下一批优先：Providers 视图、API 管理弹窗、右侧 Info 面板）。
- `src/backend/resource_manager.rs` / `src/backend/ollama_client.rs` 等（约 10~25KB）：目前仍可读，但后续如果继续加功能，建议按子模块拆分（requests/models/ui glue 等）。
- `rustup-init.exe`（约 10MB）：属于安装器二进制，通常不建议纳入源码仓库；如只是本地开发便利，建议移出仓库或加入忽略规则。

## 下一步（推荐）

- 任务输出与日志：支持流式输出、失败原因与重试策略
- 将 projects/leases 从内存存储升级为可持久化（本地文件/SQLite）
- 完善编译调度：根据 `BuildPlan`/`EnvSpec` 实际创建环境与构建产物
- 网络发现与远程节点：mDNS 发现 + 远程执行协议/鉴权
- 逐步收敛 warnings，并为关键模块补单元测试/集成测试
