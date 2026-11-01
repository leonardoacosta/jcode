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

### Read-only enforcement
Seats spawn with a Rust-side tool allowlist (read, agentgrep, webfetch/websearch). Disallowed tool calls are rejected by the executor, not by prompt.

### Lifecycle
Seats tracked in a council registry keyed by id; a drop guard and session-close hook stop them. After reload, recovery (`recover-swarm-workers-after-reload`) stops seats of councils with no live owner.

### Persistence
Transcript and verdict written as JSON under the session directory on every round.

## Risks / Trade-offs

- Vote-line parsing is brittle: fall back to abstain, never to agreement.
- Shared worker pool: council can starve other swarms; seats count against the same cap.
