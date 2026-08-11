---
name: monitor-patterns
description: "cc harness binding for the monitor-patterns concept (Monitor primitive name, helper signatures, provider recipes). Explicit-only, @skill-loaded."
user-invocable: false
allowed-tools: Read, Glob, Grep, Bash
---

# Monitor Patterns (cc binding)

The portable concept — decision matrix, three canonical patterns, poll-loop anti-pattern — is
promoted upstream (`runtime-kit`, released at
`e1de9d9680ab028e4d0777ae23009081e43582ac`). Read it there; this file states only what is true
of cc and false elsewhere.

| cc name | Binds to |
| --- | --- |
| `Monitor` primitive | Built-in tool exposed to cc agents/commands as `Monitor` |
| Helper library | `~/dev/claude/scripts/lib/monitor-helpers.sh` (`source` it before calling any helper below) |
| Project registry | **absent** — `scripts/config/projects.json` no longer exists; the registry moved to per-project `.claude/project.toml`, which carries no `monitors.{vercel,azure}` gate yet. `cc-debt: cite the stale path rather than invent a replacement, re-point once the gate has a home [beads:cc-35osa]` |
| Canonical patterns, decision matrix, poll-loop anti-pattern | `monitor-patterns` skill, `runtime-kit` kit, upstream skills repo (`~/dev/personal/skills/runtime-kit/skills/monitor-patterns/SKILL.md`) |

## Canonical helper signatures

```bash
source ~/dev/claude/scripts/lib/monitor-helpers.sh
```

| Helper | Signature | Purpose |
|--------|-----------|---------|
| `monitor_turbo_stream` | `TURBO_ARGS...` | Per-package / per-spec completion lines from a turbo or test run |
| `monitor_vercel_deploy` | `PROJECT BRANCH [POLL_SECONDS]` | READY / ERROR transitions on a deployment |
| `monitor_gh_ci` | `BRANCH [POLL_SECONDS]` | CI terminal state for a branch |
| `monitor_gh_comments` | `PR_NUMBER BOT_LOGIN [POLL_SECONDS]` | Stream bot review comments as they post |
| `monitor_lock_files` | `LOCK_DIR PATTERN_REGEX` | inotify-based file-watch with ls fallback |

Full signature list (including the provider helpers used by the reference recipes below) and all
defaults/edge cases live in the library file itself — read `scripts/lib/monitor-helpers.sh`
directly rather than duplicating it here.

## Provider recipes (cc-specific — not promoted)

These name cc's project registry and its (retired) `/monitor:*`/`/inspect:*` command surface, so
they stay local rather than moving upstream. Required by `openspec/specs/command-monitor-integration/spec.md`.

| Provider | Reference |
|---|---|
| Vercel deploy pipeline (snapshot + log tail) | `references/vercel.md` |
| Azure Pipelines (snapshot + App Insights tail) | `references/azure.md` |
| Better Stack (uptime + Logtail stream) | `references/better-stack.md` |

## Cross-Reference

Full primitive specification: `openspec/specs/command-monitor-integration/spec.md`. Portable
teaching content (decision matrix, three canonical patterns, poll-loop anti-pattern): upstream
`monitor-patterns` skill (see table above) — do not restate it here; cite it.
