---
name: rust-rescue
description: Drive a Rust workspace from broken to green — read compiler errors, fix in dependency order, re-check.
---
# Rust Rescue

When a Rust build fails:

1. Capture the full `cargo check` output; work through errors in order.
2. Fix the FIRST error before reading the rest — later errors are
   usually cascade noise.
3. Prefer the smallest correct change: match surrounding style, avoid
   new dependencies, avoid rewrites.
4. Never leave `todo!()`, `unimplemented!()` or stubbed bodies behind.
5. After the workspace checks clean, run the test suite and prove the
   fix did not change behavior beyond the intended repair.

Common Windows toolchain notes: a GNU toolchain needs w64devkit's gcc
first on PATH; rusqlite connections are !Send and cannot cross await
points or spawn_blocking closures.
