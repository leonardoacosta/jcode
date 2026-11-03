# Design

## Scope and bindings

Implement `P03.O1` diagnostics only. `FollowUpPolicy`, scheduler decisions,
fingerprints, caps, queue order, and synthetic messages remain unchanged.
Source checkout: `https://github.com/leonardoacosta/jcode`, implementation base
`e5b0c74262039cdd48a45b155b58d483aa5967d4`. The delegated lane owns
`crates/jcode-todo-policy/src/{lib.rs,evidence.rs}` and
`crates/jcode-tui/src/tui/app/input.rs`. No Git, archive, provider, shared daemon,
or worktree mutations are authorized for this worker.

Preflight: `preflight-check-v2-shadow`, initial-dispatch, 2026-11-03 UTC.
Proposal/spec/tasks and relevant source were inspected. Graph absent, direct
search used. Bounded record and TUI recording are eligible. External required
check identity/outcomes are missing at this boundary. Final independent review,
commit, archive, and persistence belong to the coordinator, not this worker.
Advisory score unavailable while required-command acceptance remains unresolved.

## Record

`ContinuationEvidence` accepts todo objects but serializes only fixed enum keys,
scalar status counts, optional `FollowUpKind`, a fixed reason enum, and booleans.
No prompt, tool output, command, todo text, dependency identity, session ID, or
fingerprint value enters the record. Omitted free text sets `truncated`, including
oversized inputs. UTF-8 slicing is unnecessary. All serializable fields have
bounded cardinality. The compact JSON record must fit 2,048 bytes.

`fingerprint_changed` is null when the decision did not compare a fingerprint.
Otherwise it reports the existing comparison used on that branch. It introduces
no new persisted fingerprint. `waiting` means a pending dispatch/turn/followup
prevents scheduling. `blocked` means a todo declares dependencies, without
asserting their cause or that an external operation is blocked.

`scheduler_checks` contains only predicates actually evaluated by the scheduler:
`ownership_delivery`, `completion_confidence`, `confidence_spike`, `gate_budget`.
Each result is `pass`, `fail`, or `not_run`. These are scheduler assessment checks,
not execution or acceptance evidence. A model's confidence or stopping prose is
never parsed into external check outcomes.

A single wrapper calls the existing decision body, then emits exactly one
`CONTINUATION_EVIDENCE <compact JSON>` log entry. Existing text logs remain.
The sink is injectable in unit tests and cannot change the returned decision.
Serialization failure skips diagnostics rather than changing policy.

## Required-command gap

`required_checks_available=false` explicitly identifies the missing observation.
This does **not** satisfy the spec scenario that records a failed required command.
`TodoGoal.feedback_loop` is prose (`jcode-task-types/src/lib.rs`), not a typed
check binding. `ToolDone` in `jcode-protocol/src/wire/events.rs` carries only
id/name/output/error. `ToolOutput.metadata` exists, but bash foreground completion
in `jcode-app-core/src/tool/bash.rs` returns the process exit code only as formatted
output. A nonzero command may still produce `Ok(ToolOutput)`. The tool error boolean
therefore cannot stand in for the command exit code or required-check identity.

Minimum follow-through requires an explicit required-check identity binding plus
typed process outcome metadata at bash completion, preservation through local
`jcode-tui/src/tui/app/turn.rs`, remote server emitters and `ToolDone`, and remote
`jcode-tui/src/tui/app/remote/server_event_handlers.rs`. These changes overlap other
lanes and exceed this worker's assigned boundary. Do not infer the binding from
commands, todo prose, self-reported success, or stdout. Keep the related acceptance
task open until the real source is integrated and tested.

## Verification

Test first through `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test`.
Policy tests cover fixed-size redaction, explicit observed scheduler results,
unknown required-check availability, and decision/state parity. TUI tests inject
one sink per scheduling call, cover queued/idle/wait branches and an actually
failed scheduler predicate. Existing auto-poke tests protect caps and fingerprints.
No rebuilt shared daemon is used. Runtime session-log acceptance remains distinct
from unit tests and coordinator terminal verification.
