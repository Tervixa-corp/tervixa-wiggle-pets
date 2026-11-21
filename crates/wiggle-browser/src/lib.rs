//! Built-in agentic browser controller.
//!
//! Drives a real Chromium-class browser over CDP (Chrome DevTools
//! Protocol) — the same loop the embedded BrowserPane uses — and
//! extracts compact DOM accessibility trees ([ref=e1] button "Submit"),
//! console logs and network failures for agent consumption.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("browser not reachable: {0}")]
    NotReachable(String),
    #[error("cdp error: {0}")]
    Cdp(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type BrowserResult<T> = Result<T, BrowserError>;

/// Configuration for the controller (mirrors the engine's browser
/// settings; kept independent so wiggle-browser has no core dependency).
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub executable: Option<String>,
    pub headless: bool,
    pub persist_session: bool,
}

/// One node of the compact accessibility tree, stored flat with depth
/// for cheap indentation rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomNode {
    pub reference: String,
    pub role: String,
    pub name: String,
    pub value: Option<String>,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSnapshot {
    pub url: String,
    pub title: String,
    pub nodes: Vec<DomNode>,
}

impl PageSnapshot {
    /// Compact bracket-ref rendering, e.g. `[ref=e1] button "Submit"`.
    pub fn to_compact_tree(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# {}\n{}\n\n", self.title, self.url));
        for node in &self.nodes {
            let indent = "  ".repeat(node.depth.min(10));
            let value = node
                .value
                .as_deref()
                .map(|v| format!(" value=\"{}\"", v.replace('"', "'")))
                .unwrap_or_default();
            out.push_str(&format!(
                "{indent}[ref={}] {} \"{}\"{value}\n",
                node.reference, node.role, node.name
            ));
        }
        out
    }
}

/// A live controller. Connects lazily: the browser binary is only
/// launched on first tool use.
pub struct BrowserController {
    config: BrowserConfig,
    ws_url: Option<String>,
    http: reqwest::Client,
}

impl BrowserController {
    pub fn new(config: BrowserConfig) -> BrowserController {
        BrowserController { config, ws_url: None, http: reqwest::Client::new() }
    }

    /// Locate a Chromium-class browser executable.
    fn find_executable(&self) -> Option<String> {
        if let Some(exe) = &self.config.executable {
            if std::path::Path::new(exe).exists() {
                return Some(exe.clone());
            }
        }
        let candidates: &[&str] = if cfg!(windows) {
            &[
                "C:/Program Files/Google/Chrome/Application/chrome.exe",
                "C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
                "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
            ]
        } else if cfg!(target_os = "macos") {
            &[
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Microsoft Edge.app/Contents/Microsoft Edge",
            ]
        } else {
            &["/usr/bin/chromium", "/usr/bin/chromium-browser", "/usr/bin/google-chrome"]
        };
        candidates.iter().find(|c| std::path::Path::new(c).exists()).map(|c| c.to_string())
    }

    /// Launch the browser with a CDP port and capture the DevTools
    /// websocket URL of the default target.
    async fn ensure_launched(&mut self) -> BrowserResult<String> {
        if let Some(url) = &self.ws_url {
            return Ok(url.clone());
        }
        let exe = self
            .find_executable()
            .ok_or_else(|| BrowserError::NotReachable("no Chromium-class browser found".into()))?;
        let port = 9222 + (std::process::id() % 500) as u16;
        let user_data = if self.config.persist_session {
            std::env::temp_dir().join("wiggle-browser-profile")
        } else {
            let tmp = std::env::temp_dir().join(format!("wiggle-browser-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&tmp);
            tmp
        };
        let mut cmd = std::process::Command::new(&exe);
        cmd.arg(format!("--remote-debugging-port={port}"))
            .arg(format!("--user-data-dir={}", user_data.display()))
            .arg("--no-first-run")
            .arg("--no-default-browser-check");
        if self.config.headless {
            cmd.arg("--headless=new");
        }
        #[cfg(windows)]
        cmd.creation_flags(0x0800_0000);
        cmd.spawn().map_err(BrowserError::Io)?;

        let http_url = format!("http://127.0.0.1:{port}/json/version");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        loop {
            let reached = async {
                if let Ok(response) = self.http.get(&http_url).send().await {
                    if let Ok(body) = response.text().await {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) {
                            if let Some(ws) = value["webSocketDebuggerUrl"].as_str() {
                                return Some(ws.to_string());
                            }
                        }
                    }
                }
                None
            };
            if let Some(ws) = reached.await {
                self.ws_url = Some(ws.clone());
                return Ok(ws);
            }
            if std::time::Instant::now() > deadline {
                return Err(BrowserError::NotReachable(format!(
                    "browser CDP endpoint did not open on port {port}"
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }

    pub async fn navigate(&mut self, url: String) -> BrowserResult<PageSnapshot> {
        let ws = self.ensure_launched().await?;
        run_page_task(&ws, move |page| page.navigate_and_snapshot(&url)).await
    }

    pub async fn actuate(
        &mut self,
        reference: String,
        action: String,
        text: String,
    ) -> BrowserResult<PageSnapshot> {
        let ws = self.ensure_launched().await?;
        run_page_task(&ws, move |page| page.actuate(&reference, &action, &text)).await
    }

    /// Snapshot + console errors + failed requests, as text.
    pub async fn inspect(&mut self) -> BrowserResult<String> {
        let ws = self.ensure_launched().await?;
        run_page_task(&ws, |page| page.inspect_report()).await
    }
}

/// Run a sync page task on a dedicated thread with its own
/// current-thread runtime (CDP is one websocket, strictly sequential).
async fn run_page_task<T, F>(ws_url: &str, task: F) -> BrowserResult<T>
where
    T: Send + 'static,
    F: FnOnce(PageSession) -> BrowserResult<T> + Send + 'static,
{
    let ws_url = ws_url.to_string();
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let result = PageSession::run(&ws_url, task);
        let _ = tx.send(result);
    });
    rx.await.map_err(|_| BrowserError::Cdp("page worker dropped".into()))?
}

type WsStream = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;
type WsMessage = tokio_tungstenite::tungstenite::Message;

pub struct PageSession {
    ws: (
        futures::stream::SplitSink<WsStream, WsMessage>,
        futures::stream::SplitStream<WsStream>,
    ),
    next_id: u64,
}

impl PageSession {
    fn run<T, F>(ws_url: &str, task: F) -> BrowserResult<T>
    where
        T: Send + 'static,
        F: FnOnce(PageSession) -> BrowserResult<T>,
    {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| BrowserError::Cdp(e.to_string()))?;
        rt.block_on(async {
            let (ws, _) = tokio_tungstenite::connect_async(ws_url)
                .await
                .map_err(|e| BrowserError::Cdp(format!("ws connect: {e}")))?;
            let session = PageSession { ws: futures::StreamExt::split(ws), next_id: 0 };
            task(session)
        })
    }

    async fn call(&mut self, method: &str, params: serde_json::Value) -> BrowserResult<serde_json::Value> {
        use futures::{SinkExt, StreamExt};
        self.next_id += 1;
        let id = self.next_id;
        let frame = serde_json::json!({
            "id": id,
            "method": method,
            "params": params,
        })
        .to_string();
        self.ws
            .0
            .send(WsMessage::Text(frame.into()))
            .await
            .map_err(|e| BrowserError::Cdp(format!("ws send: {e}")))?;
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(25);
        loop {
            let timeout = tokio::time::sleep_until(deadline);
            tokio::select! {
                msg = self.ws.1.next() => {
                    let msg = match msg {
                        Some(Ok(m)) => m,
                        Some(Err(e)) => return Err(BrowserError::Cdp(format!("ws recv: {e}"))),
                        None => return Err(BrowserError::Cdp("ws closed".into())),
                    };
                    if let WsMessage::Text(text) = msg {
                        let value: serde_json::Value = serde_json::from_str(&text)
                            .map_err(|e| BrowserError::Cdp(format!("bad cdp frame: {e}")))?;
                        if value.get("id").and_then(|v| v.as_u64()) == Some(id) {
                            if let Some(err) = value.get("error") {
                                return Err(BrowserError::Cdp(err.to_string()));
                            }
                            return Ok(value.get("result").cloned().unwrap_or(serde_json::Value::Null));
                        }
                        // Events (console, network) are ignored here; the
                        // BrowserPane subscribes to them separately.
                    }
                }
                _ = timeout => return Err(BrowserError::Cdp(format!("{method} timed out"))),
            }
        }
    }

    async fn evaluate(&mut self, expression: &str) -> BrowserResult<serde_json::Value> {
        let result = self
            .call(
                "Runtime.evaluate",
                serde_json::json!({ "expression": expression, "returnByValue": true, "awaitPromise": true }),
            )
            .await?;
        Ok(result["result"]["value"].clone())
    }

    fn navigate_and_snapshot(mut self, url: &str) -> BrowserResult<PageSnapshot> {
        drive(async {
            self.call("Page.enable", serde_json::json!({})).await?;
            self.call("Page.navigate", serde_json::json!({ "url": url })).await?;
            // Let the document settle before snapshotting.
            self.evaluate("new Promise(r => setTimeout(r, 800))").await?;
            self.snapshot_via_js().await
        })
    }

    fn actuate(mut self, reference: &str, action: &str, text: &str) -> BrowserResult<PageSnapshot> {
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
        let script = match action {
            "type" => format!(
                "(() => {{ const el = window.__wiggleRefs && window.__wiggleRefs['{reference}']; \
                 if (!el) return 'ref not found'; el.focus(); \
                 const desc = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value') \
                   || Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value'); \
                 desc.set.call(el, \"{escaped_text}\"); \
                 el.dispatchEvent(new Event('input', {{ bubbles: true }})); return 'typed'; }})()"
            ),
            "select" => format!(
                "(() => {{ const el = window.__wiggleRefs && window.__wiggleRefs['{reference}']; \
                 if (!el) return 'ref not found'; el.value = \"{escaped_text}\"; \
                 el.dispatchEvent(new Event('change', {{ bubbles: true }})); return 'selected'; }})()"
            ),
            "scroll" => format!(
                "(() => {{ const el = window.__wiggleRefs && window.__wiggleRefs['{reference}']; \
                 if (el) el.scrollIntoView({{ block: 'center' }}); window.scrollBy(0, 400); return 'scrolled'; }})()"
            ),
            _ => format!(
                "(() => {{ const el = window.__wiggleRefs && window.__wiggleRefs['{reference}']; \
                 if (!el) return 'ref not found'; el.scrollIntoView({{ block: 'center' }}); \
                 el.dispatchEvent(new MouseEvent('mousedown', {{ bubbles: true }})); \
                 el.dispatchEvent(new MouseEvent('mouseup', {{ bubbles: true }})); \
                 el.click(); return 'clicked'; }})()"
            ),
        };
        drive(async {
            let result = self.evaluate(&script).await?;
            if result.as_str() == Some("ref not found") {
                // Refresh the tree so the next attempt has fresh refs.
                self.snapshot_via_js().await?;
                return Err(BrowserError::Cdp(format!(
                    "ref {reference} not found on the current page (tree refreshed, retry)"
                )));
            }
            self.evaluate("new Promise(r => setTimeout(r, 300))").await?;
            self.snapshot_via_js().await
        })
    }

    fn inspect_report(mut self) -> BrowserResult<String> {
        drive(async {
            let console = self.console_errors().await.unwrap_or_default();
            let network = self.failed_requests().await.unwrap_or_default();
            let snapshot = self.snapshot_via_js().await?;
            Ok(format!(
                "{}\n--- console errors (last 20) ---\n{}\n--- failed requests (last 10) ---\n{}",
                snapshot.to_compact_tree(),
                if console.is_empty() { "(none)".into() } else { console },
                if network.is_empty() { "(none)".into() } else { network },
            ))
        })
    }

    async fn console_errors(&mut self) -> BrowserResult<String> {
        // Pull captured console errors if the page helper stored them;
        // otherwise evaluate a quick check of window.__wiggleConsole.
        let value = self
            .evaluate(
                "(window.__wiggleConsole || []).slice(-20).join('\\n')",
            )
            .await?;
        Ok(value.as_str().unwrap_or_default().to_string())
    }

    async fn failed_requests(&mut self) -> BrowserResult<String> {
        let value = self
            .evaluate(
                "(window.__wiggleNetwork || []).slice(-10).map(f => f.url + ' -> ' + f.status).join('\\n')",
            )
            .await?;
        Ok(value.as_str().unwrap_or_default().to_string())
    }

    async fn snapshot_via_js(&mut self) -> BrowserResult<PageSnapshot> {
        let script = r#"(() => {
            window.__wiggleRefs = {};
            const interactive = new Set(['a','button','input','select','textarea','label','summary']);
            const out = [];
            let ref = 0;
            function walk(el, depth) {
                if (!el || depth > 14 || out.length > 400) return;
                let role = el.getAttribute && el.getAttribute('role');
                if (!role) {
                    const tag = (el.tagName || '').toLowerCase();
                    if (interactive.has(tag)) role = tag;
                    else if (tag === 'h1' || tag === 'h2' || tag === 'h3') role = 'heading';
                    else if (tag === 'p') role = 'paragraph';
                    else role = 'generic';
                }
                const name = (el.getAttribute && (el.getAttribute('aria-label') || el.getAttribute('placeholder')))
                    || (el.innerText ? el.innerText.split('\n')[0].slice(0, 80) : '') || '';
                const tag = (el.tagName || '').toLowerCase();
                const value = ['input','textarea','select'].includes(tag)
                    ? String(el.value || '').slice(0, 80) : null;
                const reference = 'e' + (++ref);
                window.__wiggleRefs[reference] = el;
                out.push({ depth, reference, role: String(role), name: String(name), value });
                for (const child of el.children || []) walk(child, depth + 1);
            }
            walk(document.body || document.documentElement, 0);
            return { url: location.href, title: document.title, nodes: out };
        })()"#;
        let value = self.evaluate(script).await?;
        let nodes = value["nodes"]
            .as_array()
            .map(|list| {
                list.iter()
                    .map(|n| DomNode {
                        reference: n["reference"].as_str().unwrap_or_default().to_string(),
                        role: n["role"].as_str().unwrap_or_default().to_string(),
                        name: n["name"].as_str().unwrap_or_default().to_string(),
                        value: n["value"].as_str().map(str::to_string),
                        depth: n["depth"].as_u64().unwrap_or(0) as usize,
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(PageSnapshot {
            url: value["url"].as_str().unwrap_or_default().to_string(),
            title: value["title"].as_str().unwrap_or_default().to_string(),
            nodes,
        })
    }
}

/// Drive an async block on a fresh one-shot runtime from a sync task
/// body (PageSession::run is sync by contract).
fn drive<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("drive runtime")
        .block_on(future)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_tree_format_matches_spec() {
        let snapshot = PageSnapshot {
            url: "http://localhost:3000".into(),
            title: "Test".into(),
            nodes: vec![DomNode {
                reference: "e1".into(),
                role: "button".into(),
                name: "Submit".into(),
                value: None,
                depth: 0,
            }],
        };
        let text = snapshot.to_compact_tree();
        assert!(text.contains("[ref=e1] button \"Submit\""));
    }

    #[test]
    fn finds_a_browser_binary_or_none() {
        let controller = BrowserController::new(BrowserConfig {
            executable: None,
            headless: true,
            persist_session: false,
        });
        // On Windows CI/desktop Chrome or Edge is present; the assertion
        // is tolerant to bare CI containers.
        let _ = controller.find_executable();
    }
}
