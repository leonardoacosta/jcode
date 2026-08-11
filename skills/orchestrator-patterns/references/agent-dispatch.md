# Agent Dispatch Reference

> Fan-out/fan-in patterns for spawning and collecting parallel agents.

## Fan-Out Pattern

Spawn all independent agents in a **single message** with `run_in_background: true`. This maximizes
parallelism -- Claude Code dispatches all agents concurrently.

```
# Record start time BEFORE dispatch (captures queue + execution time)
for node in A1 A2 B1 B2; do
  orch_state_start_node "$SF" "phase_agents" "$node"
done

# Single message with N parallel Task calls
Task({ prompt: "...", run_in_background: true, timeout: 600000 })  # A1
Task({ prompt: "...", run_in_background: true, timeout: 600000 })  # A2
Task({ prompt: "...", run_in_background: true, timeout: 600000 })  # B1
Task({ prompt: "...", run_in_background: true, timeout: 600000 })  # B2
```

### Pre-Dispatch Checklist

1. Call `orch_state_start_node` for EACH agent BEFORE the dispatch message
2. Include `timeout: 600000` (10 minutes) on every Task call
3. Use `run_in_background: true` for all independent agents
4. Pass the full prompt inline -- agents cannot read orchestrator state

## Timeout and Graceful Degradation

When an agent times out:

1. Collect reports from all completed agents via `TaskOutput`
2. Record the timeout: `orch_state_update_node "$SF" "phase" "node" "failed" "timeout after 10m"`
3. Note the timed-out agent's scope as "not covered this cycle" in the report
4. Continue with remaining agents -- do NOT restart or retry the timed-out agent
5. The phase status auto-resolves to `partial` (not all nodes `completed`)

## Partial Resume on Re-Dispatch

Before spawning agents, check which ones already completed (from a prior crashed run):

```bash
AGENTS_TO_SPAWN=()
for node in A1 A2 B1 B2; do
  status=$(orch_state_node_status "$SF" "phase_agents" "$node")
  if [[ "$status" == "completed" ]]; then
    echo "Node $node already complete. Skipping."
  else
    AGENTS_TO_SPAWN+=("$node")
  fi
done

if [[ ${#AGENTS_TO_SPAWN[@]} -eq 0 ]]; then
  echo "All agents complete. Skipping to next phase."
else
  echo "Spawning agents: ${AGENTS_TO_SPAWN[*]}"
  for node in "${AGENTS_TO_SPAWN[@]}"; do
    orch_state_start_node "$SF" "phase_agents" "$node"
  done
  # Dispatch only AGENTS_TO_SPAWN in a single message
fi
```

## Agent Completion Callback

After each `TaskOutput` returns, update state:

```bash
# On success:
orch_state_update_node "$SF" "phase_agents" "A1" "completed" "$A1_REPORT"

# On failure:
orch_state_update_node "$SF" "phase_agents" "B1" "failed" "timeout after 10 minutes"
```

`wall_clock_ms` is auto-computed from the node's `started_at_ms`. No manual timing needed.

## Phase Completion Detection

`orch_state_update_node` auto-detects phase completion:

- All nodes `completed` => phase status = `completed`, `completed_at` set
- All nodes finished but some `failed` => phase status = `partial`
- Any node still `running` or `pending` => phase stays `running`

Check manually if needed:

```bash
phase_status=$(orch_state_phase_status "$SF" "phase_agents")
if [[ "$phase_status" == "completed" || "$phase_status" == "partial" ]]; then
  echo "Phase finished. Moving to next phase."
fi
```

## Choosing Agent Types

| Agent Type | Model | Use Case |
|------------|-------|----------|
| `Explore` (Haiku) | Cost-optimized | Read-only tasks: log analysis, finding collection, correlation |
| Default | Sonnet | Implementation tasks: code changes, spec creation |
| Default | Opus | Complex reasoning: architecture review, cross-domain synthesis |

Use Haiku `Explore` agents for read-only work (audits, signal collection). Reserve Sonnet/Opus for
tasks that produce code or require multi-step reasoning.

```
# Haiku agent for read-only work
Task({ subagent_type: "Explore", model: "haiku", timeout: 600000, prompt: "..." })

# Default agent for implementation work
Task({ timeout: 600000, prompt: "..." })
```

## Sequential Dependencies

When a phase depends on prior phase outputs, use explicit gates:

```
Phase 0 (preflight)  ──┐
                        ├──  Phase 1 (parallel agents)  ──┐
                        │                                   ├──  Phase 2 (correlation)
Phase 0 outputs        │                                   │
injected into          Phase 1 outputs                    Phase 2 uses Phase 1
agent prompts          passed forward                     reports as input
```

Pass prior phase outputs by:
1. Reading cached outputs from state: `jq '.phases[0].agent_outputs' "$SF"`
2. Injecting into agent prompts as context sections
3. Never relying on agents reading the state file directly
