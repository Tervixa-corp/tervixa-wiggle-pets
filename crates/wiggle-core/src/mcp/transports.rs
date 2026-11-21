//! MCP transports: stdio (child process), Streamable HTTP and SSE.

use crate::config::{McpServerConfig, McpTransport};
use crate::{CoreError, CoreResult};

pub enum Transport {
    Stdio(StdioTransport),
    Http(HttpTransport),
    Sse(SseTransport),
}

pub struct StdioTransport {
    stdin: tokio::process::ChildStdin,
    stdout: tokio::io::BufReader<tokio::process::ChildStdout>,
    child: tokio::process::Child,
}

pub struct HttpTransport {
    url: String,
    client: reqwest::Client,
    /// Streamable HTTP replies can be a single JSON body or an SSE
    /// stream; we buffer whatever the server returns.
    pending: std::collections::VecDeque<String>,
    session_id: Option<String>,
}

pub struct SseTransport {
    url: String,
    client: reqwest::Client,
    /// POST endpoint announced by the server's `endpoint` event.
    post_url: Option<String>,
    pending: std::collections::VecDeque<String>,
    session_id: Option<String>,
}

impl Transport {
    pub async fn open(config: &McpServerConfig) -> CoreResult<Transport> {
        match config.transport {
            McpTransport::Stdio => Ok(Transport::Stdio(StdioTransport::spawn(config).await?)),
            McpTransport::Http => Ok(Transport::Http(HttpTransport {
                url: config.endpoint.clone(),
                client: reqwest::Client::new(),
                pending: Default::default(),
                session_id: None,
            })),
            McpTransport::Sse => Ok(Transport::Sse(SseTransport {
                url: config.endpoint.clone(),
                client: reqwest::Client::new(),
                post_url: None,
                pending: Default::default(),
                session_id: None,
            })),
        }
    }

    pub async fn send(&mut self, raw: String) -> CoreResult<()> {
        match self {
            Transport::Stdio(t) => t.send(raw).await,
            Transport::Http(t) => t.send(raw).await,
            Transport::Sse(t) => t.send(raw).await,
        }
    }

    pub async fn receive(&mut self) -> CoreResult<String> {
        match self {
            Transport::Stdio(t) => t.receive().await,
            Transport::Http(t) => t.receive().await,
            Transport::Sse(t) => t.receive().await,
        }
    }
}

impl StdioTransport {
    async fn spawn(config: &McpServerConfig) -> CoreResult<StdioTransport> {
        let mut command = tokio::process::Command::new(&config.endpoint);
        command
            .args(&config.args)
            .envs(&config.env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true);
        #[cfg(windows)]
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        let mut child =
            command.spawn().map_err(|e| CoreError::Mcp(format!("spawn {}: {e}", config.endpoint)))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| CoreError::Mcp("child stdin unavailable".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CoreError::Mcp("child stdout unavailable".into()))?;
        Ok(StdioTransport {
            stdin,
            stdout: tokio::io::BufReader::new(stdout),
            child,
        })
    }

    async fn send(&mut self, raw: String) -> CoreResult<()> {
        use tokio::io::AsyncWriteExt;
        self.stdin
            .write_all(raw.as_bytes())
            .await
            .map_err(|e| CoreError::Mcp(format!("stdio write: {e}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|e| CoreError::Mcp(format!("stdio write: {e}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|e| CoreError::Mcp(format!("stdio write: {e}")))
    }

    async fn receive(&mut self) -> CoreResult<String> {
        use tokio::io::AsyncBufReadExt;
        let mut line = String::new();
        let n = self
            .stdout
            .read_line(&mut line)
            .await
            .map_err(|e| CoreError::Mcp(format!("stdio read: {e}")))?;
        if n == 0 {
            // Give the child a moment to surface an exit status.
            let status = self.child.try_wait().ok().flatten();
            return Err(CoreError::Mcp(match status {
                Some(s) => format!("mcp server exited: {s}"),
                None => "mcp server closed stdout".into(),
            }));
        }
        Ok(line.trim().to_string())
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

impl HttpTransport {
    async fn send(&mut self, raw: String) -> CoreResult<()> {
        let mut request = self
            .client
            .post(&self.url)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .body(raw);
        if let Some(id) = &self.session_id {
            request = request.header("mcp-session-id", id);
        }
        let response = request
            .send()
            .await
            .map_err(|e| CoreError::Mcp(format!("http: {e}")))?
            .error_for_status()
            .map_err(|e| CoreError::Mcp(format!("http: {e}")))?;
        if let Some(id) = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
        {
            self.session_id = Some(id.to_string());
        }
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = response
            .text()
            .await
            .map_err(|e| CoreError::Mcp(format!("http read: {e}")))?;
        if content_type.contains("text/event-stream") {
            self.pending.extend(extract_sse_data(&body));
        } else if !body.trim().is_empty() {
            self.pending.push_back(body);
        }
        Ok(())
    }

    async fn receive(&mut self) -> CoreResult<String> {
        self.pending
            .pop_front()
            .ok_or_else(|| CoreError::Mcp("no pending http response".into()))
    }
}

impl SseTransport {
    async fn send(&mut self, raw: String) -> CoreResult<()> {
        let post_url = self
            .post_url
            .clone()
            .ok_or_else(|| CoreError::Mcp("sse: endpoint not announced yet".into()))?;
        let mut request = self
            .client
            .post(&post_url)
            .header("content-type", "application/json")
            .header("accept", "application/json, text/event-stream")
            .body(raw);
        if let Some(id) = &self.session_id {
            request = request.header("mcp-session-id", id);
        }
        let response = request
            .send()
            .await
            .map_err(|e| CoreError::Mcp(format!("sse: {e}")))?
            .error_for_status()
            .map_err(|e| CoreError::Mcp(format!("sse: {e}")))?;
        if let Some(id) = response.headers().get("mcp-session-id").and_then(|v| v.to_str().ok()) {
            self.session_id = Some(id.to_string());
        }
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = response.text().await.map_err(|e| CoreError::Mcp(format!("sse read: {e}")))?;
        if content_type.contains("text/event-stream") {
            self.pending.extend(extract_sse_data(&body));
        } else if !body.trim().is_empty() {
            self.pending.push_back(body);
        }
        Ok(())
    }

    async fn receive(&mut self) -> CoreResult<String> {
        if let Some(next) = self.pending.pop_front() {
            return Ok(next);
        }
        // Open (or re-open) the SSE event stream and wait for the next
        // matching event. Also handles the initial `endpoint` announce.
        let response = self
            .client
            .get(&self.url)
            .header("accept", "text/event-stream")
            .send()
            .await
            .map_err(|e| CoreError::Mcp(format!("sse stream: {e}")))?
            .error_for_status()
            .map_err(|e| CoreError::Mcp(format!("sse stream: {e}")))?;
        let body = response.text().await.map_err(|e| CoreError::Mcp(format!("sse read: {e}")))?;
        for data in extract_sse_data(&body) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&data) {
                let is_endpoint = v.get("method").and_then(|m| m.as_str()) == Some("endpoint")
                    || v.get("endpoint").is_some();
                if is_endpoint {
                    if let Some(endpoint) = v.get("endpoint").and_then(|e| e.as_str()) {
                        self.post_url = Some(resolve_url(&self.url, endpoint));
                        continue;
                    }
                }
                self.pending.push_back(data);
            }
        }
        self.pending
            .pop_front()
            .ok_or_else(|| CoreError::Mcp("sse stream closed without response".into()))
    }
}

/// Extract `data:` payloads from an SSE text body, joining multi-line
/// data fields the way the SSE spec requires.
pub fn extract_sse_data(body: &str) -> Vec<String> {
    let mut frames = Vec::new();
    let mut current = String::new();
    for line in body.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(data.trim_start());
        } else if line.is_empty() && !current.is_empty() {
            frames.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        frames.push(current);
    }
    frames
}

fn resolve_url(base: &str, endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        return endpoint.to_string();
    }
    // endpoint is usually an absolute path like /messages?sessionId=...
    let scheme_end = base.find("://").map(|i| i + 3).unwrap_or(0);
    let host_end = base[scheme_end..].find('/').map(|i| scheme_end + i).unwrap_or(base.len());
    format!("{}/{}", &base[..host_end], endpoint.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_extraction_joins_data_fields() {
        let body = "event: message\ndata: {\"a\":1}\n\ndata: [DONE]\n\n: comment\ndata: second\n";
        let frames = extract_sse_data(body);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0], "{\"a\":1}");
        assert_eq!(frames[2], "second");
    }

    #[test]
    fn resolve_url_handles_relative_endpoints() {
        assert_eq!(
            resolve_url("https://mcp.example.com/sse", "/messages?x=1"),
            "https://mcp.example.com/messages?x=1"
        );
        assert_eq!(
            resolve_url("https://mcp.example.com/sse", "https://other.io/mcp"),
            "https://other.io/mcp"
        );
    }
}
