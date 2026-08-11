---
name: cc-runtime-evidence
description: >
  Prove a cc config change works at RUNTIME — hooks, session-primer, statusline, settings keys,
  preprocessor blocks, commands, agents. Maps each surface to its probe. Triggers: prove it fired,
  runtime evidence, verify hook, test the hook, did the primer, statusline change, settings change,
  verify config, wired vs live.
allowed-tools: Read, Bash, Grep, Glob
---

# CC Runtime Evidence

Harness binding for the portable **`runtime-evidence`** skill (promoted, revision
`e1de9d968`, `leo-core/skills/runtime-evidence/SKILL.md`) — read that skill for the
Wired/Fired/Effected ladder, the naming-the-assertion procedure, and the NEVER table. This file
supplies only what the promoted copy deliberately generalized away: cc's concrete surfaces,
mapped one-to-one to the exact probe command for each.

## Probe Table (cc surface -> exact probe)

| Surface | Probe |
| --- | --- |
| Hook script (new/changed) | (a) Execute directly with a synthetic stdin payload (shapes: `docs/research/hook-stdin-reference.md`) — paste stdout/exit; (b) trigger the real event once in a scratch context, then confirm via telemetry/marker/transcript per [references/probe-recipes.md](references/probe-recipes.md) |
| Hook WIRING (`settings.json` entry) | Rung 2 only: real-event trigger + fire evidence. If the event may be dead, cross-check the always-on `telemetry.sh` fire count on the same event — same-event silence for BOTH means dead event, not bad matcher |
| `settings.json` key (e.g. `continueOnBlock`) | jq the exact hook object for the key (rung 1), then exercise the path the key gates (e.g. force a `decision:block` and show the turn continues) |
| session-primer / SessionStart output | Run `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/session-primer` directly and paste the emitted lines; for wiring, grep the newest session transcript for the injected line |
| Statusline | Run the statusline command exactly as `settings.json` invokes it, paste rendered output; confirm cache freshness for pulse-style sources (`roadmap-pulse` cache mtime) |
| Bang-fenced preprocessor block | Run the script standalone: `--json` emits one JSON object AND `echo $?` = 0 under a broken precondition (unset env, missing dir) — a non-zero abort kills the whole command render |
| Detection script | Same as above, plus <200ms warm: `time <script> --json` |
| Command markdown | Invoke the command once (scratch args); confirm render includes injected JSON and the model pin took (transcript model ID) |
| Agent definition | Dispatch it once on a trivial in-domain task; confirm `agentType` in the new `subagents/agent-*.meta.json` and that frontmatter skills loaded |
| Skill description (auto-trigger) | Phrase a natural request that should match; confirm the skill loads (skill-list injection / transcript). For explicit-only: confirm the `Skill()` call site resolves |
| Pre-commit guard / git hook | Stage a synthetic violating file, run `git commit` (scratch branch or `--dry-run` path), paste the rejection; then unstage |
| systemd timer / cron | `systemctl --user list-timers <name>`, then output-file mtime after next window (or `systemctl --user start` the service once and paste the result-file diff) |

Recipes with exact payloads and marker paths: [references/probe-recipes.md](references/probe-recipes.md).
