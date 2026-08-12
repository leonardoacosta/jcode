---
name: explore
description: Native Jcode exploration workflow. Use to assess an idea, problem, change request, or open work before implementation, gather evidence with Jcode tools, build decision maps, rank routes, and hand off to /feature without invoking Codex or Claude-owned workflows.
---

# Native Explore

Treat `/explore` as Jcode's native intake and decision workflow. It clarifies intent, gathers scoped evidence, ranks routes, and produces reusable handoff context. It does not implement product changes and it does not activate `codex-explore` or any Claude compatibility command.

## Invocation contract

- Preserve all trailing slash-command text as the exploration topic.
- Establish the user's intended outcome, success criteria, scope boundaries, and known constraints first.
- Use `todo` for session-local planning and progress. Do not create a parallel durable ledger.
- Before mutating any repository planning integration, run the shared workflow preflight below.

## Shared workflow preflight

At the start of every native workflow run:

1. Identify the repository root and current revision.
2. Detect OpenSpec readiness, Beads readiness, and harness telemetry availability without mutating the repository.
3. If OpenSpec or Beads is missing and no repository-scoped preference exists, ask one focused consent question. If both are missing, ask one combined question that allows both, either, or neither.
4. If the user accepts initialization, run only the canonical non-interactive initializer for the accepted integration, then recheck readiness.
5. If the user declines or initialization fails, persist/report the declined or failed status and continue only in an explicit degraded route.
6. Repeat a setup prompt only after an explicit reset, repository identity change, or explicit setup request.
7. Check telemetry every run and emit best-effort workflow start, phase, route, efficiency, degradation, and completion observations when supported. Telemetry failure never changes routing or correctness.

## Evidence sequence

Follow this native phase order:

1. Intent and acceptance criteria.
2. Preflight status and degradation limits.
3. Session plan with `todo`.
4. Prior context from injected guidance, memory, session search, initiatives, active work, OpenSpec/Beads when ready, and canonical Recon records when needed.
5. Scoped repository evidence using `agentgrep`, `read`, `ls`, structured CLIs, and optional read-only `swarm` workers for independent evidence domains.
6. Synthesis of facts, assumptions, external claims, conflicts, options, risks, and unresolved decisions.
7. Ranked execution queue and exactly one default route.
8. Structured `/feature` handoff or durable initiative checkpoint.

Ask the user only for user-only judgments that cannot be discovered or safely defaulted. Lead with a recommendation and evidence.

## Decision-map mode

When the destination is known but material decisions cannot be resolved in one session:

- Create or update a durable `initiative` with decision milestones, open questions, evidence links, and next checkpoints.
- Use `side_panel` only as a live view. It is not authority.
- Do not report feature-ready handoff while critical decisions remain unresolved.

## Native-tool-first efficiency rules

Use this ladder before shelling out:

1. Already injected context.
2. `memory`, `session_search`, `initiative`, and repository guidance.
3. `agentgrep`, `read`, `ls`, and other typed Jcode tools.
4. Structured first-party CLI output with source-side filters.
5. One batched, bounded shell command with explicit timeout and capped output.
6. Optional focused read-only swarm.

Avoid broad recursive dumps, repeated polling when `bg wait` exists, shell parsing when a typed tool exists, and unbounded output.

## Output contract

End with:

- Verified facts, assumptions, external claims, conflicts, blockers, and limitations.
- Prior art consulted and freshness evidence.
- An ordered queue with evidence and rationale for ordering.
- One selected default route.
- If `/feature` is selected, a structured handoff containing destination, success criteria, provenance, evidence IDs, assumptions, alternatives, in/out scope, decisions, surface inventory, confirmed revisions, dependencies, conflicts, edge cases, done means, remaining questions, limitations, and recommended action.

The handoff is session context for native `/feature`, not a second durable ledger.
