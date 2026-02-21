# 开始使用

## 先决条件

在开始使用 Vangriten AI Swarm 之前，请确保您的系统已安装以下组件：

- **Rust 1.70+** (通过 rustup 安装)
- **Node.js 18+** (仅在使用可选的 Tauri 前端时需要)
- **目标 AI 提供商的 API 密钥** (如 OpenAI, Anthropic 等)

## 安装

1. **克隆仓库：**

    ```bash
    git clone <repo-url>
    cd vga-swarm
    ```

2. **构建项目：**

    ```bash
    cargo build --release
    ```

## 配置

1. **设置 API 密钥：**
    - 在 GUI 顶部菜单中打开 `API管理(API Manager)` 弹窗。
    - 按提示初始化并设置密码后解锁，再保存/查看各 Provider 的 APIKey（本地加密存储，查看/保存需解锁）。

2. **配置代理：**
    - 定义代理角色和能力。

3. **定义工作流：**
    - 为您的项目选择模板和对应的工作流。

## 运行

运行 Rust 原生 GUI（无 WebView，推荐）：

```bash
cargo run -p vgs
```

Windows 下会生成并运行：`target/debug/vgs.exe`。

如果你需要 Tauri 前端（可选）：

```bash
cargo tauri dev
```

运行已编译的二进制文件：

```bash
./target/release/vangriten-ai-swarm
```

或运行原生 GUI 的 release 产物（Windows）：

```bash
./target/release/vgs.exe
```

## 网络发现

- 当启动主控端时，它会自动在局域网内搜索可用的子客户端。
- 确保子客户端在同一网络下且防火墙允许相应端口的通信。
