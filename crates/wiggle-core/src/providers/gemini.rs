//! Native Google Gemini API streaming adapter (v1beta, SSE mode).

use tokio::sync::mpsc;

use super::{
    read_sse_data_lines, ChatRequest, ContentBlock, HttpProvider, LlmProvider, Role,
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
        .ok_or_else(|| CoreError::Provider(format!("provider '{}': no API key configured", provider.id())))?;

    let base = provider.config.base_url.trim_end_matches('/');
    let url = format!("{base}/models/{}:streamGenerateContent?alt=sse", req.model);

    let mut contents = Vec::with_capacity(req.messages.len());
    for msg in &req.messages {
        let role = match msg.role {
            Role::User => "user",
            Role::Assistant => "model",
        };
        let mut parts = Vec::new();
        for block in &msg.content {
            match block {
                ContentBlock::Text(t) | ContentBlock::Thinking(t) => {
                    parts.push(serde_json::json!({ "text": t }))
                }
                ContentBlock::ToolUse(c) => parts.push(serde_json::json!({
                    "functionCall": { "name": c.name, "args": c.arguments }
                })),
                ContentBlock::ToolResult(r) => {
                    // Gemini matches function responses by name.
                    parts.push(serde_json::json!({
                        "functionResponse": {
                            "name": r.call_id,
                            "response": { "result": r.content },
                        }
                    }));
                }
            }
        }
        contents.push(serde_json::json!({ "role": role, "parts": parts }));
    }

    let mut body = serde_json::json!({
        "contents": contents,
        "generationConfig": {
            "maxOutputTokens": req.max_tokens,
        },
    });
    if !req.system.is_empty() {
        body["systemInstruction"] = serde_json::json!({ "parts": [{ "text": req.system }] });
    }
    if !req.tools.is_empty() {
        body["tools"] = serde_json::json!([{
            "functionDeclarations": req.tools.iter().map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
                "parameters": t.parameters,
            })).collect::<Vec<_>>()
        }]);
    }
    if let Some(t) = req.temperature {
        body["generationConfig"]["temperature"] = serde_json::json!(t);
    }

    let response = provider
        .http()
        .post(&url)
        .header("content-type", "application/json")
        .header("x-goog-api-key", &key)
        .json(&body)
        .send()
        .await
        .map_err(|e| CoreError::Provider(format!("gemini: {e}")))?;
    let response = response.error_for_status().map_err(|e| {
        CoreError::Provider(format!(
            "gemini {}: {}",
            e.status().map(|s| s.to_string()).unwrap_or_default(),
            e
        ))
    })?;

    let (sse_tx, mut sse_rx) = mpsc::channel::<String>(64);
    let byte_stream = response.bytes_stream();
    tokio::spawn(async move {
        let _ = read_sse_data_lines(byte_stream, &sse_tx).await;
    });

    let mut call_counter = 0u64;
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
        if let Some(candidates) = event["candidates"].as_array() {
            for cand in candidates {
                if let Some(parts) = cand["content"]["parts"].as_array() {
                    for part in parts {
                        if let Some(text) = part["text"].as_str() {
                            if !text.is_empty() {
                                let _ = tx.send(StreamEvent::Text(text.to_string())).await;
                            }
                        }
                        if let Some(fc) = part.get("functionCall") {
                            call_counter += 1;
                            let name = fc["name"].as_str().unwrap_or_default().to_string();
                            let arguments =
                                fc.get("args").cloned().unwrap_or(serde_json::json!({}));
                            let _ = tx
                                .send(StreamEvent::ToolUse(ToolCall {
                                    id: format!("gemini-call-{call_counter}"),
                                    name,
                                    arguments,
                                }))
                                .await;
                        }
                    }
                }
                if cand["finishReason"].is_string() {
                    let stop = match cand["finishReason"].as_str() {
                        Some("MAX_TOKENS") => StopReason::MaxTokens,
                        Some("STOP") | _ => StopReason::EndTurn,
                    };
                    let _ = tx.send(StreamEvent::Done(stop)).await;
                }
            }
        }
        if let Some(meta) = event.get("usageMetadata").filter(|u| u.is_object()) {
            let _ = tx
                .send(StreamEvent::Usage(super::Usage {
                    input_tokens: meta["promptTokenCount"].as_u64().unwrap_or(0),
                    output_tokens: meta["candidatesTokenCount"].as_u64().unwrap_or(0),
                }))
                .await;
        }
    }
    Ok(())
}
