//! Content-addressable snapshot engine powering instant /undo and
//! /redo (OpenCode-style rollback, but filesystem-level and git-free).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::CoreResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub label: String,
    pub created_at: String,
    /// path -> content (None = file did not exist before).
    pub files: Vec<(String, Option<String>)>,
}

pub struct SnapshotEngine {
    dir: PathBuf,
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
}

impl SnapshotEngine {
    pub fn open(workspace: &Path) -> CoreResult<SnapshotEngine> {
        let dir = workspace.join(".wiggle").join("snapshots");
        std::fs::create_dir_all(&dir)?;
        Ok(SnapshotEngine { dir, undo_stack: Vec::new(), redo_stack: Vec::new() })
    }

    pub fn content_id(data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        hex::encode(&hash[..12])
    }

    /// Capture the current state of the given paths (missing files are
    /// recorded as absent so undo can delete them again).
    pub fn capture(&mut self, workspace: &Path, label: &str, paths: &[PathBuf]) -> CoreResult<String> {
        let mut files = Vec::new();
        for path in paths {
            let rel = path
                .strip_prefix(workspace)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let content = std::fs::read(path).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
            files.push((rel, content));
        }
        let id = format!(
            "{}-{}",
            chrono::Utc::now().format("%Y%m%d%H%M%S%3f"),
            Self::content_id(label.as_bytes())
        );
        let snapshot = Snapshot {
            id: id.clone(),
            label: label.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            files,
        };
        // Persist for crash recovery.
        let raw = serde_json::to_vec(&snapshot)?;
        std::fs::write(self.dir.join(format!("{id}.json")), raw)?;
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
        Ok(id)
    }

    /// Restore the most recent snapshot; returns its label. The
    /// about-to-be-reverted state is captured as the inverse snapshot so
    /// redo can re-apply the change.
    pub fn undo(&mut self, workspace: &Path) -> CoreResult<Option<String>> {
        let Some(snapshot) = self.undo_stack.pop() else {
            return Ok(None);
        };
        let label = snapshot.label.clone();
        let inverse = self.read_snapshot(workspace, &label, &snapshot)?;
        self.restore(workspace, &snapshot)?;
        self.redo_stack.push(inverse);
        Ok(Some(label))
    }

    /// Re-apply the most recently undone snapshot.
    pub fn redo(&mut self, workspace: &Path) -> CoreResult<Option<String>> {
        let Some(snapshot) = self.redo_stack.pop() else {
            return Ok(None);
        };
        let label = snapshot.label.clone();
        let inverse = self.read_snapshot(workspace, &label, &snapshot)?;
        self.restore(workspace, &snapshot)?;
        self.undo_stack.push(inverse);
        Ok(Some(label))
    }

    /// Snapshot of the *current* contents of the files covered by
    /// `model` — used as the inverse of the operation being undone/redone.
    fn read_snapshot(
        &self,
        workspace: &Path,
        label: &str,
        model: &Snapshot,
    ) -> CoreResult<Snapshot> {
        let paths: Vec<PathBuf> = model
            .files
            .iter()
            .map(|(rel, _)| workspace.join(rel))
            .collect();
        let mut files = Vec::new();
        for path in paths {
            let rel = path
                .strip_prefix(workspace)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let content =
                std::fs::read(&path).ok().map(|b| String::from_utf8_lossy(&b).into_owned());
            files.push((rel, content));
        }
        let id = format!(
            "{}-{}",
            chrono::Utc::now().format("%Y%m%d%H%M%S%3f"),
            Self::content_id(label.as_bytes())
        );
        Ok(Snapshot {
            id,
            label: label.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            files,
        })
    }

    fn restore(&self, workspace: &Path, snapshot: &Snapshot) -> CoreResult<()> {
        for (rel, content) in &snapshot.files {
            let path = workspace.join(rel);
            match content {
                Some(text) => {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(path, text)?;
                }
                None => {
                    let _ = std::fs::remove_file(path); // absent before snapshot
                }
            }
        }
        Ok(())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn history(&self) -> Vec<(String, String)> {
        self.undo_stack.iter().map(|s| (s.id.clone(), s.label.clone())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpws(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wiggle-undo-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn undo_restores_previous_content_and_redo_reapplies() {
        let ws = tmpws("redo");
        let mut engine = SnapshotEngine::open(&ws).unwrap();
        let file = ws.join("a.txt");
        std::fs::write(&file, "v1").unwrap();

        engine.capture(&ws, "write v2", &[file.clone()]).unwrap();
        std::fs::write(&file, "v2").unwrap();

        assert!(engine.undo(&ws).unwrap().is_some());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "v1");

        assert!(engine.redo(&ws).unwrap().is_some());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "v2");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn undo_restores_deleted_files() {
        let ws = tmpws("deleted");
        let mut engine = SnapshotEngine::open(&ws).unwrap();
        let file = ws.join("b.txt");
        std::fs::write(&file, "data").unwrap();

        engine.capture(&ws, "delete b", &[file.clone()]).unwrap();
        std::fs::remove_file(&file).unwrap();
        assert!(!file.exists());

        engine.undo(&ws).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "data");
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn empty_stack_returns_none() {
        let ws = tmpws("empty");
        let mut engine = SnapshotEngine::open(&ws).unwrap();
        assert!(engine.undo(&ws).unwrap().is_none());
        assert!(engine.redo(&ws).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&ws);
    }
}
