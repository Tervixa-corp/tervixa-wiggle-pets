//! Universal AI coding agent watcher.
//!
//! Two layers, both deterministic:
//!   1. Hook integration — registers native hooks//plugins into each agent's
//!      config so lifecycle + permission events stream to the daemon.
//!   2. Process watcher — polls the OS process table for agent binaries as a
//!      fallback so pets also react to agents without hooks installed.

pub mod installers;
pub mod process_watcher;

use crate::state::StateStore;
use std::sync::Arc;

pub const AGENTS: &[(&str, &str, &[&str])] = &[
    ("claude-code", "Claude Code", &["claude", "claude.exe"]),
    ("codex", "Codex CLI", &["codex", "codex.exe"]),
    ("cursor", "Cursor Agent", &["cursor-agent", "cursor-agent.exe", "cursor.cmd"]),
    ("antigravity", "Antigravity", &["antigravity", "antigravity.exe", "language_server_windows.exe"]),
    ("opencode", "OpenCode", &["opencode", "opencode.exe"]),
    ("open-interpreter", "Open Interpreter", &["interpreter", "interpreter.exe"]),
];

pub fn agent_display(agent: &str) -> &str {
    AGENTS
        .iter()
        .find(|(id, _, _)| *id == agent)
        .map(|(_, name, _)| *name)
        .unwrap_or(agent)
}

pub struct Watcher {
    pub store: Arc<StateStore>,
    pub stop: Arc<std::sync::atomic::AtomicBool>,
}

impl Watcher {
    pub fn new(store: Arc<StateStore>) -> Self {
        Watcher {
            store,
            stop: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Spawn the fallback process watcher loop (1 Hz, ~0% CPU).
    pub fn spawn(&self) {
        let store = self.store.clone();
        let stop = self.stop.clone();
        std::thread::spawn(move || {
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                for (id, _, procs) in AGENTS {
                    for p in process_watcher::find_processes(procs) {
                        store.apply_process_watch(id, p);
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });
    }
}
