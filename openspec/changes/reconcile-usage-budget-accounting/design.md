## Context

Per-run reconciliation of projected versus observed usage. Observation is post-request and read-only: nothing here reserves spend or gates dispatch (that stays with task admission).

## Decisions

- **Attribution.** `overnight/usage.rs` walks the `session_control` owner chain to the run's coordinator and stamps each observation with `session_control::current_generation`. Resumed workers therefore appear as a new generation of the same session.
- **Ledger.** One `usage.jsonl` per run (O_APPEND, one line per provider response), reconciled once at run end into `usage-reconciled.json` plus a `usage_reconciled` event. Reconcile is idempotent.
- **Fail closed.** Unreadable control records become `unsettled` sessions with source `unavailable`; a missing coordinator record is `unsettled`; skipped ledger lines add a lower-bound note. Missing counters stay `None`, never zero.
- **Scope of usage.** Provider stream responses only (`StreamUsageRecorder`). Compaction, summaries and other non-stream requests are not counted; the report says so.
- **Account.** Read from the serving account at turn start (`sync_bound_account_route`). A mid-turn failover is attributed to the account that started the turn.

## Known limits (descoped, not hidden)

- **No stable response id for dedup.** `StreamEvent::TokenUsage` carries no provider response/request id, so the event id is a random id per response attempt. Cumulative snapshots of one response share it, and a repeated append of the same id is dropped at read time. A replay that re-executes a request (restart, resumed worker re-running a turn) gets a new id and counts again. A key such as `session:generation:message_index:attempt` was rejected: context-limit retries re-create the recorder at the same message index, so it would merge distinct billed attempts and undercount spend. Over-counting is the safe direction for budget accounting. Revisit when providers expose a response id on the stream event.
- **Restart attribution.** The active-run registry is process-local. After a daemon restart the supervisor is gone and resumed workers' usage is not attributed. Resumed generations are attributed only within one process.
- **Partial counters.** If only some responses report a counter, the total sums the reported ones without a partial flag.
