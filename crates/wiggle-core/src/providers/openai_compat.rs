//! OpenAI-compatible streaming adapter. Covers OpenAI, OpenRouter,
//! DeepSeek, Zhipu GLM, Moonshot Kimi, Groq, Together, Fireworks,
//! Cerebras, xAI, Mistral, Qwen/DashScope, Perplexity, Cohere and every
//! self-hosted OpenAI dialect (Ollama, LM Studio, llama.cpp, vLLM,
//! LiteLLM, OmniRoute).

use tokio::sync::mpsc;

use super::{
    read_sse_data_lines, ChatMessage, ChatRequest, ContentBlock, HttpProvider, LlmProvider, Role,
    StopReason, StreamEvent, ToolCall,
};
use crate::{CoreError, CoreResult};

pub async fn stream(
    provider: std::sync::Arc<HttpProvider>,
    req: ChatRequest,
    tx: mpsc::Sender<StreamEvent>,
) -> CoreResult<()> {
    let base = provider.config.base_url.trim_end_matches('/');
    let url = format!("{base}/chat/completions");

    let mut messages = Vec::with_capacity(req.messages.len() + 1);
    if !req.system.is_empty() {
        messages.push(serde_json::json!({ "role": "system", "content": req.system }));
    }
    for msg in &req.messages {
        push_openai_message(&mut messages, msg);
    }

    let mut body = serde_json::json!({
        "model": req.model,
        "messages": messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });
    if !req.tools.is_empty() {
        body["tools"] = serde_json::json!(req.tools.iter().map(|t| serde_json::json!({
            "type": "function",
            "function": {
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
            }
        })).collect::<Vec<_>>());
    }
    if let Some(t) = req.temperature {
        body["temperature"] = serde_json::json!(t);
    }

    let mut request = provider.http().post(&url).header("content-type", "application/json");
    // Local servers ignore auth; sending a bogus bearer to them is fine,
    // but Ollama rejects empty headers, so only set it when present.
    if let Some(key) = provider.api_key() {
        request = request.bearer_auth(&key);
    }
    let response = request
        .json(&body)
        .send()
        .await
        .map_err(|e| CoreError::Provider(format!("{}: {e}", provider.id())))?;
    let response = response.error_for_status().map_err(|e| {
        CoreError::Provider(format!(
            "{} {}: {}",
            provider.id(),
            e.status().map(|s| s.to_string()).unwrap_or_default(),
            e
        ))
    })?;

    let (sse_tx, mut sse_rx) = mpsc::channel::<String>(64);
    let byte_stream = response.bytes_stream();
    tokio::spawn(async move {
        let _ = read_sse_data_lines(byte_stream, &sse_tx).await;
    });

    // Accumulate streamed tool calls by index (arguments arrive in deltas).
    let mut tools_by_index: std::collections::HashMap<u64, (String, String, String)> =
        std::collections::HashMap::new();

    while let Some(data) = sse_rx.recv().await {
        if data.trim() == "[DONE]" {
            break;
        }
        let event: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if let Some(err) = event["error"]["message"].as_str() {
            let _ = tx.send(StreamEvent::Fatal(err.to_string())).await;
            return Ok(());
        }
        for choice in event["choices"].as_array().into_iter().flatten() {
            let delta = &choice["delta"];
            if let Some(text) = delta["content"].as_str() {
                if !text.is_empty() {
                    let _ = tx.send(StreamEvent::Text(text.to_string())).await;
                }
            }
            if let Some(text) = delta["reasoning_content"].as_str() {
                if !text.is_empty() {
                    let _ = tx.send(StreamEvent::Thinking(text.to_string())).await;
                }
            }
            if let Some(calls) = delta["tool_calls"].as_array() {
                for call in calls {
                    let idx = call["index"].as_u64().unwrap_or(0);
                    let entry = tools_by_index.entry(idx).or_insert_with(|| {
                        (
                            call["id"].as_str().unwrap_or("").to_string(),
                            call["function"]["name"].as_str().unwrap_or("").to_string(),
                            String::new(),
                        )
                    });
                    if entry.0.is_empty() {
                        if let Some(id) = call["id"].as_str() {
                            entry.0 = id.to_string();
                        }
                    }
                    if entry.1.is_empty() {
                        if let Some(name) = call["function"]["name"].as_str() {
                            entry.1 = name.to_string();
                        }
                    }
                    if let Some(args) = call["function"]["arguments"].as_str() {
                        entry.2.push_str(args);
                    }
                }
            }
            if choice["finish_reason"].is_string() {
                for (_, (id, name, args)) in tools_by_index.drain() {
                    let arguments: serde_json::Value = serde_json::from_str(&args)
                        .unwrap_or(serde_json::json!({}));
                    let _ = tx.send(StreamEvent::ToolUse(ToolCall { id, name, arguments })).await;
                }
                let stop = match choice["finish_reason"].as_str() {
                    Some("tool_calls") | Some("function_call") => StopReason::ToolUse,
                    Some("length") => StopReason::MaxTokens,
                    _ => StopReason::EndTurn,
                };
                let _ = tx.send(StreamEvent::Done(stop)).await;
            }
        }
        if let Some(usage) = event.get("usage").filter(|u| u.is_object()) {
            let _ = tx
                .send(StreamEvent::Usage(super::Usage {
                    input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0),
                    output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0),
                }))
                .await;
        }
    }
    // Emit any tool calls that never saw a finish_reason (safety net).
    for (_, (id, name, args)) in tools_by_index.drain() {
        if !name.is_empty() {
            let arguments: serde_json::Value = serde_json::from_str(&args).unwrap_or(serde_json::json!({}));
            let _ = tx.send(StreamEvent::ToolUse(ToolCall { id, name, arguments })).await;
        }
    }
    Ok(())
}

fn push_openai_message(out: &mut Vec<serde_json::Value>, msg: &ChatMessage) {
    let role = match msg.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };
    // Split mixed assistant blocks: tool_calls ride in one object,
    // text/thinking become separate content messages.
    let mut text = String::new();
    let mut tool_calls = Vec::new();
    for block in &msg.content {
        match block {
            ContentBlock::Text(t) | ContentBlock::Thinking(t) => text.push_str(t),
            ContentBlock::ToolUse(c) => tool_calls.push(serde_json::json!({
                "id": c.id,
                "type": "function",
                "function": {
                    "name": c.name,
                    "arguments": c.arguments.to_string(),
                }
            })),
            ContentBlock::ToolResult(r) => {
                out.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": r.call_id,
                    "content": r.content,
                }));
            }
        }
    }
    if tool_calls.is_empty() {
        out.push(serde_json::json!({ "role": role, "content": text }));
    } else {
        let mut obj = serde_json::json!({ "role": role, "content": text, "tool_calls": tool_calls });
        if text.is_empty() {
            obj["content"] = serde_json::Value::Null;
        }
        out.push(obj);
    }
}
