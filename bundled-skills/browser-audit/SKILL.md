---
name: browser-audit
description: Verify a running web app through the built-in browser tools and report concrete evidence.
---
# Browser Audit

Use browser_navigate / browser_actuate / browser_inspect to:

1. Open the app URL (dev server, localhost build, or deployed page).
2. Exercise the flow the task touched — click, type, submit.
3. Report what you SAW: headings, errors, empty states, console
   messages included in the snapshot.

Rules:
- Never claim a fix works without loading the page after the change.
- Quote the evidence (element text, error strings) rather than
  summarizing vaguely.
- If the app fails to load, report the exact failure; do not guess.
