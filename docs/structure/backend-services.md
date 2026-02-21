# 后端核心服务 (Backend Services)

Van Fleet Load AI Swarm 后端采用分布式异步架构，所有核心模块均提供标准化的 Rust 接口集。

## ⚙️ 核心基础服务 (Infrastructure Services)

### 1. `ApiKeyManager` (密钥金库与记账)

管理外部 LLM 提供商密钥及其全生命周期安全。核心接口需确保**线程安全**且**低延迟**。

- **`fn vault_operation(&self, op: VaultOp) -> Result<VaultResult, Error>`**: 密钥金库的原子级统一操作入口，支持 `Store`, `Retrieve`, `Delete`, `Rotate` 等子操作。
- **`fn get_decrypted_key(&self, provider: &str) -> Result<String, Error>`**: 实时解密且触发调用计数，仅在执行推理任务时短暂驻留内存。
- **`fn update_usage_stats(&self, provider: &str, tokens: TokenUsage)`**: 异步更新特定提供商的 Token 消耗、QPS 及其成本报表。
- **`fn check_quota_availability(&self, provider: &str) -> bool`**: 预检余额或速率限制（Rate Limit）是否支持下一次并发请求。

### 2. `AgentScheduler` (加特林旋转调度器)

作为系统高并发中枢，负责原子级代理分配与实时心跳维持。

- **`fn gatling_rotate_next(&self) -> Result<Agent, Error>`**: Gatling 调度算法的核心实现。利用 `AtomicUsize` 在可用代理池中执行负载均衡的快速轮转。
- **`async fn dispatch_task(&self, task: Task) -> Result<TaskHandle, Error>`**: 实现任务与代理的绑定，并启动后台监控协程。
- **`fn get_swarm_status(&self) -> SwarmPulse`**: 获取集群存活节点数、异常节点列表和队列堆积深度。
- **`fn handle_agent_heartbeat(&mut self, agent_id: AgentId)`**: 响应分布式节点的定期心跳，更新 `Agent` 实体的 `last_active` 时间戳。

### 3. `EnvironmentManager` & `CompilationScheduler`

负责本地/分布式异构编译环境的建立与工件聚合。

- **`async fn setup_sandboxed_environment(&self, env_spec: EnvSpec) -> Result<EnvPath, Error>`**: 在独立容器或隔离目录中初始化 GCC/Conda/Rustc 环境。
- **`async fn dispatch_build_segments(&self, plan: BuildPlan) -> Stream<BuildUpdate>`**: 将编译任务分片化并下发至分布式节点，返回进度流。
- **`fn aggregate_artifacts(&self, results: Vec<BuildOutput>) -> TargetBinary`**: 处理对象文件链接、静态检查报错合并，产出最终二进制。

### 4. `NetworkDiscovery` & `ResourceManager`

主从架构下的节点同步与硬件资源监控。

- **`broadcast_presence(m: ClientMode)`**: 启动 mDNS 广播当前节点（Master/Slave）标识。
- **`acquire_cluster_gpu(r: GPUReq) -> ResourceLock`**: 锁定全局资源池的可分配显存及计算单元（CUDA）。
- **`sync_distributed_state(&mut self)`**: 基于 CRDT 保持蜂群节点间配置与任务状态的一致性。

---

## 🤖 代理行为规范 (Common Agent Interface)

所有具体代理角色（Architect, Programmer, etc.）均实现以下核心方法：

- **`pub async fn execute_instruction(&self, i: String) -> ResponseResult`**: 接收自然语言封装指令并产生相应动作产物。
- **`pub fn update_context(&mut self, c: ContextManager)`**: 更新代理对应的记忆槽和关联文档知识库。
- **`pub fn query_performance(&self) -> PerformanceStats`**: 获取该特定代理个体的 CPU 指标、响应用时及推理负载。
