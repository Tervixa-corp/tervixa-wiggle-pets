//! Tervixa Wiggle core engine.
//!
//! One engine serves both the CLI/TUI (`wiggle-cli`) and the native
//! desktop shell (`wiggle-desktop`), mirroring the OpenCode
//! client/server split: the frontends hold no agent logic of their own.

pub mod agents;
pub mod config;
pub mod identity;
pub mod mcp;
pub mod permissions;
pub mod plugins;
pub mod prompt_compiler;
pub mod providers;
pub mod skills;
pub mod tools;
pub mod undo;

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors surfaced by the core engine.
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("config error: {0}")]
    Config(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("permission denied: {0}")]
    Permission(String),
    #[error("tool error: {0}")]
    Tool(String),
    #[error("mcp error: {0}")]
    Mcp(String),
    #[error("skill error: {0}")]
    Skill(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("browser error: {0}")]
    Browser(#[from] wiggle_browser::BrowserError),
}

pub type CoreResult<T> = Result<T, CoreError>;

/// Streaming events emitted while an agent turn runs. The CLI/TUI, the
/// desktop event bridge and the swarm orchestrator all consume this one
/// enum, so every surface renders the same stream.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    TurnStarted { session: String, agent: String },
    TextDelta { text: String },
    ToolStarted { call: ToolCallView },
    ToolFinished { call: ToolCallView, ok: bool },
    TurnFinished { usage: UsageView },
    AgentFinished { summary: String },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCallView {
    pub id: String,
    pub name: String,
    pub params: serde_json::Value,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UsageView {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

/// A single file mutation recorded while agents work, so the UI can
/// render inline diffs and the undo engine can roll the workspace back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMutation {
    pub path: PathBuf,
    pub before_sha: String,
    pub after_sha: String,
    pub created: bool,
    pub deleted: bool,
}

/// Fully wired engine. Cloning `Engine` is cheap; all mutable state sits
/// behind interior mutability so both frontends can share one instance.
#[derive(Clone)]
pub struct Engine {
    /// Shared engine internals. Visible to the CLI and desktop shells
    /// (both live in this workspace and share the engine contract).
    pub inner: std::sync::Arc<EngineInner>,
}

pub struct EngineInner {
    pub config: Mutex<config::GlobalConfig>,
    pub workspace: PathBuf,
    pub tools: tools::ToolRegistry,
    pub skills: skills::SkillStore,
    pub memory: Option<std::sync::Mutex<wiggle_memory::MemoryStore>>,
    pub undo: Mutex<undo::SnapshotEngine>,
    pub coordinator: agents::swarm::FileCoordinator,
    pub mcp: tokio::sync::Mutex<mcp::McpManager>,
    pub events: tokio::sync::broadcast::Sender<AgentEvent>,
    /// The engine owns its executor. Tasks spawned by the runner, the
    /// swarm and MCP transports run here; hosts with their own runtime
    /// (Tauri) simply await the returned join handles.
    pub runtime: Option<std::sync::Arc<tokio::runtime::Runtime>>,
}

impl Drop for EngineInner {
    fn drop(&mut self) {
        // A tokio Runtime cannot be dropped from within an async context
        // (nested test runtimes, swarm tasks awaiting engine clones).
        // take() the Arc out so field teardown can never be the last
        // reference, and shut it down on a detached thread.
        if let Some(runtime) = self.runtime.take() {
            // Spawn never fails in practice; if it somehow does we leak
            // the runtime rather than panic on shutdown.
            std::thread::spawn(move || drop(runtime));
        }
    }
}

impl Engine {
    /// Open (or create) the engine for a workspace. Loads the global
    /// config from `~/.wiggle/config.toml` merged with
    /// `<workspace>/.wiggle/config.toml`, opens the memory vault, loads
    /// skills (bundled + user + project) and restores MCP connectors.
    pub fn open(workspace: impl Into<PathBuf>) -> CoreResult<Engine> {
        let workspace = workspace.into();
        let mut config = config::GlobalConfig::load()?;
        config.merge_project(&workspace);
        config.ensure_defaults();

        std::fs::create_dir_all(workspace.join(".wiggle"))?;
        std::fs::create_dir_all(config.home_dir.join("skills"))?;

        let memory = match wiggle_memory::MemoryStore::open(&config.memory_db_path()) {
            Ok(store) => Some(std::sync::Mutex::new(store)),
            Err(_) => None, // memory must never block the engine
        };

        let skills = skills::SkillStore::load(
            &config.home_dir,
            &workspace,
            config.bundled_skills_dir.as_deref(),
        )?;

        let registry = tools::ToolRegistry::default();
        let undo = undo::SnapshotEngine::open(&workspace)?;
        let mcp = mcp::McpManager::from_config(&config);
        let runtime =
            tokio::runtime::Builder::new_multi_thread().enable_all().build().map_err(|e| {
                CoreError::Config(format!("failed to start async runtime: {e}"))
            })?;

        let (events, _) = tokio::sync::broadcast::channel(4096);

        Ok(Engine {
            inner: std::sync::Arc::new(EngineInner {
                config: Mutex::new(config),
                workspace,
                tools: registry,
                skills,
                memory,
                undo: Mutex::new(undo),
                coordinator: agents::swarm::FileCoordinator::new(),
                mcp: tokio::sync::Mutex::new(mcp),
                events,
                runtime: Some(std::sync::Arc::new(runtime)),
            }),
        })
    }

    pub fn workspace(&self) -> &PathBuf {
        &self.inner.workspace
    }

    pub fn runtime(&self) -> tokio::runtime::Handle {
        self.inner
            .runtime
            .as_ref()
            .expect("engine runtime detached")
            .handle()
            .clone()
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AgentEvent> {
        self.inner.events.subscribe()
    }

    pub fn config(&self) -> std::sync::MutexGuard<'_, config::GlobalConfig> {
        self.inner.config.lock().expect("config mutex poisoned")
    }

    /// Security gate bound to this workspace and current permissions.
    pub fn permissions_gate(&self) -> permissions::PermissionGate {
        let config = self.config();
        permissions::PermissionGate::new(
            &self.inner.workspace,
            config.workspace_roots.clone(),
            config.permissions.clone(),
        )
    }

    /// Run one user prompt through the full pipeline and return the
    /// final assistant text (the event stream is available separately).
    pub async fn run_prompt(
        &self,
        session: &str,
        prompt: &str,
        agent: Option<&str>,
        model: Option<&str>,
    ) -> CoreResult<String> {
        let runner = agents::runner::AgentRunner::new(self.clone(), session.to_string());
        runner.run(prompt, agent, model).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_constant_is_immutable() {
        assert!(identity::IDENTITY.contains("Tervixa Wiggle"));
        assert!(identity::IDENTITY.contains("Tervixa Corporation"));
    }
}
