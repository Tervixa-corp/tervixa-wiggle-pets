---
name: commit-shepherd
description: Write clean, atomic Conventional Commits messages from a diff or a set of changes.
---
# Commit Shepherd

Given a diff, staged changes, or a short description of work:

1. Group changes into atomic commits — one concern per commit.
2. Use Conventional Commits: `type(scope): intent` where type is one of
   feat, fix, refactor, docs, test, chore, perf.
3. The subject line states intent, never mechanics ("cache the parsed
   config", not "change fn load to keep a field").
4. Add a body only when the why is not obvious from the code.

Never commit secrets, generated assets, or unrelated files. If the
diff touches unrelated concerns, propose the split before committing.
