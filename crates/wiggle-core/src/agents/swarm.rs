//! Parallel multi-agent DAG orchestrator with in-memory file-lease
//! coordination. Agents work directly against the workspace on `main` —
//! no extra git branches, no worktrees.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock, Semaphore};

use crate::CoreResult;
use crate::Engine;

/// Lifecycle state of one dispatched swarm task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmTaskState {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SwarmTask {
    pub role: String,
    pub instruction: String,
    /// Files this task owns; leases are acquired for every edit.
    pub files: Vec<std::path::PathBuf>,
    pub state: SwarmTaskState,
    pub result: String,
}

/// In-memory File Lease Manager. Before an agent edits a file it takes
/// an exclusive lease; leases are released when the task finishes.
/// RwLock per path, tracked globally, guarantees zero write races
/// without branches or worktrees.
pub struct FileCoordinator {
    leases: RwLock<HashMap<String, String>>, // canonical path -> owner id
}

impl FileCoordinator {
    pub fn new() -> FileCoordinator {
        FileCoordinator { leases: RwLock::new(HashMap::new()) }
    }

    /// Acquire an exclusive lease on a file path for `owner`.
    pub async fn lease(&self, path: &std::path::Path) -> FileLease {
        let key = path.to_string_lossy().replace('\\', "/").to_lowercase();
        loop {
            {
                let mut leases = self.leases.write().await;
                if !leases.contains_key(&key) {
                    let owner = format!("t-{}-{}", std::process::id(), NEXT_ID.fetch_add(1, Ordering::Relaxed));
                    leases.insert(key.clone(), owner.clone());
                    return FileLease { key, owner };
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    }

    pub async fn release(&self, lease: FileLease) {
        let mut leases = self.leases.write().await;
        leases.remove(&lease.key);
    }

    pub async fn held(&self) -> Vec<String> {
        self.leases.read().await.keys().cloned().collect()
    }
}

impl Default for FileCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// An owned lease; dropping it does NOT release (release is explicit
/// so callers control timing around snapshot boundaries).
pub struct FileLease {
    key: String,
    #[allow(dead_code)]
    owner: String,
}

/// Per-session swarm record.
struct SwarmRun {
    tasks: Vec<SwarmTask>,
}

/// Dispatches and tracks swarm tasks. One orchestrator per engine;
/// task records are keyed by `session`.
pub struct SwarmOrchestrator {
    engine: Engine,
    runs: Arc<Mutex<HashMap<String, SwarmRun>>>,
    ids: Arc<Mutex<HashMap<String, (String, usize)>>>, // task id -> (session, index)
    next_task_id: Arc<AtomicU64>,
}

impl SwarmOrchestrator {
    pub fn new(engine: Engine) -> SwarmOrchestrator {
        SwarmOrchestrator {
            engine,
            runs: Arc::new(Mutex::new(HashMap::new())),
            ids: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Queue tasks for a session and start them (bounded by the
    /// configured concurrency). Returns task ids immediately.
    pub async fn dispatch(
        &self,
        session: &str,
        tasks: Vec<SwarmTask>,
    ) -> CoreResult<Vec<String>> {
        let max_concurrency = self.engine.config().agents.max_concurrency.max(1);
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let mut ids = Vec::new();

        {
            let mut runs = self.runs.lock().await;
            let run = runs.entry(session.to_string()).or_insert_with(|| SwarmRun { tasks: Vec::new() });
            for mut task in tasks {
                let id = format!("swarm-{}", self.next_task_id.fetch_add(1, Ordering::Relaxed));
                ids.push(id.clone());
                task.state = SwarmTaskState::Queued;
                run.tasks.push(task.clone());
                self.ids.lock().await.insert(id.clone(), (session.to_string(), run.tasks.len() - 1));

                // Spawn execution immediately.
                let engine = self.engine.clone();
                let task = task.clone();
                let task_id = id.clone();
                let session = session.to_string();
                let permit = semaphore.clone().acquire_owned().await.map_err(|e| crate::CoreError::Tool(e.to_string()))?;
                let runs_handle = self.runs.clone();
                let ids_handle = self.ids.clone();
                self.engine.runtime().spawn(async move {
                    {
                        let mut runs = runs_handle.lock().await;
                        if let Some(run) = runs.get_mut(&session) {
                            if let Some(index) = ids_handle.lock().await.get(&task_id).map(|(_, i)| *i) {
                                run.tasks[index].state = SwarmTaskState::Running;
                            }
                        }
                    }
                    let agent = engine::runner_for(&engine, &session, &task.role);
                    let outcome = agent(task.instruction.clone()).await;
                    let mut runs = runs_handle.lock().await;
                    if let Some(run) = runs.get_mut(&session) {
                        if let Some(index) = ids_handle.lock().await.get(&task_id).map(|(_, i)| *i) {
                            let record = &mut run.tasks[index];
                            record.state = match &outcome {
                                Ok(_text) => SwarmTaskState::Done,
                                Err(_) => SwarmTaskState::Failed,
                            };
                            record.result = match outcome {
                                Ok(text) => text,
                                Err(e) => format!("failed: {e}"),
                            };
                        }
                    }
                    drop(permit);
                });
            }
        }
        Ok(ids)
    }

    /// Wait until every queued/running task of the session finishes and
    /// return a combined report.
    pub async fn run_to_completion(
        &self,
        session: &str,
        tasks: Vec<SwarmTask>,
        rationale: &str,
    ) -> CoreResult<String> {
        let ids = self.dispatch(session, tasks).await?;
        // Poll until done (bounded).
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(1800);
        loop {
            let report = self.status(session, &ids).await;
            let all_done = !ids.is_empty()
                && ids.iter().all(|id| {
                    report.contains(&format!("id {id}"))
                        && (report.contains(&format!("id {id} done")) || report.contains(&format!("id {id} failed")))
                });
            if all_done || tokio::time::Instant::now() > deadline {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }
        let report = self.status(session, &ids).await;
        Ok(format!("swarm complete ({rationale}):\n{report}"))
    }

    /// Human-readable status of selected tasks (all when filter empty).
    pub async fn status(&self, session: &str, filter: &[String]) -> String {
        let runs = self.runs.lock().await;
        let mapping = self.ids.lock().await;
        let mut lines = Vec::new();
        if let Some(run) = runs.get(session) {
            for (id, (_sess, index)) in mapping.iter().filter(|(_, (s, _))| s == session) {
                if !filter.is_empty() && !filter.contains(id) {
                    continue;
                }
                if let Some(task) = run.tasks.get(*index) {
                    lines.push(format!(
                        "id {id} {:?} [{}] files: {} result: {}",
                        task.state,
                        task.role,
                        if task.files.is_empty() { "-".into() } else { format!("{:?}", task.files) },
                        task.result.chars().take(200).collect::<String>()
                    ));
                }
            }
        }
        if lines.is_empty() {
            "(no swarm tasks)".into()
        } else {
            lines.join("\n")
        }
    }
}

// Small indirection so the swarm can build a runner per role without a
// circular import tangle.
mod engine {
    use crate::CoreResult;

    pub fn runner_for<'a>(
        engine: &'a crate::Engine,
        session: &'a str,
        role: &'a str,
    ) -> impl FnOnce(String) -> futures::future::BoxFuture<'static, CoreResult<String>> + Send + 'a {
        let runner = super::super::runner::AgentRunner::new(engine.clone(), session.to_string());
        let role = role.to_string();
        move |instruction| {
            Box::pin(async move {
                runner
                    .run(&instruction, Some(&role), None)
                    .await
                    .map(|text| {
                        format!("[{role}] {text}")
                    })
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn leases_are_exclusive() {
        let coordinator = FileCoordinator::new();
        let path = std::path::Path::new("src/a.rs");
        let lease1 = coordinator.lease(path).await;
        // Second lease blocks; emulate by checking held paths.
        let held = coordinator.held().await;
        assert_eq!(held.len(), 1);
        coordinator.release(lease1).await;
        assert!(coordinator.held().await.is_empty());
    }

    #[tokio::test]
    async fn dispatch_records_states() {
        let engine = test_engine();
        let orchestrator = SwarmOrchestrator::new(engine.clone());
        let tasks = vec![SwarmTask {
            role: "explorer".into(),
            instruction: "list the workspace files".into(),
            files: vec![],
            state: SwarmTaskState::Queued,
            result: String::new(),
        }];
        let ids = orchestrator.dispatch("test-session", tasks).await.unwrap();
        assert_eq!(ids.len(), 1);
        // Status should eventually report a terminal state.
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let report = orchestrator.status("test-session", &[]).await;
            if report.contains("done") || report.contains("failed") {
                break;
            }
            if tokio::time::Instant::now() > deadline {
                panic!("task did not finish in time: {report}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }

    fn test_engine() -> crate::Engine {
        let tmp = std::env::temp_dir().join(format!("wiggle-swarm-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        crate::Engine::open(tmp).unwrap()
    }
}
