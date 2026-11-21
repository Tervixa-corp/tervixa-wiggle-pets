//! wiggle-hook — the tiny hook binary registered into every AI agent.
//!
//! Usage: wiggle-hook <agent> <event>
//! Reads the agent's stdin JSON (session_id, cwd, tool_name, ...), POSTs a
//! normalized state event to the daemon, exits 0 immediately. Fails open:
//! if the daemon is down the agent continues untouched.
//!
//! For blocking permission flow the agents use the HTTP hook directly
//! (see installers.rs); this binary is state-only.

use std::io::Read;
use std::time::Duration;

fn read_stdin_json() -> serde_json::Value {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    serde_json::from_str(&buf).unwrap_or(serde_json::json!({}))
}

fn find_port() -> Option<u16> {
    wiggle_core::port_candidates()
        .into_iter()
        .find(|p| {
            std::net::TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], *p)),
                Duration::from_millis(120),
            )
            .is_ok()
        })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let agent = args.get(1).cloned().unwrap_or_else(|| "unknown".into());
    let event = args.get(2).cloned().unwrap_or_else(|| "Idle".into());

    let input = read_stdin_json();
    let session_id = input["session_id"]
        .as_str()
        .or_else(|| input["sessionId"].as_str())
        .unwrap_or("default")
        .to_string();
    let cwd = input["cwd"]
        .as_str()
        .or_else(|| input["workspace"].as_str())
        .map(|s| s.to_string());
    let tool = input["tool_name"]
        .as_str()
        .or_else(|| input["tool"].as_str())
        .map(|s| s.to_string());
    let detail = input["message"]
        .as_str()
        .or_else(|| input["notification"].as_str())
        .map(|s| s.chars().take(200).collect::<String>());

    // Watchdog: agents give hooks a short stdout budget; never exceed it.
    if let Some(port) = find_port() {
        let payload = serde_json::json!({
            "agent": agent,
            "session_id": session_id,
            "workspace": cwd,
            "event": event,
            "detail": detail,
            "tool": tool,
            "timestamp_ms": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        });
        let addr = format!("http://127.0.0.1:{port}/state");
        // Synchronous, tiny, bounded — no async runtime in the hot path.
        if let Ok(mut stream) =
            std::net::TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], port)), Duration::from_millis(300))
        {
            stream.set_read_timeout(Some(Duration::from_millis(300))).ok();
            stream.set_write_timeout(Some(Duration::from_millis(300))).ok();
            let body = payload.to_string();
            let req = format!(
                "POST /state HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = std::io::Write::write_all(&mut stream, req.as_bytes());
        }
        let _ = addr; // addr referenced for clarity
    }

    // Fail-open: print an empty decision (agent's native flow continues).
    println!("{{}}");
}
