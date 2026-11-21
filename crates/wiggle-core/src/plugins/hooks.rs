//! Deterministic plugin lifecycle hooks, mirroring the Claude Code /
//! Codex hook event set. Each hook is a command run with the event JSON
//! on stdin; JSON stdout `{ "decision": "deny", "reason": "..." }`
//! blocks the action.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::HookConfig;
use crate::{CoreError, CoreResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    Notification,
    SubagentStop,
    Stop,
}

impl HookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookEvent::UserPromptSubmit => "user_prompt_submit",
            HookEvent::PreToolUse => "pre_tool_use",
            HookEvent::PostToolUse => "post_tool_use",
            HookEvent::Notification => "notification",
            HookEvent::SubagentStop => "subagent_stop",
            HookEvent::Stop => "stop",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HookOutcome {
    /// true = continue (approve), false = block.
    pub approved: bool,
    #[serde(default)]
    pub reason: String,
    /// Hook may replace the user prompt text (UserPromptSubmit only).
    #[serde(default)]
    pub prompt_override: Option<String>,
}

impl HookOutcome {
    pub fn approve() -> HookOutcome {
        HookOutcome { approved: true, reason: String::new(), prompt_override: None }
    }
}

/// Run every enabled hook for an event. `payload` carries event
/// details (tool name, params, prompt text, session, cwd...).
pub async fn run_hooks(
    hooks: &[HookConfig],
    event: HookEvent,
    payload: serde_json::Value,
) -> CoreResult<HookOutcome> {
    let mut final_outcome = HookOutcome::approve();
    for hook in hooks.iter().filter(|h| h.enabled && h.event == event.as_str()) {
        let payload = build_payload(hook, event, payload.clone());
        let outcome = run_one(hook, &payload).await?;
        if !outcome.approved {
            return Ok(outcome); // first deny wins
        }
        if let Some(p) = outcome.prompt_override {
            final_outcome.prompt_override = Some(p);
        }
    }
    Ok(final_outcome)
}

fn build_payload(hook: &HookConfig, event: HookEvent, payload: serde_json::Value) -> String {
    json!({
        "hook": {
            "event": event.as_str(),
            "command": hook.command,
        },
        "event": payload,
        "tool": "tervixa-wiggle",
        "version": env!("CARGO_PKG_VERSION"),
    })
    .to_string()
}

async fn run_one(hook: &HookConfig, stdin_payload: &str) -> CoreResult<HookOutcome> {
    let command_line = hook.command.clone();
    let payload = stdin_payload.to_string();
    // Hook processes are synchronous std children; run them on the
    // blocking pool with a timeout guard.
    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        tokio::task::spawn_blocking(move || run_sync(&command_line, &payload)),
    )
    .await
    .map_err(|_| CoreError::Tool(format!("hook timed out: {}", hook.command)))?
    .map_err(|e| CoreError::Tool(format!("hook task: {e}")))?
}

fn run_sync(command_line: &str, stdin_payload: &str) -> CoreResult<HookOutcome> {
    use std::process::Stdio;
    let mut command = make_shell_command(command_line);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|e| CoreError::Tool(format!("hook spawn failed: {e}")))?;
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        // Write then drop without blocking: hook payloads are small.
        let _ = stdin.write_all(stdin_payload.as_bytes());
        let _ = stdin.flush();
    }
    let output = child
        .wait_with_output()
        .map_err(|e| CoreError::Tool(format!("hook run failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        // Exit status decides when no JSON verdict is printed.
        return Ok(HookOutcome {
            approved: output.status.success(),
            reason: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            prompt_override: None,
        });
    }
    // Try JSON verdict first.
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        let decision = value.get("decision").and_then(|d| d.as_str());
        let approved = match decision {
            Some("deny") | Some("block") => false,
            Some("allow") | Some("approve") | None => output.status.success(),
            _ => output.status.success(),
        };
        return Ok(HookOutcome {
            approved,
            reason: value
                .get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or_default()
                .to_string(),
            prompt_override: value
                .get("prompt")
                .or_else(|| value.get("updatedPrompt"))
                .and_then(|p| p.as_str())
                .map(str::to_string),
        });
    }
    Ok(HookOutcome { approved: output.status.success(), reason: String::new(), prompt_override: None })
}

#[cfg(windows)]
fn make_shell_command(command_line: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut c = std::process::Command::new("cmd");
    c.arg("/C");
    // cmd.exe parses quotes natively; std's arg escaping would corrupt
    // the inner `"` characters, so append the command line verbatim.
    c.raw_arg(command_line);
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    c
}

#[cfg(not(windows))]
fn make_shell_command(command_line: &str) -> std::process::Command {
    let mut c = std::process::Command::new("/bin/sh");
    c.arg("-c").arg(command_line);
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deny_verdict_blocks() {
        let hook = HookConfig {
            event: "pre_tool_use".into(),
            command: if cfg!(windows) {
                "echo {\"decision\":\"deny\",\"reason\":\"no writes\"}".into()
            } else {
                "echo '{\"decision\":\"deny\",\"reason\":\"no writes\"}'".into()
            },
            enabled: true,
        };
        let outcome = run_hooks(&[hook], HookEvent::PreToolUse, json!({"tool": "write_file"}))
            .await
            .unwrap();
        assert!(!outcome.approved);
        assert_eq!(outcome.reason, "no writes");
    }

    #[tokio::test]
    async fn failing_hook_denies() {
        let hook = HookConfig {
            event: "user_prompt_submit".into(),
            command: if cfg!(windows) { "exit /b 1".into() } else { "exit 1".into() },
            enabled: true,
        };
        let outcome = run_hooks(&[hook], HookEvent::UserPromptSubmit, json!({})).await.unwrap();
        assert!(!outcome.approved);
    }

    #[tokio::test]
    async fn empty_stdout_approves() {
        let hook = HookConfig {
            event: "stop".into(),
            command: if cfg!(windows) { "exit /b 0".into() } else { "exit 0".into() },
            enabled: true,
        };
        let outcome = run_hooks(&[hook], HookEvent::Stop, json!({})).await.unwrap();
        assert!(outcome.approved);
    }
}
