//! Agent definitions: built-in role profiles plus custom user agents.

use serde::{Deserialize, Serialize};

/// A serializable agent profile (built-ins are provided; users add
/// their own via Settings or `.wiggle/agents/*.toml`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentDef {
    pub name: String,
    pub role: String,
    pub description: String,
    /// Extra system instructions appended after the identity header.
    #[serde(default)]
    pub instructions: String,
    /// Tool names this agent may use; empty = all native + MCP tools.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    #[serde(default)]
    pub can_spawn_subagents: bool,
}

pub fn builtin_definitions() -> Vec<AgentDef> {
    vec![
        AgentDef {
            name: "agent".into(),
            role: "orchestrator".into(),
            description: "General-purpose orchestrator: plans work, runs tools, coordinates subagents".into(),
            instructions: "You are the orchestrator. Decompose the user's request, act directly for simple work, and dispatch parallel subagents via spawn_subagents for multi-file or research-heavy tasks. Always verify your edits (build/tests) before reporting completion.".into(),
            allowed_tools: vec![],
            can_spawn_subagents: true,
        },
        AgentDef {
            name: "architect".into(),
            role: "architect".into(),
            description: "Analyzes the repository, plans multi-file changes, coordinates subagents".into(),
            instructions: "You are the architect. Study the codebase first (list_dir, glob, grep, read_file), produce a precise plan with file paths, then either implement it via coder subagents or hand the plan back. Never edit files without a lease on them.".into(),
            allowed_tools: vec![],
            can_spawn_subagents: true,
        },
        AgentDef {
            name: "coder".into(),
            role: "coder".into(),
            description: "Implements code edits, writes tests, runs build verification".into(),
            instructions: "You are the coder. Make surgical edits with edit_file/multi_edit, keep surrounding style, and run the project's build/test commands via bash to verify. Never leave stubs or todo!() behind.".into(),
            allowed_tools: vec![],
            can_spawn_subagents: false,
        },
        AgentDef {
            name: "explorer".into(),
            role: "explorer".into(),
            description: "Fast codebase search (glob/grep/symbols) and web documentation research".into(),
            instructions: "You are the explorer. Answer questions about the codebase with file:line citations. Use web_fetch/web_search for external docs. Never modify files.".into(),
            allowed_tools: vec!["read_file", "list_dir", "glob", "grep", "web_fetch", "web_search"].into_iter().map(str::to_string).collect(),
            can_spawn_subagents: false,
        },
        AgentDef {
            name: "browser".into(),
            role: "browser".into(),
            description: "Operates the built-in browser: tests UI flows, inspects DOM/console, automates web actions".into(),
            instructions: "You are the browser agent. Navigate with browser_navigate, read the [ref=eN] tree, actuate elements, and report console/network problems you find.".into(),
            allowed_tools: vec!["browser_navigate", "browser_actuate", "browser_inspect", "web_fetch", "web_search"].into_iter().map(str::to_string).collect(),
            can_spawn_subagents: false,
        },
        AgentDef {
            name: "reviewer".into(),
            role: "reviewer".into(),
            description: "Audits diffs for bugs, security issues and adherence to the minimal UI rules".into(),
            instructions: "You are the reviewer. Read the changed files, hunt for bugs, security issues, and design-system violations (gradients, neon, oversized radius). Report concrete findings with file:line references and severity.".into(),
            allowed_tools: vec!["read_file", "list_dir", "glob", "grep"].into_iter().map(str::to_string).collect(),
            can_spawn_subagents: false,
        },
    ]
}

/// Resolve a built-in or custom definition by role/name.
pub fn resolve(custom: &[AgentDef], role: &str) -> AgentDef {
    custom
        .iter()
        .find(|a| a.role == role || a.name == role)
        .cloned()
        .or_else(|| builtin_definitions().into_iter().find(|a| a.role == role || a.name == role))
        .unwrap_or_else(|| builtin_definitions()[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_cover_the_five_roles() {
        let defs = builtin_definitions();
        for role in ["architect", "coder", "explorer", "browser", "reviewer", "orchestrator"] {
            assert!(defs.iter().any(|d| d.role == role), "missing role {role}");
        }
    }

    #[test]
    fn custom_overrides_builtin() {
        let custom = vec![AgentDef {
            name: "senior-coder".into(),
            role: "coder".into(),
            description: "custom".into(),
            instructions: "be strict".into(),
            allowed_tools: vec![],
            can_spawn_subagents: false,
        }];
        let resolved = resolve(&custom, "coder");
        assert_eq!(resolved.name, "senior-coder");
        let fallback = resolve(&[], "browser");
        assert_eq!(fallback.role, "browser");
    }
}
