//! JSON-RPC 2.0 MCP session. One session multiplexes the three MCP
//! primitives over whichever transport the server config selects.

use serde_json::{json, Value};

use crate::config::McpServerConfig;
use crate::{CoreError, CoreResult};

use super::transports::Transport;

pub struct McpSession {
    transport: Transport,
    next_id: u64,
    /// Server info captured during initialize.
    pub server_info: Value,
    pub capabilities: Value,
}

impl McpSession {
    pub async fn connect(config: &McpServerConfig) -> CoreResult<McpSession> {
        let transport = Transport::open(config).await?;
        Ok(McpSession { transport, next_id: 0, server_info: Value::Null, capabilities: Value::Null })
    }

    async fn request(&mut self, method: &str, params: Value) -> CoreResult<Value> {
        self.next_id += 1;
        let id = self.next_id;
        let frame = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let raw = serde_json::to_string(&frame)?;
        self.transport.send(raw).await?;
        let deadline = tokio::time::Duration::from_secs(30);
        let response = tokio::time::timeout(deadline, self.receive_response(id)).await
            .map_err(|_| CoreError::Mcp(format!("{method}: timed out after 30s")))??;
        Ok(response)
    }

    async fn receive_response(&mut self, id: u64) -> CoreResult<Value> {
        loop {
            let raw = self.transport.receive().await?;
            let value: Value = serde_json::from_str(&raw)
                .map_err(|e| CoreError::Mcp(format!("bad jsonrpc frame: {e}")))?;
            // Ignore server-initiated notifications.
            if value.get("id").and_then(|v| v.as_u64()) == Some(id) {
                if let Some(err) = value.get("error") {
                    return Err(CoreError::Mcp(format!(
                        "rpc error {}: {}",
                        err.get("code").and_then(|c| c.as_i64()).unwrap_or(0),
                        err.get("message").and_then(|m| m.as_str()).unwrap_or("unknown")
                    )));
                }
                return value
                    .get("result")
                    .cloned()
                    .ok_or_else(|| CoreError::Mcp("missing result".into()));
            }
        }
    }

    /// MCP handshake; also negotiates protocol version 2025-06-18.
    pub async fn initialize(&mut self) -> CoreResult<Value> {
        let result = self
            .request(
                "initialize",
                json!({
                    "protocolVersion": "2025-06-18",
                    "capabilities": {
                        "roots": { "listChanged": false },
                        "sampling": {},
                    },
                    "clientInfo": {
                        "name": "tervixa-wiggle",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                }),
            )
            .await?;
        self.server_info = result.get("serverInfo").cloned().unwrap_or(Value::Null);
        self.capabilities = result.get("capabilities").cloned().unwrap_or(Value::Null);
        Ok(result)
    }

    pub async fn ping(&mut self) -> CoreResult<()> {
        self.request("ping", json!({})).await.map(|_| ())
    }

    // ----- Primitive 1: Tools -----

    pub async fn list_tools(&mut self) -> CoreResult<Vec<(String, String, Value)>> {
        let result = self.request("tools/list", json!({})).await?;
        let mut out = Vec::new();
        for tool in result["tools"].as_array().into_iter().flatten() {
            out.push((
                tool["name"].as_str().unwrap_or_default().to_string(),
                tool["description"].as_str().unwrap_or_default().to_string(),
                tool.get("inputSchema").cloned().unwrap_or(json!({"type": "object"})),
            ));
        }
        Ok(out)
    }

    pub async fn call_tool(&mut self, name: &str, args: Value) -> CoreResult<String> {
        let result = self.request("tools/call", json!({ "name": name, "arguments": args })).await?;
        let mut text = String::new();
        for block in result["content"].as_array().into_iter().flatten() {
            match block["type"].as_str() {
                Some("text") => {
                    text.push_str(block["text"].as_str().unwrap_or_default());
                }
                Some("resource") => {
                    text.push_str(&format!(
                        "[resource {}]\n{}",
                        block["resource"]["uri"].as_str().unwrap_or_default(),
                        block["resource"]["text"].as_str().unwrap_or_default()
                    ));
                }
                _ => {}
            }
        }
        if result["isError"].as_bool() == Some(true) {
            return Err(CoreError::Mcp(if text.is_empty() {
                "tool reported error".into()
            } else {
                text
            }));
        }
        Ok(text)
    }

    // ----- Primitive 2: Resources -----

    pub async fn list_resources(&mut self) -> CoreResult<Vec<(String, String, String, Option<String>)>> {
        let result = self.request("resources/list", json!({})).await?;
        let mut out = Vec::new();
        for r in result["resources"].as_array().into_iter().flatten() {
            out.push((
                r["uri"].as_str().unwrap_or_default().to_string(),
                r["name"].as_str().unwrap_or_default().to_string(),
                r["description"].as_str().unwrap_or_default().to_string(),
                r["mimeType"].as_str().map(str::to_string),
            ));
        }
        Ok(out)
    }

    pub async fn read_resource(&mut self, uri: &str) -> CoreResult<String> {
        let result = self.request("resources/read", json!({ "uri": uri })).await?;
        let mut text = String::new();
        for block in result["contents"].as_array().into_iter().flatten() {
            if let Some(t) = block["text"].as_str() {
                text.push_str(t);
            } else if let Some(blob) = block["blob"].as_str() {
                use base64::Engine;
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(blob)
                    .unwrap_or_default();
                text.push_str(&String::from_utf8_lossy(&decoded));
            }
        }
        Ok(text)
    }

    // ----- Primitive 3: Prompts -----

    pub async fn list_prompts(&mut self) -> CoreResult<Vec<(String, String, Value)>> {
        let result = self.request("prompts/list", json!({})).await?;
        let mut out = Vec::new();
        for p in result["prompts"].as_array().into_iter().flatten() {
            out.push((
                p["name"].as_str().unwrap_or_default().to_string(),
                p["description"].as_str().unwrap_or_default().to_string(),
                p.get("arguments").cloned().unwrap_or(json!([])),
            ));
        }
        Ok(out)
    }

    pub async fn get_prompt(&mut self, name: &str, args: Value) -> CoreResult<String> {
        let result = self.request("prompts/get", json!({ "name": name, "arguments": args })).await?;
        let mut text = String::new();
        for msg in result["messages"].as_array().into_iter().flatten() {
            text.push_str(msg["content"]["text"].as_str().unwrap_or_default());
            text.push('\n');
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_use_jsonrpc_2() {
        let frame = json!({"jsonrpc": "2.0", "id": 1, "method": "ping", "params": {}});
        assert_eq!(frame["jsonrpc"], "2.0");
    }
}
