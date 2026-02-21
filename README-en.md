# Vangriten AI Swarm

[![CI](https://github.com/topki0325/Vangriten-AI-swarm/workflows/CI/badge.svg)](https://github.com/topki0325/Vangriten-AI-swarm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Vangriten AI Swarm is a high-concurrency AI swarm framework built with Rust, designed to coordinate multiple AI agents for collaborative development.

## ✨ Features

- 🚀 **High-Concurrency Execution**: Gatling-style scheduling system supporting thousands of AI agents working simultaneously
- 🔒 **Enterprise-Grade Security**: AES-256 encrypted API key storage with comprehensive resource consumption tracking
- 🌐 **Distributed Architecture**: Automatic LAN discovery with support for calling remote AI and GPU resources
- 🛠️ **Multi-Language Support**: Complete compilation environment management for GCC, Conda, Rust, and more
- 🤖 **Local AI Models**: Full integration with Ollama, supporting various open-source AI models running locally
- 📊 **Real-Time Monitoring**: Visual interface displaying swarm activity and agent status
- 🔧 **Modular Design**: Extensible agent system supporting custom roles

## 🏗️ Architecture

```text
VGA = Vangriten Gatling AI
├── V = Vangriten (Autonomous AI Orchestration)
├── G = Gatling (High-Concurrency Rotational Scheduling)
└── A = Architecture / AI / Autonomous (Three-Layer Architecture)
```

## 🚀 Quick Start

### System Requirements

- Rust 1.70+
- Node.js 18+
- Supported OS: Windows, macOS, Linux

### Installation

```bash
git clone https://github.com/topki0325/Vangriten-AI-swarm.git
cd vga-swarm
cargo build --release
```

### Running

```bash
cargo run
```

If you want to use the **Rust native GUI (without WebView)** (recommended):

```bash
cargo run -p vgs
```

On Windows, the corresponding executable files are:

- `target/debug/vgs.exe` (dev build)
- `target/release/vgs.exe` (release build)

The GUI window title is: `vas`.

If you prefer using the Tauri CLI (optional):

```bash
cargo install tauri-cli
cargo tauri dev
```

## 📖 Documentation

For detailed documentation, please see [docs/README-en.md](./docs/README-en.md) (English) or [docs/README.md](./docs/README.md) (Chinese).

## 🤝 Contributing

We welcome contributions of all kinds! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - Desktop application framework
- [Rust](https://www.rust-lang.org/) - Systems programming language
- All contributors

## 📞 Contact

- Project Homepage: [https://github.com/topki0325/Vangriten-AI-swarm](https://github.com/topki0325/Vangriten-AI-swarm)
- Issues: [https://github.com/topki0325/Vangriten-AI-swarm/issues](https://github.com/topki0325/Vangriten-AI-swarm/issues)
