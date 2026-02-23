# Ollama LAN Share

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Ollama LAN Share 是一个用于在局域网内安全共享 Ollama AI 模型的 GUI 应用程序。支持密码保护、模型白名单和自动发现。

## ✨ 特性

- 🌐 **LAN 自动发现**: 通过 UDP 广播自动发现网络中的 Ollama 共享实例
- 🔒 **可选密码保护**: 为共享的 Ollama API 添加密码验证
- 📋 **模型白名单**: 限制可访问的模型列表
- 🖥️ **原生 GUI**: 基于 eframe/egui 的跨平台桌面应用
- 🔄 **负载均衡**: 支持多个共享实例的轮询负载均衡
- 🛡️ **安全防护**: 防止 SSRF、DoS 和时序攻击

## 🚀 快速开始

### 系统要求

- Rust 1.70+
- Ollama (本地安装)

### 安装

```bash
git clone https://github.com/yourusername/ollama-lan-share.git
cd ollama-lan-share
cargo build --release
```

### 运行

```bash
cargo run --bin ollama_lan_share_gui
```

## 📖 使用指南

1. 启动应用程序
2. 在"Groups"选项卡中创建或加入组
3. 在"Chat"选项卡中配置共享设置：
   - 启用共享
   - 选择要共享的模型
   - 设置密码（可选）
4. 其他设备将自动发现你的共享实例

## 🔌 API 调用接口

### Ollama 共享 API

当启用共享时，应用在本地启动代理服务器 (端口 11435)。

#### 基本调用

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

#### 参数说明

- `host`: 共享主机的 IP 地址
- `password`: 如果设置了密码，则在 `x-vas-key` 头中提供
- `model`: 必须是允许的模型之一
- 其他参数与 Ollama API 相同

## 🏗️ 架构

- **网络发现**: UDP 广播协议，端口 45555
- **代理服务器**: Axum HTTP 服务器，端口 11435
- **安全**: HMAC 验证、IP 白名单、常量时间密码比较

## 🤝 贡献

欢迎贡献！请提交 Issue 或 Pull Request。

## 📄 许可证

MIT License

## 📞 联系

- 邮箱: 259901434@qq.com
- GitHub: [https://github.com/yourusername/ollama-lan-share](https://github.com/yourusername/ollama-lan-share)</content>
<parameter name="filePath">d:\文档\vga-swarm\crates\ollama-lan-share\README.md