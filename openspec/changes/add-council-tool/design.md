# Design

## Context

Swarm workers already support spawn, message, await, and stop (`communicate.rs`). Council is a thin orchestrator over them. Behavior reference is clean-room notes only; no Empryo text is reused.

## Goals / Non-Goals

Goals: bounded cost, read-only seats, deterministic termination, no orphan seats, durable transcript.
Non-goals: seats editing files, open-ended "debate until consensus", UI tabs beyond normal visible swarm workers.

## Decisions

### Actions
`open` (topic, seats `{name, brief, model?}`, rounds, timeout) spawns seats and collects opening statements. `round` sends a round prompt and waits on a barrier. `say` fans a convener message to all seats. `log` returns the transcript. `close` runs the verdict and stops seats. `run` chains them. Justification: matches the swarm primitives one-to-one and keeps `run` for agents that do not need interjection.

### State machine
```
OPEN -> OPENING(barrier) -> ROUND k(barrier) -> [all agree | k==N] -> VERDICT -> CLOSED
any state --abort/error/crash--> STOP SEATS -> CLOSED(transcript, error)
```

### Barrier
A round ends when every live seat answers or the per-round timeout fires (default `max(60s, total/(rounds+1))`). Late or errored seats are named in the result; the round proceeds.

### Votes and early stop
Each seat reply must end with `VOTE: <option>`. If all live seats vote the same option after a round, remaining rounds are skipped. Missing vote counts as abstain and blocks early stop.

### Verdict
A separate verdict call (coordinator model or `council` slot) receives the transcript and returns `PASS` (converged) or `PARTIAL` (open), tally `{option: count}`, and dissent `[seat: reason]`. Verdict is parsed; unparseable output yields `PARTIAL`.

### Caps
Seats 2-4, rounds 0-3, max calls default `seats*(rounds+1)+1`, optional cost cap. Checked before spawn; a round that would exceed the cap is refused and the council proceeds to verdict.

Current accounting counts council prompt dispatches, not every provider completion within a tool-using turn or retry. Aggregate provider-call enforcement remains open in task 4.3. Council seats cannot enable memory work, which otherwise adds uncapped auxiliary calls and memory writes.

### Read-only enforcement
`CommSpawn.council_id` is persisted on the seat Session before visible launch or headless publication. Agent construction and resume intersect configured tools with the fixed read-only allowlist (read, agentgrep, webfetch/websearch). The executor rejects disallowed tools, including custom, batch and MCP paths. A parent-process policy installed after the spawn response is not an enforcement boundary.

### Lifecycle
Seats are tracked in a process-local council registry keyed by id. Drop guards cover allocated seats during opening, rounds, messages, verdict and close. Reload loses the registry, so every persisted council seat is owner-less even if its coordinator reconnects. Recovery excludes those seats, preserves their full history, persists Closed status and clears pending visible startup input. Closed council sessions cannot resume or revive. Legacy unmarked seats are not inferred from labels. Spawn-RPC ambiguity and durable-closure write failures remain recovery risks documented in tasks.md.

### Persistence
Transcript and verdict written as JSON under the session directory on every round.

## Risks / Trade-offs

- Vote-line parsing is brittle: fall back to abstain, never to agreement.
- Shared worker pool: council can starve other swarms; seats count against the same cap.
