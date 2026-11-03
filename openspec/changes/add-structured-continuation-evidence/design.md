# Design

## Scope and bindings

Implement `P03.O1` diagnostics only. `FollowUpPolicy`, scheduler decisions,
fingerprints, caps, queue order, and synthetic messages remain unchanged.
Source checkout: `https://github.com/leonardoacosta/jcode`, implementation base
`e5b0c74262039cdd48a45b155b58d483aa5967d4`. The delegated lane owns
`crates/jcode-todo-policy/src/{lib.rs,evidence.rs}` and
`crates/jcode-tui/src/tui/app/input.rs` at the initial checkpoint. The resumed
assignment also owns the minimal typed task/protocol contract, native bash metadata,
server emitter, local/remote TUI transport, accumulator lifecycle, and matching tests.
No Git, archive, provider, shared runtime, config, or worktree mutations are authorized.

Preflight: `preflight-check-v2-shadow`, initial-dispatch, 2026-11-03 UTC.
Proposal/spec/tasks and relevant source were inspected. Graph absent, direct
search used. Bounded record and TUI recording are eligible. At the initial checkpoint, external required
check identity/outcomes were missing at this boundary. The resumed explicit scope
includes the typed process-to-client path described below. Final independent review,
commit, archive, and persistence belong to the coordinator, not this worker.
Runtime session-log acceptance and final closeout remain coordinator-owned.

## Record

`ContinuationEvidence` accepts todo objects but serializes only fixed enum keys,
scalar status counts, optional `FollowUpKind`, a fixed reason enum, booleans, and
bounded required-check pseudonym/result pairs.
No prompt, tool output, command, todo text, dependency identity, session ID, or
fingerprint value enters the record. Omitted free text sets `truncated`, including
oversized inputs. UTF-8 slicing is unnecessary. All emitted enum fields and required-check observations have
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

## Required-command observation

The optional native bash `required_check_id` explicitly binds one invocation to a
required check. The tool validates 1-64 ASCII letters, digits, or `_.:-` before
execution. The binding does not grant execution authority or bypass the existing
destructive-command, hook, or tool-permission gates. No command, todo prose,
stdout, stderr, or model success claim supplies an identity or result.

Both foreground completion implementations attach the real child process state
under existing `ToolOutput.metadata.process`: `foreground`, `completed`, and
nullable `exit_code`. A check passes only for completed foreground execution with
`exit_code=0` and no tool error. Nonzero exits, signal termination (null exit), and
actual tool errors never pass. Explicit background starts and foreground timeout
promotions have no completed required-check outcome. Later background completion
is not integrated by this change and cannot supply a pass.

`RequiredCheckOutcome` lives in `jcode-task-types`; `ToolDone.required_check` is an
additive optional typed wire field. Legacy events omit it. Both normal agent and
manual server tool execution extract it before discarding `ToolOutput.metadata`.
Provider-reported/SDK prose-only and replay events carry no typed observation.
Local TUI execution and remote event handling feed the same bounded accumulator.

The accumulator retains at most eight validated IDs (64 bytes each) per turn,
deduplicates explicit IDs, and never overwrites a failure with later success or
unrelated tool completion. Diagnostics replace supplied names with turn-local
`check_1` through `check_8` pseudonyms; these are not cross-turn identifiers or
cryptographic fingerprints. Omitted names, invalid IDs, and overflow mark the
record `truncated`. The maximum-size diagnostic record remains 2,048 bytes.

Clear the accumulator at local interactive-turn entry, successful remote message
or input-shell dispatch, externally started remote-turn adoption, and remote
session switch. Preserve it through model/tool rounds within the same turn and
attach it only after the existing scheduler decision body returns. FollowUpPolicy,
queue ordering, caps, fingerprints, and synthetic messages remain unchanged.

## Verification

Test first through `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test`.
Policy tests cover fixed-size redaction, explicit observed scheduler results,
unknown required-check availability, and decision/state parity. TUI tests inject
one sink per scheduling call, cover queued/idle/wait branches and an actually
failed scheduler predicate. Existing auto-poke tests protect caps and fingerprints.
No rebuilt shared daemon is used. Runtime session-log acceptance remains distinct
from unit tests and coordinator terminal verification.

## Batch integration follow-through

The native batch tool aggregates only its actual native bash child completions,
using each normalized explicit child binding and result before metadata is discarded.
It does not parse formatted batch output or trust process metadata from unrelated
child tools. Original child order determines alias order; duplicate identities keep
failure sticky. `RequiredCheckBatch` retains at most eight valid identities and an
explicit overflow flag. Top-level batch `ToolOutput.metadata.required_checks` transfers
through additive optional typed `ToolDone.required_checks`, separate from the legacy
singular bash field. Both local and remote accumulators re-bound received aggregates,
preserve overflow, and redact names. Native nested batches remain rejected by the
existing execution guard; this change does not enable nesting or bypass permissions.

Root NDJSON and ACP renderers explicitly ignore these diagnostic wire fields,
preserving their existing output contracts without exposing supplied check identities.
