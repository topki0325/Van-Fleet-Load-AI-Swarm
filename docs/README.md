# Vangriten AI Swarm - 蜂群框架文档库

欢迎阅读 Vangriten AI Swarm 框架的详细文档。本文档库旨在提供对系统架构、实现细节、最佳实践以及使用指南的全面了解。

## 文档目录

- [核心架构](architecture.md)：概述系统的核心理念、组件和三层方法。
- [代理角色与能力](agents.md)：详细介绍不同类型的 AI 代理及其通信协议。
- [技术实现](technical-implementation.md)：包括高并发设计、模块化编译调度、网络发现和 API 密钥安全的实现细节。
- [项目结构与模块设计](structure/file-tree.md)：详细的项目目录布局与模块化设计理念。
  - [前端 API 接口](structure/frontend-api.md)：Tauri 模式下的 UI 逻辑与后端指令集。
  - [后端核心服务](structure/backend-services.md)：加特林调度、环境管理与分布式算力节点。
  - [共享数据模型](structure/data-models.md)：统一的项目、代理与任务实体规范。
- [开始使用](getting-started.md)：环境设置、安装步骤、配置指南和运行说明。
- [组件规则](component-rules.md)：GUI 子功能组组件化规范（便于拆分编译、降低重编译时间）。
- [最佳实践](best-practices.md)：代理专业化、并发管理、安全和性能优化的建议。
- [未来增强](future.md)：关于多模态、联合学习以及 CI/CD 集成的规划。
- [项目进度](progress.md)：当前开发进度、可验证功能与下一步计划。

## 功能模块文档

- [API 密钥管理](api-key-management.md)：管理多个 AI 服务提供商的 API 密钥，支持加密存储和使用统计。
  - [English Version](api-key-management-en.md)
- [资源管理代理](resource-manager.md)：局域网内 AI 模型资源发现、分布式 GPU 资源分配、网络负载均衡等功能。
  - [English Version](resource-manager-en.md)
- [C 语言编译环境管理](c-compiler.md)：GCC 实例自动发现、轮流编译、并行编译等功能。
  - [English Version](c-compiler-en.md)
- [Ollama 集成](ollama.md)：在本地运行多种开源 AI 模型，支持聊天、文本生成和向量嵌入。
  - [English Version](ollama-en.md)

## 其他重要文件

- [README](../README.md)：项目概况和快速入门。
- [CONTRIBUTING](../CONTRIBUTING.md)：如何参与项目开发。
- [CHANGELOG](../CHANGELOG.md)：版本更新记录。
- [LICENSE](../LICENSE)：MIT 许可证说明。
- [CODE_OF_CONDUCT](../CODE_OF_CONDUCT.md)：社区行为准则。

## Skills（给 AI/Agents 使用的技能库）

- [skills/README.md](../skills/README.md)：技能库说明与约定
