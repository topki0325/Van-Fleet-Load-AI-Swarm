# 资源管理器独立运行指南

资源管理器可以作为独立的CLI程序运行，不依赖主GUI界面。

## 编译

### 编译所有二进制文件
```bash
cargo build --release
```

### 只编译资源管理器
```bash
cargo build --release --bin resource-manager
```

### 编译调试版本
```bash
cargo build --bin resource-manager
```

## 运行

### 交互式模式
```bash
cargo run --bin resource-manager
```

或者直接运行编译好的二进制文件：
```bash
./target/release/resource-manager
```

### 命令行模式

#### 发现节点
```bash
cargo run --bin resource-manager discover
```

#### 查看状态
```bash
cargo run --bin resource-manager status
```

#### 显示帮助
```bash
cargo run --bin resource-manager help
```

## 交互式命令

启动交互式模式后，可以使用以下命令：

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

### 1. 启动资源管理器
```bash
cargo run --bin resource-manager
```

### 2. 发现节点
```
resource-manager> discover

Discovering nodes...
Found 2 node(s):
  - abc123 @ 192.168.1.100:8080 (Status: Online)
    CPU: 8 cores, Memory: 16384 MB, GPUs: 1
  - def456 @ 192.168.1.101:8080 (Status: Online)
    CPU: 4 cores, Memory: 8192 MB, GPUs: 0
```

### 3. 创建群组
```
resource-manager> create-group
Enter group name: AI Research Group
Enter max members: 10
Created group: group-uuid-here
```

### 4. 请求资源
```
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

### 5. 执行健康检查
```
resource-manager> health
Enter node ID: abc123

Health check result:
  Node: abc123
  Status: Online
  Response time: 15 ms
  Active allocations: 2
  CPU load: 45.0%
```

### 6. 设置负载均衡策略
```
resource-manager> set-strategy
Enter strategy number (1-5): 1
Strategy updated!
```

可用策略：
1. LeastLoaded - 最少负载
2. RoundRobin - 轮询
3. Weighted - 加权
4. Geographic - 地理位置
5. Random - 随机

## 独立部署

资源管理器可以独立部署到服务器上，作为资源管理服务运行：

### 1. 编译发布版本
```bash
cargo build --release --bin resource-manager
```

### 2. 复制到目标服务器
```bash
scp target/release/resource-manager user@server:/path/to/deploy/
```

### 3. 在服务器上运行
```bash
ssh user@server
cd /path/to/deploy/
./resource-manager
```

### 4. 使用systemd管理服务（可选）
创建 `/etc/systemd/system/resource-manager.service`：

```ini
[Unit]
Description=Vangriten AI Swarm Resource Manager
After=network.target

[Service]
Type=simple
User=your-user
WorkingDirectory=/path/to/deploy
ExecStart=/path/to/deploy/resource-manager
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

启动服务：
```bash
sudo systemctl daemon-reload
sudo systemctl enable resource-manager
sudo systemctl start resource-manager
sudo systemctl status resource-manager
```

## 网络配置

资源管理器使用UDP广播进行节点发现，默认端口为8080。

### 防火墙配置

如果使用防火墙，需要允许UDP端口8080：

**Ubuntu/Debian:**
```bash
sudo ufw allow 8080/udp
```

**CentOS/RHEL:**
```bash
sudo firewall-cmd --add-port=8080/udp --permanent
sudo firewall-cmd --reload
```

**Windows:**
```powershell
New-NetFirewallRule -DisplayName "Resource Manager" -Direction Inbound -Protocol UDP -LocalPort 8080 -Action Allow
```

## 日志

资源管理器使用 `tracing` 进行日志记录。

### 设置日志级别
```bash
RUST_LOG=debug cargo run --bin resource-manager
```

### 保存日志到文件
```bash
RUST_LOG=info cargo run --bin resource-manager 2>&1 | tee resource-manager.log
```

## 故障排除

### 无法发现节点
1. 检查防火墙设置
2. 确保所有节点在同一局域网
3. 检查UDP端口8080是否被占用
4. 启用远程访问：`remote` 命令

### 资源分配失败
1. 检查节点状态：`list` 命令
2. 执行健康检查：`health` 命令
3. 检查资源需求是否合理
4. 尝试不同的负载均衡策略

### 端口冲突
如果端口8080被占用，可以修改 `src/backend/resource_manager.rs` 中的 `broadcast_port` 常量。

## 性能优化

### 编译优化
```bash
cargo build --release --bin resource-manager
```

Release模式会启用以下优化：
- 优化级别3
- 链接时优化（LTO）
- 单代码生成单元
- 去除调试符号

### 运行时优化
- 使用 `tokio` 异步运行时
- 连接池管理
- 批量操作减少网络开销

## 与主GUI集成

资源管理器也可以通过Tauri命令从主GUI调用，参见 `src/frontend/mod.rs` 中的命令实现。

## 许可证

MIT License