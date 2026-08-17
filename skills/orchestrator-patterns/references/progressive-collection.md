# Progressive Collection Reference

> Stream outputs as agent groups complete. Do not wait for all agents before acting on results.

## Core Principle

Process findings incrementally as each agent finishes. This enables downstream work (spec creation,
correlation, synthesis) to overlap with ongoing agent execution.

## Streaming Output Pattern

Instead of waiting for all N agents, act on results as groups complete:

```
Timeline:
  A1 completes ──> collect A1 report
  A2 completes ──> collect A2 report ──> run downstream on {A1, A2}
  B1 completes ──> collect B1 report
  B2 completes ──> collect B2 report ──> run downstream on {B1, B2}
  C1 completes ──> collect C1 report ──> run final synthesis on {all}
```

### Implementation

Track completed agents and trigger downstream work at group boundaries:

```bash
COMPLETED_AGENTS=()

# After each TaskOutput:
COMPLETED_AGENTS+=("$AGENT_NAME")

# Check if a group is fully done
if all_in_group "attendee" "${COMPLETED_AGENTS[@]}"; then
  # Trigger incremental downstream (e.g., /audit-waves for attendee findings)
fi
```

## Incremental Correlation Buffer

For commands with a correlation phase (cross-referencing findings), run correlation incrementally
as each agent group finishes:

```
CORRELATED_AGENTS=[]

for agent in [completed_agents not in CORRELATED_AGENTS]:
  # Spawn Haiku correlator with:
  #   - The newly completed agent's report
  #   - Prior correlation results (C1a findings, etc.)
  #   - List of already-correlated finding IDs
  Task({
    subagent_type: "Explore",
    model: "haiku",
    timeout: 120000,
    prompt: "Correlate findings from $AGENT with production signals.
      Skip already-correlated IDs: $ALREADY_CORRELATED_IDS
      Return: { agent, new_correlations: [...], skipped_dupes: N }"
  })
  CORRELATED_AGENTS.push(agent)
```

### Deduplication Across Batches

Each incremental correlation batch receives the set of already-processed finding IDs from prior
batches. The correlator MUST:

1. Skip findings whose ID appears in `$ALREADY_CORRELATED_IDS`
2. Only emit NEW correlations not seen in prior batches
3. Return all correlated IDs (old + new) for the next batch

```
Batch 1: IDs [1,2,3] -> correlations for 1,3 -> ALREADY = [1,3]
Batch 2: IDs [4,5,6] + skip [1,3] -> correlations for 5 -> ALREADY = [1,3,5]
Batch 3: IDs [7,8] + skip [1,3,5] -> correlations for 7,8 -> ALREADY = [1,3,5,7,8]
Final:   Cross-agent patterns only, skip [1,3,5,7,8]
```

## Progressive Downstream Dispatch

Run downstream commands as early results arrive, not after all agents finish:

```
Agent Group      Downstream Action           When
─────────────    ─────────────────────────    ─────────────────────
Attendee (A1+A2) /audit-waves (incremental)  Both attendee agents done
Staff (B1+B2)    /audit-waves (incremental)  Both staff agents done
Design (D1)      Include in next waves run   D1 done
Signals (C1a)    Enable correlation           C1a done
Correlation      /audit-waves --full          All agents + correlation done
```

This overlaps spec creation with ongoing auditing -- specs from attendee findings are being
created while staff agents are still running.

## Final Synthesis Pass

After ALL agents complete, run a final pass that:

1. Cross-references findings across ALL agent groups (not just within groups)
2. Identifies patterns that span multiple domains
3. Deduplicates findings that were reported by both attendee and staff agents
4. Produces the unified report with merged per-domain sections

The final pass receives all prior incremental results and focuses on cross-cutting concerns only --
it does not re-process findings already handled incrementally.

## Anti-Patterns

| Pattern | Problem | Fix |
|---------|---------|-----|
| Wait for all agents before acting | Wastes wall-clock time | Process groups incrementally |
| Re-process all findings in final pass | Duplicates work, inflates token cost | Track processed IDs, final pass does cross-cutting only |
| Run downstream without dedup state | Creates duplicate specs/findings | Always pass `$ALREADY_CORRELATED_IDS` forward |
| Spawn correlator before prerequisite ready | Correlator has no data to cross-reference | Gate on prerequisite agent (e.g., C1a) completion |
