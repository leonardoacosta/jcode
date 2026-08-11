# Vercel Live Monitor (snapshot + stream)

Carried forward from the archived `/monitor:vercel` command
(`script-spec-sync-and-fold-monitor` change) — LIVE present-tense visibility into a Vercel
project's deploy pipeline and runtime log stream. For RETROSPECTIVE log queries (time-window
aggregation, historical errors), use `/inspect:better-stack` instead.

## Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--prod` | Target production branch | off |
| `--dev` | Target dev branch | on |
| `--deploys` | Snapshot deploy state and exit | off |
| `--logs` | Skip snapshot, tail logs only | off |
| `--json` | Structural JSON output for the snapshot | off |

Default behaviour (no `--deploys` / `--logs`): two-phase — snapshot first, then
stream logs until the Monitor lifetime ends.

## Project Registry

Reads from `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/config/projects.json`'s `monitors.vercel.project`
field per project code. Projects with `monitors.vercel.enabled=false` (or no
`monitors` block) are not Vercel-monitorable; print instructive error and exit.

## Prerequisites

Requires `VERCEL_TOKEN` environment variable (from `.env`) and `vercel` CLI
authenticated.

## Phase 1: Live Deploy Snapshot

```bash
source ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/lib/monitor-helpers.sh
PROJECT=$(jq -r --arg c "$PROJECT_CODE" '.projects[] | select(.code == $c) | .monitors.vercel.project' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/config/projects.json)
BRANCH=$([ "$TARGET" = "prod" ] && echo "main" || echo "dev")
monitor_vercel_deploy "$PROJECT" "$BRANCH" 15
```

Emits `<state>\t<deploy_id>\t<url>` once on terminal state. Exits when the
deploy reaches READY / ERROR / CANCELED. Skipped if `--logs` is passed.

## Phase 2: Live Log Tail

```bash
monitor_vercel_logs "$PROJECT" "$BRANCH" 15
```

Emits `<level>\t<route>\t<msg>` per error/warning event. Stream-only — no
terminal state; the Monitor invocation bounds runtime.

## Cross-References

- `/inspect:better-stack` — RETRO log queries (time-window aggregation)
- `/ci:gh` — CI/build failure diagnosis and auto-fix
- `/monitor:triage` — upstream router (when symptoms aren't yet localized)
- `references/azure.md` — peer (Azure pipeline snapshot + App Insights tail)
