//! 1-click MCP connector catalog: well-known servers installable from
//! the Settings > MCP Connectors panel.

use crate::config::{McpServerConfig, McpTransport};

pub struct ConnectorPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub endpoint: &'static str,
    pub args: &'static [&'static str],
    pub transport: McpTransport,
}

pub static CATALOG: &[ConnectorPreset] = &[
    ConnectorPreset {
        name: "filesystem",
        description: "Read/write/search files in an allowed directory tree (@modelcontextprotocol/server-filesystem)",
        endpoint: "npx",
        args: &["-y", "@modelcontextprotocol/server-filesystem", "."],
        transport: McpTransport::Stdio,
    },
    ConnectorPreset {
        name: "git",
        description: "Git repository inspection and operations (@modelcontextprotocol/server-git)",
        endpoint: "uvx",
        args: &["mcp-server-git", "--repository", "."],
        transport: McpTransport::Stdio,
    },
    ConnectorPreset {
        name: "fetch",
        description: "Fetch web pages and convert to markdown (mcp-server-fetch)",
        endpoint: "uvx",
        args: &["mcp-server-fetch"],
        transport: McpTransport::Stdio,
    },
    ConnectorPreset {
        name: "memory",
        description: "Persistent knowledge-graph memory (@modelcontextprotocol/server-memory)",
        endpoint: "npx",
        args: &["-y", "@modelcontextprotocol/server-memory"],
        transport: McpTransport::Stdio,
    },
    ConnectorPreset {
        name: "github",
        description: "GitHub API: repos, issues, PRs, files (official)",
        endpoint: "https://api.githubcopilot.com/mcp/",
        args: &[],
        transport: McpTransport::Http,
    },
    ConnectorPreset {
        name: "sequential-thinking",
        description: "Structured step-by-step reasoning engine (@modelcontextprotocol/server-sequential-thinking)",
        endpoint: "npx",
        args: &["-y", "@modelcontextprotocol/server-sequential-thinking"],
        transport: McpTransport::Stdio,
    },
    ConnectorPreset {
        name: "puppeteer-browser",
        description: "Headless browser automation (puppeteer-mcp-server)",
        endpoint: "npx",
        args: &["-y", "@modelcontextprotocol/server-puppeteer"],
        transport: McpTransport::Stdio,
    },
];

pub fn to_config(preset: &ConnectorPreset) -> McpServerConfig {
    McpServerConfig {
        name: preset.name.to_string(),
        transport: preset.transport,
        endpoint: preset.endpoint.to_string(),
        args: preset.args.iter().map(|s| s.to_string()).collect(),
        env: Default::default(),
        enabled: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_presets_convert() {
        for preset in CATALOG {
            let cfg = to_config(preset);
            assert_eq!(cfg.name, preset.name);
            assert!(!cfg.endpoint.is_empty());
        }
    }
}
