# TERVIXA WIGGLE

**An AI coding agent, owned end-to-end: a Rust engine, a terminal UI, and a native desktop workbench.**

Tervixa Wiggle imitates the architecture of the best agents in the
field — OpenCode, Claude Code, Codex, Cursor, Antigravity — and keeps
the whole stack local: your keys, your files, your memory vault, your
swarm.

```
┌────────────────────────────────────────────────────────────┐
│  TERVIXA WIGGLE 1.0.0                       [aux] [set]    │
├────┬───────────┬─────────────────────────────┬────────────┤
│ »  │ Providers │  you: ship the auth module  │  output    │
│ f  │ ● Anthr.  │                             │  memory    │
│ m  │ ● OpenAI  │  wiggle: 7-stage plan, 3    │  browser   │
│ x  │ ○ Gemini  │  swarm agents dispatched…   │  raw       │
├────┴───────────┴─────────────────────────────┴────────────┤
│ idle · anthropic · claude-fable-5.2 · skills 5 · mcp 2    │
└────────────────────────────────────────────────────────────┘
```

## Layout

| Crate | What it is |
|---|---|
| `crates/wiggle-core` | The engine: config, providers, prompt compiler, tools, agents/swarm, MCP, skills, hooks, permissions, undo |
| `crates/wiggle-memory` | SQLite + FTS5 memory vault with hybrid vector/keyword recall |
| `crates/wiggle-browser` | CDP browser controller (Chrome/Edge) driving web verification |
| `crates/wiggle-cli` | `wiggle` CLI + ratatui monochrome TUI |
| `crates/wiggle-desktop` | Tauri v2 native shell |
| `apps/desktop-ui` | React 19 + Vite flat monochrome workbench |

## Identity (immutable)

- **Product:** Tervixa Wiggle
- **Prefix:** every session starts as `Tervixa Wiggle, ver <version>…`
- **7-stage prompt compiler:** normalize → anchor identity → load
  skills → recall memory → route task (single/swarm) → gate
  permissions → emit system prompt.

## Feature map

- **BYOK providers** — 3 wire dialects (Anthropic, OpenAI-compatible,
  Gemini), 20+ built-in presets, and the models.dev catalog (300+
  providers) cached to `~/.wiggle/models-dev.json`. Local runtimes
  included: ollama, vllm, litellm/OmniRoute gateway, LM Studio,
  llama.cpp, colibri.
- **Parallel swarm** — multi-agent dispatch with in-memory file
  leases (never git branches or worktrees). The compiler routes
  "…and then…", "in parallel", "meanwhile" prompts to a coder +
  explorer pair automatically.
- **MCP client** — stdio, SSE and streamable HTTP transports; tools,
  resources and prompts; one-click import of Claude Desktop configs.
- **Skills** — `SKILL.md` packages (bundled + `~/.wiggle/skills` +
  `.wiggle/skills`), importable from Claude Code or any GitHub repo.
- **Lifecycle hooks** — UserPromptSubmit, PreToolUse, PostToolUse,
  Notification, SubagentStop, Stop. A hook prints
  `{"decision":"deny","reason":"…"}` to block an action.
- **Memory** — every session learns; durable facts, preferences and
  summaries land in `~/.wiggle/memory/wiggle.db` (SQLite + FTS5 +
  64-dim hashed embeddings). If the vault can't open, the engine runs
  without memory — never blocked.
- **Undo/redo** — content snapshots in `.wiggle/snapshots/`; every
  write/edit is reversible, `/undo` and `/redo` included in the TUI.
- **Permissions** — three sandbox tiers (suggest / auto-edit /
  full-auto) plus per-tool allow/ask/deny and command allow/deny
  lists, mirroring Codex.
- **Tools** — read_file, write_file, edit_file, multi_edit, glob,
  grep, list_dir, bash (persistent shell), web_fetch, web_search,
  browser_navigate, browser_actuate, browser_inspect, subagent, and
  every MCP tool as `mcp_<server>_<name>`.

## Build

```sh
cargo check --workspace        # Rust engine + shells
cargo test --workspace
(cd apps/desktop-ui && npm install && npm run build)
cargo build --release -p wiggle-cli       # the `wiggle` binary
cargo build --release -p wiggle-desktop   # the native desktop app
```

## Run

```sh
wiggle run "refactor the auth module and then update the tests"
wiggle tui                      # full-screen monochrome TUI
wiggle status                   # providers, skills, memory
wiggle providers                # provider presets + env keys
wiggle skills                   # loaded SKILL.md packages
wiggle undo | redo              # snapshot rollback
wiggle compile "task text"      # inspect the 7-stage compiled prompt
```

The desktop app opens the workbench window: fixed 32px titlebar,
44px rail, 240px sidebar, 380px aux pane, 24px status bar — flat
monochrome, no gradients, no glow.

## Releases

Push a `v*` tag. The `release` workflow builds Windows/Linux/macOS
artifacts, packages them with `scripts/package-releases.sh` into the
`releases/` installer layout, and attaches them to the GitHub
release with `SHA256SUMS.txt`.

## License

MIT — see [LICENSE](LICENSE). Part of the **Tervixa Forge**
toolchain.
