# Ollama 集成文档

## 概述

Ollama 是一个开源的大语言模型运行环境，支持在本地运行多种开源模型。Vangriten AI Swarm 完整集成了 Ollama，允许您在本地使用各种开源 AI 模型，无需 API 密钥，完全免费。

## 功能特性

### 1. 模型管理
- **列出模型** - 查看所有已安装的 Ollama 模型
- **拉取模型** - 从 Ollama 仓库下载新模型
- **删除模型** - 删除不需要的模型
- **查看模型信息** - 获取模型的详细信息

### 2. 聊天功能
- **简单聊天** - 快速发送单条消息并获得回复
- **高级聊天** - 支持多轮对话、自定义参数的完整聊天 API

### 3. 文本生成
- **简单生成** - 基于提示词生成文本
- **高级生成** - 支持自定义参数的文本生成

### 4. 向量嵌入
- **生成嵌入** - 将文本转换为向量表示，用于语义搜索和相似度计算

### 5. 使用统计
- **查看统计** - 跟踪请求次数、Token 使用量和响应时间
- **重置统计** - 清零使用统计数据

## 支持的模型

Ollama 支持多种开源模型，以下是一些常用模型：

### Llama 系列
- `llama3` - Meta Llama 3（最新版本）
- `llama3:8b` - Llama 3 8B 参数版本
- `llama3:70b` - Llama 3 70B 参数版本
- `llama2` - Meta Llama 2

### Mistral 系列
- `mistral` - Mistral AI 的基础模型
- `mistral:7b` - Mistral 7B 参数版本
- `mixtral` - Mixtral 8x7B 混合专家模型
- `mixtral:8x7b` - Mixtral 8x7B 版本

### CodeLlama 系列
- `codellama` - 代码生成专用模型
- `codellama:7b` - CodeLlama 7B 参数版本
- `codellama:13b` - CodeLlama 13B 参数版本

### 其他模型
- `gemma` - Google Gemma 模型
- `gemma:2b` - Gemma 2B 参数版本
- `gemma:7b` - Gemma 7B 参数版本
- `phi3` - Microsoft Phi-3 模型
- `phi3:mini` - Phi-3 Mini 版本
- `qwen` - 阿里通义千问
- `qwen:7b` - 通义千问 7B 参数版本
- `qwen:14b` - 通义千问 14B 参数版本

## 安装 Ollama

### Windows
1. 访问 [Ollama 官网](https://ollama.ai/)
2. 下载 Windows 安装程序
3. 运行安装程序并按照提示完成安装
4. 打开命令行，运行 `ollama --version` 验证安装

### macOS
```bash
brew install ollama
```

### Linux
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

### 启动 Ollama 服务
安装完成后，Ollama 服务会自动启动。默认监听 `http://localhost:11434`

## Web 界面使用

### 1. 检查连接
1. 在 Web 界面中找到 "Ollama" 部分
2. 点击 "Check Connection" 按钮
3. 查看连接状态和 Ollama 版本

### 2. 列出模型
1. 点击 "List Models" 按钮
2. 查看所有已安装的模型列表
3. 每个模型显示名称、大小和修改时间

### 3. 拉取模型
1. 在 "Model Name" 输入框中输入模型名称（如 `llama3`）
2. 点击 "Pull Model" 按钮
3. 等待模型下载完成（首次下载可能需要较长时间）

### 4. 删除模型
1. 在 "Model Name" 输入框中输入要删除的模型名称
2. 点击 "Delete Model" 按钮
3. 确认删除操作

### 5. 查看模型信息
1. 在 "Model Name" 输入框中输入模型名称
2. 点击 "Show Model Info" 按钮
3. 查看模型的详细信息，包括许可证、参数等

### 6. 简单聊天
1. 在 "Model" 输入框中输入模型名称（如 `llama3`）
2. 在 "Prompt" 文本框中输入您的问题或消息
3. 点击 "Chat" 按钮
4. 查看模型的回复

### 7. 文本生成
1. 在 "Model" 输入框中输入模型名称
2. 在 "Prompt" 文本框中输入提示词
3. 点击 "Generate" 按钮
4. 查看生成的文本

### 8. 生成向量嵌入
1. 在 "Model" 输入框中输入模型名称
2. 在 "Input Text" 文本框中输入要嵌入的文本
3. 点击 "Generate Embedding" 按钮
4. 查看生成的向量（显示前 10 个值和向量长度）

### 9. 查看使用统计
1. 点击 "Get Usage Stats" 按钮
2. 查看总请求数、总 Token 数、总响应时间
3. 查看每个模型的详细使用统计

### 10. 重置使用统计
1. 点击 "Reset Usage Stats" 按钮
2. 所有统计数据将被清零

## 代码调用示例

### 初始化 Ollama 客户端
```rust
use vangriten_ai_swarm::backend::OllamaManager;

// 使用默认地址 (http://localhost:11434)
let manager = OllamaManager::new(None).await;

// 使用自定义地址
let manager = OllamaManager::new(Some("http://192.168.1.100:11434".to_string())).await;
```

### 检查连接
```rust
let status = manager.check_connection().await;
if status.is_connected {
    println!("Connected to Ollama version: {}", status.version.unwrap());
} else {
    println!("Connection failed: {}", status.error.unwrap());
}
```

### 列出模型
```rust
let models = manager.list_models().await?;
for model in models {
    println!("Model: {} (Size: {} bytes)", model.name, model.size);
}
```

### 拉取模型
```rust
let result = manager.pull_model("llama3").await?;
println!("{}", result);
```

### 简单聊天
```rust
let response = manager.chat_simple("llama3", "Hello, how are you?").await?;
println!("Response: {}", response);
```

### 高级聊天
```rust
use vangriten_ai_swarm::backend::ollama_client::{ChatRequest, ChatMessage, ChatOptions};

let request = ChatRequest {
    model: "llama3".to_string(),
    messages: vec![
        ChatMessage {
            role: "user".to_string(),
            content: "What is Rust?".to_string(),
            images: None,
        },
    ],
    stream: Some(false),
    format: None,
    options: Some(ChatOptions {
        temperature: Some(0.7),
        top_p: Some(0.9),
        top_k: Some(40),
        num_predict: Some(512),
        num_ctx: Some(2048),
        repeat_penalty: Some(1.1),
        stop: None,
    }),
};

let response = manager.chat(request).await?;
println!("Response: {}", response.message.content);
```

### 文本生成
```rust
let response = manager.generate_simple("llama3", "Write a short poem about programming.").await?;
println!("Generated: {}", response);
```

### 生成向量嵌入
```rust
let embedding = manager.embed("llama3", "Hello world").await?;
println!("Embedding length: {}", embedding.len());
```

### 查看使用统计
```rust
let stats = manager.get_usage_stats().await;
println!("Total requests: {}", stats.total_requests);
println!("Total tokens: {}", stats.total_tokens);
println!("Total duration: {} ms", stats.total_duration_ms);

for (model_name, model_stats) in stats.model_stats {
    println!("Model {}: {} requests, {} tokens", 
             model_name, model_stats.requests, model_stats.tokens);
}
```

## 高级配置

### 自定义参数

#### Temperature (温度)
- 范围: 0.0 - 2.0
- 默认: 0.7
- 说明: 控制输出的随机性。值越低输出越确定，值越高输出越随机

#### Top P (核采样)
- 范围: 0.0 - 1.0
- 默认: 0.9
- 说明: 限制从概率最高的 tokens 中选择

#### Top K
- 范围: 1 - 100
- 默认: 40
- 说明: 只考虑概率最高的 K 个 tokens

#### Num Predict (最大生成长度)
- 范围: 1 - 4096
- 默认: 512
- 说明: 限制生成的最大 token 数量

#### Num Ctx (上下文长度)
- 范围: 1 - 4096
- 默认: 2048
- 说明: 模型的上下文窗口大小

#### Repeat Penalty (重复惩罚)
- 范围: 0.0 - 2.0
- 默认: 1.1
- 说明: 惩罚重复的 token，值越高重复越少

## 性能优化

### 1. 选择合适的模型
- **小模型** (2B-7B): 快速响应，适合简单任务
- **中等模型** (7B-13B): 平衡性能和质量
- **大模型** (13B-70B): 高质量输出，适合复杂任务

### 2. 调整参数
- 降低 `temperature` 提高确定性
- 减少 `num_predict` 加快响应速度
- 增加 `num_ctx` 支持更长上下文

### 3. 批量处理
对于大量请求，考虑使用并行处理

## 故障排除

### 连接失败
**问题**: 无法连接到 Ollama 服务

**解决方案**:
1. 确认 Ollama 服务正在运行
2. 检查 Ollama 是否在默认端口 11434 监听
3. 验证防火墙设置
4. 尝试重启 Ollama 服务

### 模型下载失败
**问题**: 拉取模型时出错

**解决方案**:
1. 检查网络连接
2. 确认模型名称正确
3. 确保有足够的磁盘空间
4. 尝试使用不同的模型版本

### 响应缓慢
**问题**: 模型响应时间过长

**解决方案**:
1. 使用更小的模型
2. 减少 `num_predict` 参数
3. 增加 GPU 资源（如果可用）
4. 关闭其他占用资源的程序

### 内存不足
**问题**: 运行大模型时内存不足

**解决方案**:
1. 使用更小的模型版本
2. 增加 `num_ctx` 限制
3. 关闭其他应用程序
4. 考虑使用量化模型

## 最佳实践

### 1. 模型选择
- 对于简单任务，使用小模型（如 `llama3:8b`）
- 对于复杂任务，使用大模型（如 `llama3:70b`）
- 对于代码生成，使用 CodeLlama 系列

### 2. 参数调优
- 从默认参数开始
- 根据任务需求逐步调整
- 记录最佳参数配置

### 3. 提示词工程
- 使用清晰、具体的提示词
- 提供足够的上下文
- 使用示例引导模型

### 4. 资源管理
- 定期清理不需要的模型
- 监控使用统计
- 合理分配系统资源

## API 参考

### OllamaManager
```rust
pub struct OllamaManager {
    // 内部实现
}
```

#### 方法
- `new(base_url: Option<String>) -> Self` - 创建新的 Ollama 管理器
- `check_connection(&self) -> OllamaConnectionStatus` - 检查连接状态
- `list_models(&self) -> Result<Vec<OllamaModel>, String>` - 列出所有模型
- `show_model_info(&self, model_name: &str) -> Result<OllamaModelInfo, String>` - 获取模型信息
- `pull_model(&self, model_name: &str) -> Result<String, String>` - 拉取模型
- `delete_model(&self, model_name: &str) -> Result<String, String>` - 删除模型
- `chat(&self, request: ChatRequest) -> Result<ChatResponse, String>` - 聊天
- `chat_simple(&self, model: &str, prompt: &str) -> Result<String, String>` - 简单聊天
- `generate(&self, request: GenerateRequest) -> Result<GenerateResponse, String>` - 生成文本
- `generate_simple(&self, model: &str, prompt: &str) -> Result<String, String>` - 简单生成
- `embed(&self, model: &str, input: &str) -> Result<Vec<f32>, String>` - 生成嵌入
- `get_version(&self) -> Result<String, String>` - 获取版本
- `get_usage_stats(&self) -> OllamaUsageStats` - 获取使用统计
- `reset_usage_stats(&self)` - 重置使用统计

## 相关链接

- [Ollama 官网](https://ollama.ai/)
- [Ollama GitHub](https://github.com/ollama/ollama)
- [Ollama 模型库](https://ollama.ai/library)
- [Vangriten AI Swarm 文档](./README.md)

## 许可证

Ollama 集成功能遵循 Vangriten AI Swarm 的许可证。
