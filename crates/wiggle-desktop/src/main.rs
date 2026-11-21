//! Tervixa Wiggle desktop shell — Tauri v2 IPC bridge.
//!
//! The React workbench (apps/desktop-ui) talks to the engine through
//! these commands and streams every AgentEvent over the `agent://`
//! event channel; the window holds zero agent logic (OpenCode desktop
//! pattern).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use wiggle_core::config::GlobalConfig;
use wiggle_core::Engine;

type Cmd<T> = Result<T, String>;

#[derive(Clone)]
struct EngineState {
    engine: Engine,
}

#[tauri::command]
async fn engine_status(state: tauri::State<'_, EngineState>) -> Cmd<serde_json::Value> {
    let engine = &state.engine;
    let config = engine.config().clone();
    let mcp_ready = engine.inner.mcp.lock().await.ready_count();
    Ok(serde_json::json!({
        "product": wiggle_core::identity::PRODUCT,
        "version": wiggle_core::identity::VERSION,
        "workspace": engine.workspace().display().to_string(),
        "skills": engine.inner.skills.count(),
        "mcp_ready": mcp_ready,
        "providers": config.providers.iter().map(|p| serde_json::json!({
            "id": p.id,
            "label": p.label,
            "api": p.api,
            "base_url": p.base_url,
            "models": p.models.iter().map(|m| m.id.clone()).collect::<Vec<_>>(),
            "enabled": p.enabled,
            "has_key": p.env_keys.iter().any(|k| std::env::var(k).is_ok()) || p.api_key.is_some(),
        })).collect::<Vec<_>>(),
        "default_provider": config.default_provider,
        "default_model": config.default_model,
    }))
}

#[tauri::command]
fn engine_settings(state: tauri::State<EngineState>) -> Cmd<GlobalConfig> {
    Ok(state.engine.config().clone())
}

#[tauri::command]
fn save_settings(state: tauri::State<EngineState>, config: GlobalConfig) -> Cmd<()> {
    let mut current = state.engine.config();
    *current = config;
    current.ensure_defaults();
    current.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn send_prompt(
    state: tauri::State<'_, EngineState>,
    app: tauri::AppHandle,
    session: String,
    prompt: String,
    agent: Option<String>,
    model: Option<String>,
) -> Cmd<String> {
    let engine = state.engine.clone();
    // Stream engine events to the UI over the `agent://` event channel.
    let mut events = engine.subscribe();
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        use tauri::Emitter;
        while let Ok(event) = events.recv().await {
            let _ = app_handle.emit("agent://", &event);
        }
    });
    engine
        .run_prompt(&session, &prompt, agent.as_deref(), model.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct SwarmTaskInput {
    pub role: String,
    pub instruction: String,
    #[serde(default)]
    pub files: Vec<String>,
}

#[tauri::command]
async fn spawn_swarm(
    state: tauri::State<'_, EngineState>,
    session: String,
    tasks: Vec<SwarmTaskInput>,
) -> Cmd<Vec<String>> {
    let engine = state.engine.clone();
    let orchestrator = wiggle_core::agents::swarm::SwarmOrchestrator::new(engine);
    let swarm_tasks: Vec<wiggle_core::agents::swarm::SwarmTask> = tasks
        .into_iter()
        .map(|t| wiggle_core::agents::swarm::SwarmTask {
            role: t.role,
            instruction: t.instruction,
            files: t.files.into_iter().map(std::path::PathBuf::from).collect(),
            state: wiggle_core::agents::swarm::SwarmTaskState::Queued,
            result: String::new(),
        })
        .collect();
    orchestrator
        .dispatch(&session, swarm_tasks)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn swarm_status(
    state: tauri::State<'_, EngineState>,
    session: String,
) -> Cmd<String> {
    let engine = state.engine.clone();
    let orchestrator = wiggle_core::agents::swarm::SwarmOrchestrator::new(engine);
    Ok(orchestrator.status(&session, &[]).await)
}

#[tauri::command]
fn read_workspace_file(state: tauri::State<EngineState>, path: String) -> Cmd<String> {
    let engine = &state.engine;
    let full = engine.workspace().join(&path);
    std::fs::read_to_string(full).map_err(|e| format!("read {path}: {e}"))
}

#[tauri::command]
fn list_workspace_tree(state: tauri::State<EngineState>) -> Cmd<serde_json::Value> {
    let engine = &state.engine;
    Ok(tree_json(engine.workspace(), 0))
}

fn tree_json(dir: &std::path::Path, depth: usize) -> serde_json::Value {
    let mut children = Vec::new();
    if depth < 6 {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                let skip = matches!(
                    name.as_str(),
                    "target" | "node_modules" | ".git" | "dist" | ".wiggle"
                );
                if skip {
                    continue;
                }
                let is_dir = path.is_dir();
                children.push(serde_json::json!({
                    "name": name,
                    "dir": is_dir,
                    "children": if is_dir { tree_json(&path, depth + 1) } else { serde_json::Value::Null },
                }));
            }
        }
    }
    serde_json::Value::Array(children)
}

#[tauri::command]
fn write_workspace_file(
    state: tauri::State<EngineState>,
    path: String,
    content: String,
) -> Cmd<()> {
    let engine = &state.engine;
    let full = engine.workspace().join(&path);
    if !full.starts_with(engine.workspace()) {
        return Err("path escapes workspace".into());
    }
    if let Some(parent) = full.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(full, content).map_err(|e| format!("write {path}: {e}"))
}

#[tauri::command]
fn undo_workspace(state: tauri::State<EngineState>) -> Cmd<Option<String>> {
    let engine = &state.engine;
    let mut undo = engine.inner.undo.lock().expect("undo");
    undo.undo(engine.workspace()).map_err(|e| e.to_string())
}

#[tauri::command]
fn redo_workspace(state: tauri::State<EngineState>) -> Cmd<Option<String>> {
    let engine = &state.engine;
    let mut undo = engine.inner.undo.lock().expect("undo");
    undo.redo(engine.workspace()).map_err(|e| e.to_string())
}

#[tauri::command]
fn memory_entries(
    state: tauri::State<EngineState>,
    limit: usize,
) -> Cmd<Vec<wiggle_memory::MemoryEntry>> {
    match &state.engine.inner.memory {
        Some(memory) => {
            let store = memory.lock().expect("memory");
            store.list(limit).map_err(|e| e.to_string())
        }
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
fn memory_add(state: tauri::State<EngineState>, content: String) -> Cmd<i64> {
    match &state.engine.inner.memory {
        Some(memory) => {
            let mut store = memory.lock().expect("memory");
            store.add_fact(&content).map_err(|e| e.to_string())
        }
        None => Err("memory vault unavailable".into()),
    }
}

#[tauri::command]
fn memory_delete(state: tauri::State<EngineState>, id: i64) -> Cmd<()> {
    match &state.engine.inner.memory {
        Some(memory) => {
            let mut store = memory.lock().expect("memory");
            store.delete(id).map_err(|e| e.to_string())
        }
        None => Err("memory vault unavailable".into()),
    }
}

#[tauri::command]
fn memory_pin(state: tauri::State<EngineState>, id: i64, pinned: bool) -> Cmd<()> {
    match &state.engine.inner.memory {
        Some(memory) => {
            let mut store = memory.lock().expect("memory");
            store.pin(id, pinned).map_err(|e| e.to_string())
        }
        None => Err("memory vault unavailable".into()),
    }
}

#[tauri::command]
async fn import_skills_github(
    state: tauri::State<'_, EngineState>,
    repo: String,
) -> Cmd<usize> {
    let _ = state;
    let mut parts = repo.splitn(2, '/');
    let (owner, name) = match (parts.next(), parts.next()) {
        (Some(o), Some(r)) if !o.is_empty() && !r.is_empty() => (o.to_string(), r.to_string()),
        _ => return Err("repo must be owner/name".into()),
    };
    // Run off the IPC thread.
    tauri::async_runtime::spawn(async move {
        wiggle_core::skills::importer::import_github(&owner, &name).await
    })
    .await
    .map_err(|e| e.to_string())?
    .map(|list| list.len())
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn import_skills_claude(state: tauri::State<EngineState>) -> Cmd<usize> {
    let _ = state;
    let source = wiggle_core::skills::importer::ImportSource::ClaudeCode;
    match wiggle_core::skills::importer::home_skills_dir(&source) {
        Some(dir) if dir.exists() => wiggle_core::skills::importer::import_dir(&dir)
            .map(|list| list.len())
            .map_err(|e| e.to_string()),
        _ => Ok(0),
    }
}

#[tauri::command]
async fn connect_mcp_all(state: tauri::State<'_, EngineState>) -> Cmd<Vec<serde_json::Value>> {
    let engine = state.engine.clone();
    let mut manager = engine.inner.mcp.lock().await;
    manager.connect_all().await;
    Ok(manager
        .states()
        .iter()
        .map(|s| {
            serde_json::json!({
                "name": s.config.name,
                "transport": s.config.transport,
                "connected": s.connected,
                "error": s.error,
                "tools": s.tools.len(),
                "resources": s.resources.len(),
                "prompts": s.prompts.len(),
            })
        })
        .collect::<Vec<_>>())
}

#[tauri::command]
fn mcp_catalog() -> Cmd<Vec<serde_json::Value>> {
    Ok(wiggle_core::mcp::registry::CATALOG
        .iter()
        .map(|c| serde_json::json!({ "name": c.name, "description": c.description }))
        .collect())
}

fn main() {
    let engine = match Engine::open(std::env::current_dir().unwrap_or_else(|_| ".".into())) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("engine failed to start: {e}");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .manage(EngineState { engine })
        .invoke_handler(tauri::generate_handler![
            engine_status,
            engine_settings,
            save_settings,
            send_prompt,
            spawn_swarm,
            swarm_status,
            read_workspace_file,
            write_workspace_file,
            list_workspace_tree,
            undo_workspace,
            redo_workspace,
            memory_entries,
            memory_add,
            memory_delete,
            memory_pin,
            import_skills_claude,
            import_skills_github,
            connect_mcp_all,
            mcp_catalog,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
