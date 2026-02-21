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
- [最佳实践](best-practices.md)：代理专业化、并发管理、安全和性能优化的建议。
- [未来增强](future.md)：关于多模态、联合学习以及 CI/CD 集成的规划。
- [项目进度](progress.md)：当前开发进度、可验证功能与下一步计划。

## 其他重要文件

- [README](../README.md)：项目概况和快速入门。
- [CONTRIBUTING](../CONTRIBUTING.md)：如何参与项目开发。
- [CHANGELOG](../CHANGELOG.md)：版本更新记录。
- [LICENSE](../LICENSE)：MIT 许可证说明。
- [CODE_OF_CONDUCT](../CODE_OF_CONDUCT.md)：社区行为准则。
