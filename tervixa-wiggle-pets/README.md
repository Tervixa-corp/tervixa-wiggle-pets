# TERVIXA WIGGLE PETS

**A deterministic desktop pet that lives on your screen and watches your AI coding agents — by [Tervixa Forge](https://forge.tervixa.in)** · v1.0.0 · MIT

A transparent always-on-top pet overlay for Windows / Linux / macOS. Your pet walks, runs, sleeps, climbs screen bezels, fetches balls and stares at your cursor — and reacts in real time to what Claude Code, Codex, Cursor, OpenCode, Open Interpreter and friends are doing.

**The engine is 100% deterministic, zero-AI.** Reminders, hydration nudges, pomodoro, agent watching, permission approvals — all native Rust, no tokens, no network calls, fully offline. AI is *only* used for the optional pet chat, and only if you bring a model.

```
┌──────────────────────────────────────────────────────────┐
│  ☁ Nimbus · "stormy flashes gold when an agent fails"    │
│                                                          │
│   ┌──────────────────────────────────────┐               │
│   │ CLAUDE CODE · my-app                 │               │
│   │ Permission needed:                   │               │
│   │ > Bash: rm -rf node_modules          │               │
│   │ [Approve] [Always Allow] [Deny] [Reply]│             │
│   └──────────────────────────────────────┘               │
│                                                          │
│                                    ● claude working      │
│                                 ○ codex idle            │
└──────────────────────────────────────────────────────────┘
```

## Features

### 🐾 23 bundled pets
- **18 sprite pets** from [VS Code Pets](https://github.com/tonybaloney/vscode-pets) (MIT) — akita & shiba dogs, cats, cyber fox, Ferris crab, panda, rubber duck, snail, turtle, cockatiel, raccoon, rat, chicken, horse, monkey. Full credit in [`assets/pets/ATTRIBUTION.md`](assets/pets/ATTRIBUTION.md).
- **Cloud Pets** — the signature original collection: Nimbus, Stormy, Aurora, Cotton. Procedurally drawn on canvas: they bob, rain code while your agent works, flash lightning on notifications, and rain droplets during hydration breaks.
- **Wiggle Bot** — original Tervixa pixel coding companion with a status LED and waving arm.
- **GitHub Pet Explorer** — install any community pet by `user/repo` manifest URL.

### 🤖 AI agent watcher (deterministic)
Installs **marker-scoped, idempotent hooks** into each agent's own config — uninstall anytime, and hooks **fail open** (quit the pet, everything behaves exactly as stock):

| Agent | Mechanism |
|---|---|
| Claude Code | hooks in `~/.claude/settings.json` (UserPromptSubmit, PreToolUse, PermissionRequest blocking HTTP hook, Stop…) |
| Codex | `~/.codex/hooks.json` (SessionStart, PreToolUse, PermissionRequest, Stop, Interrupt…) |
| Cursor | `~/.cursor/hooks.json` (beforeSubmitPrompt, beforeToolUse…) |
| OpenCode | plugin in `~/.config/opencode/plugin/` (permission.asked, session.status…) |
| Anything else | process watcher — detects running agent CLIs, one-click **Interrupt** |

Live agent states map to pet behaviour: `thinking → idle-sniff`, `working → walk/run`, `attention → jump & wave`, `notification → swipe + bubble`, `error → lie down X-eyes`.

### ✅ Permission control from the pet
When any agent asks for permission, a minimal monochrome card pops over the pet:

- **Approve** (Enter) · **Always Allow** (policy rule) · **Deny** (Esc) · **Reply** (send feedback to the agent)
- **Auto-pilot policy engine** (pure rules, no AI): auto-approves safe read-only commands (`ls`, `cat`, `git status`, `cargo check`, `pytest`…), always surfaces dangerous ones (`rm -rf`, `git push --force`, `sudo`, credentials…).

### ⏰ Timed skills (zero AI, SQLite-backed)
- **Hydration** — interval + daily goal + active hours + streak tracking, water-drinking pet animation.
- **Reminders** — natural-language parsed by a deterministic Rust parser: *"remind me every monday and thursday at 6pm to back up the database"*, *"in 20 minutes check the build"*, *"daily at 7am stretch"*. No LLM. Ever.
- **Pomodoro** — 25/5 focus rounds with pet focus pose.

### 💬 Lightweight AI chat (optional)
Alt+Space quick command. Reminders/notes typed here are still parsed with **zero AI**. For chat replies only, pick:

- **100% No-AI mode** (default) — chat disabled, everything else works.
- **Local** — Ollama / LM Studio / llama.cpp, 1-click pull of tiny models (Qwen 0.5B–1.5B, Llama 3.2 1B, Gemma 2 2B, Phi-3 mini).
- **Cloud BYOK** — OpenAI, Anthropic, OpenRouter, DeepSeek, GLM, Kimi, Groq, Mistral. Keys are read from env vars — never stored, never logged.

## Architecture

```
tervixa-wiggle-pets/
├── src-core/          Rust deterministic engine (std-only HTTP, rusqlite, no async runtime)
│   ├── protocol.rs    wire types + agent-event → pet-state mapping
│   ├── timed_skills/  hydration, reminders, pomodoro + NL time parser (unit tested)
│   ├── agent_watcher/ process watcher + hook installers for 4 agents
│   ├── permission_bridge/ auto-approve policy + pending queue
│   ├── pet_registry/  pet.json manifests + GitHub downloader
│   ├── ai_router/     chat-only model catalog + clients
│   └── server.rs      daemon on 127.0.0.1:23340 (/state, /hook/*, /api/*)
├── src-ui/            Electron overlay: pet scene, bubbles, Control Center
├── assets/pets/       23 pets + ATTRIBUTION.md
├── releases/          per-OS packaging
└── .github/workflows/ build + release CI
```

Hook transport: agents POST to `127.0.0.1:23340-23347` with a ~100 ms fail-open timeout; the UI long-polls `/api/events`. Loopback only — nothing leaves your machine.

## Run from source

```bash
# 1. Rust core
cd src-core
cargo build --release        # windows-gnu: cargo +stable-x86_64-pc-windows-gnu build --release
# 2. UI
cd ..
npm install                  # postinstall builds the core if Rust is present
npm start
```

## Licence

MIT © Tervixa Corporation. Bundled sprites are MIT with per-artist credits in [`assets/pets/ATTRIBUTION.md`](assets/pets/ATTRIBUTION.md). AGPL-licensed pet projects inspired the hook architecture but **no artwork or code** was copied from them.
