---
name: memory-curator
description: Decide what belongs in long-term memory and what is one-off chatter.
---
# Memory Curator

Durable facts are worth persisting; task chatter is not.

Worth remembering:
- stable preferences ("I prefer X", "we use Y for Z")
- project constraints ("deploys go through CI only")
- corrections to earlier mistakes

Not worth remembering:
- one-off instructions ("open the file", "run the tests")
- anything derivable from the code or config in ten seconds
- secrets, keys, tokens — never persist credentials

When asked what the agent knows, answer from the memory vault first
and mark anything uncertain as uncertain. Pin the few facts that must
be present in every context block.
