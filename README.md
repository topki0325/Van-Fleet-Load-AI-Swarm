# Vangriten AI Swarm

[![CI](https://github.com/topki0325/Vangriten-AI-swarm/workflows/CI/badge.svg)](https://github.com/topki0325/Vangriten-AI-swarm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Vangriten AI Swarm 是一个高并发 AI 蜂群框架，基于 Rust 构建，旨在协调多个 AI 代理进行协作开发。

## ✨ 特性

- 🚀 **高并发执行**：加特林式调度系统，支持数千个 AI 代理同时工作
- 🔒 **企业级安全**：AES-256 加密 API 密钥，完整的资源消耗统计
- 🌐 **分布式架构**：局域网自动发现，支持调用远程 AI 和 GPU 资源
- 🛠️ **多语言支持**：GCC、Conda、Rust 等完整编译环境管理
- 🤖 **本地 AI 模型**：完整集成 Ollama，支持在本地运行多种开源 AI 模型
- 📊 **实时监控**：可视化界面展示蜂群活动和代理状态
- 🔧 **模块化设计**：可扩展的代理系统，支持自定义角色

## 🏗️ 架构

```text
VGA = Vangriten Gatling AI
├── V = Vangriten (自主 AI 编排)
├── G = Gatling (高并发旋转调度)
└── A = Architecture / AI / Autonomous (三层架构)
```

## 🚀 快速开始

### 系统要求

- Rust 1.70+
- Node.js 18+
- 支持的操作系统：Windows, macOS, Linux

### 安装

```bash
git clone https://github.com/topki0325/vga-swarm.git
cd vga-swarm
cargo build --release
```

### 运行

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

## 📖 文档

详细文档请查看 [docs/README.md](./docs/README.md)（中文）或 [docs/README-en.md](./docs/README-en.md)（英文）。

## 🤝 贡献

我们欢迎各种形式的贡献！请查看 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解详情。

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](./LICENSE) 文件了解详情。

## 🙏 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- 所有贡献者

## 📞 联系

- 项目主页: [https://github.com/topki0325/Vangriten-AI-swarm](https://github.com/topki0325/Vangriten-AI-swarm)
- Issues: [https://github.com/topki0325/Vangriten-AI-swarm/issues](https://github.com/topki0325/Vangriten-AI-swarm/issues)
