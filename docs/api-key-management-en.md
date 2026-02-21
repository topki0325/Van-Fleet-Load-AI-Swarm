# API Key Management

## Overview

API Key Management is a secure vault system for managing API keys for multiple AI service providers. It uses AES-256 encryption to store API keys securely and provides comprehensive usage statistics tracking.

## Features

### 1. Secure Storage
- **AES-256 Encryption**: Military-grade encryption for all stored API keys
- **Password Protection**: Vault is protected by a master password
- **Encrypted Files**: All keys are stored in encrypted files on disk

### 2. Multi-Provider Support
- **Chinese AI Providers**: Zhipu AI, Baichuan, MiniMax, DeepSeek, Moonshot, Tongyi Qianwen, ERNIE Bot
- **US AI Providers**: OpenAI, Anthropic Claude, Google Gemini, Cohere, Mistral AI
- **Global Providers**: Meta Llama, Hugging Face
- **Local Providers**: Ollama

### 3. Usage Statistics
- **Request Tracking**: Track number of requests made to each provider
- **Token Usage**: Monitor token consumption
- **Last Used**: Track when each provider was last accessed

### 4. Easy Management
- **Store Keys**: Securely store API keys for any provider
- **Retrieve Keys**: Get decrypted API keys when needed
- **Delete Keys**: Remove API keys from the vault
- **List Providers**: View all providers with stored keys
- **Set Default**: Set a default provider for quick access

## Supported Providers

### Chinese AI Providers

#### Zhipu AI (智谱AI)
- **Models**: glm-4, glm-4-plus, glm-4-0520, glm-4-0916, glm-4-air, glm-4-flash, glm-3-turbo
- **API Endpoint**: https://open.bigmodel.cn/api/paas/v4/chat/completions
- **Pricing**: ¥0.1 per 1K tokens (input/output)
- **Free Tier**: 100,000 tokens

#### Baichuan (百川智能)
- **Models**: baichuan-7b, baichuan-13b, baichuan-turbo
- **API Endpoint**: https://api.baichuan-ai.com/v1/chat/completions
- **Pricing**: ¥0.05 per 1K tokens (input/output)
- **Free Tier**: 200,000 tokens

#### MiniMax
- **Models**: abab5.5-chat, abab5.5-chat-pro, abab6-chat, abab6-chat-pro
- **API Endpoint**: https://api.minimax.chat/v1/text/chatcompletion_v2
- **Pricing**: ¥0.015 per 1K tokens (input/output)
- **Free Tier**: 1,000,000 tokens

#### DeepSeek (深度求索)
- **Models**: deepseek-chat, deepseek-coder
- **API Endpoint**: https://api.deepseek.com/chat/completions
- **Pricing**: ¥0.001 per 1K input tokens, ¥0.002 per 1K output tokens
- **Free Tier**: 5,000,000 tokens

#### Moonshot AI (月之暗面)
- **Models**: moonshot-v1-8k, moonshot-v1-32k, moonshot-v1-128k
- **API Endpoint**: https://api.moonshot.cn/v1/chat/completions
- **Pricing**: ¥0.012 per 1K tokens (input/output)
- **Free Tier**: 2,000,000 tokens

#### Tongyi Qianwen (通义千问)
- **Models**: qwen-turbo, qwen-plus, qwen-max, qwen-longcontext
- **API Endpoint**: https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation
- **Pricing**: ¥0.008 per 1K tokens (input/output)
- **Free Tier**: 1,000,000 tokens

#### ERNIE Bot (文心一言)
- **Models**: ernie-bot-4, ernie-bot-turbo, ernie-bot-pro
- **API Endpoint**: https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions
- **Pricing**: ¥0.012 per 1K tokens (input/output)
- **Free Tier**: 500,000 tokens

### US AI Providers

#### OpenAI
- **Models**: gpt-4, gpt-4-turbo, gpt-4-turbo-preview, gpt-3.5-turbo, gpt-3.5-turbo-16k, gpt-3.5-turbo-instruct
- **API Endpoint**: https://api.openai.com/v1/chat/completions
- **Pricing**: $0.03 per 1K input tokens, $0.06 per 1K output tokens

#### Anthropic Claude
- **Models**: claude-3-opus, claude-3-sonnet, claude-3-haiku
- **API Endpoint**: https://api.anthropic.com/v1/messages
- **Pricing**: $0.015 per 1K input tokens, $0.075 per 1K output tokens

#### Google Gemini
- **Models**: gemini-pro, gemini-pro-vision, gemini-1.5-pro, gemini-1.5-flash
- **API Endpoint**: https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent
- **Pricing**: $0.00025 per 1K input tokens, $0.0005 per 1K output tokens
- **Free Tier**: 15,000,000 tokens

#### Cohere
- **Models**: command, command-light, command-r
- **API Endpoint**: https://api.cohere.ai/v1/chat
- **Pricing**: $0.015 per 1K input tokens, $0.03 per 1K output tokens

#### Mistral AI
- **Models**: mistral-large, mistral-medium, mistral-small, mixtral-8x7b, mixtral-8x22b
- **API Endpoint**: https://api.mistral.ai/v1/chat/completions
- **Pricing**: €0.003 per 1K input tokens, €0.006 per 1K output tokens

### Global Providers

#### Meta Llama
- **Models**: llama-3-70b, llama-3-8b, llama-2-70b
- **API Endpoint**: https://api.meta.com/v1/chat/completions
- **Pricing**: Free
- **API Key**: Not required

#### Hugging Face
- **Models**: meta-llama-3-70b-instruct, mistralai-mixtral-8x7b
- **API Endpoint**: https://api-inference.huggingface.co/models
- **Pricing**: $0.0001 per 1K tokens (input/output)
- **Free Tier**: 1,000,000 tokens

### Local Providers

#### Ollama
- **Models**: llama3, mistral, codellama, gemma, phi3, qwen, and more
- **API Endpoint**: http://localhost:11434/api/chat
- **Pricing**: Free
- **API Key**: Not required

## Web Interface Usage

### Initialize Vault
1. Open the web interface
2. Navigate to "Vault" section
3. Enter a strong password
4. Click "Initialize" (only required once)

### Unlock Vault
1. Enter the vault password
2. Click "Unlock" to access stored keys

### Store API Key
1. Enter provider ID (e.g., "openai", "zhipu-ai")
2. Enter your API key
3. Click "Store" to save the key

### Retrieve API Key
1. Enter provider ID
2. Click "Retrieve" to get the decrypted key

### List Providers
1. Click "List Providers" to view all providers with stored keys

### Delete API Key
1. Enter provider ID
2. Click "Delete" to remove the key from the vault

### View Usage Statistics
1. Click "Usage" to view statistics for all providers
2. See request counts, token usage, and last used timestamps

## Code Usage Examples

### Initialize Vault
```rust
use vangriten_ai_swarm::backend::ApiKeyManager;

let api_manager = ApiKeyManager::new().await;

// Initialize vault with password (only once)
api_manager.vault_initialize("your-strong-password").unwrap();

// Unlock vault
api_manager.vault_unlock("your-strong-password").unwrap();
```

### Store API Key
```rust
use vangriten_ai_swarm::shared::models::VaultOp;

let result = api_manager.vault_operation(VaultOp::Store {
    provider: "openai".to_string(),
    key: "sk-...".to_string(),
}).unwrap();
```

### Retrieve API Key
```rust
use vangriten_ai_swarm::shared::models::VaultOp;

let result = api_manager.vault_operation(VaultOp::Retrieve {
    provider: "openai".to_string(),
}).unwrap();

if let VaultResult::Key(key) = result {
    println!("API Key: {}", key);
}
```

### List Providers
```rust
use vangriten_ai_swarm::shared::models::VaultOp;

let result = api_manager.vault_operation(VaultOp::List).unwrap();

if let VaultResult::Providers(providers) = result {
    for provider in providers {
        println!("Provider: {}", provider);
    }
}
```

### Get Provider Configuration
```rust
use vangriten_ai_swarm::shared::models::VaultOp;

let result = api_manager.vault_operation(VaultOp::GetProviderConfig {
    provider: "openai".to_string(),
}).unwrap();

if let VaultResult::ProviderConfig(config) = result {
    println!("Name: {}", config.name);
    println!("Models: {:?}", config.models);
    println!("Pricing: {:?}", config.pricing);
}
```

### Set Default Provider
```rust
use vangriten_ai_swarm::shared::models::VaultOp;

let result = api_manager.vault_operation(VaultOp::SetDefaultProvider {
    provider: "openai".to_string(),
}).unwrap();

if let VaultResult::DefaultProvider(provider) = result {
    println!("Default provider: {}", provider);
}
```

### Get Usage Statistics
```rust
let usage_entries = api_manager.get_usage_entries().await;

for entry in usage_entries {
    println!("Provider: {}", entry.provider);
    println!("Requests: {}", entry.requests_made);
    println!("Last Used: {}", entry.last_used);
}
```

## Security Best Practices

### 1. Strong Password
- Use a password with at least 12 characters
- Include uppercase, lowercase, numbers, and special characters
- Don't reuse passwords from other services

### 2. Secure Storage
- The vault is stored in `vault/keys.enc`
- All keys are encrypted with AES-256
- Salt is stored separately in `vault/salt.bin`

### 3. Access Control
- Always unlock the vault before accessing keys
- Lock the vault when not in use
- Never share your vault password

### 4. Backup
- Regularly backup your vault directory
- Store backups in a secure location
- Test backup restoration

## API Reference

### ApiKeyManager
```rust
pub struct ApiKeyManager {
    // Internal implementation
}
```

#### Methods
- `new() -> Self` - Create a new API key manager
- `vault_initialize(&self, password: &str) -> Result<(), VgaError>` - Initialize vault
- `vault_unlock(&self, password: &str) -> Result<(), VgaError>` - Unlock vault
- `vault_lock(&self)` - Lock vault
- `vault_operation(&self, op: VaultOp) -> Result<VaultResult, VgaError>` - Perform vault operation
- `get_decrypted_key(&self, provider: &str) -> Result<String, VgaError>` - Get decrypted key
- `update_usage_stats(&self, provider: &str)` - Update usage statistics
- `get_usage_entries(&self) -> Vec<VaultUsageEntry>` - Get usage statistics
- `check_quota_availability(&self, provider: &str) -> bool` - Check quota availability

### VaultOp
```rust
pub enum VaultOp {
    Store { provider: String, key: String },
    Retrieve { provider: String },
    Delete { provider: String },
    List,
    GetProviders,
    GetProviderConfig { provider: String },
    SetDefaultProvider { provider: String },
}
```

### VaultResult
```rust
pub enum VaultResult {
    Success,
    Key(String),
    Providers(Vec<String>),
    ProviderConfigs(Vec<ProviderConfig>),
    ProviderConfig(ProviderConfig),
    DefaultProvider(String),
}
```

## Troubleshooting

### Vault Already Initialized
**Problem**: Error when trying to initialize vault

**Solution**: The vault is already initialized. Use `vault_unlock` instead.

### Invalid Password
**Problem**: Cannot unlock vault

**Solution**: Verify your password is correct. If forgotten, you'll need to delete the vault directory and reinitialize.

### Provider Not Found
**Problem**: Error when retrieving provider configuration

**Solution**: Check the provider ID is correct. Use `GetProviders` to list available providers.

### Encryption Error
**Problem**: Error when storing or retrieving keys

**Solution**: Ensure the vault is unlocked and the password is correct.

## Related Links

- [Provider Configuration](../src/backend/provider_config.rs) - Source code for provider configurations
- [Ollama Integration](ollama-en.md) - Local AI model support
- [Resource Manager](resource-manager-en.md) - Distributed resource management

## License

API Key Management features follow the Vangriten AI Swarm license.
