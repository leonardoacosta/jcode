---
name: monitor
description: Monitor live CI, test, and deployment runs by routing to the correct script for Vercel, GitHub Actions, Depot CI, Azure DevOps pipelines, or Azure classic releases. Use this when a user asks to watch a run, poll for completion, or report terminal state updates. Tests poll every 5 minutes by default; deploys poll every minute by default.
user-invocable: false
allowed-tools: Read, Glob, Grep, Bash(${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-*)
---

# Monitor

Use this skill when the task is to watch a live remote workflow until it reaches
terminal state. This skill is agent-agnostic: it routes to plain shell scripts
with stable stdout and exit-code semantics instead of depending on first-class
command primitives. Tests poll every 5 minutes by default. Deploys poll every
minute by default.

## Routing

| Surface | Script | Default poll |
| --- | --- | --- |
| Vercel deployment | `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-vercel-deploy PROJECT BRANCH` | 60s |
| GitHub Actions / hosted test workflows | `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-gh-actions BRANCH OWNER/REPO` | 300s |
| Depot CI run | `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-depot-ci RUN_ID` | 300s |
| Azure DevOps pipeline | `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-azure-pipeline ORG PROJECT [PIPELINE_ID]` | 300s |
| Azure DevOps classic release | `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/monitor-azure-release ORG PROJECT DEFINITION_ID [ENVIRONMENT]` | 60s |

## Contract

- Emit concise state lines only. Do not dump full JSON unless the user asks.
- Success terminal state exits `0`.
- Failure terminal state exits `1`.
- Usage or configuration failure exits `2`.
- Timeout exits `124` when `--timeout-seconds` is supplied.

## Clarification Rules

- If the provider is explicit, route directly.
- If the user says "deploy", prefer the deploy monitors.
- If the user says "tests", "CI", "workflow", or "jobs", prefer the test/CI monitors.
- Ask only for the minimum missing identifier:
  Vercel needs `PROJECT` and `BRANCH`.
  GitHub Actions needs `BRANCH` and `OWNER/REPO`.
  Depot needs `RUN_ID`.
  Azure pipeline needs `ORG` and `PROJECT`, with optional `PIPELINE_ID`.
  Azure classic release needs `ORG`, `PROJECT`, and `DEFINITION_ID`, with optional environment selector.

## How To Use

1. Read [providers.md](./references/providers.md) for the target surface only.
2. Choose the provider script from the routing table.
3. Pass `--poll-seconds` only when the user wants a non-default cadence.
4. Pass `--timeout-seconds` when the watch should stop after a fixed window.
5. Report the latest state line and whether the exit code was success, failure, config error, or timeout.

## Examples

See [examples.md](./references/examples.md) for concrete request-to-command mappings.
