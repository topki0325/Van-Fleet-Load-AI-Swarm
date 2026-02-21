use crate::shared::models::{ProviderConfig, ProviderRegion, PricingInfo};

pub fn get_predefined_providers() -> Vec<ProviderConfig> {
    vec![
        // 中国AI服务提供商
        ProviderConfig {
            id: "zhipu-ai".to_string(),
            name: "智谱AI (Zhipu AI)".to_string(),
            region: ProviderRegion::China,
            api_endpoint: "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string(),
            models: vec![
                "glm-4".to_string(),
                "glm-4-plus".to_string(),
                "glm-4-0520".to_string(),
                "glm-4-0916".to_string(),
                "glm-4-air".to_string(),
                "glm-4-flash".to_string(),
                "glm-3-turbo".to_string(),
            ],
            pricing: PricingInfo {
                currency: "CNY".to_string(),
                input_price_per_1k: 0.1,
                output_price_per_1k: 0.1,
                free_tier_limit: Some(100000),
            },
            requires_api_key: true,
            description: "智谱AI是中国领先的大模型研发公司，提供GLM系列大语言模型。".to_string(),
        },
        ProviderConfig {
            id: "baichuan".to_string(),
            name: "百川智能 (Baichuan)".to_string(),
            region: ProviderRegion::China,
            api_endpoint: "https://api.baichuan-ai.com/v1/chat/completions".to_string(),
            models: vec![
                "baichuan-7b".to_string(),
                "baichuan-13b".to_string(),
                "baichuan-turbo".to_string(),
            ],
            pricing: PricingInfo {
                currency: "CNY".to_string(),
                input_price_per_1k: 0.05,
                output_price_per_1k: 0.05,
                free_tier_limit: Some(200000),
            },
            requires_api_key: true,
            description: "百川智能提供开源大模型，支持多模态交互。".to_string(),
        },
        ProviderConfig {
            id: "minimax".to_string(),
            name: "MiniMax".to_string(),
            region: ProviderRegion::China,
            api_endpoint: "https://api.minimax.chat/v1/text/chatcompletion_v2".to_string(),
            models: vec![
                "abab5.5-chat".to_string(),
                "abab5.5-chat-pro".to_string(),
                "abab6-chat".to_string(),
                "abab6-chat-pro".to_string(),
            ],
            pricing: PricingInfo {
                currency: "CNY".to_string(),
                input_price_per_1k: 0.015,
                output_price_per_1k: 0.015,
                free_tier_limit: Some(1000000),
            },
            requires_api_key: true,
            description: "MiniMax提供高性能大语言模型API服务。".to_string(),
        },
        ProviderConfig {
            id: "deepseek".to_string(),
            name: "深度求索 (DeepSeek)".to_string(),
            region: ProviderRegion::China,
            api_endpoint: "https://api.deepseek.com/chat/completions".to_string(),
            models: vec![
                "deepseek-chat".to_string(),
                "deepseek-coder".to_string(),
            ],
            pricing: PricingInfo {
                currency: "CNY".to_string(),
                input_price_per_1k: 0.001,
                output_price_per_1k: 0.002,
                free_tier_limit: Some(5000000),
            },
            requires_api_key: true,
            description: "深度求索提供开源大语言模型，支持代码生成。".to_string(),
        },
        ProviderConfig {
            id: "moonshot".to_string(),
            name: "月之暗面 (Moonshot AI)".to_string(),
            region: ProviderRegion::China,
            api_endpoint: "https://api.moonshot.cn/v1/chat/completions".to_string(),
            models: vec![
                "moonshot-v1-8k".to_string(),
                "moonshot-v1-32k".to_string(),
                "moonshot-v1-128k".to_string(),
            ],
            pricing: PricingInfo {
                currency: "CNY".to_string(),
                input_price_per_1k: 0.012,
                output_price_per_1k: 0.012,
                free_tier_limit: Some(2000000),
            },
            requires_api_key: true,
            description: "月之暗面提供长上下文大语言模型。".to_string(),
        },
        ProviderConfig {
            id: "tongyi".to_string(),
            name: "通义千问 (Tongyi Qianwen)".to_string(),
            region: ProviderRegion::China,
            api_endpoint: "https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation".to_string(),
            models: vec![
                "qwen-turbo".to_string(),
                "qwen-plus".to_string(),
                "qwen-max".to_string(),
                "qwen-longcontext".to_string(),
            ],
            pricing: PricingInfo {
                currency: "CNY".to_string(),
                input_price_per_1k: 0.008,
                output_price_per_1k: 0.008,
                free_tier_limit: Some(1000000),
            },
            requires_api_key: true,
            description: "通义千问是阿里云推出的大语言模型。".to_string(),
        },
        ProviderConfig {
            id: "ernie-bot".to_string(),
            name: "文心一言 (ERNIE Bot)".to_string(),
            region: ProviderRegion::China,
            api_endpoint: "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions".to_string(),
            models: vec![
                "ernie-bot-4".to_string(),
                "ernie-bot-turbo".to_string(),
                "ernie-bot-pro".to_string(),
            ],
            pricing: PricingInfo {
                currency: "CNY".to_string(),
                input_price_per_1k: 0.012,
                output_price_per_1k: 0.012,
                free_tier_limit: Some(500000),
            },
            requires_api_key: true,
            description: "文心一言是百度推出的大语言模型。".to_string(),
        },
        
        // 美国AI服务提供商
        ProviderConfig {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            region: ProviderRegion::USA,
            api_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            models: vec![
                "gpt-4".to_string(),
                "gpt-4-turbo".to_string(),
                "gpt-4-turbo-preview".to_string(),
                "gpt-3.5-turbo".to_string(),
                "gpt-3.5-turbo-16k".to_string(),
                "gpt-3.5-turbo-instruct".to_string(),
            ],
            pricing: PricingInfo {
                currency: "USD".to_string(),
                input_price_per_1k: 0.03,
                output_price_per_1k: 0.06,
                free_tier_limit: None,
            },
            requires_api_key: true,
            description: "OpenAI提供GPT系列大语言模型，业界领先。".to_string(),
        },
        ProviderConfig {
            id: "anthropic".to_string(),
            name: "Anthropic Claude".to_string(),
            region: ProviderRegion::USA,
            api_endpoint: "https://api.anthropic.com/v1/messages".to_string(),
            models: vec![
                "claude-3-opus".to_string(),
                "claude-3-sonnet".to_string(),
                "claude-3-haiku".to_string(),
            ],
            pricing: PricingInfo {
                currency: "USD".to_string(),
                input_price_per_1k: 0.015,
                output_price_per_1k: 0.075,
                free_tier_limit: None,
            },
            requires_api_key: true,
            description: "Anthropic提供Claude系列大语言模型，以安全著称。".to_string(),
        },
        ProviderConfig {
            id: "google".to_string(),
            name: "Google Gemini".to_string(),
            region: ProviderRegion::USA,
            api_endpoint: "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent".to_string(),
            models: vec![
                "gemini-pro".to_string(),
                "gemini-pro-vision".to_string(),
                "gemini-1.5-pro".to_string(),
                "gemini-1.5-flash".to_string(),
            ],
            pricing: PricingInfo {
                currency: "USD".to_string(),
                input_price_per_1k: 0.00025,
                output_price_per_1k: 0.0005,
                free_tier_limit: Some(15000000),
            },
            requires_api_key: true,
            description: "Google提供Gemini系列多模态大模型。".to_string(),
        },
        ProviderConfig {
            id: "cohere".to_string(),
            name: "Cohere".to_string(),
            region: ProviderRegion::USA,
            api_endpoint: "https://api.cohere.ai/v1/chat".to_string(),
            models: vec![
                "command".to_string(),
                "command-light".to_string(),
                "command-r".to_string(),
            ],
            pricing: PricingInfo {
                currency: "USD".to_string(),
                input_price_per_1k: 0.015,
                output_price_per_1k: 0.03,
                free_tier_limit: None,
            },
            requires_api_key: true,
            description: "Cohere提供Command系列大语言模型。".to_string(),
        },
        ProviderConfig {
            id: "mistral".to_string(),
            name: "Mistral AI".to_string(),
            region: ProviderRegion::USA,
            api_endpoint: "https://api.mistral.ai/v1/chat/completions".to_string(),
            models: vec![
                "mistral-large".to_string(),
                "mistral-medium".to_string(),
                "mistral-small".to_string(),
                "mixtral-8x7b".to_string(),
                "mixtral-8x22b".to_string(),
            ],
            pricing: PricingInfo {
                currency: "EUR".to_string(),
                input_price_per_1k: 0.003,
                output_price_per_1k: 0.006,
                free_tier_limit: None,
            },
            requires_api_key: true,
            description: "Mistral AI提供开源大语言模型。".to_string(),
        },
        ProviderConfig {
            id: "meta".to_string(),
            name: "Meta Llama".to_string(),
            region: ProviderRegion::Global,
            api_endpoint: "https://api.meta.com/v1/chat/completions".to_string(),
            models: vec![
                "llama-3-70b".to_string(),
                "llama-3-8b".to_string(),
                "llama-2-70b".to_string(),
            ],
            pricing: PricingInfo {
                currency: "USD".to_string(),
                input_price_per_1k: 0.0,
                output_price_per_1k: 0.0,
                free_tier_limit: None,
            },
            requires_api_key: false,
            description: "Meta提供开源Llama系列大语言模型，可免费使用。".to_string(),
        },
        ProviderConfig {
            id: "huggingface".to_string(),
            name: "Hugging Face".to_string(),
            region: ProviderRegion::Global,
            api_endpoint: "https://api-inference.huggingface.co/models".to_string(),
            models: vec![
                "meta-llama-3-70b-instruct".to_string(),
                "mistralai-mixtral-8x7b".to_string(),
            ],
            pricing: PricingInfo {
                currency: "USD".to_string(),
                input_price_per_1k: 0.0001,
                output_price_per_1k: 0.0001,
                free_tier_limit: Some(1000000),
            },
            requires_api_key: true,
            description: "Hugging Face提供开源模型推理API。".to_string(),
        },
    ]
}

pub fn get_provider_by_id(id: &str) -> Option<ProviderConfig> {
    get_predefined_providers().into_iter().find(|p| p.id == id)
}

pub fn get_providers_by_region(region: ProviderRegion) -> Vec<ProviderConfig> {
    get_predefined_providers()
        .into_iter()
        .filter(|p| p.region == region)
        .collect()
}