---
description: Toggle llmtrim subscription rerouting for this Claude Code window only.
disable-model-invocation: true
argument-hint: "on [codex|kimi|grok]|off|status"
allowed-tools: Bash('/home/nyaptor/.local/share/mise/installs/node/25.0.0/lib/node_modules/@llmtrim/cli/node_modules/@llmtrim/linux-x64/bin/llmtrim' window-sub slash *)
---

<!-- llmtrim-owned-window-sub -->
Window-local subscription override (does not change other windows or the global
`llmtrim sub` setting).

- `/sub on` — enable using the last provider for this window, or the global `sub`
- `/sub on codex` / `/sub on kimi` / `/sub on grok` — enable a specific provider
- `/sub off` — force Anthropic (with compression) for this window
- `/sub status` — show this window's override

!`'/home/nyaptor/.local/share/mise/installs/node/25.0.0/lib/node_modules/@llmtrim/cli/node_modules/@llmtrim/linux-x64/bin/llmtrim' window-sub slash "$ARGUMENTS"`
