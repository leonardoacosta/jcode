# Design

## Context

See proposal.md for scope. `ContinuationEvidence` already bounds and redacts scheduler facts and typed required-check outcomes. `schedule_auto_poke_followup_with_evidence` records each deterministic TUI decision. `JevClient` already supports a supervision purpose, and `JevShadowEvaluator` provides observation-only advice with request/time bounds. Explicit configured routing is a prerequisite, not part of this implementation.

## Decisions

- Observe after the native decision. The observer receives the existing redacted record and the actual queued/not-queued result. It cannot modify policy, queue work, acknowledge checks or mark tasks complete. Prefer this existing boundary over introducing another scheduler.
- Default off. Use the explicit supervision resolver only when the shadow opt-in is enabled. Missing configuration must not select another purpose or credential route. Do not mutate user configuration or activate this feature as part of delivery.
- Bound observation latency and outstanding work. Transport failures, timeout, malformed responses and unavailable evidence yield `uncertain`, never a synthetic stop/continue or retry loop. Record counts and latency independently of decision agreement. Do not log credentials, raw command output, todo prose or transport error bodies.
- Reuse the current transport/evaluator where its contract fits. Its `wait` and `abstain` advice are not permission to execute semantic decisions. The public shadow contract remains `continue`/`stop`/`uncertain`; any mapping must retain uncertainty rather than claim verified completion.
- Offline replay consumes captured evidence plus actual deterministic decisions and recorded verdicts. Reject missing evidence. Report missed required checks, hypothetical redundant turns, call count, latency and uncertainty separately. Missing or not-run checks never count as passed. Negative controls cover waits, blocked work, premature stop and failed required checks.

## Implementation contract

- `[features].semantic_poke_shadow` defaults to `false`. The TUI observes only after the deterministic decision and required-check evidence are captured. `SEMANTIC_POKE_SHADOW` logs one self-contained JSON record with `decision_id`, `deterministic_queued`, `evidence`, `verdict`, `calls`, and measured `latency_ms`. IDs bind delayed results to their original decision, never the current turn. The captured local/remote session owns the log context.
- A process-lifetime evaluator preserves its 48-request/600-second budget across decisions and opt-out/opt-in transitions. One permit spans submission, evaluation, cancellation accounting and logging. Busy observations record `uncertain` immediately, without tasks or backlog. Missing configuration can recover on a later decision. After successful initialization, a changed supervision table records `uncertain` until process restart rather than replacing the evaluator and resetting its cap. Credential/endpoint environment changes likewise require restart to affect an existing client.
- Each request has a two-second async timeout. Cancellation drops a record guard that records `uncertain` and the actual started evaluation count. `wait` and `abstain` map conservatively to `uncertain`, not verified completion. No provider error bodies enter logs. Missing required-check provenance abstains without a request.
- `jcode semantic-poke-replay <JSON>` reads a regular file with a four-MiB cap before config/provider bootstrap. Input is a JSON array of the exact logged records, not raw log lines. Extract the JSON suffix after `SEMANTIC_POKE_SHADOW ` into an array. Empty arrays, absent/incomplete evidence, duplicate identities, unknown enums, invalid check bindings and overflow fail. Unknown required checks are counted separately. Hypothetical redundant turns require a queued native decision, a semantic `stop`, known passing checks, and explicit wait/blocked/unchanged evidence. A stop disagreement alone is not redundancy. No GO or savings result is produced.

## Verification and boundaries

Run actual project tests and public replay/runtime acceptance over final bytes. Loopback fixtures can establish integration and transport failure behavior, not semantic quality. Preserve actual failures and distinguish synthetic controls from actual-session observations. The roadmap's redacted actual-session inference replay remains dependent on an authorized inference path and explicit spending/disclosure bounds. Do not substitute mocks for that acceptance or enable production decisions after a GO result.

## Risks and rollback

Required-check completeness is separate from generic redaction: the
`required_checks_complete` evidence field is true only when observed checks
exist and none were dropped by validation, batch bounds or accumulator capacity.
Legacy records without this field default to incomplete. Incomplete provenance
causes the observer to abstain without a request; replay counts unknown check
coverage and never credits hypothetical redundant turns from a retained pass
prefix. Generic `truncated` still records pseudonym/prose redaction and cannot
establish check completeness in either direction.

Observation work could delay the TUI or leak context. Bound payloads and async work, preserve redaction, and verify the production caller with failure controls. Disabling the opt-in restores the deterministic-only observation path without data migration. No new provider, service, permission, deployment or overnight executor is included.
