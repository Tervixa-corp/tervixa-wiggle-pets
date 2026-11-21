# WIGGLE.md — Tervixa Wiggle Project Instructions

This file is automatically injected into every prompt compiled by
Tervixa Wiggle when working inside this repository (Step 3 of the
7-Stage Prompt Compiler).

## Project

Tervixa Wiggle is an autonomous AI software engineering agent distributed as:

- `wiggle` — a standalone CLI / TUI binary (`crates/wiggle-cli`)
- Tervixa Wiggle Desktop — a native cross-platform desktop app
  (Tauri v2 shell in `crates/wiggle-desktop`, React workbench in
  `apps/desktop-ui`)

Both frontends share 100% of the engine in `crates/wiggle-core` and the
personal memory engine in `crates/wiggle-memory`.

## Engineering rules

- Rust workspace at the repository root; every crate is `crates/*`.
- No `todo!()`, no unimplemented stubs, no empty placeholder files.
- Errors are `thiserror` types; no `unwrap()` outside tests and `main()`.
- Blocking I/O (SQLite, process spawns) never runs on the async executor
  without `spawn_blocking` or an explicit thread.
- The memory engine must degrade gracefully: if `~/.wiggle/memory/wiggle.db`
  cannot be opened, the prompt compiler proceeds without personal context.

## UI rules (apps/desktop-ui)

- Strictly flat, utilitarian, monochrome design. Zero gradients, zero
  glow shadows, zero neon accents, border-radius at most 4px (6px for
  modals). Color tokens are defined once in `src/styles/minimal.css`.
- The workbench layout is fixed: 44px activity rail, 240px sidebar,
  flexible center, 380px auxiliary pane, 32px titlebar, 24px status bar.

## Git rules

- All work happens directly on the `main` branch. No feature branches,
  no worktrees, no pull requests from secondary branches.
- Commit messages are clear milestone descriptions.
