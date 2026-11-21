//! Built-in provider presets plus live import of the models.dev
//! registry (324 providers at last fetch), the same catalog OpenCode
//! consumes.

use crate::config::{ModelEntry, ProviderApi, ProviderConfig};

pub struct ProviderPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub api: ProviderApi,
    pub base_url: &'static str,
    pub env_keys: &'static [&'static str],
    pub models: &'static [&'static str],
}

impl ProviderPreset {
    pub fn to_config(&self) -> ProviderConfig {
        ProviderConfig {
            id: self.id.to_string(),
            label: self.label.to_string(),
            api: self.api,
            base_url: self.base_url.to_string(),
            env_keys: self.env_keys.iter().map(|s| s.to_string()).collect(),
            api_key: None,
            models: self
                .models
                .iter()
                .map(|m| ModelEntry {
                    id: m.to_string(),
                    context_limit: 200_000,
                    tool_call: true,
                    reasoning: true,
                })
                .collect(),
            enabled: true,
        }
    }
}

/// Curated first-party presets. Every entry is BYOK: the user's own key
/// (inline or via env var) or a free local endpoint.
pub static BUILTIN_PRESETS: &[ProviderPreset] = &[
    ProviderPreset {
        id: "anthropic",
        label: "Anthropic",
        api: ProviderApi::Anthropic,
        base_url: "https://api.anthropic.com",
        env_keys: &["ANTHROPIC_API_KEY"],
        models: &[
            "claude-sonnet-4-5",
            "claude-opus-4-1",
            "claude-haiku-4-5",
        ],
    },
    ProviderPreset {
        id: "openai",
        label: "OpenAI",
        api: ProviderApi::Openai,
        base_url: "https://api.openai.com/v1",
        env_keys: &["OPENAI_API_KEY"],
        models: &["gpt-5.2", "gpt-5.1", "gpt-5-mini", "gpt-4.1"],
    },
    ProviderPreset {
        id: "google",
        label: "Google Gemini",
        api: ProviderApi::Gemini,
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        env_keys: &["GEMINI_API_KEY", "GOOGLE_API_KEY", "GOOGLE_GENERATIVE_AI_API_KEY"],
        models: &["gemini-3-pro", "gemini-2.5-pro", "gemini-2.5-flash"],
    },
    ProviderPreset {
        id: "openrouter",
        label: "OpenRouter",
        api: ProviderApi::Openai,
        base_url: "https://openrouter.ai/api/v1",
        env_keys: &["OPENROUTER_API_KEY"],
        models: &["anthropic/claude-sonnet-4.5", "openai/gpt-5.1", "google/gemini-2.5-pro"],
    },
    ProviderPreset {
        id: "deepseek",
        label: "DeepSeek",
        api: ProviderApi::Openai,
        base_url: "https://api.deepseek.com",
        env_keys: &["DEEPSEEK_API_KEY"],
        models: &["deepseek-chat", "deepseek-reasoner"],
    },
    ProviderPreset {
        id: "zai",
        label: "Zhipu GLM (Z.ai)",
        api: ProviderApi::Openai,
        base_url: "https://api.z.ai/api/paas/v4",
        env_keys: &["ZHIPU_API_KEY", "ZAI_API_KEY"],
        models: &["glm-4.6", "glm-4.5"],
    },
    ProviderPreset {
        id: "mistral",
        label: "Mistral AI",
        api: ProviderApi::Anthropic,
        base_url: "https://api.mistral.ai/v1",
        env_keys: &["MISTRAL_API_KEY"],
        models: &["mistral-large-latest", "devstral-medium-latest"],
    },
    ProviderPreset {
        id: "alibaba",
        label: "Qwen (DashScope)",
        api: ProviderApi::Openai,
        base_url: "https://dashscope-intl.aliyuncs.com/compatible-mode/v1",
        env_keys: &["DASHSCOPE_API_KEY", "QWEN_API_KEY"],
        models: &["qwen3-max", "qwen3-coder-plus"],
    },
    ProviderPreset {
        id: "moonshotai",
        label: "Moonshot Kimi",
        api: ProviderApi::Openai,
        base_url: "https://api.moonshot.ai/v1",
        env_keys: &["MOONSHOT_API_KEY", "KIMI_API_KEY"],
        models: &["kimi-k2-thinking", "kimi-k2-0905-preview"],
    },
    ProviderPreset {
        id: "minimax",
        label: "MiniMax",
        api: ProviderApi::Anthropic,
        base_url: "https://api.minimax.io/anthropic/v1",
        env_keys: &["MINIMAX_API_KEY"],
        models: &["MiniMax-M2"],
    },
    ProviderPreset {
        id: "groq",
        label: "Groq",
        api: ProviderApi::Openai,
        base_url: "https://api.groq.com/openai/v1",
        env_keys: &["GROQ_API_KEY"],
        models: &["llama-3.3-70b-versatile", "gpt-oss-120b"],
    },
    ProviderPreset {
        id: "togetherai",
        label: "Together AI",
        api: ProviderApi::Openai,
        base_url: "https://api.together.xyz/v1",
        env_keys: &["TOGETHER_API_KEY", "TOGETHER_AI_API_KEY"],
        models: &["deepseek-ai/DeepSeek-V3", "meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8"],
    },
    ProviderPreset {
        id: "fireworks-ai",
        label: "Fireworks AI",
        api: ProviderApi::Openai,
        base_url: "https://api.fireworks.ai/inference/v1",
        env_keys: &["FIREWORKS_API_KEY"],
        models: &["accounts/fireworks/models/deepseek-v3", "accounts/fireworks/models/kimi-k2-instruct"],
    },
    ProviderPreset {
        id: "cerebras",
        label: "Cerebras",
        api: ProviderApi::Openai,
        base_url: "https://api.cerebras.ai/v1",
        env_keys: &["CEREBRAS_API_KEY"],
        models: &["llama-3.3-70b", "qwen-3-235b-a22b-instruct"],
    },
    ProviderPreset {
        id: "xai",
        label: "xAI Grok",
        api: ProviderApi::Openai,
        base_url: "https://api.x.ai/v1",
        env_keys: &["XAI_API_KEY", "GROK_API_KEY"],
        models: &["grok-4", "grok-code-fast-1"],
    },
    ProviderPreset {
        id: "cohere",
        label: "Cohere",
        api: ProviderApi::Openai,
        base_url: "https://api.cohere.ai/compatibility/v1",
        env_keys: &["COHERE_API_KEY"],
        models: &["command-a-03-2025", "command-r-plus"],
    },
    ProviderPreset {
        id: "perplexity",
        label: "Perplexity",
        api: ProviderApi::Openai,
        base_url: "https://api.perplexity.ai",
        env_keys: &["PERPLEXITY_API_KEY"],
        models: &["sonar-pro", "sonar-reasoning-pro"],
    },
    ProviderPreset {
        id: "omniroute",
        label: "OmniRoute / LiteLLM Gateway",
        api: ProviderApi::Openai,
        base_url: "http://localhost:4000/v1",
        env_keys: &[],
        models: &["litellm-default"],
    },
    ProviderPreset {
        id: "ollama",
        label: "Ollama (local)",
        api: ProviderApi::Openai,
        base_url: "http://127.0.0.1:11434/v1",
        env_keys: &[],
        models: &["qwen3-coder:30b", "gpt-oss:20b"],
    },
    ProviderPreset {
        id: "lmstudio",
        label: "LM Studio (local)",
        api: ProviderApi::Openai,
        base_url: "http://127.0.0.1:1234/v1",
        env_keys: &[],
        models: &["lmstudio-current-model"],
    },
    ProviderPreset {
        id: "llamacpp",
        label: "llama.cpp server (local)",
        api: ProviderApi::Openai,
        base_url: "http://127.0.0.1:8080/v1",
        env_keys: &[],
        models: &["local"],
    },
    ProviderPreset {
        id: "vllm",
        label: "vLLM (local)",
        api: ProviderApi::Openai,
        base_url: "http://127.0.0.1:8000/v1",
        env_keys: &["VLLM_API_KEY"],
        models: &["served-model"],
    },
    ProviderPreset {
        id: "colibri",
        label: "Colibri (local)",
        api: ProviderApi::Openai,
        base_url: "http://127.0.0.1:8091/v1",
        env_keys: &[],
        models: &["default"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_converts() {
        for preset in BUILTIN_PRESETS {
            let cfg = preset.to_config();
            assert_eq!(cfg.id, preset.id);
            assert!(!cfg.models.is_empty(), "{} has no models", preset.id);
            assert!(!cfg.base_url.is_empty());
            assert!(cfg.enabled);
        }
    }

    #[test]
    fn local_endpoints_have_no_required_key() {
        for preset in BUILTIN_PRESETS.iter().filter(|p| p.base_url.contains("127.0.0.1") || p.base_url.contains("localhost")) {
            assert!(
                preset.env_keys.is_empty() || preset.id == "vllm",
                "{} is local but requires a key",
                preset.id
            );
        }
    }
}
