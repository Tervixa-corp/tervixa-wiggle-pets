//! Native Anthropic Messages API streaming adapter with prompt caching
//! and extended thinking support.

use tokio::sync::mpsc;

use super::{
    read_sse_data_lines, ChatRequest, ContentBlock, ChatMessage, HttpProvider, LlmProvider, Role,
    StopReason, StreamEvent, ToolCall,
};
use crate::{CoreError, CoreResult};

pub async fn stream(
    provider: std::sync::Arc<HttpProvider>,
    req: ChatRequest,
    tx: mpsc::Sender<StreamEvent>,
) -> CoreResult<()> {
    let key = provider
        .api_key()
        .ok_or_else(|| CoreError::Provider(format!("provider '{}': no API key configured (set one of {:?} or add an inline key)", provider.id(), provider.config.env_keys)))?;

    let base = provider.config.base_url.trim_end_matches('/');
    let url = format!("{base}/v1/messages");

    // System blocks with prompt caching enabled.
    let mut system = vec![serde_json::json!({
        "type": "text",
        "text": req.system,
    })];
    if req.system.chars().count() > 2048 {
        system[0]["cache_control"] = serde_json::json!({ "type": "ephemeral" });
    }

    let messages = to_anthropic_messages(&req.messages);
    let tools: Vec<serde_json::Value> = req
        .tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.parameters,
            })
        })
        .collect();

    let mut body = serde_json::json!({
        "model": req.model,
        "max_tokens": req.max_tokens,
        "system": system,
        "messages": messages,
        "stream": true,
    });
    if !tools.is_empty() {
        body["tools"] = serde_json::json!(tools);
    }
    if let Some(t) = req.temperature {
        body["temperature"] = serde_json::json!(t);
    }
    if req.thinking {
        body["thinking"] = serde_json::json!({ "type": "enabled", "budget_tokens": 8192 });
    }

    let response = provider
        .http()
        .post(&url)
        .header("x-api-key", &key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| CoreError::Provider(format!("anthropic: {e}")))?;

    let response = response
        .error_for_status()
        .map_err(|e| CoreError::Provider(format!("anthropic {}: {}", e.status().map(|s| s.to_string()).unwrap_or_default(), e.to_string())))?;

    let (sse_tx, mut sse_rx) = mpsc::channel::<String>(64);
    let byte_stream = response.bytes_stream();
    tokio::spawn(async move {
        let _ = read_sse_data_lines(byte_stream, &sse_tx).await;
    });

    // Per-index accumulators for streaming tool_use arguments.
    let mut partial_tools: std::collections::HashMap<u64, (String, String, String)> =
        std::collections::HashMap::new();

    while let Some(data) = sse_rx.recv().await {
        if data == "[DONE]" {
            break;
        }
        let event: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(_) => continue,
        };
        match event["type"].as_str() {
            Some("message_start") => {
                let input = event["message"]["usage"]["input_tokens"].as_u64().unwrap_or(0);
                let _ = tx.send(StreamEvent::Usage(super::Usage { input_tokens: input, output_tokens: 0 })).await;
            }
            Some("content_block_start") => {
                let block = &event["content_block"];
                if block["type"] == "tool_use" {
                    let id = block["id"].as_str().unwrap_or_default().to_string();
                    let name = block["name"].as_str().unwrap_or_default().to_string();
                    partial_tools
                        .insert(event["index"].as_u64().unwrap_or(0), (id, name, String::new()));
                }
            }
            Some("content_block_delta") => {
                let delta = &event["delta"];
                match delta["type"].as_str() {
                    Some("text_delta") => {
                        if let Some(t) = delta["text"].as_str() {
                            let _ = tx.send(StreamEvent::Text(t.to_string())).await;
                        }
                    }
                    Some("thinking_delta") => {
                        if let Some(t) = delta["thinking"].as_str() {
                            let _ = tx.send(StreamEvent::Thinking(t.to_string())).await;
                        }
                    }
                    Some("input_json_delta") => {
                        if let Some(p) = delta["partial_json"].as_str() {
                            if let Some((_, _, acc)) = partial_tools.get_mut(&event["index"].as_u64().unwrap_or(0)) {
                                acc.push_str(p);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Some("content_block_stop") => {
                if let Some((id, name, args)) =
                    partial_tools.remove(&event["index"].as_u64().unwrap_or(0))
                {
                    let arguments: serde_json::Value =
                        if args.trim().is_empty() { serde_json::json!({}) } else { serde_json::from_str(&args).unwrap_or(serde_json::json!({})) };
                    let _ = tx.send(StreamEvent::ToolUse(ToolCall { id, name, arguments })).await;
                }
            }
            Some("message_delta") => {
                if let Some(out) = event["usage"]["output_tokens"].as_u64() {
                    let _ = tx.send(StreamEvent::Usage(super::Usage { input_tokens: 0, output_tokens: out })).await;
                }
                let stop = match event["delta"]["stop_reason"].as_str() {
                    Some("tool_use") => StopReason::ToolUse,
                    Some("max_tokens") => StopReason::MaxTokens,
                    Some("end_turn") | Some("stop_sequence") | None => StopReason::EndTurn,
                    _ => StopReason::EndTurn,
                };
                let _ = tx.send(StreamEvent::Done(stop)).await;
            }
            Some("error") => {
                let msg = event["error"]["message"].as_str().unwrap_or("unknown anthropic error");
                let _ = tx.send(StreamEvent::Fatal(msg.to_string())).await;
                return Ok(());
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn to_anthropic_messages(messages: &[ChatMessage]) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for msg in messages {
        let role = match msg.role {
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        let mut content = Vec::new();
        for block in &msg.content {
            match block {
                ContentBlock::Text(t) => content.push(serde_json::json!({ "type": "text", "text": t })),
                ContentBlock::Thinking(t) => content.push(serde_json::json!({ "type": "thinking", "thinking": t })),
                ContentBlock::ToolUse(c) => content.push(serde_json::json!({
                    "type": "tool_use",
                    "id": c.id,
                    "name": c.name,
                    "input": c.arguments,
                })),
                ContentBlock::ToolResult(r) => content.push(serde_json::json!({
                    "type": "tool_result",
                    "tool_use_id": r.call_id,
                    "content": r.content,
                    "is_error": r.is_error,
                })),
            }
        }
        if content.is_empty() {
            content.push(serde_json::json!({ "type": "text", "text": "(empty)" }));
        }
        out.push(serde_json::json!({ "role": role, "content": content }));
    }
    out
}
