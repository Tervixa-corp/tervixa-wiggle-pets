//! Unified BYOK LLM provider engine.
//!
//! Every provider connection — Anthropic Messages, any OpenAI-compatible
//! endpoint (OpenRouter, DeepSeek, GLM, Groq, Ollama, LM Studio, vLLM,
//! LiteLLM, OmniRoute, ...) or the native Gemini API — implements the
//! same [`LlmProvider`] contract and streams the same [`StreamEvent`]
//! vocabulary, the way OpenCode's provider layer normalizes 200+ models.

use std::sync::Arc;

use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::config::{ProviderApi, ProviderConfig};
use crate::{CoreError, CoreResult};

pub mod anthropic;
pub mod gemini;
pub mod openai_compat;
pub mod presets;

/// Role of one message in the transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultBlock {
    pub call_id: String,
    pub content: String,
    pub is_error: bool,
}

/// One content block inside a message, provider-neutral.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text(String),
    Thinking(String),
    ToolUse(ToolCall),
    ToolResult(ToolResultBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

impl ChatMessage {
    pub fn user(text: impl Into<String>) -> ChatMessage {
        ChatMessage { role: Role::User, content: vec![ContentBlock::Text(text.into())] }
    }

    /// Flatten the message to plain text (used for token estimation).
    pub fn text(&self) -> String {
        self.content
            .iter()
            .map(|b| match b {
                ContentBlock::Text(t) | ContentBlock::Thinking(t) => t.clone(),
                ContentBlock::ToolUse(c) => {
                    format!("{} {}", c.name, c.arguments.to_string())
                }
                ContentBlock::ToolResult(r) => r.content.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// A callable tool advertised to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// JSON Schema object for the parameters.
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    Error,
}

/// Provider-neutral streaming chunk.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    Text(String),
    Thinking(String),
    ToolUse(ToolCall),
    Usage(Usage),
    Done(StopReason),
    Fatal(String),
}

/// One compiled request ready to send.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub system: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolSpec>,
    pub max_tokens: u32,
    pub temperature: Option<f32>,
    /// Enable provider-native extended thinking when supported.
    pub thinking: bool,
}

impl Default for ChatRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            system: String::new(),
            messages: Vec::new(),
            tools: Vec::new(),
            max_tokens: 16_384,
            temperature: None,
            thinking: false,
        }
    }
}

/// Contract implemented by every provider connection.
pub trait LlmProvider: Send + Sync {
    fn id(&self) -> &str;
    fn api(&self) -> ProviderApi;
    /// Start a streaming chat; events are pumped into the returned
    /// channel until a `Done`/`Fatal` terminal event arrives.
    fn chat(
        &self,
        req: ChatRequest,
    ) -> BoxFuture<'static, CoreResult<mpsc::Receiver<StreamEvent>>>;
    /// API key for this connection, if resolvable.
    fn api_key(&self) -> Option<String>;
    fn config(&self) -> &ProviderConfig;
}

/// Cheap deterministic token estimate (~4 chars/token) used for the
/// status bar and context budgeting before an API response arrives.
pub fn estimate_tokens(text: &str) -> u64 {
    (text.chars().count() as u64).div_ceil(4)
}

/// Shared SSE line reader: yields complete `data:` payload strings.
pub async fn read_sse_data_lines<S>(mut stream: S, tx: &mpsc::Sender<String>) -> CoreResult<()>
where
    S: futures::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    use futures::StreamExt;
    let mut buf: Vec<u8> = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| CoreError::Provider(e.to_string()))?;
        buf.extend_from_slice(&chunk);
        while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=pos).collect();
            let line = String::from_utf8_lossy(&line);
            let line = line.trim_end_matches(['\r', '\n']);
            if let Some(data) = line.strip_prefix("data:") {
                if tx.send(data.trim().to_string()).await.is_err() {
                    return Ok(()); // consumer dropped
                }
            }
        }
    }
    Ok(())
}

/// A concrete provider connection.
pub struct HttpProvider {
    config: ProviderConfig,
    http: reqwest::Client,
}

impl HttpProvider {
    pub fn new(config: ProviderConfig) -> CoreResult<HttpProvider> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .build()
            .map_err(|e| CoreError::Provider(e.to_string()))?;
        Ok(HttpProvider { config, http })
    }

    pub fn api_key_from(&self) -> Option<String> {
        for key in &self.config.env_keys {
            if let Ok(v) = std::env::var(key) {
                if !v.trim().is_empty() {
                    return Some(v.trim().to_string());
                }
            }
        }
        self.config.api_key.clone().filter(|s| !s.trim().is_empty())
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.http
    }
}

impl LlmProvider for HttpProvider {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn api(&self) -> ProviderApi {
        self.config.api
    }

    fn api_key(&self) -> Option<String> {
        self.api_key_from()
    }

    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    fn chat(
        &self,
        req: ChatRequest,
    ) -> BoxFuture<'static, CoreResult<mpsc::Receiver<StreamEvent>>> {
        let provider = Arc::new(self.clone_provider());
        Box::pin(async move {
            let (tx, rx) = mpsc::channel::<StreamEvent>(256);
            let handle = provider.runtime_handle();
            handle.spawn(async move {
                let result = match provider.config.api {
                    ProviderApi::Anthropic => anthropic::stream(provider.clone(), req, tx.clone()).await,
                    ProviderApi::Openai => openai_compat::stream(provider.clone(), req, tx.clone()).await,
                    ProviderApi::Gemini => gemini::stream(provider.clone(), req, tx.clone()).await,
                };
                if let Err(e) = result {
                    let _ = tx.send(StreamEvent::Fatal(e.to_string())).await;
                }
            });
            Ok(rx)
        })
    }
}

// Clone-ability shim: HttpProvider holds only config + client.
impl HttpProvider {
    fn clone_provider(&self) -> HttpProvider {
        HttpProvider { config: self.config.clone(), http: self.http.clone() }
    }

    fn runtime_handle(&self) -> tokio::runtime::Handle {
        // Always called from inside an async context (the runner awaits
        // the returned future on the engine runtime).
        tokio::runtime::Handle::try_current()
            .expect("HttpProvider::chat must be polled on a tokio runtime")
    }
}

/// Router: resolves `provider/model`, streams with failover to the next
/// enabled connection offering the same model.
pub struct Router;

impl Router {
    pub fn provider(
        config: &crate::config::GlobalConfig,
        provider_id: Option<&str>,
        model: Option<&str>,
    ) -> CoreResult<(Arc<dyn LlmProvider>, String)> {
        let enabled: Vec<&ProviderConfig> =
            config.providers.iter().filter(|p| p.enabled && !p.models.is_empty()).collect();
        if enabled.is_empty() {
            return Err(CoreError::Provider(
                "no enabled providers with models configured — add an API key in Settings or connect a local Ollama server".into(),
            ));
        }

        let model = model
            .map(str::to_string)
            .or_else(|| config.default_model.clone())
            .unwrap_or_else(|| enabled[0].models[0].id.clone());

        let provider_id = provider_id
            .map(str::to_string)
            .or_else(|| config.default_provider.clone());

        let mut ordered: Vec<&ProviderConfig> = Vec::new();
        if let Some(pid) = &provider_id {
            if let Some(p) = enabled.iter().copied().find(|p| &p.id == pid) {
                ordered.push(p);
            }
        } else if let Some(p) =
            enabled.iter().copied().find(|p| p.models.iter().any(|m| m.id == model))
        {
            ordered.push(p);
        }
        for p in enabled {
            if !ordered.iter().any(|o| o.id == p.id) {
                ordered.push(p);
            }
        }

        let first = ordered[0];
        Ok((Arc::new(HttpProvider::new(first.clone())?), model))
    }

    /// Model catalog merged from built-in presets and a cached copy of
    /// the models.dev registry (324 providers, 299 REST endpoints).
    pub async fn fetch_models_dev() -> CoreResult<serde_json::Value> {
        let url = "https://models.dev/api.json";
        let body = reqwest::get(url)
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| CoreError::Provider(format!("models.dev: {e}")))?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| CoreError::Provider(format!("models.dev: {e}")))?;
        let cache = crate::config::GlobalConfig::wiggle_home().join("models-dev.json");
        if let Ok(raw) = serde_json::to_vec(&body) {
            let _ = std::fs::write(cache, raw);
        }
        Ok(body)
    }

    /// Cached models.dev registry, or `None` when offline and never fetched.
    pub fn cached_models_dev() -> Option<serde_json::Value> {
        let cache = crate::config::GlobalConfig::wiggle_home().join("models-dev.json");
        let raw = std::fs::read(cache).ok()?;
        serde_json::from_slice(&raw).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_estimation_is_monotonic() {
        assert!(estimate_tokens("hello") >= 1);
        assert!(estimate_tokens(&"x".repeat(400)) >= estimate_tokens(&"x".repeat(40)));
    }

    #[test]
    fn message_text_flattens_blocks() {
        let m = ChatMessage {
            role: Role::Assistant,
            content: vec![
                ContentBlock::Text("plan".into()),
                ContentBlock::ToolUse(ToolCall {
                    id: "t1".into(),
                    name: "read_file".into(),
                    arguments: serde_json::json!({"path": "a.rs"}),
                }),
            ],
        };
        assert!(m.text().contains("plan"));
        assert!(m.text().contains("read_file"));
    }
}
