//! Lightweight AI chat router — invoked ONLY for chat replies.
//! Local-first (Ollama, LM Studio, llama.cpp server, any OpenAI-compatible
//! endpoint) with optional BYOK cloud providers. Deterministic skills never
//! touch this module.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String, // ollama | lm-studio | openai-compat | anthropic | openrouter | ...
    pub params: String,   // "0.5B", "1.5B", ...
    pub ram_hint: String,
}

/// Curated lightweight local catalog (first-run selector).
pub fn lightweight_catalog() -> Vec<ModelInfo> {
    [
        ("qwen2.5:0.5b", "Qwen 2.5 0.5B Instruct", "0.5B", "< 600 MB"),
        ("qwen2.5:1.5b", "Qwen 2.5 1.5B Instruct", "1.5B", "~1 GB"),
        ("qwen3:1.7b", "Qwen 3 1.7B", "1.7B", "~1.2 GB"),
        ("llama3.2:1b", "Llama 3.2 1B Instruct", "1B", "~800 MB"),
        ("llama3.2:3b", "Llama 3.2 3B Instruct", "3B", "~2 GB"),
        ("smollm2:360m", "SmolLM2 360M", "360M", "< 400 MB"),
        ("smollm2:1.7b", "SmolLM2 1.7B", "1.7B", "~1.1 GB"),
        ("gemma3:1b", "Gemma 3 1B", "1B", "~800 MB"),
        ("phi4-mini", "Phi-4 Mini", "3.8B", "~2.5 GB"),
    ]
    .iter()
    .map(|(id, name, params, ram)| ModelInfo {
        id: id.to_string(),
        name: name.to_string(),
        provider: "ollama".into(),
        params: params.to_string(),
        ram_hint: ram.to_string(),
    })
    .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// "local" | "cloud" | "off" (100% no-AI deterministic mode)
    pub mode: String,
    pub local_base_url: String,
    pub local_model: String,
    pub cloud_provider: String,
    pub cloud_model: String,
    /// API key is read from env or this field (never logged).
    pub api_key_env: String,
}

impl Default for RouterConfig {
    fn default() -> Self {
        RouterConfig {
            mode: "local".into(),
            local_base_url: "http://localhost:11434".into(),
            local_model: "qwen2.5:1.5b".into(),
            cloud_provider: "openai".into(),
            cloud_model: "gpt-4o-mini".into(),
            api_key_env: "OPENAI_API_KEY".into(),
        }
    }
}

/// Detect well-known local inference servers.
pub fn detect_local() -> Vec<(String, String)> {
    let probes = [
        ("http://localhost:11434", "Ollama"),
        ("http://localhost:1234", "LM Studio"),
        ("http://localhost:8080", "llama.cpp server"),
        ("http://localhost:8000", "vLLM"),
    ];
    probes
        .into_iter()
        .filter(|(url, _)| {
            ureq::get(&format!("{url}/v1/models"))
                .timeout(std::time::Duration::from_millis(600))
                .call()
                .is_ok()
        })
        .map(|(u, n)| (u.to_string(), n.to_string()))
        .collect()
}

const PET_SYSTEM: &str = "You are the Tervixa Wiggle Pet — a tiny, friendly desktop companion watching over the user's AI coding agents. Reply in 1-3 short sentences, warm and playful but never verbose. You may mention agent states, reminders and hydration only if the user asks.";

pub fn chat(cfg: &RouterConfig, user_msg: &str, context: &str) -> Result<String, String> {
    if cfg.mode == "off" {
        return Err("AI chat is off (deterministic mode)".into());
    }
    let system = format!("{PET_SYSTEM}{context}");
    match cfg.mode.as_str() {
        "local" => chat_openai_compat(&cfg.local_base_url, &cfg.local_model, &system, user_msg, None),
        "cloud" => {
            let key = std::env::var(&cfg.api_key_env).unwrap_or_default();
            if cfg.cloud_provider == "anthropic" {
                chat_anthropic(&key, &cfg.cloud_model, &system, user_msg)
            } else {
                // openai / openrouter / deepseek / glm / kimi / groq are all
                // OpenAI-compatible; base URL comes from env override.
                let base = std::env::var("TERVIXA_AI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".into());
                chat_openai_compat(&base, &cfg.cloud_model, &system, user_msg, Some(key))
            }
        }
        other => Err(format!("unknown router mode: {other}")),
    }
}

fn chat_openai_compat(
    base: &str,
    model: &str,
    system: &str,
    user_msg: &str,
    key: Option<String>,
) -> Result<String, String> {
    let mut req = ureq::post(&format!("{base}/v1/chat/completions").replace("/v1/v1", "/v1"))
        .timeout(std::time::Duration::from_secs(60))
        .set("Content-Type", "application/json");
    if let Some(k) = key {
        if !k.is_empty() {
            req = req.set("Authorization", &format!("Bearer {k}"));
        }
    }
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user_msg }
        ],
        "max_tokens": 300
    });
    let resp = req
        .send_json(body)
        .map_err(|e| format!("local AI request failed: {e}"))?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    v["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| format!("unexpected AI response: {v}"))
}

fn chat_anthropic(key: &str, model: &str, system: &str, user_msg: &str) -> Result<String, String> {
    if key.is_empty() {
        return Err("missing ANTHROPIC_API_KEY".into());
    }
    let resp = ureq::post("https://api.anthropic.com/v1/messages")
        .timeout(std::time::Duration::from_secs(60))
        .set("x-api-key", key)
        .set("anthropic-version", "2023-06-01")
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "model": model,
            "max_tokens": 300,
            "system": system,
            "messages": [{ "role": "user", "content": user_msg }]
        }))
        .map_err(|e| format!("anthropic request failed: {e}"))?;
    let v: serde_json::Value = resp.into_json().map_err(|e| e.to_string())?;
    v["content"][0]["text"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| format!("unexpected anthropic response: {v}"))
}
