//! Live session state — the single source of truth the UI polls.

use crate::protocol::AgentStateEvent;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Session {
    pub event: AgentStateEvent,
    pub state: String,
    pub updated_ms: u64,
    /// Monotonic activity marker for process-watcher fallbacks.
    pub from_process_watch: bool,
}

pub struct StateStore {
    sessions: Mutex<HashMap<String, Session>>,
    seq: std::sync::atomic::AtomicU64,
    event_log: Mutex<Vec<crate::protocol::UiEvent>>,
    start_ms: std::time::Instant,
}

fn now_ms(start: &std::time::Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

impl StateStore {
    pub fn new() -> Self {
        StateStore {
            sessions: Mutex::new(HashMap::new()),
            seq: std::sync::atomic::AtomicU64::new(0),
            event_log: Mutex::new(Vec::new()),
            start_ms: std::time::Instant::now(),
        }
    }

    pub fn next_seq(&self) -> u64 {
        self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }

    pub fn push_event(&self, kind: &str, payload: serde_json::Value) -> crate::protocol::UiEvent {
        let ev = crate::protocol::UiEvent {
            seq: self.next_seq(),
            kind: kind.to_string(),
            payload,
            timestamp_ms: now_ms(&self.start_ms),
        };
        let mut log = self.event_log.lock().unwrap();
        log.push(ev.clone());
        // Keep the log bounded.
        if log.len() > 2000 {
            log.drain(0..500);
        }
        ev
    }

    pub fn events_since(&self, since: u64) -> Vec<crate::protocol::UiEvent> {
        let log = self.event_log.lock().unwrap();
        log.iter().filter(|e| e.seq > since).cloned().collect()
    }

    pub fn apply(&self, ev: AgentStateEvent) {
        let key = format!("{}:{}", ev.agent, ev.session_id);
        let state = ev.state().to_string();
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(
            key,
            Session {
                state: state.clone(),
                updated_ms: now_ms(&self.start_ms),
                event: ev.clone(),
                from_process_watch: false,
            },
        );
        drop(sessions);
        self.push_event(
            "state",
            serde_json::json!({
                "agent": ev.agent,
                "session_id": ev.session_id,
                "workspace": ev.workspace,
                "state": state,
                "detail": ev.detail,
                "tool": ev.tool,
                "subagent_count": ev.subagent_count,
            }),
        );
    }

    /// Fallback session synthesized by the process watcher.
    pub fn apply_process_watch(&self, agent: &str, pid: u32) {
        let key = format!("{agent}:pid-{pid}");
        let mut sessions = self.sessions.lock().unwrap();
        if sessions.contains_key(&key) {
            // refresh heartbeat only
            if let Some(s) = sessions.get_mut(&key) {
                s.updated_ms = now_ms(&self.start_ms);
            }
            return;
        }
        let ev = AgentStateEvent {
            agent: agent.to_string(),
            session_id: format!("pid-{pid}"),
            workspace: None,
            event: "SessionStart".into(),
            detail: Some("detected running (no hooks installed)".into()),
            tool: None,
            subagent_count: None,
            timestamp_ms: None,
        };
        sessions.insert(
            key,
            Session {
                state: "idle".into(),
                updated_ms: now_ms(&self.start_ms),
                event: ev,
                from_process_watch: true,
            },
        );
    }

    /// Sessions with a heartbeat newer than `ttl_ms` (hook telemetry expires
    /// after 10 min, process-watch entries after 15 s of missed polls).
    pub fn live_sessions(&self) -> Vec<Session> {
        let now = now_ms(&self.start_ms);
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|_, s| now.saturating_sub(s.updated_ms) < if s.from_process_watch { 15_000 } else { 600_000 });
        sessions.values().cloned().collect()
    }

    pub fn remove_session(&self, agent: &str, session_id: &str) {
        let key = format!("{agent}:{session_id}");
        self.sessions.lock().unwrap().remove(&key);
        self.push_event(
            "state",
            serde_json::json!({ "agent": agent, "session_id": session_id, "state": "ended" }),
        );
    }
}
