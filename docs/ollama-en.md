# Ollama Integration

## Overview

Ollama is an open-source large language model runtime environment that supports running various open-source models locally. Vangriten AI Swarm fully integrates with Ollama, allowing you to use various open-source AI models locally without API keys and completely free.

## Features

### 1. Model Management
- **List Models** - View all installed Ollama models
- **Pull Models** - Download new models from the Ollama repository
- **Delete Models** - Remove unwanted models
- **View Model Info** - Get detailed information about models

### 2. Chat Functionality
- **Simple Chat** - Quickly send a single message and get a response
- **Advanced Chat** - Full chat API supporting multi-turn conversations and custom parameters

### 3. Text Generation
- **Simple Generation** - Generate text based on prompts
- **Advanced Generation** - Text generation with custom parameters

### 4. Vector Embeddings
- **Generate Embeddings** - Convert text to vector representations for semantic search and similarity calculation

### 5. Usage Statistics
- **View Statistics** - Track request count, token usage, and response time
- **Reset Statistics** - Clear usage statistics

## Supported Models

Ollama supports various open-source models. Here are some commonly used models:

### Llama Series
- `llama3` - Meta Llama 3 (latest version)
- `llama3:8b` - Llama 3 8B parameter version
- `llama3:70b` - Llama 3 70B parameter version
- `llama2` - Meta Llama 2

### Mistral Series
- `mistral` - Mistral AI's base model
- `mistral:7b` - Mistral 7B parameter version
- `mixtral` - Mixtral 8x7B mixture of experts model
- `mixtral:8x7b` - Mixtral 8x7B version

### CodeLlama Series
- `codellama` - Code generation specialized model
- `codellama:7b` - CodeLlama 7B parameter version
- `codellama:13b` - CodeLlama 13B parameter version

### Other Models
- `gemma` - Google Gemma model
- `gemma:2b` - Gemma 2B parameter version
- `gemma:7b` - Gemma 7B parameter version
- `phi3` - Microsoft Phi-3 model
- `phi3:mini` - Phi-3 Mini version
- `qwen` - Alibaba Tongyi Qianwen
- `qwen:7b` - Tongyi Qianwen 7B parameter version
- `qwen:14b` - Tongyi Qianwen 14B parameter version

## Installing Ollama

### Windows
1. Visit the [Ollama official website](https://ollama.ai/)
2. Download the Windows installer
3. Run the installer and follow the prompts
4. Open a command line and run `ollama --version` to verify installation

### macOS
```bash
brew install ollama
```

### Linux
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

### Starting Ollama Service
After installation, the Ollama service starts automatically. It listens on `http://localhost:11434` by default.

## Web Interface Usage

### 1. Check Connection
1. Find the "Ollama" section in the web interface
2. Click the "Check Connection" button
3. View connection status and Ollama version

### 2. List Models
1. Click the "List Models" button
2. View all installed models
3. Each model shows name, size, and modification time

### 3. Pull Model
1. Enter the model name in the "Model Name" input field (e.g., `llama3`)
2. Click the "Pull Model" button
3. Wait for the model download to complete (first download may take a while)

### 4. Delete Model
1. Enter the model name to delete in the "Model Name" input field
2. Click the "Delete Model" button
3. Confirm the deletion

### 5. View Model Info
1. Enter the model name in the "Model Name" input field
2. Click the "Show Model Info" button
3. View detailed model information including license and parameters

### 6. Simple Chat
1. Enter the model name in the "Model" input field (e.g., `llama3`)
2. Enter your question or message in the "Prompt" text area
3. Click the "Chat" button
4. View the model's response

### 7. Text Generation
1. Enter the model name in the "Model" input field
2. Enter your prompt in the "Prompt" text area
3. Click the "Generate" button
4. View the generated text

### 8. Generate Vector Embeddings
1. Enter the model name in the "Model" input field
2. Enter the text to embed in the "Input Text" text area
3. Click the "Generate Embedding" button
4. View the generated vector (shows first 10 values and vector length)

### 9. View Usage Statistics
1. Click the "Get Usage Stats" button
2. View total requests, total tokens, and total response time
3. View detailed usage statistics for each model

### 10. Reset Usage Statistics
1. Click the "Reset Usage Stats" button
2. All statistics will be cleared

## Code Usage Examples

### Initialize Ollama Client
```rust
use vangriten_ai_swarm::backend::OllamaManager;

// Use default address (http://localhost:11434)
let manager = OllamaManager::new(None).await;

// Use custom address
let manager = OllamaManager::new(Some("http://192.168.1.100:11434".to_string())).await;
```

### Check Connection
```rust
let status = manager.check_connection().await;
if status.is_connected {
    println!("Connected to Ollama version: {}", status.version.unwrap());
} else {
    println!("Connection failed: {}", status.error.unwrap());
}
```

### List Models
```rust
let models = manager.list_models().await?;
for model in models {
    println!("Model: {} (Size: {} bytes)", model.name, model.size);
}
```

### Pull Model
```rust
let result = manager.pull_model("llama3").await?;
println!("{}", result);
```

### Simple Chat
```rust
let response = manager.chat_simple("llama3", "Hello, how are you?").await?;
println!("Response: {}", response);
```

### Advanced Chat
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

### Text Generation
```rust
let response = manager.generate_simple("llama3", "Write a short poem about programming.").await?;
println!("Generated: {}", response);
```

### Generate Vector Embeddings
```rust
let embedding = manager.embed("llama3", "Hello world").await?;
println!("Embedding length: {}", embedding.len());
```

### View Usage Statistics
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

## Advanced Configuration

### Custom Parameters

#### Temperature
- Range: 0.0 - 2.0
- Default: 0.7
- Description: Controls output randomness. Lower values make output more deterministic, higher values make it more random.

#### Top P (Nucleus Sampling)
- Range: 0.0 - 1.0
- Default: 0.9
- Description: Limits selection to the highest probability tokens.

#### Top K
- Range: 1 - 100
- Default: 40
- Description: Only considers the K highest probability tokens.

#### Num Predict (Max Generation Length)
- Range: 1 - 4096
- Default: 512
- Description: Limits the maximum number of tokens to generate.

#### Num Ctx (Context Length)
- Range: 1 - 4096
- Default: 2048
- Description: The model's context window size.

#### Repeat Penalty
- Range: 0.0 - 2.0
- Default: 1.1
- Description: Penalizes repeated tokens. Higher values mean less repetition.

## Performance Optimization

### 1. Choose the Right Model
- **Small models** (2B-7B): Fast response, suitable for simple tasks
- **Medium models** (7B-13B): Balanced performance and quality
- **Large models** (13B-70B): High quality output, suitable for complex tasks

### 2. Adjust Parameters
- Lower `temperature` for more deterministic output
- Reduce `num_predict` for faster response
- Increase `num_ctx` for longer context support

### 3. Batch Processing
For large numbers of requests, consider using parallel processing.

## Troubleshooting

### Connection Failed
**Problem**: Unable to connect to Ollama service

**Solutions**:
1. Ensure Ollama service is running
2. Check if Ollama is listening on default port 11434
3. Verify firewall settings
4. Try restarting Ollama service

### Model Download Failed
**Problem**: Error when pulling model

**Solutions**:
1. Check network connection
2. Verify model name is correct
3. Ensure sufficient disk space
4. Try a different model version

### Slow Response
**Problem**: Model response time is too long

**Solutions**:
1. Use a smaller model
2. Reduce `num_predict` parameter
3. Increase GPU resources (if available)
4. Close other resource-intensive programs

### Out of Memory
**Problem**: Insufficient memory when running large models

**Solutions**:
1. Use a smaller model version
2. Increase `num_ctx` limit
3. Close other applications
4. Consider using quantized models

## Best Practices

### 1. Model Selection
- For simple tasks, use small models (e.g., `llama3:8b`)
- For complex tasks, use large models (e.g., `llama3:70b`)
- For code generation, use CodeLlama series

### 2. Parameter Tuning
- Start with default parameters
- Adjust gradually based on task requirements
- Record best parameter configurations

### 3. Prompt Engineering
- Use clear, specific prompts
- Provide sufficient context
- Use examples to guide the model

### 4. Resource Management
- Regularly clean up unused models
- Monitor usage statistics
- Allocate system resources reasonably

## API Reference

### OllamaManager
```rust
pub struct OllamaManager {
    // Internal implementation
}
```

#### Methods
- `new(base_url: Option<String>) -> Self` - Create a new Ollama manager
- `check_connection(&self) -> OllamaConnectionStatus` - Check connection status
- `list_models(&self) -> Result<Vec<OllamaModel>, String>` - List all models
- `show_model_info(&self, model_name: &str) -> Result<OllamaModelInfo, String>` - Get model information
- `pull_model(&self, model_name: &str) -> Result<String, String>` - Pull model
- `delete_model(&self, model_name: &str) -> Result<String, String>` - Delete model
- `chat(&self, request: ChatRequest) -> Result<ChatResponse, String>` - Chat
- `chat_simple(&self, model: &str, prompt: &str) -> Result<String, String>` - Simple chat
- `generate(&self, request: GenerateRequest) -> Result<GenerateResponse, String>` - Generate text
- `generate_simple(&self, model: &str, prompt: &str) -> Result<String, String>` - Simple generation
- `embed(&self, model: &str, input: &str) -> Result<Vec<f32>, String>` - Generate embedding
- `get_version(&self) -> Result<String, String>` - Get version
- `get_usage_stats(&self) -> OllamaUsageStats` - Get usage statistics
- `reset_usage_stats(&self)` - Reset usage statistics

## Related Links

- [Ollama Official Website](https://ollama.ai/)
- [Ollama GitHub](https://github.com/ollama/ollama)
- [Ollama Model Library](https://ollama.ai/library)
- [Vangriten AI Swarm Documentation](./README-en.md)

## License

Ollama integration features follow the Vangriten AI Swarm license.
