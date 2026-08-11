# Better Stack Live Monitor (stream-only)

Carried forward from the archived `/monitor:better-stack` command
(`script-spec-sync-and-fold-monitor` change) — LIVE present-tense visibility into Better Stack
uptime monitor state and a filtered error log tail. RETROSPECTIVE log queries (last-N-hours
aggregations, top-error grouping, time-window analysis) live in `/inspect:better-stack`.

## Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--uptime` | Stream uptime monitor state changes only | off |
| `--logs` | Stream Logtail error events only | off |

Default behaviour (no flag): stream both — uptime + logs interleaved.

## Prerequisites

- `BETTERSTACK_API_TOKEN` env var (Bearer token, monitor read scope)
- `LOGTAIL_SOURCE_TOKEN` env var for `--logs` (per-source token)
- `mcp__betterstack__uptime_list_monitors_tool` MCP available as fallback
  (helper uses REST API directly; MCP is for richer triage workflows)

## Stream: Live Uptime + Logtail

```bash
source ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/lib/monitor-helpers.sh

# Run both helpers in parallel — Monitor invocation captures both stdouts.
if [ "$MODE" != "logs" ]; then
  monitor_bstack_uptime 30 &
fi
if [ "$MODE" != "uptime" ]; then
  monitor_bstack_logtail "$LOGTAIL_SOURCE_TOKEN" 15 &
fi
wait
```

Stream-only — no terminal state. The Monitor invocation bounds runtime.

## Cross-References

- `references/vercel.md` — peer (Vercel deploy + log stream)
- `/monitor:posthog` — peer (PostHog error + trace stream)
- `/monitor:triage` — upstream router (when symptoms aren't yet localized)
- `/inspect:better-stack` — RETRO sibling (time-window aggregation, last-N-hours)
- `/inspect:posthog` — RETRO sibling (analytics dashboards)
