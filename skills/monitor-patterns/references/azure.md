# Azure Pipeline Live Monitor (snapshot + stream)

Carried forward from the archived `/monitor:azure` command
(`script-spec-sync-and-fold-monitor` change) — LIVE present-tense visibility into an Azure
DevOps project's pipeline state and App Insights error stream. For RETROSPECTIVE log queries
(last-24h aggregations, top exceptions), use `/inspect:better-stack` instead.

## Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--prod` | Target production pipeline | off |
| `--dev` | Target dev pipeline | on |
| `--deploys` | Snapshot pipeline state and exit | off |
| `--logs` | Skip snapshot, tail App Insights only | off |

Default behaviour: two-phase — snapshot pipeline first, then tail App Insights
until the Monitor lifetime ends.

## Project Registry

Reads from `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/config/projects.json`'s `monitors.azure.{org,project}`
fields per project code. Projects with `monitors.azure.enabled=false` (or no
`monitors` block) are not Azure-monitorable; print instructive error and exit.

## Prerequisites

Requires `az` CLI authenticated (`az login`) with `azure-devops` extension and
App Insights read permission.

## Phase 1: Live Pipeline Snapshot

```bash
source ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/lib/monitor-helpers.sh
ORG=$(jq -r --arg c "$PROJECT_CODE" '.projects[] | select(.code == $c) | .monitors.azure.org' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/config/projects.json)
AZP=$(jq -r --arg c "$PROJECT_CODE" '.projects[] | select(.code == $c) | .monitors.azure.project' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/config/projects.json)
monitor_azure_pipeline "$ORG" "$AZP" "" 20
```

Emits `<state>\t<run_id>\t<url>` once on terminal state. Exits when the run
reaches succeeded / failed / canceled.

## Phase 2: Live App Insights Tail

```bash
APP_INSIGHTS_ID="${APP_INSIGHTS_ID:?APP_INSIGHTS_ID env var required}"
monitor_azure_logs "$APP_INSIGHTS_ID" 15
```

Emits `<severity>\t<operation>\t<msg>` per error event. Stream-only.

## Cross-References

- `references/vercel.md` — peer (Vercel deploy + log stream)
- `/inspect:better-stack` — RETRO log queries (time-window aggregation)
- `/monitor:triage` — upstream router (when symptoms aren't yet localized)
