# State Management Reference

> Crash-safe state persistence for multi-phase orchestrated commands.

## Naming Convention

All state and telemetry files follow a canonical naming pattern:

- **Directory:** `$HOME/.claude/scripts/state/`
- **State files:** `{command}-state.json` (e.g., `apply-state.json`, `p2p-state.json`)
- **Telemetry:** `{command}-telemetry.jsonl` (e.g., `feature-telemetry.jsonl`)
- **Cache:** `{command}-cache.json` (e.g., `stats-cache.json`)

Use the helper functions to derive paths instead of hardcoding:

```bash
SF=$(orch_state_path "my-command")       # → $HOME/.claude/scripts/state/my-command-state.json
TF=$(orch_telemetry_path "my-command")   # → $HOME/.claude/scripts/state/my-command-telemetry.jsonl
```

**Never** use `~/.claude/state/` (stale path) or relative paths (break across working directories).

## State File Schema

```json
{
  "git_sha": "abc1234",
  "started_at": "2026-03-08T14:00:00+00:00",
  "started_at_ms": 1741444800000,
  "phases": [
    {
      "name": "phase_0_preflight",
      "status": "completed",
      "started_at": "2026-03-08T14:00:00+00:00",
      "completed_at": "2026-03-08T14:01:30+00:00",
      "agent_outputs": {
        "result_key": "cached value for downstream phases"
      }
    },
    {
      "name": "phase_1_agents",
      "status": "partial",
      "started_at": "2026-03-08T14:01:30+00:00",
      "completed_at": null,
      "agent_outputs": {
        "agent_A": {
          "status": "completed",
          "report": "...",
          "started_at_ms": 1741444890000,
          "wall_clock_ms": 312000
        },
        "agent_B": {
          "status": "failed",
          "error": "timeout",
          "started_at_ms": 1741444890000,
          "wall_clock_ms": 600000
        }
      }
    },
    {
      "name": "phase_2_synthesis",
      "status": "pending",
      "started_at": null,
      "completed_at": null,
      "agent_outputs": {}
    }
  ]
}
```

## Phase Status Values

| Status | Meaning |
|--------|---------|
| `pending` | Not yet started |
| `running` | Currently executing (at least one node started) |
| `partial` | All nodes finished but some failed |
| `completed` | All nodes succeeded |
| `failed` | Phase-level failure (not node-level) |

## Crash Resume Flow

On every invocation, run `orch_check_resume` before any work:

```
1. orch_check_resume $STATE_FILE [--fresh]
   ├─ --fresh flag present    → delete state, return 1 (fresh start)
   ├─ No state file           → return 1 (fresh start)
   ├─ SHA mismatch            → delete state, return 1 (fresh start)
   └─ SHA matches             → print completed phases, return 0 (resume)

2. If fresh (return 1):
   orch_state_init $STATE_FILE $SHA phase_0 phase_1 phase_2

3. Before each phase:
   status=$(orch_state_phase_status $STATE_FILE phase_N)
   if [[ "$status" == "completed" ]]; then
     # Restore cached outputs from agent_outputs
     CACHED=$(jq -r '.phases[] | select(.name == "phase_N") | .agent_outputs.key' $STATE_FILE)
     # Skip to next phase
   fi
```

## Atomic Write Pattern

All state mutations use the write-tmp-then-mv pattern:

```bash
jq '...' "$state_file" > "${state_file}.tmp" && mv -f "${state_file}.tmp" "$state_file"
```

This prevents partial writes from corrupting state on crash. The `mv` is atomic on POSIX
filesystems (same partition). All `orch_state_*` functions use this pattern internally.

## Single-Writer Invariant

Only one process writes state at a time. The orchestrator is the sole writer -- agents return
reports to the orchestrator, which calls `orch_state_update_node`.

If concurrent writes are suspected, check mtime:

```bash
mtime_ms=$(stat -c %Y%3N "$STATE_FILE" 2>/dev/null || echo 0)
now_ms=$(date +%s%3N)
if [[ $((now_ms - mtime_ms)) -lt 500 ]]; then
  echo "WARNING: State file written <500ms ago. Possible concurrent writer."
fi
```

## Phase Skip Logic with Cached Output Restoration

Completed phases store their outputs in `agent_outputs`. Downstream phases restore these values
instead of re-executing:

```bash
if [[ "$(orch_state_phase_status "$SF" phase_0)" == "completed" ]]; then
  echo "Phase 0 already complete. Restoring cached outputs."
  STRIPE_STATUS=$(jq -r '.phases[0].agent_outputs.stripe_status' "$SF")
  REG_RESULTS=$(jq -r '.phases[0].agent_outputs.regression_results' "$SF")
else
  # Run phase 0, then persist outputs:
  orch_state_complete "$SF" "phase_0" \
    "{\"stripe_status\":\"$STRIPE_STATUS\",\"regression_results\":\"$REG_RESULTS\"}"
fi
```

## Partial Phase Resume (Node-Level)

For phases with multiple agents, check each node individually before re-dispatching:

```bash
AGENTS_TO_SPAWN=()
for node in A1 A2 B1 B2; do
  status=$(orch_state_node_status "$SF" "phase_1" "$node")
  if [[ "$status" == "completed" ]]; then
    echo "Node $node already complete. Skipping."
  else
    AGENTS_TO_SPAWN+=("$node")
  fi
done

# Only dispatch incomplete nodes
for node in "${AGENTS_TO_SPAWN[@]}"; do
  orch_state_start_node "$SF" "phase_1" "$node"
  # dispatch agent...
done
```

## State Cleanup

After the final phase completes successfully, remove the state file:

```bash
orch_state_complete "$SF" "phase_final" '{"report_generated": true}'
echo "All phases complete. Cleaning up state file."
rm -f "$SF"
```

Keep the state file if any phase is `partial` or `failed` -- it enables targeted re-runs.
