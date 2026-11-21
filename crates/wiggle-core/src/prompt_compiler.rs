//! 7-stage prompt compiler.
//!
//! Every outgoing model request is rebuilt here: identity + environment,
//! personal memory, project instructions, skills, tool bindings, hook
//! transformations, and task routing.

use serde::{Deserialize, Serialize};

use crate::config::GlobalConfig;
use crate::{CoreResult, Engine};

/// Environment facts captured once per turn (Step 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os: String,
    pub arch: String,
    pub shell: String,
    pub cwd: std::path::PathBuf,
    pub git_branch: Option<String>,
    pub git_status: Option<String>,
}

impl EnvironmentInfo {
    pub fn capture(cwd: &std::path::Path) -> EnvironmentInfo {
        let (git_branch, git_status) = crate::identity::git_snapshot(cwd);
        EnvironmentInfo {
            os: crate::identity::os_name(),
            arch: crate::identity::arch_name(),
            shell: crate::identity::default_shell(),
            cwd: cwd.to_path_buf(),
            git_branch,
            git_status,
        }
    }
}

/// Routing decision from Step 7.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum TaskRouting {
    /// Single agent handles everything.
    Single { agent: String },
    /// Parallel subagents are warranted; instructions per role.
    Swarm {
        lead: String,
        tasks: Vec<SwarmDirective>,
        rationale: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwarmDirective {
    pub role: String,
    pub instruction: String,
}

/// The fully compiled prompt: what actually gets sent to the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledPrompt {
    pub system: String,
    pub user_message: String,
    pub active_skills: Vec<String>,
    pub tool_names: Vec<String>,
    pub routing: TaskRouting,
    pub usage_note: String,
}

pub struct PromptCompiler<'a> {
    engine: &'a Engine,
}

impl<'a> PromptCompiler<'a> {
    pub fn new(engine: &'a Engine) -> PromptCompiler<'a> {
        PromptCompiler { engine }
    }

    /// Run all seven stages and return the compiled request text.
    pub async fn compile(
        &self,
        session: &str,
        raw_prompt: &str,
        agent_role: Option<&str>,
    ) -> CoreResult<CompiledPrompt> {
        let config: GlobalConfig = self.engine.config().clone();
        let workspace = self.engine.workspace().clone();

        // ---- Step 1: identity & environment header ----
        let env = EnvironmentInfo::capture(&workspace);
        let mut system = crate::identity::system_header(&env);
        // User persona extension (never replaces the immutable block).
        if let Some(override_text) = &config.identity_override {
            system.push_str(&format!("\n<user_persona>\n{override_text}\n</user_persona>"));
        }
        if let Some(extra) = &config.system_prompt {
            system.push_str(&format!("\n<extra_instructions>\n{extra}\n</extra_instructions>"));
        }

        // ---- Step 2: personal context & past chat memory ----
        if config.memory_enabled {
            if let Some(memory) = &self.engine.inner.memory {
                if let Ok(store) = memory.lock() {
                    let block = store.context_block(raw_prompt, 12);
                    if !block.is_empty() {
                        system.push_str(&format!("\n{block}"));
                    }
                }
            }
        }

        // ---- Step 3: project instructions (WIGGLE.md / CLAUDE.md / AGENTS.md / .cursorrules) ----
        for candidate in ["WIGGLE.md", "CLAUDE.md", "AGENTS.md", ".cursorrules"] {
            let path = workspace.join(candidate);
            if let Ok(content) = std::fs::read_to_string(&path) {
                if !content.trim().is_empty() {
                    system.push_str(&format!(
                        "\n<project_instructions source=\"{candidate}\">\n{}\n</project_instructions>",
                        truncate(&content, 12_000)
                    ));
                    break; // first match wins; they overlap heavily
                }
            }
        }

        // ---- Step 6 runs before Step 4/5 in effect: hooks may rewrite
        // the prompt itself (UserPromptSubmit). ----
        let hook_outcome = crate::plugins::hooks::run_hooks(
            &config.hooks,
            crate::plugins::hooks::HookEvent::UserPromptSubmit,
            serde_json::json!({
                "session": session,
                "prompt": raw_prompt,
                "cwd": workspace.display().to_string(),
            }),
        )
        .await
        .unwrap_or_else(|_| crate::plugins::hooks::HookOutcome::approve());
        if !hook_outcome.approved {
            return Err(crate::CoreError::Permission(format!(
                "prompt blocked by UserPromptSubmit hook: {}",
                hook_outcome.reason
            )));
        }
        let prompt = hook_outcome.prompt_override.unwrap_or_else(|| raw_prompt.to_string());

        // ---- Step 4: dynamic skill attachment ----
        let mut active_skills = Vec::new();
        for skill in self.engine.inner.skills.match_prompts(&prompt, 3) {
            if let Ok(instructions) = skill.instructions() {
                system.push_str(&format!(
                    "\n<active_skill name=\"{}\" path=\"{}\">\n{}\n</active_skill>",
                    skill.name,
                    skill.path.display(),
                    truncate(&instructions, 8_000)
                ));
                active_skills.push(skill.name);
            }
        }

        // ---- Step 5: MCP & native tool binding ----
        let native = self.engine.inner.tools.specs();
        let mut tool_names: Vec<String> = native.iter().map(|t| t.name.clone()).collect();
        let mut specs = native;
        {
            let mcp = self.engine.inner.mcp.lock().await;
            for tool in mcp.all_tools() {
                specs.push(crate::providers::ToolSpec {
                    name: format!("mcp_{}_{}", sanitize(&tool.server), sanitize(&tool.name)),
                    description: format!("[MCP:{}{}] {}", tool.server, tool.name, tool.description),
                    parameters: if tool.input_schema.is_object() {
                        tool.input_schema.clone()
                    } else {
                        serde_json::json!({"type": "object"})
                    },
                });
                tool_names.push(tool.name.clone());
            }
        }

        // Agent profile instructions.
        let role = agent_role.unwrap_or("agent");
        let def = crate::agents::definitions::resolve(&config.agents.custom, role);
        if !def.instructions.is_empty() {
            system.push_str(&format!("\n<agent_profile role=\"{}\">\n{}\n</agent_profile>", def.role, def.instructions));
        }

        // ---- Step 7: task decomposition & agent routing ----
        let routing = route_task(&prompt);

        let usage_note = format!(
            "compiled: {} stage(s) | skills attached: {} | tools bound: {} | routing: {}",
            7,
            active_skills.len(),
            tool_names.len(),
            match &routing {
                TaskRouting::Single { agent } => format!("single/{agent}"),
                TaskRouting::Swarm { tasks, .. } => format!("swarm({})", tasks.len()),
            }
        );

        Ok(CompiledPrompt { system, user_message: prompt, active_skills, tool_names, routing, usage_note })
    }
}

/// Heuristic Step 7 router: detect parallelizable multi-part prompts.
pub fn route_task(prompt: &str) -> TaskRouting {
    let lower = prompt.to_lowercase();
    let split_markers = [
        " and then ", "in parallel", "meanwhile", " while ", " at the same time", "同时",
        "for each of", "both ",
    ];
    let wants_parallel = split_markers.iter().any(|m| lower.contains(m))
        || lower.matches(" and ").count() >= 2;

    if lower.contains("browser") || lower.contains("localhost") && lower.contains("test") {
        return TaskRouting::Swarm {
            lead: "agent".into(),
            rationale: "prompt references a browser or live app verification".into(),
            tasks: vec![
                SwarmDirective { role: "coder".into(), instruction: prompt.to_string() },
                SwarmDirective {
                    role: "browser".into(),
                    instruction: format!("Verify the result in the built-in browser: {prompt}"),
                },
            ],
        };
    }

    if wants_parallel {
        // Split at the first parallel marker so each agent gets a real
        // instruction; when no marker splits cleanly both agents see the
        // full prompt (coder implements, explorer verifies context).
        let split_at = split_markers
            .iter()
            .find_map(|m| lower.find(m).map(|i| i + m.len()));
        let (first, second) = match split_at {
            Some(s) if s > 0 && s < prompt.len() => {
                (prompt[..s].trim(), prompt[s..].trim())
            }
            _ => (prompt.trim(), prompt.trim()),
        };
        return TaskRouting::Swarm {
            lead: "agent".into(),
            rationale: "multi-part task detected; splitting across specialist agents".into(),
            tasks: vec![
                SwarmDirective { role: "coder".into(), instruction: first.to_string() },
                SwarmDirective { role: "explorer".into(), instruction: second.to_string() },
            ],
        };
    }

    TaskRouting::Single { agent: "agent".into() }
}

fn sanitize(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { '_' }).collect()
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        text.to_string()
    } else {
        format!("{}...", text.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_detects_parallel_intent() {
        let routing = route_task("Refactor the auth module and then update all the tests and also the docs");
        assert!(matches!(routing, TaskRouting::Swarm { .. }));
        let simple = route_task("fix the typo in readme.md");
        assert!(matches!(simple, TaskRouting::Single { .. }));
    }

    #[test]
    fn routing_detects_browser_verification() {
        let routing = route_task("use the browser to check that localhost:3000 loads");
        assert!(matches!(routing, TaskRouting::Swarm { ref tasks, .. } if tasks.iter().any(|t| t.role == "browser")));
    }

    #[test]
    fn sanitize_mcp_names() {
        assert_eq!(sanitize("my-server"), "my-server");
        assert_eq!(sanitize("server.name/x"), "server_name_x");
    }
}
