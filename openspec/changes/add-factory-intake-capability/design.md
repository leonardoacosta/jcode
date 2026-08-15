## Context

Research for this change is in `docs/inbox-factory-extensibility.md`. This document records only the decisions that constrain implementation, and the measurements behind them.

The operating context is a single-operator, local-first system. That weakens blast-radius arguments that would dominate a multi-tenant design, and it strengthens arguments about recoverability and audit, because there is no second operator to reconstruct what happened.

## Goals / Non-Goals

**Goals:**

- Make the second transport adapter cheap, and prove that at spec level before writing the first.
- Keep chat history non-authoritative, so deleting a conversation loses nothing but conversation.
- Record everything received, permanently, before interpreting any of it.
- Make promotion from message to work an explicit, audited transition.

**Non-Goals:**

- Reimplementing the initiative lifecycle inside a chat provider.
- Conversational agent behavior, threading UX, or rich interactive surfaces.
- Multi-operator access control. Identity mapping exists to bind approvals, not to model a team.

## Decisions

### Intake is stricter than the terminal

Terminal input is trusted because it requires physical presence at the machine. Chat input requires only a phone. Intake therefore defaults to producing proposals rather than executing, with two read-only exceptions (research, status). This is deliberately more restrictive than the same instruction typed into a session, and the asymmetry is the point: the lowest-friction input path gets the tightest default.

### Dedupe keys never derive from transport sequence numbers

Telegram's API documentation states that `update_id` may be randomized after a period of bot inactivity. A single-operator inbox is idle most of the time, so this is the normal case, not an edge case. Any dedupe scheme keyed on provider sequence would silently break after a quiet weekend. Keys derive from content and identity instead.

### Storage: embedded database, justified narrowly

This introduces the first embedded database in the workspace. Nothing in any workspace `Cargo.toml` currently matches `rusqlite|sqlx|libsql|redb|sled|duckdb`; existing state is JSON files with `.bak` siblings (`ambient/queue.json`, `memory/global.json`, roughly nine such files). This change sets a precedent, so it is justified by measurement rather than preference.

Benchmarked on records of 1739 B, sized from the real `ambient/queue.json`:

| Operation | N | JSON rewrite | JSONL append | SQLite |
|---|---|---|---|---|
| Writes | 100 | 0.028 s | 0.001 s | 0.002 s |
| Writes | 1000 | 2.341 s | 0.007 s | 0.019 s |
| Writes | 5000 | 60.378 s | 0.036 s | 0.100 s |
| Dedupe lookup | 50 000 | — | 52.47 ms | 0.09 ms |
| Pending-approval scan | 50 000 | — | 24.10 ms | 11.28 ms |
| File size | 50 000 | — | 85.2 MB | 108.8 MB |

Whole-file JSON rewrite is quadratic and disqualified. Between the remaining two, the deciding factor is the dedupe lookup, which runs on *every inbound message*: 0.09 ms versus 52.47 ms, a 616× difference that grows with permanent retention.

Two results argue against this choice and are recorded rather than omitted: the pending-approval scan is only 2.1× faster, and the database file is 28% *larger* than the equivalent JSONL. If the dedupe path were removed from the design, JSONL would be the better choice.

### Retention is maximal; redaction is narrow

Everything is retained permanently, including duplicates, unauthorized senders, throttled messages, and unhandled update variants. Tiered or expiring retention was considered and rejected.

Redaction is not distrust of the operator. The single hazard is a credential pasted by accident, so the scrub is scoped to credential-shaped strings at ingress, and everything else is stored verbatim. Because retention is permanent and ingress-time scrubbing is the only control, there is no override token to bypass it: an override would permanently defeat the one control the design has.

The provider copy is transient and the local copy is permanent. Telegram retains undelivered updates for at most 24 hours, so the durable record is the local one, which is what makes ingress-time scrubbing the correct location rather than a defense-in-depth nicety.

### Group activation requires an explicit mention

Group chats forward only messages that mention the bot or reply to it. Broader activation modes were considered for later; requiring an explicit address avoids ingesting unrelated group traffic into a permanent store on day one.

## Risks / Trade-offs

- **First embedded DB in the workspace.** Mitigated by narrow justification above; the fallback if the dedupe path changes is append-only JSONL, which the same benchmark supports.
- **Permanent retention grows without bound.** Accepted. At the measured record size, 50 000 messages is ~109 MB, and a single-operator inbox does not approach that quickly.
- **Spec-level neutrality is not implementation neutrality.** The boundary check reads specification text. It cannot prove the eventual code respects the seam. The second adapter is the real test, and it is expected to be the conformance test for this design.

## Open Questions

None. Identity was briefly framed as an open question about a mapping table, which was an error: the system is single-operator by decision, so the allowlist holds one entry and there is nothing to map. The operator reads their own sender identifier from the first unauthorized attempt, which the adapter records for exactly this purpose.
