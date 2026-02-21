# API密钥集中管理

## 概述

Vangriten AI Swarm 提供了强大的API密钥集中管理功能，支持中国和美国主流AI服务提供商。通过预置的配置模板，用户可以快速配置和管理多个AI服务的API密钥。

## 支持的AI服务提供商

### 中国AI服务提供商

| 提供商 | ID | 模型 | 价格 (CNY/1K tokens) | 免费额度 |
|--------|-----|------|---------------------|----------|
| 智谱AI | zhipu-ai | glm-4, glm-4-plus, glm-4-0520, glm-4-0916, glm-4-air, glm-4-flash, glm-3-turbo | 0.10 | 100,000 |
| 百川智能 | baichuan | baichuan-7b, baichuan-13b, baichuan-turbo | 0.05 | 200,000 |
| MiniMax | minimax | abab5.5-chat, abab5.5-chat-pro, abab6-chat, abab6-chat-pro | 0.015 | 1,000,000 |
| 深度求索 | deepseek | deepseek-chat, deepseek-coder | 0.001 | 5,000,000 |
| 月之暗面 | moonshot | moonshot-v1-8k, moonshot-v1-32k, moonshot-v1-128k | 0.012 | 2,000,000 |
| 通义千问 | tongyi | qwen-turbo, qwen-plus, qwen-max, qwen-longcontext | 0.008 | 1,000,000 |
| 文心一言 | ernie-bot | ernie-bot-4, ernie-bot-turbo, ernie-bot-pro | 0.012 | 500,000 |

### 美国AI服务提供商

| 提供商 | ID | 模型 | 价格 (USD/1K tokens) | 免费额度 |
|--------|-----|------|---------------------|----------|
| OpenAI | openai | gpt-4, gpt-4-turbo, gpt-4-turbo-preview, gpt-3.5-turbo, gpt-3.5-turbo-16k | 0.03-0.06 | 无 |
| Anthropic Claude | anthropic | claude-3-opus, claude-3-sonnet, claude-3-haiku | 0.015-0.075 | 无 |
| Google Gemini | google | gemini-pro, gemini-pro-vision, gemini-1.5-pro, gemini-1.5-flash | 0.00025-0.0005 | 15,000,000 |
| Cohere | cohere | command, command-light, command-r | 0.015-0.03 | 无 |
| Mistral AI | mistral | mistral-large, mistral-medium, mistral-small, mixtral-8x7b, mixtral-8x22b | 0.003-0.006 | 无 |

### 全球AI服务提供商

| 提供商 | ID | 模型 | 价格 | 免费额度 |
|--------|-----|------|------|----------|
| Meta Llama | meta | llama-3-70b, llama-3-8b, llama-2-70b | 免费 | 无 |
| Hugging Face | huggingface | meta-llama-3-70b-instruct, mistralai-mixtral-8x7b | 0.0001 | 1,000,000 |

## 核心功能

### 1. 预置提供商配置

系统内置了所有主流AI服务提供商的配置信息，包括：
- API端点地址
- 支持的模型列表
- 定价信息
- 免费额度
- 是否需要API密钥

### 2. 安全密钥存储

- 使用AES-256-GCM加密算法存储API密钥
- 密钥文件保存在 `vault/` 目录下
- 每个提供商的密钥独立加密存储

### 3. 使用统计

- 自动记录每个API密钥的使用次数
- 记录最后一次使用时间
- 跟踪Token使用量

### 4. 默认提供商设置

- 可以设置默认使用的AI服务提供商
- 方便快速切换不同的AI服务

## 使用方法

### 通过Web界面管理

1. **加载提供商列表**
   - 点击 "Load Providers" 按钮查看所有预置的AI服务提供商
   - 可以按地区筛选（中国/美国/全部）

2. **查看提供商配置**
   - 在 "Provider ID" 输入框中输入提供商ID（如 `openai`、`zhipu-ai`）
   - 点击 "Get Config" 查看该提供商的详细配置信息

3. **存储API密钥**
   - 在 "Provider" 输入框中输入提供商ID
   - 在 "Key" 输入框中输入API密钥
   - 点击 "Store" 按钮保存密钥

4. **检索API密钥**
   - 在 "Provider" 输入框中输入提供商ID
   - 点击 "Retrieve" 按钮获取已存储的密钥

5. **查看使用统计**
   - 点击 "Usage" 按钮查看所有API密钥的使用情况

6. **设置默认提供商**
   - 在 "Provider ID" 输入框中输入提供商ID
   - 点击 "Set Default" 按钮设置为默认提供商

### 通过代码调用

```rust
use crate::shared::models::{VaultOp, VaultResult};
use crate::backend::ApiKeyManager;

let api_manager = ApiKeyManager::new().await;

// 存储API密钥
let result = api_manager.vault_operation(VaultOp::Store {
    provider: "openai".to_string(),
    key: "sk-xxxxxxxxxxxxxxxx".to_string(),
});

// 检索API密钥
let result = api_manager.vault_operation(VaultOp::Retrieve {
    provider: "openai".to_string(),
});

// 获取所有预置提供商
let result = api_manager.vault_operation(VaultOp::GetProviders);

// 获取特定提供商配置
let result = api_manager.vault_operation(VaultOp::GetProviderConfig {
    provider: "zhipu-ai".to_string(),
});

// 设置默认提供商
let result = api_manager.vault_operation(VaultOp::SetDefaultProvider {
    provider: "openai".to_string(),
});
```

## API密钥获取指南

### 中国AI服务提供商

#### 智谱AI (Zhipu AI)
- 官网: https://open.bigmodel.cn/
- 注册账号并创建API密钥
- 支持GLM系列大模型

#### 百川智能 (Baichuan)
- 官网: https://www.baichuan-ai.com/
- 注册账号并申请API密钥
- 提供开源大模型

#### MiniMax
- 官网: https://api.minimax.chat/
- 注册账号并创建API密钥
- 高性能大语言模型

#### 深度求索 (DeepSeek)
- 官网: https://platform.deepseek.com/
- 注册账号并获取API密钥
- 开源大语言模型

#### 月之暗面 (Moonshot AI)
- 官网: https://platform.moonshot.cn/
- 注册账号并创建API密钥
- 长上下文大模型

#### 通义千问 (Tongyi Qianwen)
- 官网: https://dashscope.aliyuncs.com/
- 阿里云账号并开通服务
- 获取API Key

#### 文心一言 (ERNIE Bot)
- 官网: https://cloud.baidu.com/product/wenxinworkshop
- 百度智能云账号并开通服务
- 获取API Key和Secret Key

### 美国AI服务提供商

#### OpenAI
- 官网: https://platform.openai.com/
- 注册账号并创建API密钥
- 支持GPT-4和GPT-3.5系列

#### Anthropic Claude
- 官网: https://console.anthropic.com/
- 注册账号并创建API密钥
- Claude系列大模型

#### Google Gemini
- 官网: https://makersuite.google.com/
- Google账号并创建API密钥
- Gemini系列多模态模型

#### Cohere
- 官网: https://dashboard.cohere.ai/
- 注册账号并创建API密钥
- Command系列大模型

#### Mistral AI
- 官网: https://console.mistral.ai/
- 注册账号并创建API密钥
- 开源大模型

### 全球AI服务提供商

#### Meta Llama
- 官网: https://llama.meta.com/
- 开源模型，无需API密钥
- 可本地部署或使用第三方服务

#### Hugging Face
- 官网: https://huggingface.co/
- 注册账号并创建API Token
- 提供开源模型推理API

## 安全建议

1. **密钥保护**
   - 不要在代码中硬编码API密钥
   - 使用环境变量或密钥管理工具
   - 定期轮换API密钥

2. **访问控制**
   - 限制API密钥的权限范围
   - 设置使用限额和预算
   - 监控异常使用情况

3. **存储安全**
   - 系统使用AES-256-GCM加密存储密钥
   - 密钥文件权限应设置为仅限当前用户访问
   - 定期备份密钥文件

## 最佳实践

1. **选择合适的提供商**
   - 根据需求选择模型（代码生成、对话、长文本等）
   - 考虑成本和性能平衡
   - 利用免费额度进行测试

2. **多提供商策略**
   - 配置多个提供商作为备选
   - 根据任务类型自动切换
   - 实现负载均衡

3. **监控使用情况**
   - 定期查看使用统计
   - 分析成本和性能
   - 优化API调用策略

## 故障排除

### 常见问题

1. **API密钥无效**
   - 检查密钥是否正确复制
   - 确认密钥是否已激活
   - 检查账户余额是否充足

2. **请求失败**
   - 检查网络连接
   - 确认API端点地址正确
   - 查看错误日志获取详细信息

3. **配额限制**
   - 查看提供商的使用限制
   - 考虑升级套餐
   - 实现请求队列和重试机制

## 扩展功能

### 添加自定义提供商

如果需要添加自定义AI服务提供商，可以修改 `src/backend/provider_config.rs` 文件，在 `get_predefined_providers()` 函数中添加新的提供商配置：

```rust
ProviderConfig {
    id: "custom-provider".to_string(),
    name: "Custom AI Provider".to_string(),
    region: ProviderRegion::Global,
    api_endpoint: "https://api.custom-provider.com/v1/chat/completions".to_string(),
    models: vec![
        "custom-model-1".to_string(),
        "custom-model-2".to_string(),
    ],
    pricing: PricingInfo {
        currency: "USD".to_string(),
        input_price_per_1k: 0.01,
        output_price_per_1k: 0.02,
        free_tier_limit: Some(100000),
    },
    requires_api_key: true,
    description: "Custom AI provider description".to_string(),
}
```

## 相关文档

- [资源管理代理](resource-manager.md)
- [数据模型](structure/data-models.md)
- [后端服务](structure/backend-services.md)