//! Permission bridge: the floating approval responder, deterministic
//! auto-approve rule engine, and the interrupter.

use crate::protocol::{PermissionAnswer, PermissionAsk, PermissionDecision};
use crate::state::StateStore;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Deterministic auto-approve policy engine (zero AI)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct PolicyConfig {
    pub enabled: bool,
    /// Read-only commands always auto-approved when enabled.
    pub auto_read_only: bool,
    /// Auto-approve file edits inside trusted roots.
    pub trusted_dirs: Vec<String>,
    /// Dangerous command fragments — never auto-approved, always surfaced.
    pub dangerous: Vec<String>,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        PolicyConfig {
            enabled: false,
            auto_read_only: true,
            trusted_dirs: vec![],
            dangerous: [
                "rm -rf",
                "git push --force",
                "git push -f",
                "sudo",
                "format ",
                "del /f",
                "rd /s",
                "remove-item -recurse",
                "mkfs",
                "drop table",
                ":(){:|:&};:",
                ".ssh",
                "id_rsa",
                ".aws/credentials",
                ".npmrc",
                ".netrc",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }
}

const READ_ONLY: &[&str] = &[
    "ls", "cat", "head", "tail", "grep", "rg", "find", "pwd", "echo", "which", "where",
    "git status", "git diff", "git log", "git show", "git branch", "git remote",
    "cargo check", "cargo test", "cargo fmt", "cargo clippy", "cargo doc",
    "npm test", "npm run lint", "npm ls", "npx tsc", "tsc", "node --version",
    "python --version", "pytest", "rustc --version", "go test", "go vet",
    "type", "dir", "get-childitem", "get-content", "test-path", "wc", "whoami",
];

pub fn decide_automatically(policy: &PolicyConfig, ask: &PermissionAsk) -> Option<PermissionDecision> {
    if !policy.enabled {
        return None;
    }
    let subject = ask
        .subject
        .as_deref()
        .unwrap_or("")
        .to_lowercase();

    // 1. Dangerous? Never auto-approve — surface to the human.
    for d in &policy.dangerous {
        if subject.contains(d) {
            return None;
        }
    }
    // 2. Read-only commands.
    if policy.auto_read_only {
        let first_words: String = subject.trim().to_string();
        for ro in READ_ONLY {
            if first_words.starts_with(ro) {
                return Some(PermissionDecision::Allow);
            }
        }
    }
    // 3. File edits in trusted dirs (subject mentions the trusted root path).
    for t in &policy.trusted_dirs {
        let t = t.to_lowercase();
        if !t.is_empty() && subject.contains(&t) {
            return Some(PermissionDecision::Allow);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Pending permission queue (UI answers via POST /api/permission/answer)
// ---------------------------------------------------------------------------

pub struct PermissionBridge {
    pub policy: std::sync::Mutex<PolicyConfig>,
    pending: std::sync::Mutex<HashMap<String, std::sync::mpsc::Sender<PermissionDecision>>>,
    pub store: Arc<StateStore>,
}

impl PermissionBridge {
    pub fn new(store: Arc<StateStore>) -> Self {
        PermissionBridge {
            policy: std::sync::Mutex::new(PolicyConfig::default()),
            pending: std::sync::Mutex::new(HashMap::new()),
            store,
        }
    }

    /// Blocking entry point used by the HTTP permission hook.
    /// Returns the decision to print back to the agent.
    pub fn ask(&self, ask: PermissionAsk, timeout: Duration) -> PermissionDecision {
        // Deterministic policy pass first.
        if let Some(d) = {
            let policy = self.policy.lock().unwrap();
            decide_automatically(&policy, &ask)
        } {
            self.store.push_event(
                "permission",
                serde_json::json!({
                    "resolved": "auto",
                    "decision": d,
                    "ask": ask,
                }),
            );
            return d;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        self.pending.lock().unwrap().insert(ask.id.clone(), tx);
        self.store.push_event(
            "permission",
            serde_json::json!({ "resolved": "pending", "ask": ask }),
        );
        let decision = rx
            .recv_timeout(timeout)
            .unwrap_or(PermissionDecision::Deny); // fail-closed watchdog
        self.pending.lock().unwrap().remove(&ask.id);
        decision
    }

    pub fn answer(&self, answer: PermissionAnswer) -> Result<(), String> {
        let tx = self.pending.lock().unwrap().remove(&answer.id);
        if let Some(tx) = tx {
            tx.send(answer.decision.clone()).map_err(|_| "responder gone".to_string())
        } else {
            Err("no such pending permission".into())
        }
    }
}

// ---------------------------------------------------------------------------
// Hook response rendering (what the daemon prints back to Claude Code)
// ---------------------------------------------------------------------------

pub fn claude_permission_response(d: &PermissionDecision) -> serde_json::Value {
    let decision = match d {
        PermissionDecision::Allow | PermissionDecision::AllowAlways => {
            serde_json::json!({ "behavior": "allow" })
        }
        PermissionDecision::Deny => serde_json::json!({
            "behavior": "deny",
            "message": "Denied from Tervixa Wiggle Pets"
        }),
        PermissionDecision::Reply(text) => serde_json::json!({
            "behavior": "allow",
            "updatedInput": { "answer": text }
        }),
    };
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PermissionRequest",
            "decision": decision
        }
    })
}
