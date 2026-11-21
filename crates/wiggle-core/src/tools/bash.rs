//! Persistent cross-platform shell executor with timeout, streaming
//! output capture and command allow/deny gating.

use std::sync::Arc;

use serde_json::json;
use tokio::sync::Mutex as AsyncMutex;

use super::{ToolDef, ToolOutput};
use crate::permissions::Decision;
use crate::{CoreError, CoreResult};

/// One process-wide persistent shell session: the working directory
/// carries across commands (like Claude Code's shell snapshotting).
static GLOBAL_SHELL: AsyncMutex<Option<ShellSession>> = AsyncMutex::const_new(None);

/// A persistent shell session: one working directory carried across
/// commands (like Claude Code's shell snapshotting).
pub struct ShellSession {
    pub cwd: std::path::PathBuf,
    pub last_exit: Option<i32>,
}

impl ShellSession {
    pub fn new(workspace: &std::path::Path) -> ShellSession {
        ShellSession { cwd: workspace.to_path_buf(), last_exit: None }
    }

    fn shell_command(command: &str) -> std::process::Command {
        if cfg!(windows) {
            let mut cmd = std::process::Command::new("powershell");
            cmd.args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", command]);
            cmd
        } else {
            let mut cmd = std::process::Command::new("sh");
            cmd.arg("-c").arg(command);
            cmd
        }
    }

    /// Run one command, stream-combining stdout/stderr, with timeout.
    pub async fn run(
        &mut self,
        engine: &crate::Engine,
        command: &str,
        timeout_secs: u64,
        cwd: Option<String>,
    ) -> CoreResult<(i32, String)> {
        let decision = engine.permissions_gate().check_command(command);
        match decision {
            Decision::Allowed => {}
            Decision::NeedsApproval { reason } | Decision::Denied { reason } => {
                return Err(CoreError::Permission(reason));
            }
        }
        if let Some(dir) = cwd {
            self.cwd = engine.workspace().join(dir);
        }

        let mut cmd = Self::shell_command(command);
        cmd.current_dir(&self.cwd);
        cmd.env("WIGGLE", "1");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            tokio::task::spawn_blocking(move || cmd.output()),
        )
        .await;

        let result = match output {
            Ok(Ok(Ok(out))) => out,
            Ok(Ok(Err(e))) => return Err(CoreError::Tool(format!("shell spawn: {e}"))),
            Ok(Err(e)) => return Err(CoreError::Tool(format!("shell task: {e}"))),
            Err(_) => {
                self.last_exit = Some(-1);
                return Err(CoreError::Tool(format!(
                    "command timed out after {timeout_secs}s: {command}"
                )));
            }
        };

        let exit = result.status.code().unwrap_or(-1);
        self.last_exit = Some(exit);
        let mut combined = String::from_utf8_lossy(&result.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&result.stderr);
        if !stderr.trim().is_empty() {
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(stderr.trim_end());
        }
        Ok((exit, combined))
    }
}

pub fn definitions() -> Vec<ToolDef> {
    vec![ToolDef {
        name: "bash",
        description: "Execute a shell command in the workspace (PowerShell on Windows, sh on Unix). Output is captured with a timeout; the working directory persists between calls.",
        parameters: super::obj_schema(json!({
            "command": { "type": "string" },
            "timeout_secs": { "type": "integer", "description": "Default 120" },
            "cwd": { "type": "string", "description": "Optional directory relative to workspace root" }
        })),
        mutating: true,
        run: Arc::new(|ctx, args| Box::pin(async move {
            let command = args["command"].as_str().ok_or_else(|| CoreError::Tool("missing 'command'".into()))?.to_string();
            let timeout = args["timeout_secs"].as_u64().unwrap_or(120).clamp(1, 900);
            let cwd = args["cwd"].as_str().map(str::to_string);
            let mut guard = GLOBAL_SHELL.lock().await;
            let session = guard.get_or_insert_with(|| ShellSession::new(ctx.engine.workspace()));
            let (exit, output) = session.run(&ctx.engine, &command, timeout, cwd).await?;
            let truncated = if output.len() > 20_000 {
                format!("{}\n... (output truncated)", &output[..20_000])
            } else {
                output
            };
            Ok(ToolOutput::text(format!("exit {exit}\n{truncated}")))
        })),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_engine(name: &str) -> crate::Engine {
        let tmp =
            std::env::temp_dir().join(format!("wiggle-bash-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let engine = crate::Engine::open(tmp).unwrap();
        engine.config().permissions.mode =
            Some(crate::config::SandboxMode::FullAuto);
        engine
    }

    #[tokio::test]
    async fn runs_commands_and_captures_exit() {
        let engine = test_engine("run");
        let mut shell = ShellSession::new(engine.workspace());
        let (exit, out) = shell.run(&engine, "echo hello", 30, None).await.unwrap();
        assert_eq!(exit, 0);
        assert!(out.contains("hello"));
    }

    #[tokio::test]
    async fn denylist_blocks_commands() {
        let engine = test_engine("deny");
        engine.config().permissions.command_denylist.push("echo".into());
        let mut shell = ShellSession::new(engine.workspace());
        let err = shell.run(&engine, "echo blocked", 30, None).await.unwrap_err();
        assert!(err.to_string().contains("denylist"));
    }
}
