# 项目进度（截至 2026-02-21）

本文件记录当前开发阶段的"已完成/可验证/待办"，便于协作与继续推进。

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
- ✅ Ollama 完整集成（本地 AI 模型支持）
- ✅ C 编译器环境管理（GCC 实例发现与调度）
- ✅ 双语文档支持（中文 + 英语）

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

### 7) Ollama 集成（本地 AI 模型支持）

- ✅ 完整的 Ollama 客户端实现
  - 连接管理（默认 `http://localhost:11434`）
  - 模型管理（列出、拉取、删除、查看信息）
  - 聊天功能（简单聊天和高级聊天）
  - 文本生成
  - 向量嵌入
  - 使用统计跟踪
- ✅ Ollama 提供商配置
  - 支持本地 Ollama 作为 AI 提供商
  - 模型配置和参数设置
- ✅ Tauri 命令集成（12 个命令）
  - 模型管理：`ollama_list_models`, `ollama_pull_model`, `ollama_delete_model`, `ollama_show_model_info`
  - 聊天功能：`ollama_chat_simple`, `ollama_chat_advanced`
  - 文本生成：`ollama_generate`
  - 向量嵌入：`ollama_embeddings`
  - 统计信息：`ollama_get_stats`, `ollama_reset_stats`
  - 连接测试：`ollama_test_connection`
- ✅ Web 界面集成
  - Ollama 管理面板
  - 模型列表和操作
  - 聊天界面
  - 统计信息展示

### 8) C 编译器环境管理

- ✅ GCC 实例自动发现
  - 系统范围内搜索 GCC 实例
  - 版本检测和路径识别
  - 可用性状态跟踪
- ✅ 轮流编译策略
  - 负载均衡分配编译任务
  - 公平调度机制
  - 任务状态跟踪
- ✅ 并行编译支持
  - 多文件同时编译
  - 最大并发数控制
  - 结果聚合和错误处理
- ✅ 编译器状态监控
  - 实时状态查看
  - 活动和已完成任务跟踪
  - 性能指标统计
- ✅ Tauri 命令集成（4 个命令）
  - GCC 管理：`gcc_list_instances`, `gcc_get_status`
  - 编译执行：`gcc_compile_round_robin`, `gcc_compile_parallel`
- ✅ Web 界面集成
  - GCC 实例列表
  - 轮流编译界面
  - 并行编译界面
  - 编译结果展示

### 9) 双语文档支持

- ✅ 完整的英语文档
  - 项目主文档（README-en.md）
  - 文档索引（docs/README-en.md）
  - 功能模块文档（4 个）
- ✅ 文档命名规范
  - 中文文档：无后缀（如 `ollama.md`）
  - 英语文档：`-en.md` 后缀（如 `ollama-en.md`）
- ✅ 文档内容对应
  - 功能概述
  - 安装指南
  - 使用说明
  - 代码示例
  - API 参考
  - 故障排除
- ✅ 更新中文文档
  - 添加英语文档链接
  - 同步新功能内容

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
4. Ollama 功能验证（需先安装 Ollama）：
  - 确保 Ollama 服务运行：`ollama serve`
  - 在 GUI 中切换到 Ollama 管理面板
  - 点击 `List Models` 查看已安装模型
  - 点击 `Pull Model` 拉取新模型（如 `llama3`）
  - 使用聊天界面与模型交互
  - 查看统计信息
5. C 编译器功能验证：
  - 确保 GCC 已安装：`gcc --version`
  - 在 GUI 中切换到 C 编译器管理面板
  - 点击 `List GCC Instances` 查看发现的 GCC 实例
  - 点击 `Get Status` 查看编译器状态
  - 使用轮流编译或并行编译功能
  - 查看编译结果和输出

（可选）如果你需要验证 Tauri 路径：`cargo tauri dev`

## 已知限制/技术债

- 部分模块仍是骨架实现：网络发现、编译调度、真实 token/cost 统计等部分未完成
- agents 的"智能逻辑"是占位实现（主要用于验证调用链路），未接入真实模型/工具
- 仍存在部分 `dead_code` 类 warning（结构体字段暂未读写），不影响运行
- 任务调度与队列是内存实现，尚无持久化与重启恢复
- 构建会提示 `net2 v0.2.39` future-incompat warning（当前不影响编译/运行）
- Ollama 功能需要用户手动安装和配置 Ollama 服务
- C 编译器功能需要系统安装 GCC

## 大文件检查（拆分/清理建议）

本仓库按"排除 target/.git/dist/icons"扫描后，较大的文件主要集中在：

- `src/bin/vga_gui.rs`（约 44KB / 1000+ 行）：建议继续按组件拆分（下一批优先：Providers 视图、API 管理弹窗、右侧 Info 面板）
- `src/backend/resource_manager.rs`（约 23KB）：资源管理代理实现，包含节点发现、资源分配、负载均衡等功能
- `src/backend/ollama_client.rs`（约 18KB）：Ollama 客户端实现，包含模型管理、聊天、生成等功能
- `src/backend/provider_config.rs`（约 13KB）：AI 提供商配置，包含多个提供商的配置管理
- `src/backend/c_compiler.rs`（约 13KB）：C 编译器调度器，包含 GCC 发现和编译调度
- `src/backend/api_manager.rs`（约 12KB）：API 密钥管理器，包含加密存储和密钥管理
- `src/backend/agent_scheduler.rs`（约 10KB）：Agent 调度器，包含任务队列和状态管理
- `src/backend/network_discovery.rs`（约 10KB）：网络发现模块，包含 mDNS 和 UDP 发现
- `rustup-init.exe`（约 10MB）：属于安装器二进制，通常不建议纳入源码仓库；如只是本地开发便利，建议移出仓库或加入忽略规则

## 下一步（推荐）

### 短期目标
- 任务输出与日志：支持流式输出、失败原因与重试策略
- 将 projects/leases 从内存存储升级为可持久化（本地文件/SQLite）
- 完善 Ollama 集成：添加更多模型支持、流式响应、自定义参数
- 完善 C 编译器管理：添加更多编译器支持（Clang、MSVC）、编译缓存、增量编译
- 逐步收敛 warnings，并为关键模块补单元测试/集成测试

### 中期目标
- 完善编译调度：根据 `BuildPlan`/`EnvSpec` 实际创建环境与构建产物
- 网络发现与远程节点：mDNS 发现 + 远程执行协议/鉴权
- 资源管理代理：完善分布式资源调度、负载均衡、健康检查
- 多语言编译环境：支持 Python、JavaScript、Rust 等多种语言的编译和执行

### 长期目标
- 完整的 AI Agent 智能逻辑：接入真实模型和工具
- 分布式任务执行：跨节点任务调度和结果聚合
- 完整的监控和日志系统：实时监控、日志聚合、性能分析
- 插件系统：支持自定义插件和扩展
