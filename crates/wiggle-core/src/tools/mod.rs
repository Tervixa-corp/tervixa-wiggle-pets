//! Native tool registry and JSON-schema generator. Tools are the
//! model-facing surface of the engine; MCP tools are merged in at
//! compile time by the prompt compiler.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::json;

use crate::CoreResult;

pub mod bash;
pub mod fs;
pub mod subagent;
pub mod web;

/// Execution context handed to every tool invocation.
#[derive(Clone)]
pub struct ToolContext {
    pub engine: crate::Engine,
    pub session: String,
    pub agent_role: String,
}

pub type ToolFn = dyn Fn(ToolContext, serde_json::Value) -> futures::future::BoxFuture<'static, CoreResult<ToolOutput>>
    + Send
    + Sync;

pub struct ToolOutput {
    /// Text shown back to the model.
    pub content: String,
    /// Structured side-channel for the UI (diffs, screenshots...).
    pub display: Option<serde_json::Value>,
}

impl ToolOutput {
    pub fn text(content: impl Into<String>) -> ToolOutput {
        ToolOutput { content: content.into(), display: None }
    }
}

/// A registered native tool.
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: serde_json::Value,
    pub mutating: bool,
    pub run: Arc<ToolFn>,
}

/// Registry of built-in tools, addressable by name.
pub struct ToolRegistry {
    tools: BTreeMap<&'static str, Arc<ToolDef>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        let mut tools: BTreeMap<&'static str, Arc<ToolDef>> = BTreeMap::new();
        for def in fs::definitions() {
            tools.insert(def.name, Arc::new(def));
        }
        for def in bash::definitions() {
            tools.insert(def.name, Arc::new(def));
        }
        for def in web::definitions() {
            tools.insert(def.name, Arc::new(def));
        }
        for def in subagent::definitions() {
            tools.insert(def.name, Arc::new(def));
        }
        ToolRegistry { tools }
    }
}

impl ToolRegistry {
    pub fn get(&self, name: &str) -> Option<Arc<ToolDef>> {
        self.tools.get(name).cloned()
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }

    /// Model-facing tool specs (OpenAI function-schema shape, which the
    /// provider adapters translate for Anthropic / Gemini).
    pub fn specs(&self) -> Vec<crate::providers::ToolSpec> {
        self.tools
            .values()
            .map(|t| crate::providers::ToolSpec {
                name: t.name.to_string(),
                description: t.description.to_string(),
                parameters: t.parameters.clone(),
            })
            .collect()
    }
}

/// Shorthand for building a JSON object schema.
pub fn obj_schema(properties: serde_json::Value) -> serde_json::Value {
    json!({ "type": "object", "properties": properties, "required": [] })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_loads_all_native_tools() {
        let registry = ToolRegistry::default();
        for expected in [
            "read_file", "write_file", "edit_file", "multi_edit", "glob", "grep", "list_dir",
            "bash", "web_fetch", "web_search", "browser_navigate", "browser_actuate",
            "browser_inspect", "spawn_subagents", "task_status",
        ] {
            assert!(registry.get(expected).is_some(), "missing tool {expected}");
        }
        assert_eq!(registry.specs().len(), registry.names().len());
    }
}
