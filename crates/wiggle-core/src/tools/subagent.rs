//! Subagent tools: spawn_subagents (parallel swarm dispatch) and
//! task_status (poll running swarm tasks), the Antigravity Agent
//! Manager pattern exposed to the model.

use std::sync::Arc;

use serde_json::json;

use super::{ToolDef, ToolOutput};
use crate::agents::swarm::{SwarmTask, SwarmTaskState};
use crate::CoreError;

pub fn definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "spawn_subagents",
            description: "Dispatch one or more specialized subagents to run in parallel (Antigravity Agent Manager style). Each task gets a role, an instruction and optionally files it owns. Returns task ids immediately.",
            parameters: super::obj_schema(json!({
                "tasks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "role": { "type": "string", "description": "architect | coder | explorer | browser | reviewer" },
                            "instruction": { "type": "string" },
                            "files": { "type": "array", "items": { "type": "string" }, "description": "Optional file paths this task owns (file-lease coordination)" }
                        },
                        "required": []
                    }
                }
            })),
            mutating: true,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let tasks: Vec<SwarmTask> = args["tasks"]
                    .as_array()
                    .ok_or_else(|| CoreError::Tool("missing 'tasks' array".into()))?
                    .iter()
                    .filter_map(|t| {
                        Some(SwarmTask {
                            role: t["role"].as_str().unwrap_or("explorer").to_string(),
                            instruction: t["instruction"].as_str().unwrap_or_default().to_string(),
                            files: t["files"]
                                .as_array()
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|f| f.as_str())
                                        .map(std::path::PathBuf::from)
                                        .collect()
                                })
                                .unwrap_or_default(),
                            state: SwarmTaskState::Queued,
                            result: String::new(),
                        })
                    })
                    .collect();
                if tasks.is_empty() {
                    return Err(CoreError::Tool("no tasks given".into()));
                }
                let orchestrator = crate::agents::swarm::SwarmOrchestrator::new(ctx.engine.clone());
                let ids = orchestrator.dispatch(&ctx.session, tasks).await?;
                Ok(ToolOutput::text(format!(
                    "dispatched {} task(s): {}",
                    ids.len(),
                    ids.join(", ")
                )))
            })),
        },
        ToolDef {
            name: "task_status",
            description: "Poll dispatched subagent tasks. Pass no ids to report every task in the current session.",
            parameters: super::obj_schema(json!({
                "ids": { "type": "array", "items": { "type": "string" } }
            })),
            mutating: false,
            run: Arc::new(|ctx, args| Box::pin(async move {
                let filter: Vec<String> = args["ids"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                    .unwrap_or_default();
                let orchestrator = crate::agents::swarm::SwarmOrchestrator::new(ctx.engine.clone());
                let report = orchestrator.status(&ctx.session, &filter).await;
                Ok(ToolOutput::text(report))
            })),
        },
    ]
}
