//! Local hook/IPC HTTP server — the spine of the deterministic engine.
//! Hand-rolled HTTP/1.1 over std::net (no async runtime, ~0% idle CPU).
//! Hooks fail open (100 ms budget); the UI polls /api/events.

use crate::protocol::{PermissionAnswer, PermissionAsk, PermissionDecision};
use crate::state::StateStore;
use crate::{agent_watcher, db::Db, permission_bridge, pet_registry, timed_skills, ai_router};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct App {
    pub db: Db,
    pub store: Arc<StateStore>,
    pub bridge: permission_bridge::PermissionBridge,
    pub stop: Arc<AtomicBool>,
    pub chat_history: Mutex<Vec<(String, String)>>,
}

pub fn serve(app: Arc<App>, port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    for stream in listener.incoming() {
        if app.stop.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        match stream {
            Ok(s) => {
                let app = app.clone();
                std::thread::spawn(move || handle_conn(app, s));
            }
            Err(_) => continue,
        }
    }
    Ok(())
}

fn handle_conn(app: Arc<App>, stream: TcpStream) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(700)));
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });
    let mut stream = stream;

    let (method, path, body) = match read_request(&mut reader) {
        Ok(r) => r,
        Err(_) => return,
    };

    let (status, json) = route(&app, &method, &path, &body);
    let reason = if status == 200 { "OK" } else { "ERR" };
    let body_bytes = json.into_bytes();
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
        body_bytes.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(&body_bytes);
    let _ = stream.flush();
}

fn read_request(reader: &mut BufReader<TcpStream>) -> std::io::Result<(String, String, String)> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();
    let mut content_length = 0usize;
    loop {
        let mut h = String::new();
        reader.read_line(&mut h)?;
        let h = h.trim();
        if h.is_empty() {
            break;
        }
        let lower = h.to_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; content_length.min(10 * 1024 * 1024)];
    reader.read_exact(&mut body)?;
    Ok((method, path, String::from_utf8_lossy(&body).to_string()))
}

fn ok(v: serde_json::Value) -> (u16, String) {
    (200, serde_json::json!({ "ok": true, "data": v }).to_string())
}
fn err(status: u16, msg: &str) -> (u16, String) {
    (
        status,
        serde_json::json!({ "ok": false, "error": msg }).to_string(),
    )
}

fn route(app: &Arc<App>, method: &str, path: &str, body: &str) -> (u16, String) {
    let path_q = path.split('?').collect::<Vec<_>>();
    let (path, query) = (path_q[0], path_q.get(1).copied().unwrap_or(""));
    let json_body = serde_json::from_str::<serde_json::Value>(body).unwrap_or(serde_json::json!({}));

    // ---- Hook endpoints (from agent hooks, hot path) ----
    if path == "/state" && method == "POST" {
        match serde_json::from_value::<crate::protocol::AgentStateEvent>(json_body) {
            Ok(ev) => {
                app.store.apply(ev);
                (200, "{\"ok\":true}".into())
            }
            Err(e) => return err(400, &format!("bad state event: {e}")),
        }
    } else if let Some(agent) = path.strip_prefix("/hook/").and_then(|p| p.strip_suffix("/permission")) {
        if method == "POST" {
            return hook_permission(app, agent, &json_body);
        }
        return err(405, "method not allowed");
    } else {
        // ---- UI API ----
        return match (method, path) {
            ("GET", "/health") => ok(serde_json::json!({
                "app": "tervixa-wiggle-pets", "version": env!("CARGO_PKG_VERSION"),
            })),
            ("GET", "/api/events") => {
                let since: u64 = query_param(query, "since").and_then(|s| s.parse().ok()).unwrap_or(0);
                ok(serde_json::json!({ "events": app.store.events_since(since) }))
            }
            ("GET", "/api/sessions") => {
                let sessions: Vec<serde_json::Value> = app
                    .store
                    .live_sessions()
                    .iter()
                    .map(|s| {
                        serde_json::json!({
                            "agent": s.event.agent,
                            "agent_name": agent_watcher::agent_display(&s.event.agent),
                            "session_id": s.event.session_id,
                            "workspace": s.event.workspace,
                            "state": s.state,
                            "detail": s.event.detail,
                            "tool": s.event.tool,
                        })
                    })
                    .collect();
                ok(serde_json::json!({ "sessions": sessions }))
            }
            ("POST", "/api/permission/answer") => {
                match serde_json::from_value::<PermissionAnswer>(json_body) {
                    Ok(a) => match app.bridge.answer(a) {
                        Ok(()) => (200, "{\"ok\":true}".into()),
                        Err(e) => err(404, &e),
                    },
                    Err(e) => err(400, &e.to_string()),
                }
            }
            ("POST", "/api/interrupt") => {
                let pid = json_body["pid"].as_u64().unwrap_or(0) as u32;
                if pid == 0 {
                    return err(400, "pid required (from session_id 'pid-<pid>')");
                }
                match agent_watcher::process_watcher::interrupt_pid(pid) {
                    Ok(()) => {
                        let agent = json_body["agent"].as_str().unwrap_or("unknown");
                        let sid = json_body["session_id"].as_str().unwrap_or("");
                        app.store.remove_session(agent, sid);
                        ok(serde_json::json!({ "interrupted": pid }))
                    }
                    Err(e) => err(500, &e),
                }
            }
            ("GET", "/api/skills/hydration") => ok(serde_json::to_value(timed_skills::hydration_status(&app.db)).unwrap()),
            ("POST", "/api/skills/hydration/config") => {
                let cfg = timed_skills::HydrationConfig {
                    interval_min: json_body["interval_min"].as_i64().unwrap_or(45),
                    goal_ml: json_body["goal_ml"].as_i64().unwrap_or(2500),
                    cup_ml: json_body["cup_ml"].as_i64().unwrap_or(250),
                    active_start: json_body["active_start"].as_str().unwrap_or("08:00").into(),
                    active_end: json_body["active_end"].as_str().unwrap_or("22:00").into(),
                    enabled: json_body["enabled"].as_bool().unwrap_or(true),
                };
                match timed_skills::hydration_set_cfg(&app.db, &cfg) {
                    Ok(()) => ok(serde_json::to_value(cfg).unwrap()),
                    Err(e) => err(500, &e),
                }
            }
            ("POST", "/api/skills/hydration/log") => {
                let ml = json_body["ml"].as_i64().unwrap_or(250);
                match timed_skills::hydration_log(&app.db, ml) {
                    Ok(()) => ok(serde_json::to_value(timed_skills::hydration_status(&app.db)).unwrap()),
                    Err(e) => err(500, &e),
                }
            }
            ("GET", "/api/reminders") => ok(serde_json::json!({ "reminders": timed_skills::list_reminders(&app.db) })),
            ("POST", "/api/reminders") => {
                let text = json_body["text"].as_str().unwrap_or_default();
                if text.is_empty() {
                    return err(400, "text required");
                }
                match timed_skills::parser::parse_and_store(&app.db, text) {
                    Ok(msg) => ok(serde_json::json!({ "message": msg })),
                    Err(e) => err(500, &e),
                }
            }
            ("DELETE", p) if p.starts_with("/api/reminders/") => {
                let id: i64 = p.rsplit('/').next().and_then(|s| s.parse().ok()).unwrap_or(0);
                match timed_skills::disable_reminder(&app.db, id) {
                    Ok(()) => (200, "{\"ok\":true}".into()),
                    Err(e) => err(500, &e),
                }
            }
            ("GET", "/api/pomodoro") => ok(app.db_pomodoro()),
            ("POST", "/api/pomodoro") => {
                let active = json_body["active"].as_bool().unwrap_or(false);
                match timed_skills::set_pomodoro(&app.db, active) {
                    Ok(()) => ok(app.db_pomodoro()),
                    Err(e) => err(500, &e),
                }
            }
            ("GET", "/api/pets") => {
                let reg = pet_registry::PetRegistry::load();
                ok(serde_json::to_value(&reg.pets).unwrap_or(serde_json::json!([])))
            }
            ("POST", "/api/pets/install") => {
                let spec = json_body["spec"].as_str().unwrap_or_default();
                match pet_registry::download_pet(spec) {
                    Ok(m) => ok(serde_json::to_value(&m).unwrap()),
                    Err(e) => err(400, &e),
                }
            }
            ("GET", "/api/ai/catalog") => ok(serde_json::json!({ "models": ai_router::lightweight_catalog() })),
            ("POST", "/api/ai/detect") => {
                let found = ai_router::detect_local();
                ok(serde_json::json!({ "servers": found }))
            }
            ("POST", "/api/ai/config") => {
                let cfg: ai_router::RouterConfig =
                    serde_json::from_value(json_body).unwrap_or_default();
                match app.db.set_setting("ai_router", &serde_json::to_string(&cfg).unwrap()) {
                    Ok(()) => ok(serde_json::to_value(&cfg).unwrap()),
                    Err(e) => err(500, &e),
                }
            }
            ("GET", "/api/ai/config") => {
                let cfg = load_ai_cfg(&app.db);
                ok(serde_json::to_value(&cfg).unwrap())
            }
            ("POST", "/api/chat") => {
                let msg = json_body["message"].as_str().unwrap_or_default().to_string();
                if msg.is_empty() {
                    return err(400, "message required");
                }
                let cfg = load_ai_cfg(&app.db);
                let history = app.chat_history.lock().unwrap();
                let ctx = sessions_context(app);
                drop(history);
                match ai_router::chat(&cfg, &msg, &ctx) {
                    Ok(reply) => {
                        let mut h = app.chat_history.lock().unwrap();
                        h.push((msg.clone(), reply.clone()));
                        let n = h.len();
                        if n > 50 {
                            h.drain(0..n - 50);
                        }
                        drop(h);
                        app.store.push_event("chat", serde_json::json!({ "q": msg, "a": reply }));
                        ok(serde_json::json!({ "reply": reply }))
                    }
                    Err(e) => err(502, &e),
                }
            }
            ("POST", "/api/quick-command") => {
                let text = json_body["text"].as_str().unwrap_or_default().to_string();
                if text.is_empty() {
                    return err(400, "text required");
                }
                // Deterministic first (reminders / notes / interrupt).
                let lower = text.to_lowercase();
                if lower.starts_with("stop") || lower.starts_with("interrupt") {
                    let target = lower.split_whitespace().nth(1).unwrap_or("");
                    let hit = app
                        .store
                        .live_sessions()
                        .into_iter()
                        .find(|s| s.event.agent.contains(target) || target.is_empty());
                    if let Some(s) = hit {
                        let pid: u32 = s
                            .event
                            .session_id
                            .strip_prefix("pid-")
                            .and_then(|p| p.parse().ok())
                            .unwrap_or(0);
                        if pid > 0 {
                            let _ = agent_watcher::process_watcher::interrupt_pid(pid);
                            app.store.remove_session(&s.event.agent, &s.event.session_id);
                            return ok(serde_json::json!({ "action": "interrupted", "agent": s.event.agent }));
                        }
                        return err(400, "hook-based session: use the Interrupt button on its card");
                    }
                    return err(404, "no matching agent session");
                }
                match timed_skills::parser::parse_and_store(&app.db, &text) {
                    Ok(reply) => {
                        app.store.push_event("quick", serde_json::json!({ "reply": reply }));
                        ok(serde_json::json!({ "reply": reply }))
                    }
                    Err(e) => err(500, &e),
                }
            }
            ("POST", "/api/agents/install") => {
                let results: Vec<serde_json::Value> = agent_watcher::installers::install_all()
                    .into_iter()
                    .map(|(name, r)| match r {
                        Ok(()) => serde_json::json!({ "agent": name, "installed": true }),
                        Err(e) => serde_json::json!({ "agent": name, "installed": false, "error": e }),
                    })
                    .collect();
                ok(serde_json::json!({ "results": results }))
            }
            ("POST", "/api/agents/uninstall") => {
                let results: Vec<serde_json::Value> = agent_watcher::installers::uninstall_all()
                    .into_iter()
                    .map(|(name, r)| match r {
                        Ok(()) => serde_json::json!({ "agent": name, "removed": true }),
                        Err(e) => serde_json::json!({ "agent": name, "removed": false, "error": e }),
                    })
                    .collect();
                ok(serde_json::json!({ "results": results }))
            }
            ("GET", "/api/notes") => ok(app.db_notes()),
            ("GET", _) => err(404, "not found"),
            ("POST", _) => err(404, "not found"),
            _ => err(405, "method not allowed"),
        };
    }
}

fn hook_permission(app: &Arc<App>, agent: &str, body: &serde_json::Value) -> (u16, String) {
    let ask = PermissionAsk {
        id: body["id"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}-{}", agent, uuid_lite())),
        agent: agent.to_string(),
        session_id: body["session_id"].as_str().unwrap_or("unknown").to_string(),
        workspace: body["workspace"].as_str().or_else(|| body["cwd"].as_str()).map(|s| s.to_string()),
        tool: body["tool"].as_str().or_else(|| body["tool_name"].as_str()).map(|s| s.to_string()),
        subject: body["subject"]
            .as_str()
            .or_else(|| body["command"].as_str())
            .or_else(|| body["message"].as_str())
            .map(|s| s.chars().take(400).collect()),
        kind: Some("permission".into()),
        timeout_secs: 590,
    };
    // If the hook gave us a reply URL (OpenCode plugin), relay the decision there.
    let reply_url = body["reply_url"].as_str().map(|s| s.to_string());
    let decision = app.bridge.ask(
        ask,
        std::time::Duration::from_secs(body["timeout_secs"].as_u64().unwrap_or(590).min(600)),
    );
    if let Some(url) = reply_url {
        let payload = serde_json::json!({
            "decision": match decision {
                PermissionDecision::Allow => "allow",
                PermissionDecision::AllowAlways => "allow",
                PermissionDecision::Deny => "deny",
                PermissionDecision::Reply(_) => "allow",
            }
        });
        let _ = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send_json(payload);
    }
    (200, permission_bridge::claude_permission_response(&decision).to_string())
}

fn sessions_context(app: &App) -> String {
    let sessions: Vec<serde_json::Value> = app
        .store
        .live_sessions()
        .iter()
        .map(|s| {
            serde_json::json!({
                "agent": s.event.agent,
                "state": s.state,
                "workspace": s.event.workspace,
            })
        })
        .collect();
    let hyd = timed_skills::hydration_status(&app.db);
    format!(
        "\n\n[Live context] agent sessions: {} | hydration today: {}/{} ml",
        serde_json::to_string(&sessions).unwrap_or_default(),
        hyd.drunk_ml,
        hyd.goal_ml
    )
}

fn load_ai_cfg(db: &Db) -> ai_router::RouterConfig {
    db.get_setting("ai_router")
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn uuid_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ d.as_secs())
        .unwrap_or(0);
    format!("{n:x}{:x}", std::process::id())
}

// Small helpers so route() stays readable.
impl App {
    fn db_pomodoro(&self) -> serde_json::Value {
        timed_skills::pomodoro_state(&self.db)
    }
    fn db_notes(&self) -> serde_json::Value {
        let notes = self
            .db
            .with_conn(|c| {
                let mut stmt = c.prepare("SELECT id, text, ts FROM notes ORDER BY id DESC LIMIT 100")?;
                let rows = stmt
                    .query_map([], |r| {
                        Ok(serde_json::json!({ "id": r.get::<_, i64>(0)?, "text": r.get::<_, String>(1)?, "ts": r.get::<_, i64>(2)? }))
                    })?
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>();
                Ok(rows)
            })
            .unwrap_or_default();
        serde_json::json!({ "notes": notes })
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<String> {
    query.split('&').find_map(|kv| {
        let mut it = kv.splitn(2, '=');
        let k = it.next()?;
        if k == key {
            it.next().map(|v| v.to_string())
        } else {
            None
        }
    })
}

/// Background deterministic scheduler: fires reminders + hydration nudges.
pub fn spawn_scheduler(app: Arc<App>) {
    std::thread::spawn(move || {
        let mut last_minute = 0u64;
        loop {
            if app.stop.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            // Reminders (checked every second for precision on `once`).
            for r in timed_skills::due_reminders(&app.db) {
                app.store.push_event(
                    "reminder",
                    serde_json::json!({ "id": r.id, "text": r.text, "kind": r.kind }),
                );
            }
            // Hydration (once per minute when due).
            let minute = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() / 60)
                .unwrap_or(0);
            if minute != last_minute {
                last_minute = minute;
                let status = timed_skills::hydration_status(&app.db);
                if status.due {
                    app.store.push_event(
                        "hydration",
                        serde_json::json!({
                            "cup_ml": status.config.cup_ml,
                            "drunk_ml": status.drunk_ml,
                            "goal_ml": status.goal_ml,
                            "cups": status.cups,
                            "streak": status.streak,
                        }),
                    );
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
}
