//! Global (`~/.wiggle/config.toml`) and per-project
//! (`.wiggle/config.toml`) configuration.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

fn home_dir() -> PathBuf {
    std::env::var_os("WIGGLE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).map(|h| h.join(".wiggle")))
        .unwrap_or_else(|| PathBuf::from(".wiggle"))
}

/// Which native API dialect a provider speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderApi {
    Anthropic,
    Openai,
    Gemini,
}

impl ProviderApi {
    pub fn from_models_dev_npm(npm: &str) -> Option<ProviderApi> {
        match npm {
            "@ai-sdk/anthropic" | "@ai-sdk/amazon-bedrock" | "@ai-sdk/mistral" => {
                Some(ProviderApi::Anthropic)
            }
            "@ai-sdk/google" | "@ai-sdk/google-vertex" => Some(ProviderApi::Gemini),
            _ => Some(ProviderApi::Openai),
        }
    }
}

/// One BYOK provider connection configured by the user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    pub id: String,
    pub label: String,
    pub api: ProviderApi,
    pub base_url: String,
    /// Environment variables tried in order for the API key.
    #[serde(default)]
    pub env_keys: Vec<String>,
    /// Inline API key (BYOK). Env keys win when both are present.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Models offered by this connection: id -> (context limit, supports tools).
    #[serde(default)]
    pub models: Vec<ModelEntry>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelEntry {
    pub id: String,
    #[serde(default)]
    pub context_limit: u64,
    #[serde(default = "default_true")]
    pub tool_call: bool,
    #[serde(default)]
    pub reasoning: bool,
}

/// Per-role model binding for the swarm.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RoleBindings {
    #[serde(default)]
    pub orchestrator: Option<String>,
    #[serde(default)]
    pub architect: Option<String>,
    #[serde(default)]
    pub coder: Option<String>,
    #[serde(default)]
    pub explorer: Option<String>,
    #[serde(default)]
    pub browser: Option<String>,
    #[serde(default)]
    pub reviewer: Option<String>,
    #[serde(default)]
    pub memory_extractor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SandboxMode {
    /// Never execute mutating tools; propose patches for review (Codex "suggest").
    Suggest,
    /// Edit files automatically; shell commands still ask (Codex "auto-edit").
    AutoEdit,
    /// Full autonomy within the workspace (Codex "full-auto").
    FullAuto,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolPermission {
    Allow,
    Ask,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Permissions {
    #[serde(default)]
    pub mode: Option<SandboxMode>,
    /// tool name -> permission
    #[serde(default)]
    pub tools: std::collections::BTreeMap<String, ToolPermission>,
    /// Shell command prefixes always allowed without prompting.
    #[serde(default)]
    pub command_allowlist: Vec<String>,
    /// Shell command prefixes always denied.
    #[serde(default)]
    pub command_denylist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: McpTransport,
    /// stdio: executable; http/sse: full URL.
    pub endpoint: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    Stdio,
    Http,
    Sse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HookConfig {
    /// Event name: user_prompt_submit | pre_tool_use | post_tool_use |
    /// notification | subagent_stop | stop
    pub event: String,
    /// Command executed with the event JSON on stdin. Its stdout may be
    /// JSON `{"decision": "deny", "reason": "..."}` to block an action.
    pub command: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BrowserConfig {
    /// Executable to drive over CDP; auto-detected when empty.
    #[serde(default)]
    pub executable: Option<String>,
    #[serde(default = "default_search_engine")]
    pub search_engine: String,
    #[serde(default)]
    pub headless: bool,
    #[serde(default = "default_true")]
    pub persist_session: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            executable: None,
            search_engine: default_search_engine(),
            headless: false,
            persist_session: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitConfig {
    #[serde(default)]
    pub auto_commit: bool,
    #[serde(default = "default_main_branch")]
    pub primary_branch: String,
    #[serde(default)]
    pub auto_push: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self { auto_commit: false, primary_branch: default_main_branch(), auto_push: false }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceConfig {
    /// "dark" (Flat Carbon Dark) or "light" (Crisp Light).
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    #[serde(default)]
    pub keybindings: std::collections::BTreeMap<String, String>,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self { theme: default_theme(), font_size: default_font_size(), keybindings: Default::default() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentsConfig {
    /// Maximum concurrently running swarm agents.
    #[serde(default = "default_concurrency")]
    pub max_concurrency: usize,
    /// Maximum tool-use turns per agent run.
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
    #[serde(default)]
    pub role_models: RoleBindings,
    /// Custom agent definitions loaded from .wiggle/agents/*.toml
    #[serde(default)]
    pub custom: Vec<crate::agents::definitions::AgentDef>,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            max_concurrency: default_concurrency(),
            max_turns: default_max_turns(),
            role_models: Default::default(),
            custom: Vec::new(),
        }
    }
}

/// Complete configuration. Serialized to TOML in both scopes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GlobalConfig {
    /// Optional user override of the identity/persona header.
    #[serde(default)]
    pub identity_override: Option<String>,
    /// Extra system instructions appended after the immutable identity.
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub default_provider: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub agents: AgentsConfig,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
    #[serde(default)]
    pub hooks: Vec<HookConfig>,
    #[serde(default)]
    pub memory_enabled: bool,
    #[serde(default)]
    pub browser: BrowserConfig,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    /// Extra workspace roots the agent may touch outside the workspace.
    #[serde(default)]
    pub workspace_roots: Vec<PathBuf>,

    #[serde(skip)]
    pub home_dir: PathBuf,
    #[serde(skip)]
    pub bundled_skills_dir: Option<PathBuf>,
}

fn default_true() -> bool {
    true
}
fn default_search_engine() -> String {
    "https://duckduckgo.com/html/?q=".into()
}
fn default_main_branch() -> String {
    "main".into()
}
fn default_theme() -> String {
    "dark".into()
}
fn default_font_size() -> u32 {
    13
}
fn default_concurrency() -> usize {
    5
}
fn default_max_turns() -> u32 {
    48
}

impl GlobalConfig {
    pub fn wiggle_home() -> PathBuf {
        static HOME: OnceLock<PathBuf> = OnceLock::new();
        HOME.get_or_init(home_dir).clone()
    }

    pub fn global_path() -> PathBuf {
        Self::wiggle_home().join("config.toml")
    }

    pub fn project_path(workspace: &Path) -> PathBuf {
        workspace.join(".wiggle").join("config.toml")
    }

    pub fn memory_db_path(&self) -> PathBuf {
        self.home_dir.join("memory").join("wiggle.db")
    }

    /// Load the global config; missing file yields defaults.
    pub fn load() -> CoreResult<GlobalConfig> {
        let path = Self::global_path();
        let mut config = if path.exists() {
            let raw = std::fs::read_to_string(&path)?;
            toml::from_str::<GlobalConfig>(&raw).map_err(|e| CoreError::Config(e.to_string()))?
        } else {
            GlobalConfig::default()
        };
        config.home_dir = Self::wiggle_home();
        config.bundled_skills_dir = default_bundled_skills_dir();
        Ok(config)
    }

    /// Merge `<workspace>/.wiggle/config.toml` over the global config.
    pub fn merge_project(&mut self, workspace: &Path) {
        let path = Self::project_path(workspace);
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(project) = toml::from_str::<GlobalConfig>(&raw) {
                if project.identity_override.is_some() {
                    self.identity_override = project.identity_override;
                }
                if project.system_prompt.is_some() {
                    self.system_prompt = project.system_prompt;
                }
                if project.default_provider.is_some() {
                    self.default_provider = project.default_provider;
                }
                if project.default_model.is_some() {
                    self.default_model = project.default_model;
                }
                for p in project.providers {
                    match self.providers.iter_mut().find(|e| e.id == p.id) {
                        Some(existing) => *existing = p,
                        None => self.providers.push(p),
                    }
                }
                self.mcp_servers.extend(project.mcp_servers);
                self.hooks.extend(project.hooks);
                self.workspace_roots.extend(project.workspace_roots);
                if !project.agents.custom.is_empty() {
                    self.agents.custom.extend(project.agents.custom);
                }
                if let Some(mode) = project.permissions.mode {
                    self.permissions.mode = Some(mode);
                }
                for (k, v) in project.permissions.tools {
                    self.permissions.tools.insert(k, v);
                }
            }
        }
    }

    /// Fill in built-in provider connections the user has not disabled
    /// and make sure required directories exist.
    pub fn ensure_defaults(&mut self) {
        if self.home_dir.as_os_str().is_empty() {
            self.home_dir = Self::wiggle_home();
        }
        if self.bundled_skills_dir.is_none() {
            self.bundled_skills_dir = default_bundled_skills_dir();
        }
        let existing: std::collections::HashSet<String> =
            self.providers.iter().map(|p| p.id.clone()).collect();
        for preset in crate::providers::presets::BUILTIN_PRESETS.iter() {
            if existing.contains(&preset.id.to_string()) {
                continue;
            }
            self.providers.push(preset.to_config());
        }
    }

    /// Persist the global config to `~/.wiggle/config.toml`.
    pub fn save(&self) -> CoreResult<()> {
        std::fs::create_dir_all(&self.home_dir)?;
        let raw = toml::to_string_pretty(self).map_err(|e| CoreError::Config(e.to_string()))?;
        std::fs::write(Self::global_path(), raw)?;
        Ok(())
    }

    /// Resolve the model string (`provider/model`) for a role binding.
    pub fn resolve_model(&self, role: &str) -> Option<(ProviderConfig, String)> {
        let binding = match role {
            "architect" => self.agents.role_models.architect.as_deref(),
            "coder" => self.agents.role_models.coder.as_deref(),
            "explorer" => self.agents.role_models.explorer.as_deref(),
            "browser" => self.agents.role_models.browser.as_deref(),
            "reviewer" => self.agents.role_models.reviewer.as_deref(),
            _ => self.agents.role_models.orchestrator.as_deref(),
        };
        let model = binding.or(self.default_model.as_deref());
        let provider_id = self.default_provider.clone().or_else(|| {
            self.providers.iter().find(|p| p.enabled && !p.models.is_empty()).map(|p| p.id.clone())
        });
        match (model, provider_id) {
            (Some(model), Some(pid)) => self
                .providers
                .iter()
                .find(|p| p.id == pid && p.enabled)
                .cloned()
                .map(|p| (p, model.to_string())),
            (Some(model), None) => self
                .providers
                .iter()
                .find(|p| p.enabled && p.models.iter().any(|m| m.id == model))
                .cloned()
                .map(|p| (p, model.to_string())),
            _ => None,
        }
    }
}

fn default_bundled_skills_dir() -> Option<PathBuf> {
    // 1. $WIGGLE_BUNDLED_SKILLS (set by installers)
    // 2. ../bundled-skills relative to the executable (repo checkout)
    if let Some(v) = std::env::var_os("WIGGLE_BUNDLED_SKILLS") {
        return Some(PathBuf::from(v));
    }
    std::env::current_exe().ok().and_then(|exe| {
        let dir = exe.parent()?.join("../../bundled-skills").canonicalize().ok();
        dir.filter(|p| p.is_dir()).or_else(|| {
            exe.parent()
                .map(|p| p.join("bundled-skills"))
                .filter(|p| p.is_dir())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_serializes_roundtrip() {
        let mut c = GlobalConfig::default();
        c.ensure_defaults();
        let raw = toml::to_string_pretty(&c).unwrap();
        let back: GlobalConfig = toml::from_str(&raw).unwrap();
        assert_eq!(back.providers.len(), c.providers.len());
    }

    #[test]
    fn builtin_presets_cover_major_vendor_apis() {
        let ids: Vec<&str> =
            crate::providers::presets::BUILTIN_PRESETS.iter().map(|p| p.id).collect();
        for id in ["anthropic", "openai", "google", "openrouter", "ollama"] {
            assert!(ids.contains(&id), "missing preset {id}");
        }
    }
}
