//! Model Context Protocol client: JSON-RPC 2.0 over stdio, SSE and
//! Streamable HTTP, with all three primitives (tools, resources,
//! prompts) and Claude-Desktop-compatible `mcpServers` config import.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::{GlobalConfig, McpServerConfig, McpTransport};
use crate::{CoreError, CoreResult};

pub mod client;
pub mod registry;
pub mod transports;

/// One tool / resource / prompt exposed by a connected MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub server: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub server: String,
    pub uri: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub server: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// Connection state of one configured server.
#[derive(Debug, Clone, Serialize)]
pub struct McpServerState {
    pub config: McpServerConfig,
    pub connected: bool,
    #[serde(skip_serializing)]
    pub error: Option<String>,
    pub tools: Vec<McpTool>,
    pub resources: Vec<McpResource>,
    pub prompts: Vec<McpPrompt>,
}

/// Registry of all configured MCP servers plus their live sessions.
pub struct McpManager {
    servers: BTreeMap<String, McpServerState>,
    sessions: BTreeMap<String, Mutex<client::McpSession>>,
}

impl McpManager {
    pub fn from_config(config: &GlobalConfig) -> McpManager {
        let mut servers = BTreeMap::new();
        for server in &config.mcp_servers {
            servers.insert(
                server.name.clone(),
                McpServerState {
                    config: server.clone(),
                    connected: false,
                    error: None,
                    tools: Vec::new(),
                    resources: Vec::new(),
                    prompts: Vec::new(),
                },
            );
        }
        McpManager { servers, sessions: BTreeMap::new() }
    }

    /// Connect to one server and cache its three primitive lists.
    pub async fn connect(&mut self, name: &str) -> CoreResult<()> {
        let state = self
            .servers
            .get(name)
            .ok_or_else(|| CoreError::Mcp(format!("unknown server '{name}'")))?
            .config
            .clone();
        let mut session = client::McpSession::connect(&state).await?;
        session.initialize().await?;

        let tools = session.list_tools().await.unwrap_or_default();
        let resources = session.list_resources().await.unwrap_or_default();
        let prompts = session.list_prompts().await.unwrap_or_default();

        if let Some(state) = self.servers.get_mut(name) {
            state.connected = true;
            state.error = None;
            state.tools = tools
                .into_iter()
                .map(|(name, description, input_schema)| McpTool {
                    server: state.config.name.clone(),
                    name,
                    description,
                    input_schema,
                })
                .collect();
            state.resources = resources
                .into_iter()
                .map(|(uri, name, description, mime_type)| McpResource {
                    server: state.config.name.clone(),
                    uri,
                    name,
                    description,
                    mime_type,
                })
                .collect();
            state.prompts = prompts
                .into_iter()
                .map(|(name, description, arguments)| McpPrompt {
                    server: state.config.name.clone(),
                    name,
                    description,
                    arguments,
                })
                .collect();
        }
        self.sessions.insert(name.to_string(), Mutex::new(session));
        Ok(())
    }

    /// Connect to every enabled server, recording per-server errors.
    pub async fn connect_all(&mut self) {
        let names: Vec<String> = self
            .servers
            .values()
            .filter(|s| s.config.enabled)
            .map(|s| s.config.name.clone())
            .collect();
        for name in names {
            if let Err(e) = self.connect(&name).await {
                if let Some(state) = self.servers.get_mut(&name) {
                    state.connected = false;
                    state.error = Some(e.to_string());
                }
            }
        }
    }

    /// Call a tool on a connected server.
    pub async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        args: serde_json::Value,
    ) -> CoreResult<String> {
        let mut session = self
            .sessions
            .get(server)
            .ok_or_else(|| CoreError::Mcp(format!("server '{server}' not connected")))?
            .lock()
            .await;
        session.call_tool(tool, args).await
    }

    /// Read a resource from a connected server.
    pub async fn read_resource(&self, server: &str, uri: &str) -> CoreResult<String> {
        let mut session = self
            .sessions
            .get(server)
            .ok_or_else(|| CoreError::Mcp(format!("server '{server}' not connected")))?
            .lock()
            .await;
        session.read_resource(uri).await
    }

    /// Render a prompt template from a connected server.
    pub async fn get_prompt(
        &self,
        server: &str,
        name: &str,
        args: serde_json::Value,
    ) -> CoreResult<String> {
        let mut session = self
            .sessions
            .get(server)
            .ok_or_else(|| CoreError::Mcp(format!("server '{server}' not connected")))?
            .lock()
            .await;
        session.get_prompt(name, args).await
    }

    pub fn states(&self) -> Vec<McpServerState> {
        self.servers.values().cloned().collect()
    }

    pub fn all_tools(&self) -> Vec<McpTool> {
        self.servers
            .values()
            .filter(|s| s.connected && s.config.enabled)
            .flat_map(|s| s.tools.iter().cloned())
            .collect()
    }

    pub fn ready_count(&self) -> usize {
        self.servers.values().filter(|s| s.connected).count()
    }
}

/// Parse a Claude Desktop / Claude Code `mcpServers` JSON block and
/// return our config entries; 100% compatible field names.
pub fn parse_claude_mcp_json(raw: &str) -> CoreResult<Vec<McpServerConfig>> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| CoreError::Mcp(e.to_string()))?;
    let servers = value
        .get("mcpServers")
        .or(Some(&value))
        .ok_or_else(|| CoreError::Mcp("missing mcpServers".into()))?;
    let mut out = Vec::new();
    for (name, spec) in servers.as_object().into_iter().flatten() {
        if let Some(url) = spec["url"].as_str() {
            out.push(McpServerConfig {
                name: name.clone(),
                transport: if spec["type"].as_str() == Some("sse") {
                    McpTransport::Sse
                } else {
                    McpTransport::Http
                },
                endpoint: url.to_string(),
                args: Vec::new(),
                env: Default::default(),
                enabled: true,
            });
        } else if let Some(cmd) = spec["command"].as_str() {
            let args = spec["args"]
                .as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let env = spec["env"]
                .as_object()
                .map(|o| {
                    o.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect()
                })
                .unwrap_or_default();
            out.push(McpServerConfig {
                name: name.clone(),
                transport: McpTransport::Stdio,
                endpoint: cmd.to_string(),
                args,
                env,
                enabled: true,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_claude_desktop_config() {
        let raw = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
                    "env": {"HOME": "/tmp"}
                },
                "remote": { "url": "https://mcp.example.com/mcp" }
            }
        }"#;
        let parsed = parse_claude_mcp_json(raw).unwrap();
        assert_eq!(parsed.len(), 2);
        let fs = parsed.iter().find(|p| p.name == "filesystem").unwrap();
        assert_eq!(fs.transport, McpTransport::Stdio);
        assert_eq!(fs.endpoint, "npx");
        assert_eq!(fs.args.len(), 3);
        let remote = parsed.iter().find(|p| p.name == "remote").unwrap();
        assert_eq!(remote.endpoint, "https://mcp.example.com/mcp");
    }
}
