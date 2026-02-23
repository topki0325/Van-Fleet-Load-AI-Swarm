# Van Fleet Load AI Swarm

[![CI](https://github.com/topki0325/Van-Fleet-Load-AI-Swarm/workflows/CI/badge.svg)](https://github.com/topki0325/Van-Fleet-Load-AI-Swarm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[English](#english) | [中文](#中文)

---

## English

Van Fleet Load AI Swarm is a high-concurrency AI swarm framework built with Rust, designed to coordinate multiple AI agents for collaborative development.

### ✨ Features

- 🚀 **High-Concurrency Execution**: Gatling-style scheduling system supporting thousands of AI agents working simultaneously
- 🔒 **Enterprise-Grade Security**: AES-256 encrypted API keys with complete resource consumption statistics
- 🌐 **Distributed Architecture**: LAN auto-discovery, supporting remote AI and GPU resource calls
- 🛠️ **Multi-Language Support**: Complete compilation environment management for GCC, Conda, Rust, etc.
- 🤖 **Local AI Models**: Fully integrated Ollama, supporting running various open-source AI models locally
- 📊 **Real-time Monitoring**: Visual interface displaying swarm activity and agent status
- 🌟 **Project Wizard**: Automatic workflow for "Article Quick-Write" with directory picker and task orchestration
- 💥 **Burst Mode**: One-click creation of 1-10 AI entity clones for maximum API concurrency
- 🔗 **Custom Relay Support**: Support for custom API providers with flexible URL and header settings
- 🔧 **Modular Design**: Extensible agent system supporting custom roles

### 🏗️ Architecture

Van Fleet Load AI Swarm draws inspiration from the "Van Fleet load" - the legendary artillery barrage tactic from the Korean War, reimagined as a coordinated AI agent framework:

```
VFLAS = Van Fleet Load AI Swarm
├── VFL = Van Fleet Load (Van Fleet Load)
│   └── Inspired by the overwhelming artillery barrage tactic
│       that consumed massive ammunition in the Korean War
├── A = AI (Artificial Intelligence Agents)
│   └── Coordinated autonomous AI systems
└── S = Swarm (Collaborative AI Agent Collective)
    ├── Swarm: Collective intelligence of AI agents working together
    ├── Collaborative: Multi-agent teamwork and coordination
    └── Collective: Unified AI agent ecosystem for complex tasks
```

**Van Fleet Concept**: Named after General James Van Fleet's legendary artillery tactic during the Korean War, where unprecedented ammunition consumption (36,000 artillery shells in 9 days for a single hill) demonstrated overwhelming firepower saturation. This framework applies similar saturation principles to AI agent coordination and resource orchestration.

### 🚀 Quick Start

#### System Requirements

- Rust 1.70+
- Node.js 18+
- Supported OS: Windows, macOS, Linux

#### Installation

```bash
git clone https://github.com/topki0325/vga-swarm.git
cd vga-swarm
cargo build --release
```

#### Running

```bash
cargo run
```

For **Rust Native GUI (No WebView)** (Recommended):

```bash
cargo run -p vgs
```

Windows executables:
- `target/debug/vgs.exe` (dev build)
- `target/release/vgs.exe` (release build)

GUI window title: `vas`

If you prefer Tauri CLI (Optional):

```bash
cargo install tauri-cli
cargo tauri dev
```

### 📖 Documentation

Detailed documentation: [docs/README-en.md](./docs/README-en.md) (English) or [docs/README.md](./docs/README.md) (Chinese).

### 🤝 Contributing

We welcome contributions of all kinds! See [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

### 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

### 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Desktop application framework
- [Rust](https://www.rust-lang.org/) - Systems programming language
- All contributors

### 📞 Contact

- Email: 259901434@qq.com
- Project Home: [https://github.com/topki0325/Van-Fleet-Load-AI-Swarm](https://github.com/topki0325/Van-Fleet-Load-AI-Swarm)
- Issues: [https://github.com/topki0325/Van-Fleet-Load-AI-Swarm/issues](https://github.com/topki0325/Van-Fleet-Load-AI-Swarm/issues)

### 🔌 API 调用接口

#### Ollama LAN 共享 API

当启用 Ollama 共享时，应用会在本地启动一个代理服务器 (默认端口 11435)，提供安全的 Ollama API 访问。

##### 基本用法

1. 在 GUI 中启用共享，选择要共享的模型，并设置密码（可选）。
2. 其他 LAN 设备可以通过发现协议找到你的共享实例。
3. 调用 API 时使用以下格式：

```bash
curl -X POST http://<host>:11435/api/chat \
  -H "Content-Type: application/json" \
  -H "x-vas-key: <password>" \
  -d '{
    "model": "llama2",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": false
  }'
```

##### 参数说明

- `host`: 共享主机的 IP 地址
- `password`: 如果设置了共享密码，则必须在 `x-vas-key` 头中提供
- `model`: 必须是共享主机允许的模型之一
- 其他参数与标准 Ollama API 相同

##### MCP (Model Context Protocol) 集成

项目支持作为 MCP 服务器运行，提供以下工具：

- **mcp_pylance_mcp_s_pylanceDocString**: 获取 Python 符号的文档字符串
- **mcp_pylance_mcp_s_pylanceDocuments**: 搜索 Pylance 文档
- **mcp_pylance_mcp_s_pylanceFileSyntaxErrors**: 检查 Python 文件语法错误
- **mcp_pylance_mcp_s_pylanceImports**: 分析工作区导入
- **mcp_pylance_mcp_s_pylanceInstalledTopLevelModules**: 获取已安装的顶级模块
- **mcp_pylance_mcp_s_pylanceInvokeRefactoring**: 应用代码重构
- **mcp_pylance_mcp_s_pylancePythonEnvironments**: 获取 Python 环境信息
- **mcp_pylance_mcp_s_pylanceRunCodeSnippet**: 执行 Python 代码片段
- **mcp_pylance_mcp_s_pylanceSettings**: 获取 Pylance 设置
- **mcp_pylance_mcp_s_pylanceSyntaxErrors**: 检查代码片段语法
- **mcp_pylance_mcp_s_pylanceUpdatePythonEnvironment**: 切换 Python 环境
- **mcp_pylance_mcp_s_pylanceWorkspaceRoots**: 获取工作区根目录
- **mcp_pylance_mcp_s_pylanceWorkspaceUserFiles**: 获取用户 Python 文件

要启动 MCP 服务器：

```bash
cargo run --bin mcp-server
```

然后在 MCP 客户端中配置连接到该服务器。

---

## 中文

Van Fleet Load AI Swarm 是一个高并发 AI 蜂群架构，基于 Rust 构建，旨在协调多个 AI 代理来进行协作开发。

### ✨ 特性

- 🚀 **高并发执行**：加特林式调度系统，支持数千个 AI 代理同时工作
- 🔒 **企业级安全**：AES-256 加密 API 密钥，完整的资源消耗统计
- 🌐 **分布式架构**：局域网自动发现，支持调用远程 AI 和 GPU 资源
- 🛠️ **多语言支持**：GCC、Conda、Rust 等完整编译环境管理
- 🤖 **本地 AI 模型**：完整集成 Ollama，支持在本地运行多种开源 AI 模型
- 📊 **实时监控**：可视化界面展示蜂群活动和代理状态
- 🌟 **项目向导**：“文章快速写”自动化工作流，集成文件夹选择与任务编排
- 💥 **裂变模式**：一键生成 1-10 个 AI 实体副本，支持同一 API 商的高并发满载
- 🔗 **自定义提供商**：支持手动添加 Relay 中转站，灵活配置 URL 与 Header
- 🔧 **模块化设计**：可扩展的代理系统，支持自定义角色

### 🏗️ 架构

Van Fleet Load AI Swarm 的灵感来源于朝鲜战争中的"Van Fleet load" - 传奇的火力压制战术，将其重新想象为协调的 AI 代理框架：

```
VFLAS = Van Fleet Load AI Swarm
├── VFL = Van Fleet Load (Van Fleet Load)
│   └── 灵感来源于朝鲜战争中的压倒性火炮轰击战术
│       9天内消耗3.6万发炮弹夺取一座小山的传奇战例
├── A = AI (人工智能代理)
│   └── 协调的自主 AI 系统
└── S = Swarm (协作式AI代理集群)
    ├── Swarm: AI代理群体智能协同工作
    ├── Collaborative: 多代理团队合作与协调
    └── Collective: 统一AI代理生态系统处理复杂任务
```

**Van Fleet 概念**：以朝鲜战争中范弗里特将军的传奇火炮战术命名，当时前所未有的弹药消耗量（9天内3.6万发炮弹夺取一座小山）展示了压倒性的火力饱和能力。该框架将类似的饱和原理应用于 AI 代理协调和资源编排。

### 🚀 快速开始

#### 系统要求

- Rust 1.70+
- Node.js 18+
- 支持的操作系统：Windows, macOS, Linux

#### 安装

```bash
git clone https://github.com/topki0325/vga-swarm.git
cd vga-swarm
cargo build --release
```

#### 运行

```bash
cargo run
```

如果你想使用 **Rust 原生 GUI（无 WebView）**（推荐）：

```bash
cargo run -p vgs
```

Windows 下对应可执行文件为：

- `target/debug/vgs.exe`（dev 构建）
- `target/release/vgs.exe`（release 构建）

GUI 窗口标题为：`vas`。

如果你偏好使用 Tauri CLI（可选）：

```bash
cargo install tauri-cli
cargo tauri dev
```

### 📖 文档

详细文档请查看 [docs/README.md](./docs/README.md)（中文）或 [docs/README-en.md](./docs/README-en.md)（英文）。

### 🤝 贡献

我们欢迎各种形式的贡献！请查看 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解详情。

### 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](./LICENSE) 文件了解详情。

### 🙏 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- 所有贡献者

### 📞 联系

- 邮箱: 259901434@qq.com
- 项目主页: [https://github.com/topki0325/Van-Fleet-Load-AI-Swarm](https://github.com/topki0325/Van-Fleet-Load-AI-Swarm)
- Issues: [https://github.com/topki0325/Van-Fleet-Load-AI-Swarm/issues](https://github.com/topki0325/Van-Fleet-Load-AI-Swarm/issues)

### 🔌 API 调用接口

#### Ollama LAN 共享 API

启用 Ollama 共享后，应用会在本地启动代理服务器 (默认端口 11435)，提供安全的 Ollama API 访问。

##### 基本用法

1. 在 GUI 中启用共享，选择要共享的模型，并设置密码（可选）。
2. 其他 LAN 设备可通过发现协议找到你的共享实例。
3. 调用 API 时使用以下格式：

```bash
curl -X POST http://<host>:11435/api/chat \
  -H "Content-Type: application/json" \
  -H "x-vas-key: <password>" \
  -d '{
    "model": "llama2",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": false
  }'
```

##### 参数说明

- `host`: 共享主机的 IP 地址
- `password`: 如果设置了共享密码，则必须在 `x-vas-key` 头中提供
- `model`: 必须是共享主机允许的模型之一
- 其他参数与标准 Ollama API 相同

##### MCP (Model Context Protocol) 集成

项目支持作为 MCP 服务器运行，提供以下工具：

- **mcp_pylance_mcp_s_pylanceDocString**: 获取 Python 符号的文档字符串
- **mcp_pylance_mcp_s_pylanceDocuments**: 搜索 Pylance 文档
- **mcp_pylance_mcp_s_pylanceFileSyntaxErrors**: 检查 Python 文件语法错误
- **mcp_pylance_mcp_s_pylanceImports**: 分析工作区导入
- **mcp_pylance_mcp_s_pylanceInstalledTopLevelModules**: 获取已安装的顶级模块
- **mcp_pylance_mcp_s_pylanceInvokeRefactoring**: 应用代码重构
- **mcp_pylance_mcp_s_pylancePythonEnvironments**: 获取 Python 环境信息
- **mcp_pylance_mcp_s_pylanceRunCodeSnippet**: 执行 Python 代码片段
- **mcp_pylance_mcp_s_pylanceSettings**: 获取 Pylance 设置
- **mcp_pylance_mcp_s_pylanceSyntaxErrors**: 检查代码片段语法
- **mcp_pylance_mcp_s_pylanceUpdatePythonEnvironment**: 切换 Python 环境
- **mcp_pylance_mcp_s_pylanceWorkspaceRoots**: 获取工作区根目录
- **mcp_pylance_mcp_s_pylanceWorkspaceUserFiles**: 获取用户 Python 文件

要启动 MCP 服务器：

```bash
cargo run --bin mcp-server
```

然后在 MCP 客户端中配置连接到该服务器。
