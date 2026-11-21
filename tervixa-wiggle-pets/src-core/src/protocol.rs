//! Shared wire protocol between agent hooks, the daemon and the UI.
//!
//! Hooks (Claude Code, Codex, Cursor, OpenCode plugin, process watcher)
//! POST JSON to the daemon:
//!   POST /state      -> AgentStateEvent  (session lifecycle, best-effort)
//!   POST /permission -> PermissionAsk    (blocking: daemon answers decision)
//!   POST /elicitation-> ElicitationAsk   (blocking: question cards)
//! The UI subscribes with long-poll: GET /api/events?since=<seq>.

use serde::{Deserialize, Serialize};

/// Normalized animation states shared by every pet package.
pub const STATES: &[&str] = &[
    "idle",
    "thinking",
    "working",
    "notification",
    "attention",
    "error",
    "sleeping",
    "drinking_water",
    "focus",
    "drag",
];

/// Deterministic event -> state mapping (reference: lifecycle semantics
/// shared by Claude Code / Codex / Cursor / OpenCode hook surfaces).
pub fn event_to_state(event: &str) -> &'static str {
    match event {
        "UserPromptSubmit" | "beforeSubmitPrompt" | "step_started" => "thinking",
        "PreToolUse" | "preToolUse" | "PostToolUse" | "postToolUse" | "tool_called"
        | "tool_success" | "tool_failed_wait" | "message_part" => "working",
        "PostToolUseFailure" | "postToolUseFailure" | "StopFailure" | "session_error"
        | "tool_error" | "execution_failed" => "error",
        "Stop" | "stop" | "PostCompact" | "execution_succeeded" | "session_idle" => "attention",
        "PermissionRequest" | "preToolUsePermission" | "permission_asked" | "Notification"
        | "notification" | "request_user_input" => "notification",
        "PreCompact" | "preCompact" => "working",
        "SubagentStart" | "subagentStart" | "tool_error_juggle" => "working",
        "SessionStart" | "sessionStart" | "session_created" | "session_status" => "working",
        _ => "idle",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateEvent {
    /// Agent identifier: claude-code | codex | cursor | antigravity |
    /// opencode | open-interpreter | custom.
    pub agent: String,
    /// Stable session id (hook-provided or synthesized from pid).
    pub session_id: String,
    /// Workspace / cwd label when available.
    #[serde(default)]
    pub workspace: Option<String>,
    /// Normalized lifecycle event name.
    pub event: String,
    /// Human detail line (tool name, notification text, ...).
    #[serde(default)]
    pub detail: Option<String>,
    /// Tool name for working/error states.
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub subagent_count: Option<u32>,
    #[serde(default)]
    pub timestamp_ms: Option<u64>,
}

impl AgentStateEvent {
    pub fn state(&self) -> &'static str {
        event_to_state(&self.event)
    }
}

/// A blocking permission request coming from an agent hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAsk {
    pub id: String,
    pub agent: String,
    pub session_id: String,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    /// The exact command / diff / question preview.
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub kind: Option<String>, // permission | question
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    600
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionDecision {
    Allow,
    AllowAlways,
    Deny,
    /// Free-form reply used to answer agent questions.
    Reply(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAnswer {
    pub id: String,
    pub decision: PermissionDecision,
}

/// Any UI-visible event (fed to bubbles + pet animation driver).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiEvent {
    pub seq: u64,
    pub kind: String, // state | permission | elicitation | reminder | hydration | chat | system
    #[serde(default)]
    pub payload: serde_json::Value,
    pub timestamp_ms: u64,
}
