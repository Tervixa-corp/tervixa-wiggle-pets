//! Autonomous ReAct / tool-use agent execution loop.
//!
//! Streams from the provider, executes tool calls through the
//! permission gate and file-lease coordinator, records undo snapshots,
//! feeds tool results back until the model stops or the turn budget is
//! exhausted.

use serde_json::json;

use crate::config::SandboxMode;
use crate::permissions::Decision;
use crate::providers::{
    ChatMessage, ChatRequest, ContentBlock, Router, StreamEvent, ToolResultBlock,
};
use crate::{AgentEvent, CoreError, CoreResult, Engine, ToolCallView};

pub struct AgentRunner {
    engine: Engine,
    session: String,
}

impl AgentRunner {
    pub fn new(engine: Engine, session: String) -> AgentRunner {
        AgentRunner { engine, session }
    }

    /// Full 7-stage pipeline + tool loop. Returns the final text.
    pub async fn run(
        &self,
        raw_prompt: &str,
        agent_role: Option<&str>,
        model_override: Option<&str>,
    ) -> CoreResult<String> {
        let config = self.engine.config().clone();
        let compiler = crate::prompt_compiler::PromptCompiler::new(&self.engine);
        let compiled = compiler.compile(&self.session, raw_prompt, agent_role).await?;
        let role = match &compiled.routing {
            crate::prompt_compiler::TaskRouting::Single { agent } => agent.clone(),
            crate::prompt_compiler::TaskRouting::Swarm { tasks, .. } => tasks[0].role.clone(),
        };

        let _ = self.engine.inner.events.send(AgentEvent::TurnStarted {
            session: self.session.clone(),
            agent: role.clone(),
        });

        // Swarms are orchestrated by the swarm module.
        if let crate::prompt_compiler::TaskRouting::Swarm { tasks, rationale, .. } =
            &compiled.routing
        {
            let orchestrator = super::swarm::SwarmOrchestrator::new(self.engine.clone());
            let swarm_tasks: Vec<super::swarm::SwarmTask> = tasks
                .iter()
                .map(|d| super::swarm::SwarmTask {
                    role: d.role.clone(),
                    instruction: d.instruction.clone(),
                    files: Vec::new(),
                    state: super::swarm::SwarmTaskState::Queued,
                    result: String::new(),
                })
                .collect();
            let summary = orchestrator
                .run_to_completion(&self.session, swarm_tasks, rationale)
                .await?;
            let _ = self.engine.inner.events.send(AgentEvent::AgentFinished {
                summary: summary.clone(),
            });
            return Ok(summary);
        }

        let result = self
            .single_agent_loop(&compiled, &role, model_override, &config)
            .await;
        match &result {
            Ok(text) => {
                // Learn from the exchange (memory extractor).
                if config.memory_enabled {
                    if let Some(memory) = &self.engine.inner.memory {
                        if let Ok(mut store) = memory.lock() {
                            let extractor = wiggle_memory::MemoryExtractor;
                            let _ = extractor.learn(&mut store, raw_prompt, text);
                        }
                    }
                }
                let _ = self.engine.inner.events.send(AgentEvent::AgentFinished {
                    summary: text.clone(),
                });
            }
            Err(e) => {
                let _ = self.engine.inner.events.send(AgentEvent::Error {
                    message: e.to_string(),
                });
            }
        }
        result
    }

    async fn single_agent_loop(
        &self,
        compiled: &crate::prompt_compiler::CompiledPrompt,
        role: &str,
        model_override: Option<&str>,
        config: &crate::config::GlobalConfig,
    ) -> CoreResult<String> {
        let (provider, model) = Router::provider(
            config,
            model_override
                .and_then(|m| m.split('/').next())
                .or(config.default_provider.as_deref()),
            model_override.or(Some(&model_of(config, role))),
        )?;

        let max_turns = config.agents.max_turns;
        let sandbox = self.engine.permissions_gate().mode();
        let tool_names: Vec<String> = compiled.tool_names.clone();

        let mut transcript: Vec<ChatMessage> =
            vec![ChatMessage::user(compiled.user_message.clone())];
        let mut final_text = String::new();
        let mut total_usage = crate::providers::Usage::default();

        for _turn in 0..max_turns {
            let request = ChatRequest {
                model: model.clone(),
                system: compiled.system.clone(),
                messages: transcript.clone(),
                tools: self.engine.inner.tools.specs(),
                max_tokens: 16_384,
                temperature: None,
                thinking: false,
            };

            let mut rx = provider.chat(request).await?;
            let mut turn_text = String::new();
            let mut tool_uses: Vec<crate::providers::ToolCall> = Vec::new();
            let mut stop = crate::providers::StopReason::EndTurn;
            let mut turn_usage = crate::providers::Usage::default();

            while let Some(event) = rx.recv().await {
                match event {
                    StreamEvent::Text(t) => {
                        final_text.push_str(&t);
                        turn_text.push_str(&t);
                        let _ = self
                            .engine
                            .inner
                            .events
                            .send(AgentEvent::TextDelta { text: t });
                    }
                    StreamEvent::Thinking(_) => { /* surfaced in UI via Thinking deltas later */ }
                    StreamEvent::ToolUse(call) => tool_uses.push(call),
                    StreamEvent::Usage(u) => {
                        turn_usage.input_tokens += u.input_tokens;
                        turn_usage.output_tokens += u.output_tokens;
                    }
                    StreamEvent::Done(reason) => {
                        stop = reason;
                        break;
                    }
                    StreamEvent::Fatal(msg) => {
                        return Err(CoreError::Provider(msg));
                    }
                }
            }

            total_usage.input_tokens += turn_usage.input_tokens;
            total_usage.output_tokens += turn_usage.output_tokens;

            if tool_uses.is_empty() {
                transcript.push(ChatMessage {
                    role: crate::providers::Role::Assistant,
                    content: vec![ContentBlock::Text(turn_text)],
                });
                let _ = self.engine.inner.events.send(AgentEvent::TurnFinished {
                    usage: crate::UsageView {
                        input_tokens: total_usage.input_tokens,
                        output_tokens: total_usage.output_tokens,
                        total_tokens: total_usage.input_tokens + total_usage.output_tokens,
                    },
                });
                return Ok(final_text);
            }

            // Record assistant tool_use block, then execute each call.
            let mut assistant_blocks: Vec<ContentBlock> = Vec::new();
            if !turn_text.is_empty() {
                assistant_blocks.push(ContentBlock::Text(turn_text));
            }
            for call in &tool_uses {
                assistant_blocks.push(ContentBlock::ToolUse(call.clone()));
            }
            transcript.push(ChatMessage {
                role: crate::providers::Role::Assistant,
                content: assistant_blocks,
            });

            let mut result_blocks: Vec<ContentBlock> = Vec::new();
            for call in tool_uses {
                // PreToolUse hooks may deny.
                let hook_ok = crate::plugins::hooks::run_hooks(
                    &config.hooks,
                    crate::plugins::hooks::HookEvent::PreToolUse,
                    json!({
                        "session": self.session,
                        "tool": call.name,
                        "params": call.arguments,
                    }),
                )
                .await
                .unwrap_or_else(|_| crate::plugins::hooks::HookOutcome::approve());

                let call_view = ToolCallView {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    params: call.arguments.clone(),
                    output: String::new(),
                };
                let _ = self
                    .engine
                    .inner
                    .events
                    .send(AgentEvent::ToolStarted { call: call_view.clone() });

                let outcome = if !hook_ok.approved {
                    Err(CoreError::Permission(format!(
                        "blocked by PreToolUse hook: {}",
                        hook_ok.reason
                    )))
                } else {
                    self.execute_tool(&call, sandbox.clone(), &tool_names).await
                };

                let (content, is_error) = match outcome {
                    Ok(output) => (output.content, false),
                    Err(e) => (format!("error: {e}"), true),
                };

                let _ = self.engine.inner.events.send(AgentEvent::ToolFinished {
                    call: ToolCallView {
                        id: call.id.clone(),
                        name: call.name.clone(),
                        params: call.arguments.clone(),
                        output: content.chars().take(400).collect(),
                    },
                    ok: !is_error,
                });

                result_blocks.push(ContentBlock::ToolResult(ToolResultBlock {
                    call_id: call.id.clone(),
                    content,
                    is_error,
                }));
            }
            transcript.push(ChatMessage {
                role: crate::providers::Role::User,
                content: result_blocks,
            });
            let _ = stop;
        }

        Err(CoreError::Tool(format!(
            "agent reached the turn budget ({max_turns}) without a final answer"
        )))
    }

    async fn execute_tool(
        &self,
        call: &crate::providers::ToolCall,
        sandbox: SandboxMode,
        tool_names: &[String],
    ) -> CoreResult<crate::tools::ToolOutput> {
        let _ = sandbox;
        // MCP tools are prefixed mcp_<server>_<name>.
        if let Some(rest) = call.name.strip_prefix("mcp_") {
            // Resolve the target under the async lock, then call while
            // holding the same guard (tokio guards are Send).
            let manager = self.engine.inner.mcp.lock().await;
            for tool in manager.all_tools() {
                let full = format!("mcp_{}_{}", tool.server.replace(['-', '.', '/'], "_"), tool.name);
                if full == format!("mcp_{rest}") {
                    return manager
                        .call_tool(&tool.server, &tool.name, call.arguments.clone())
                        .await
                        .map(crate::tools::ToolOutput::text);
                }
            }
            return Err(CoreError::Mcp(format!("mcp tool '{}' not found", call.name)));
        }

        let def = self
            .engine
            .inner
            .tools
            .get(&call.name)
            .ok_or_else(|| {
                CoreError::Tool(format!(
                    "unknown tool '{}'. available: {}",
                    call.name,
                    tool_names.join(", ")
                ))
            })?;

        // Permission gate for mutating native tools.
        let gate = self.engine.permissions_gate();
        match gate.check_tool(def.name, def.mutating) {
            Decision::Allowed => {}
            Decision::NeedsApproval { reason } | Decision::Denied { reason } => {
                return Err(CoreError::Permission(reason));
            }
        }

        let ctx = crate::tools::ToolContext {
            engine: self.engine.clone(),
            session: self.session.clone(),
            agent_role: "agent".into(),
        };
        (def.run)(ctx, call.arguments.clone()).await
    }
}

fn model_of(config: &crate::config::GlobalConfig, role: &str) -> String {
    config
        .resolve_model(role)
        .map(|(_, m)| m)
        .or_else(|| config.default_model.clone())
        .unwrap_or_else(|| {
            config
                .providers
                .iter()
                .find(|p| p.enabled)
                .and_then(|p| p.models.first().map(|m| m.id.clone()))
                .unwrap_or_default()
        })
}
