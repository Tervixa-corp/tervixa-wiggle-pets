//! Filesystem tools: read_file, write_file, edit_file, multi_edit,
//! glob, grep, list_dir — all sandbox-validated and snapshot-protected.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;

use super::{ToolContext, ToolDef, ToolOutput};
use crate::permissions::Decision;
use crate::{CoreError, CoreResult};

fn check(ctx: &ToolContext, tool: &str, path: &str, mutating: bool) -> CoreResult<PathBuf> {
    let gate = ctx.engine.permissions_gate();
    let decision = gate.validate_path(std::path::Path::new(path));
    match decision {
        Decision::Allowed => {}
        Decision::NeedsApproval { reason } | Decision::Denied { reason } => {
            return Err(CoreError::Permission(reason));
        }
    }
    if mutating {
        match gate.check_tool(tool, true) {
            Decision::Allowed => {}
            Decision::NeedsApproval { reason } | Decision::Denied { reason } => {
                return Err(CoreError::Permission(reason));
            }
        }
    }
    Ok(PathBuf::from(path))
}

fn resolve(ctx: &ToolContext, path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        ctx.engine.workspace().join(p)
    }
}

pub fn definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "read_file",
            description: "Read a text file from the workspace. Returns the full content.",
            parameters: super::obj_schema(json!({
                "path": { "type": "string", "description": "File path relative to workspace root" },
                "start_line": { "type": "integer", "description": "Optional 1-based start line" },
                "end_line": { "type": "integer", "description": "Optional 1-based end line (inclusive)" }
            })),
            mutating: false,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let path = args["path"].as_str().ok_or_else(|| CoreError::Tool("missing 'path'".into()))?;
                check(&ctx, "read_file", path, false)?;
                let full = resolve(&ctx, path);
                let raw = std::fs::read_to_string(&full)
                    .map_err(|e| CoreError::Tool(format!("read {path}: {e}")))?;
                let start = args["start_line"].as_u64().unwrap_or(1).max(1) as usize;
                let end = args["end_line"].as_u64().map(|e| e as usize);
                let selected: String = raw
                    .lines()
                    .enumerate()
                    .skip(start - 1)
                    .take(end.map(|e| e - start + 1).unwrap_or(usize::MAX))
                    .map(|(i, l)| format!("{:>5}: {l}\n", i + 1))
                    .collect();
                Ok(ToolOutput::text(if selected.is_empty() { "(empty or out of range)".into() } else { selected }))
            })),
        },
        ToolDef {
            name: "write_file",
            description: "Create or overwrite a text file in the workspace. An undo snapshot is recorded automatically.",
            parameters: super::obj_schema(json!({
                "path": { "type": "string", "description": "File path relative to workspace root" },
                "content": { "type": "string", "description": "Full new file content" }
            })),
            mutating: true,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let path = args["path"].as_str().ok_or_else(|| CoreError::Tool("missing 'path'".into()))?.to_string();
                let content = args["content"].as_str().unwrap_or_default();
                let rel = check(&ctx, "write_file", &path, true)?;
                let lease = ctx.engine.inner.coordinator.lease(&rel).await;
                let result = write_with_snapshot(&ctx, &path, content).await;
                ctx.engine.inner.coordinator.release(lease).await;
                result
            })),
        },
        ToolDef {
            name: "edit_file",
            description: "Replace the first exact occurrence of a string in a file with new text.",
            parameters: super::obj_schema(json!({
                "path": { "type": "string" },
                "old": { "type": "string", "description": "Exact existing text" },
                "new": { "type": "string", "description": "Replacement text" },
                "replace_all": { "type": "boolean", "description": "Replace every occurrence (default false)" }
            })),
            mutating: true,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let path = args["path"].as_str().ok_or_else(|| CoreError::Tool("missing 'path'".into()))?.to_string();
                let old = args["old"].as_str().ok_or_else(|| CoreError::Tool("missing 'old'".into()))?;
                let new = args["new"].as_str().unwrap_or_default().to_string();
                let replace_all = args["replace_all"].as_bool().unwrap_or(false);
                check(&ctx, "edit_file", &path, true)?;
                let lease = ctx.engine.inner.coordinator.lease(std::path::Path::new(&path)).await;
                let result = apply_edits(&ctx, &path, vec![(old.to_string(), new, replace_all)]).await;
                ctx.engine.inner.coordinator.release(lease).await;
                result
            })),
        },
        ToolDef {
            name: "multi_edit",
            description: "Apply several exact find/replace edits to one file in a single atomic operation.",
            parameters: super::obj_schema(json!({
                "path": { "type": "string" },
                "edits": {
                    "type": "array",
                    "description": "Array of {old, new, replace_all?} objects applied in order",
                    "items": {
                        "type": "object",
                        "properties": {
                            "old": { "type": "string" },
                            "new": { "type": "string" },
                            "replace_all": { "type": "boolean" }
                        }
                    }
                }
            })),
            mutating: true,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let path = args["path"].as_str().ok_or_else(|| CoreError::Tool("missing 'path'".into()))?.to_string();
                check(&ctx, "multi_edit", &path, true)?;
                let mut edits = Vec::new();
                for e in args["edits"].as_array().into_iter().flatten() {
                    edits.push((
                        e["old"].as_str().ok_or_else(|| CoreError::Tool("edit missing 'old'".into()))?.to_string(),
                        e["new"].as_str().unwrap_or_default().to_string(),
                        e["replace_all"].as_bool().unwrap_or(false),
                    ));
                }
                let lease = ctx.engine.inner.coordinator.lease(std::path::Path::new(&path)).await;
                let result = apply_edits(&ctx, &path, edits).await;
                ctx.engine.inner.coordinator.release(lease).await;
                result
            })),
        },
        ToolDef {
            name: "glob",
            description: "List workspace files matching a glob pattern (e.g. 'src/**/*.rs').",
            parameters: super::obj_schema(json!({
                "pattern": { "type": "string" }
            })),
            mutating: false,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let pattern = args["pattern"].as_str().ok_or_else(|| CoreError::Tool("missing 'pattern'".into()))?;
                let root = ctx.engine.workspace();
                let mut results = Vec::new();
                collect_matching(root, root, pattern, &mut results, 0)?;
                results.sort();
                let out = if results.is_empty() {
                    "(no matches)".to_string()
                } else {
                    results.iter().take(500).map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n")
                };
                Ok(ToolOutput::text(out))
            })),
        },
        ToolDef {
            name: "grep",
            description: "Regex search across workspace files. Returns 'path:line: text' matches.",
            parameters: super::obj_schema(json!({
                "pattern": { "type": "string", "description": "Regular expression" },
                "include": { "type": "string", "description": "Optional glob filter like '*.rs'" }
            })),
            mutating: false,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let pattern = args["pattern"].as_str().ok_or_else(|| CoreError::Tool("missing 'pattern'".into()))?;
                let include = args["include"].as_str().map(str::to_string);
                let re = regex::Regex::new(pattern)
                    .map_err(|e| CoreError::Tool(format!("bad regex: {e}")))?;
                let mut results = Vec::new();
                let root = ctx.engine.workspace().clone();
                grep_recursive(&root, &root, &re, include.as_deref(), &mut results, 0);
                let out = if results.is_empty() {
                    "(no matches)".to_string()
                } else {
                    results.iter().take(300).cloned().collect::<Vec<_>>().join("\n")
                };
                Ok(ToolOutput::text(out))
            })),
        },
        ToolDef {
            name: "list_dir",
            description: "List the immediate entries of a workspace directory.",
            parameters: super::obj_schema(json!({
                "path": { "type": "string", "description": "Directory path ('.' for workspace root)" }
            })),
            mutating: false,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let path = args["path"].as_str().unwrap_or(".");
                check(&ctx, "read_file", path, false)?;
                let full = resolve(&ctx, path);
                let mut lines = Vec::new();
                for entry in std::fs::read_dir(&full)
                    .map_err(|e| CoreError::Tool(format!("list {path}: {e}")))?
                    .flatten()
                {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let kind = if entry.path().is_dir() { "dir " } else { "file" };
                    lines.push(format!("{kind} {name}"));
                }
                lines.sort();
                Ok(ToolOutput::text(if lines.is_empty() { "(empty)".into() } else { lines.join("\n") }))
            })),
        },
    ]
}

async fn write_with_snapshot(ctx: &ToolContext, path: &str, content: &str) -> CoreResult<ToolOutput> {
    let full = resolve(ctx, path);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let engine = ctx.engine.clone();
    let paths = vec![full.clone()];
    let label = format!("write_file {path}");
    let workspace = engine.workspace().clone();
    let text = content.to_string();
    // Snapshot capture is synchronous SQLite I/O; run it inline (the
    // connection is not Send, so it cannot cross into spawn_blocking).
    {
        let mut undo = engine.inner.undo.lock().expect("undo mutex");
        undo.capture(&workspace, &label, &paths)?;
    }

    std::fs::write(&full, text).map_err(|e| CoreError::Tool(format!("write {path}: {e}")))?;
    Ok(ToolOutput::text(format!("wrote {} ({} bytes)", path, content.len())))
}

async fn apply_edits(
    ctx: &ToolContext,
    path: &str,
    edits: Vec<(String, String, bool)>,
) -> CoreResult<ToolOutput> {
    let full = resolve(ctx, path);
    let raw = std::fs::read_to_string(&full)
        .map_err(|e| CoreError::Tool(format!("read {path}: {e}")))?;
    let mut content = raw.clone();
    for (old, new, replace_all) in &edits {
        if !content.contains(old.as_str()) {
            return Err(CoreError::Tool(format!(
                "edit failed: '{}' not found in {path}",
                truncate_snippet(old)
            )));
        }
        if *replace_all {
            content = content.replace(old.as_str(), new);
        } else {
            content = content.replacen(old.as_str(), new, 1);
        }
    }
    let engine = ctx.engine.clone();
    let paths = vec![full.clone()];
    let label = format!("edit_file {path}");
    let workspace = engine.workspace().clone();
    {
        let mut undo = engine.inner.undo.lock().expect("undo mutex");
        undo.capture(&workspace, &label, &paths)?;
    }

    std::fs::write(&full, content).map_err(|e| CoreError::Tool(format!("write {path}: {e}")))?;
    Ok(ToolOutput::text(format!("applied {} edit(s) to {path}", edits.len())))
}

fn truncate_snippet(s: &str) -> String {
    let mut s = s.chars().take(40).collect::<String>();
    if s.len() < s.chars().count() {
        s.push('…');
    }
    s
}

fn collect_matching(
    root: &std::path::Path,
    dir: &std::path::Path,
    pattern: &str,
    out: &mut Vec<PathBuf>,
    depth: usize,
) -> CoreResult<()> {
    if depth > 12 || out.len() > 2000 {
        return Ok(());
    }
    // The `glob` crate is intentionally avoided: a small hand-rolled
    // matcher handles ** / * / ? which is all agents emit.
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        let p = entry.path();
        let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
        if p.is_dir() {
            collect_matching(root, &p, pattern, out, depth + 1)?;
        } else if glob_match(pattern, &rel) {
            out.push(PathBuf::from(rel));
        }
    }
    Ok(())
}

/// Minimal glob supporting `**`, `*`, `?` and `/` separators.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    fn rec(p: &[u8], t: &[u8]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        if p.starts_with(b"**") {
            let rest = &p[if p.len() > 2 && p[2] == b'/' { 3 } else { 2 }..];
            // `**` matches zero or more path segments.
            for i in 0..=t.len() {
                if rec(rest, &t[i..]) {
                    return true;
                }
            }
            return false;
        }
        if t.is_empty() {
            return false;
        }
        match p[0] {
            // A single `*` must not cross a `/` separator (`**` above does).
            b'*' if t[0] != b'/' => rec(&p[1..], &t[1..]) || rec(p, &t[1..]),
            b'*' => false,
            b'?' => rec(&p[1..], &t[1..]),
            c => c == t[0] && rec(&p[1..], &t[1..]),
        }
    }
    rec(pattern.as_bytes(), text.as_bytes())
}

fn grep_recursive(
    root: &std::path::Path,
    dir: &std::path::Path,
    re: &regex::Regex,
    include: Option<&str>,
    out: &mut Vec<String>,
    depth: usize,
) {
    if depth > 12 || out.len() > 300 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if name == "target" || name == "node_modules" || name == ".git" || name == "dist" {
                continue;
            }
            grep_recursive(root, &p, re, include, out, depth + 1);
        } else if let Ok(raw) = std::fs::read_to_string(&p) {
            if let Some(inc) = include {
                let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                if !glob_match(inc, &name) {
                    continue;
                }
            }
            let rel = p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/");
            for (i, line) in raw.lines().enumerate() {
                if re.is_match(line) {
                    out.push(format!("{rel}:{}: {}", i + 1, line.trim()));
                    if out.len() > 300 {
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_common_patterns() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("src/**/*.rs", "src/tools/fs.rs"));
        assert!(glob_match("**/*.ts", "apps/desktop-ui/src/App.tsx") == false);
        assert!(glob_match("**/*.tsx", "apps/desktop-ui/src/App.tsx"));
        assert!(!glob_match("*.rs", "src/main.rs"));
        assert!(glob_match("a?c.txt", "abc.txt"));
    }

    #[test]
    fn grep_skips_ignored_dirs() {
        let tmp = std::env::temp_dir().join("wiggle-grep-test");
        let _ = std::fs::remove_dir_all(&tmp);
        let sub = tmp.join("node_modules").join("dep");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("x.js"), "findme").unwrap();
        std::fs::write(tmp.join("a.txt"), "findme here").unwrap();
        let re = regex::Regex::new("findme").unwrap();
        let mut out = Vec::new();
        grep_recursive(&tmp, &tmp, &re, None, &mut out, 0);
        assert_eq!(out.len(), 1);
        assert!(out[0].contains("a.txt"));
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
