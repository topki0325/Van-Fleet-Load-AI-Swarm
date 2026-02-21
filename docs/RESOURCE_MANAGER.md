# 资源管理器 (Resource Manager)

独立的资源管理CLI工具，用于管理Vangriten AI Swarm的分布式计算资源。

## 快速开始

### 编译
```bash
cargo build --release --bin resource-manager
```

### 运行
```bash
cargo run --bin resource-manager
```

### 命令行模式
```bash
# 发现节点
cargo run --bin resource-manager discover

# 查看状态
cargo run --bin resource-manager status

# 显示帮助
cargo run --bin resource-manager help
```

## 主要功能

- **节点发现**：自动发现局域网内的计算节点
- **群组管理**：创建和管理AI蜂群群组
- **资源分配**：智能调度和分配CPU、内存、GPU资源
- **负载均衡**：支持多种负载均衡策略
- **健康检查**：监控节点健康状态和资源使用
- **远程访问**：控制是否允许远程访问本机资源

## 交互式命令

| 命令 | 描述 |
|------|------|
| `help` | 显示帮助信息 |
| `discover` | 发现局域网内的节点 |
| `list` | 列出所有已发现的节点 |
| `groups` | 列出所有群组 |
| `pools` | 列出所有资源池 |
| `create-group` | 创建新的群组 |
| `request` | 请求资源分配 |
| `strategy` | 显示当前负载均衡策略 |
| `set-strategy` | 设置负载均衡策略 |
| `remote` | 切换远程访问 |
| `health` | 对节点执行健康检查 |
| `clear` | 清屏 |
| `quit` / `exit` | 退出程序 |

## 使用示例

### 启动并发现节点
```bash
cargo run --bin resource-manager

resource-manager> discover
Found 2 node(s):
  - abc123 @ 192.168.1.100:8080 (Status: Online)
    CPU: 8 cores, Memory: 16384 MB, GPUs: 1
```

### 创建群组
```bash
resource-manager> create-group
Enter group name: AI Research Group
Enter max members: 10
Created group: group-uuid-here
```

### 请求资源
```bash
resource-manager> request
Enter CPU cores (default: 2): 4
Enter memory in MB (default: 4096): 8192
GPU required? (y/n, default: n): y

Resources allocated!
  Allocation ID: allocation-uuid-here
  Node: abc123
  CPU: 4 cores
  Memory: 8192 MB
  GPU: gpu-0 (8192 MB)
```

## 独立部署

资源管理器可以独立部署到服务器上运行：

```bash
# 编译
cargo build --release --bin resource-manager

# 复制到服务器
scp target/release/resource-manager user@server:/path/to/deploy/

# 在服务器上运行
ssh user@server
cd /path/to/deploy/
./resource-manager
```

详细部署指南请参考 [resource-manager-cli.md](resource-manager-cli.md)

## 网络配置

- 默认UDP端口：8080
- 需要防火墙允许UDP 8080端口

## 文档

- [资源管理代理文档](resource-manager.md) - 完整的功能文档
- [CLI使用指南](resource-manager-cli.md) - 独立运行和部署指南

## 许可证

MIT License