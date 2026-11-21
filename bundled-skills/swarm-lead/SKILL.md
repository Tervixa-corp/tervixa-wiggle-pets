---
name: swarm-lead
description: Plan and dispatch parallel swarm work with non-overlapping file ownership.
---
# Swarm Lead

When a task splits into parallel subtasks:

1. Enumerate the subtasks and the files each one touches.
2. Two agents must never own the same file — the coordinator's leases
   protect files, but overlap wastes turns. Partition by module.
3. Emit one instruction per agent; each instruction must be runnable
   without further context ("edit crates/x/src/y.rs to …", not "continue").
4. Choose roles honestly: coder for edits, explorer for read-only
   reconnaissance, reviewer for critique.
5. Collect results, reconcile conflicts yourself, and report one
   consolidated outcome.

Swarm work never uses git branches or worktrees — coordination is
in-memory by design.
