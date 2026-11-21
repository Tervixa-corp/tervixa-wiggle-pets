//! Allow / Ask / Deny security gate and workspace path validator.
//!
//! The three-tier sandbox mirrors Codex (suggest / auto-edit /
//! full-auto) with Claude Code's per-tool rules layered on top.

use std::path::{Path, PathBuf};

use crate::config::{Permissions, SandboxMode, ToolPermission};

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Allowed,
    NeedsApproval { reason: String },
    Denied { reason: String },
}

/// Static gate evaluated before any tool runs.
pub struct PermissionGate {
    pub workspace: PathBuf,
    pub extra_roots: Vec<PathBuf>,
    pub permissions: Permissions,
}

impl PermissionGate {
    pub fn new(workspace: &Path, extra_roots: Vec<PathBuf>, permissions: Permissions) -> PermissionGate {
        PermissionGate {
            workspace: workspace.to_path_buf(),
            extra_roots,
            permissions,
        }
    }

    /// Sandbox tier in effect for tool execution.
    pub fn mode(&self) -> SandboxMode {
        self.permissions.mode.clone().unwrap_or(SandboxMode::AutoEdit)
    }

    /// Validate a filesystem path stays inside the sandbox.
    pub fn validate_path(&self, path: &Path) -> Decision {
        let path = match absolutize(path) {
            Ok(p) => p,
            Err(e) => return Decision::Denied { reason: e },
        };
        for root in std::iter::once(self.workspace.clone()).chain(self.extra_roots.iter().cloned())
        {
            // Roots live through the same normalization so POSIX-style
            // test roots and Windows rooted paths compare fairly.
            let root = match absolutize(&root) {
                Ok(r) => r,
                Err(_) => root.clone(),
            };
            if path.starts_with(&root) {
                return Decision::Allowed;
            }
        }
        Decision::Denied {
            reason: format!(
                "path '{}' escapes the workspace sandbox",
                path.display()
            ),
        }
    }

    /// Gate a tool call. `mutating` tools are downgraded per sandbox tier.
    pub fn check_tool(&self, tool: &str, mutating: bool) -> Decision {
        let explicit = self.permissions.tools.get(tool);
        match explicit {
            Some(ToolPermission::Deny) => {
                return Decision::Denied { reason: format!("tool '{tool}' is denied in settings") };
            }
            Some(ToolPermission::Allow) => return Decision::Allowed,
            Some(ToolPermission::Ask) => {
                return Decision::NeedsApproval { reason: format!("tool '{tool}' requires approval") };
            }
            None => {}
        }
        if mutating {
            return match self.mode() {
                SandboxMode::Suggest => Decision::NeedsApproval {
                    reason: "sandbox is in suggest mode: edits require approval".into(),
                },
                SandboxMode::AutoEdit | SandboxMode::FullAuto => Decision::Allowed,
            };
        }
        Decision::Allowed
    }

    /// Gate one shell command against the allow/deny lists.
    pub fn check_command(&self, command: &str) -> Decision {
        let cmd = command.trim().to_lowercase();
        for deny in &self.permissions.command_denylist {
            if cmd.starts_with(&deny.to_lowercase()) {
                return Decision::Denied { reason: format!("command matches denylist entry '{deny}'") };
            }
        }
        for allow in &self.permissions.command_allowlist {
            if cmd.starts_with(&allow.to_lowercase()) {
                return Decision::Allowed;
            }
        }
        match self.mode() {
            SandboxMode::FullAuto => Decision::Allowed,
            _ => Decision::NeedsApproval {
                reason: format!("command requires approval: {command}"),
            },
        }
    }
}

fn absolutize(path: &Path) -> Result<PathBuf, String> {
    // std::path::absolute resolves relative and rooted (e.g. `/ws`) paths
    // against the current directory / current drive on every platform.
    let p = std::path::absolute(path).map_err(|e| e.to_string())?;
    // Lexically normalize `..` segments without requiring the path to exist.
    let mut out = PathBuf::new();
    for component in p.components() {
        match component {
            std::path::Component::ParentDir => {
                if !out.pop() || out.as_os_str().is_empty() {
                    return Err("path escapes root via '..'".into());
                }
            }
            c => out.push(c),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gate(mode: SandboxMode) -> PermissionGate {
        PermissionGate::new(Path::new("/ws"), Vec::new(), Permissions { mode: Some(mode), ..Default::default() })
    }

    #[test]
    fn workspace_paths_pass_and_escaped_fail() {
        let g = gate(SandboxMode::FullAuto);
        assert_eq!(g.validate_path(Path::new("/ws/src/a.rs")), Decision::Allowed);
        assert!(matches!(g.validate_path(Path::new("/etc/passwd")), Decision::Denied { .. }));
    }

    #[test]
    fn traversal_is_normalized() {
        let g = gate(SandboxMode::FullAuto);
        assert_eq!(g.validate_path(Path::new("/ws/src/../Cargo.toml")), Decision::Allowed);
        assert!(matches!(g.validate_path(Path::new("/ws/../../etc/passwd")), Decision::Denied { .. }));
    }

    #[test]
    fn sandbox_tiers_downgrade_mutating_tools() {
        let g = gate(SandboxMode::Suggest);
        assert!(matches!(g.check_tool("write_file", true), Decision::NeedsApproval { .. }));
        assert_eq!(g.check_tool("read_file", false), Decision::Allowed);

        let g = gate(SandboxMode::AutoEdit);
        assert_eq!(g.check_tool("write_file", true), Decision::Allowed);
        assert!(matches!(g.check_command("cargo build"), Decision::NeedsApproval { .. }));
    }

    #[test]
    fn command_lists_win_over_mode() {
        let mut permissions = Permissions { mode: Some(SandboxMode::AutoEdit), ..Default::default() };
        permissions.command_allowlist.push("cargo".into());
        permissions.command_denylist.push("rm -rf".into());
        let g = PermissionGate::new(Path::new("/ws"), Vec::new(), permissions);
        assert_eq!(g.check_command("cargo test"), Decision::Allowed);
        assert!(matches!(g.check_command("rm -rf /"), Decision::Denied { .. }));
    }
}
