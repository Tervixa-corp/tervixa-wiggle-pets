//! One-click hook/plugin installers — registers native, marker-scoped
//! integrations into each AI coding agent's own config files. All edits are
//! idempotent merges (existing user config is preserved) and reversible
//! (`uninstall_all`).

use serde_json::{json, Value};
use std::path::PathBuf;

use crate::DEFAULT_SERVER_PORT;

pub fn home() -> PathBuf {
    crate::dirs_home()
}

fn exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "wiggle-hook".into())
}

fn hook_cmd(agent: &str, event: &str) -> String {
    // Quote for JSON embedding; agents exec via shell on unix, direct on win.
    let exe = exe_path();
    if cfg!(windows) {
        format!("\"{exe}\" {agent} {event}")
    } else {
        format!("'{exe}' {agent} {event}")
    }
}

fn server_port() -> u16 {
    std::env::var("TERVIXA_WIGGLE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_SERVER_PORT)
}

fn permission_url(agent: &str) -> String {
    format!("http://127.0.0.1:{}/hook/{}/permission", server_port(), agent)
}

/// Read + parse a JSON config file (missing => empty object).
fn read_json(path: &PathBuf) -> Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}))
}

fn write_json(path: &PathBuf, v: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // Backup before first write of this run.
    if path.exists() {
        let _ = std::fs::copy(path, path.with_extension("bak"));
    }
    let body = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(path, body).map_err(|e| e.to_string())
}

const MARKER: &str = "tervixa-wiggle-pets";

fn is_ours(entry: &Value) -> bool {
    entry.get("name").and_then(|n| n.as_str()) == Some(MARKER)
        || entry
            .get("hooks")
            .map(|hs| hs.as_array().map(|a| a.iter().any(is_ours_hook)).unwrap_or(false))
            .unwrap_or(false)
}

fn is_ours_hook(h: &Value) -> bool {
    h.get("name").and_then(|n| n.as_str()) == Some(MARKER)
}

/// Remove marker-scoped entries from a hook map before merging fresh ones.
fn scrub_ours(hooks: &mut Value) {
    if let Some(map) = hooks.as_object_mut() {
        for (_, groups) in map.iter_mut() {
            if let Some(arr) = groups.as_array_mut() {
                arr.retain(|g| !is_ours(g));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Claude Code — ~/.claude/settings.json
// ---------------------------------------------------------------------------

const CLAUDE_STATE_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "Stop",
    "SubagentStart",
    "SubagentStop",
    "PreCompact",
    "PostCompact",
    "Notification",
    "SessionEnd",
];
const CLAUDE_HTTP_EVENTS: &[&str] = &["PermissionRequest", "Elicitation"];

pub fn install_claude() -> Result<(), String> {
    let path = home().join(".claude").join("settings.json");
    let mut cfg = read_json(&path);
    let hooks = cfg
        .as_object_mut()
        .ok_or("settings.json is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));
    scrub_ours(hooks);
    let hook_entry = |inner: Value| json!({ "matcher": "", "name": MARKER, "hooks": [inner] });
    for event in CLAUDE_STATE_EVENTS {
        hooks[event] = json!([]);
        hooks[event]
            .as_array_mut()
            .unwrap()
            .push(hook_entry(json!({
                "type": "command",
                "command": hook_cmd("claude-code", event),
                "timeout": 10
            })));
    }
    for event in CLAUDE_HTTP_EVENTS {
        hooks[event] = json!([]);
        hooks[event]
            .as_array_mut()
            .unwrap()
            .push(hook_entry(json!({
                "type": "http",
                "url": permission_url("claude"),
                "timeout": 600
            })));
    }
    write_json(&path, &cfg)
}

// ---------------------------------------------------------------------------
// Codex CLI — ~/.codex/hooks.json (same merge strategy)
// ---------------------------------------------------------------------------

const CODEX_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PreCompact",
    "PostCompact",
    "Stop",
    "Interrupt",
    "SessionEnd",
];

pub fn install_codex() -> Result<(), String> {
    let path = home().join(".codex").join("hooks.json");
    let mut cfg = read_json(&path);
    let hooks = cfg
        .as_object_mut()
        .ok_or("hooks.json is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));
    scrub_ours(hooks);
    for event in CODEX_EVENTS {
        hooks[event] = json!([]);
        hooks[event].as_array_mut().unwrap().push(json!({
            "name": MARKER,
            "type": "command",
            "command": hook_cmd("codex", event),
            "timeout": if *event == "PermissionRequest" { 600 } else { 10 }
        }));
    }
    write_json(&path, &cfg)
}

// ---------------------------------------------------------------------------
// Cursor — ~/.cursor/hooks.json
// ---------------------------------------------------------------------------

const CURSOR_EVENTS: &[&str] = &[
    "sessionStart",
    "sessionEnd",
    "beforeSubmitPrompt",
    "preToolUse",
    "postToolUse",
    "postToolUseFailure",
    "subagentStart",
    "subagentStop",
    "preCompact",
    "stop",
];

pub fn install_cursor() -> Result<(), String> {
    let path = home().join(".cursor").join("hooks.json");
    let mut cfg = read_json(&path);
    let hooks = cfg
        .as_object_mut()
        .ok_or("hooks.json is not an object")?
        .entry("hooks")
        .or_insert_with(|| json!({}));
    scrub_ours(hooks);
    for event in CURSOR_EVENTS {
        hooks[event] = json!([]);
        hooks[event].as_array_mut().unwrap().push(json!({
            "name": MARKER,
            "command": hook_cmd("cursor", event),
            "timeout": 10
        }));
    }
    write_json(&path, &cfg)
}

// ---------------------------------------------------------------------------
// OpenCode — ~/.config/opencode/plugin/tervixa-wiggle-pets.mjs
// ---------------------------------------------------------------------------

const OPENCODE_PLUGIN: &str = r#"
// TERVIXA WIGGLE PETS — OpenCode integration plugin (installed by wiggle-server).
// Forwards session lifecycle + permission events to the local pet daemon.
// Fail-open: if the daemon is not running, OpenCode behaves exactly as stock.
const SERVER = "__SERVER__";
async function post(path, body) {
  try {
    const ctrl = new AbortController();
    const t = setTimeout(() => ctrl.abort(), 1500);
    const res = await fetch(SERVER + path, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
      signal: ctrl.signal,
    });
    clearTimeout(t);
    return await res.json().catch(() => ({}));
  } catch { return {}; }
}

let last = { state: "", ts: 0 };

export default async function TervixaWigglePets({ client, project, directory }) {
  const ws = directory || project || "";
  async function sendState(sessionID, event, detail, tool) {
    const now = Date.now();
    if (event === last.state && now - last.ts < 700) return; // dedup bursts
    last = { state: event, ts: now };
    await post("/state", {
      agent: "opencode",
      session_id: sessionID || "opencode",
      workspace: ws,
      event,
      detail: detail || null,
      tool: tool || null,
    });
  }
  return {
    event: async ({ event }) => {
      const type = event?.type || "";
      const p = event?.properties || {};
      switch (type) {
        case "session.created":
        case "session.updated":
          break;
        case "session.status":
          await sendState(p.sessionID, "session_status");
          break;
        case "session.idle":
          await sendState(p.sessionID, "session_idle");
          break;
        case "session.error":
          await sendState(p.sessionID, "session_error");
          break;
        case "message.part.updated": {
          const part = p.part || {};
          if (part.type === "tool") {
            await sendState(part.sessionID, part.state === "completed" ? "tool_success" : "tool_called", null, part.tool);
          } else if (part.type === "reasoning") {
            await sendState(part.sessionID, "step_started");
          } else if (part.type === "text") {
            await sendState(part.sessionID, "message_part");
          }
          break;
        }
        case "permission.asked": {
          // Blocking bubble: daemon answers, we relay through the host router.
          const id = p.id || `${Date.now()}`;
          const answer = await post(`/hook/opencode/permission`, {
            id, agent: "opencode",
            session_id: p.sessionID || "opencode",
            workspace: ws,
            tool: p.type || "tool",
            subject: p.title || p.pattern || JSON.stringify(p).slice(0, 300),
          });
          if (answer && answer.behavior) return answer;
          if (answer && answer.action) return answer;
          return; // fail-open: native permission prompt
        }
        default:
          break;
      }
    },
  };
}
"#;

pub fn install_opencode() -> Result<(), String> {
    let dir = home().join(".config").join("opencode").join("plugin");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("tervixa-wiggle-pets.mjs");
    let body = OPENCODE_PLUGIN.replace(
        "__SERVER__",
        &format!("http://127.0.0.1:{}", server_port()),
    );
    std::fs::write(path, body).map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Uninstall
// ---------------------------------------------------------------------------

pub fn uninstall_claude() -> Result<(), String> {
    let path = home().join(".claude").join("settings.json");
    if !path.exists() {
        return Ok(());
    }
    let mut cfg = read_json(&path);
    if let Some(hooks) = cfg.get_mut("hooks") {
        scrub_ours(hooks);
    }
    write_json(&path, &cfg)
}

pub fn uninstall_codex() -> Result<(), String> {
    let path = home().join(".codex").join("hooks.json");
    if !path.exists() {
        return Ok(());
    }
    let mut cfg = read_json(&path);
    if let Some(hooks) = cfg.get_mut("hooks") {
        scrub_ours(hooks);
    }
    write_json(&path, &cfg)
}

pub fn uninstall_cursor() -> Result<(), String> {
    let path = home().join(".cursor").join("hooks.json");
    if !path.exists() {
        return Ok(());
    }
    let mut cfg = read_json(&path);
    if let Some(hooks) = cfg.get_mut("hooks") {
        scrub_ours(hooks);
    }
    write_json(&path, &cfg)
}

pub fn uninstall_opencode() -> Result<(), String> {
    let path = home().join(".config").join("opencode").join("plugin").join("tervixa-wiggle-pets.mjs");
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn install_all() -> Vec<(String, Result<(), String>)> {
    vec![
        ("claude-code".into(), install_claude()),
        ("codex".into(), install_codex()),
        ("cursor".into(), install_cursor()),
        ("opencode".into(), install_opencode()),
    ]
}

pub fn uninstall_all() -> Vec<(String, Result<(), String>)> {
    vec![
        ("claude-code".into(), uninstall_claude()),
        ("codex".into(), uninstall_codex()),
        ("cursor".into(), uninstall_cursor()),
        ("opencode".into(), uninstall_opencode()),
    ]
}
