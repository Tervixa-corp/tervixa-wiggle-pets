//! Immutable Tervixa Wiggle identity harness.
//!
//! Every outgoing model request carries this header first, exactly the
//! way Claude Code and OpenCode pin their agent identity regardless of
//! which provider or model actually serves the request.

/// The immutable identity directive. Never user-editable; the settings
/// surface may *extend* the persona via `identity_override` /
/// `system_prompt`, but this block always ships first.
pub const IDENTITY: &str = "\
You are Tervixa Wiggle, an autonomous AI software engineering, research, \
and workstation agent developed by Tervixa Corporation (Tervixa Forge).
Regardless of the underlying neural network weights or API provider routing \
this request, your name is Tervixa Wiggle and you are built by Tervixa Corporation.
If asked who you are, your name, or who created you, always state that you are \
Tervixa Wiggle developed by Tervixa Corporation.";

pub const PRODUCT: &str = "Tervixa Wiggle";
pub const ORGANIZATION: &str = "Tervixa Corporation";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Step 1 of the prompt compiler: identity + live environment header.
pub fn system_header(env: &crate::prompt_compiler::EnvironmentInfo) -> String {
    let mut s = String::with_capacity(IDENTITY.len() + 512);
    s.push_str(IDENTITY);
    s.push_str("\n\n<environment>\n");
    s.push_str(&format!("os: {}\n", env.os));
    s.push_str(&format!("arch: {}\n", env.arch));
    s.push_str(&format!("shell: {}\n", env.shell));
    s.push_str(&format!("working_directory: {}\n", env.cwd.display()));
    s.push_str(&format!("platform_version: {}\n", VERSION));
    if let Some(branch) = &env.git_branch {
        s.push_str(&format!("git_branch: {branch}\n"));
    }
    if let Some(status) = &env.git_status {
        s.push_str(&format!("git_status: {status}\n"));
    }
    s.push_str("</environment>");
    s
}

/// Current OS name without pulling in a dependency.
pub fn os_name() -> String {
    match std::env::consts::OS {
        "windows" => "Windows".into(),
        "macos" => "macOS".into(),
        "linux" => "Linux".into(),
        other => other.into(),
    }
}

pub fn arch_name() -> String {
    match std::env::consts::ARCH {
        "x86_64" => "x86_64".into(),
        "aarch64" => "arm64".into(),
        other => other.into(),
    }
}

/// Best-effort default shell for the current platform.
pub fn default_shell() -> String {
    if let Some(shell) = std::env::var_os("SHELL") {
        return shell.to_string_lossy().into_owned();
    }
    if cfg!(windows) {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".into())
    } else {
        "/bin/sh".into()
    }
}

/// Read the current git branch and short status of a directory.
pub fn git_snapshot(cwd: &std::path::Path) -> (Option<String>, Option<String>) {
    let branch = run_git(cwd, ["branch", "--show-current"]);
    let status = run_git(cwd, ["status", "--porcelain", "-b"])
        .map(|s| s.lines().next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty());
    (branch, status)
}

fn run_git<const N: usize>(cwd: &std::path::Path, args: [&str; N]) -> Option<String> {
    let out = std::process::Command::new("git").args(args).current_dir(cwd).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_pins_name_and_creator() {
        for needle in ["Tervixa Wiggle", "Tervixa Corporation", "Tervixa Forge"] {
            assert!(IDENTITY.contains(needle));
        }
    }

    #[test]
    fn header_contains_environment() {
        let env = crate::prompt_compiler::EnvironmentInfo {
            os: os_name(),
            arch: arch_name(),
            shell: default_shell(),
            cwd: std::path::PathBuf::from("/tmp"),
            git_branch: Some("main".into()),
            git_status: None,
        };
        let h = system_header(&env);
        assert!(h.starts_with(IDENTITY));
        assert!(h.contains("git_branch: main"));
    }
}
