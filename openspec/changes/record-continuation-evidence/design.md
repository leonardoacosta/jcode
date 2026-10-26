# Continuation evidence design

## Context

See [proposal.md](proposal.md) for intent. This is a proposed P01 implementation design, not a claim that the feature exists.

Fresh local research on 2026-10-26:

| Surface | Observed source contract | Design consequence |
| --- | --- | --- |
| `crates/jcode-todo-policy/src/followup.rs` | Stateful pure `FollowUpPolicy::decide`, five gate attempts, existing fingerprint/digest/confidence guards | Observe its existing decisions, do not replace guards |
| `crates/jcode-tui/src/tui/app/input.rs` | Separate `App::schedule_auto_poke_followup_if_needed`, including pending/queued guards | Treat TUI and SDK as distinct engines; do not assume parity |
| `crates/jcode-todo-policy/src/store.rs` | `load_snapshot` and `apply_effects` are separate | Recording is not acknowledgement |
| `crates/jcode-harness-api-server/src/translate.rs` | `get_todo_state` reads, `ack_todo_follow_up` applies effects | No new endpoint or wire change in this slice |
| `crates/jcode-harness-api/src/events.rs` | `ToolDone` supplies session/call/name/output and optional error | Observe native return references, never parse `PASS` or copy output |
| `crates/jcode-tui/src/tui/app/tests/auto_poke_cycles.rs` | `with_temp_jcode_home`/`create_test_app` call the actual scheduler | Reuse isolated test infrastructure |
| `crates/jcode-build-meta/src/lib.rs` | `version()`/`git_hash()` report the current process build; TUI already depends on this crate | Controller process identity is separate from daemon and checkout identity |
| Existing local workflow specifications | Generic evidence/control/evaluation principles, not a native receipt model; local-only here | Do not import private historical specs as committed dependencies |

OpenSpec 1.13.2 resolves this checkout with `spec-driven`. Optional Graft freshness failed with `Maximum call stack size exceeded`; ordinary local inspection continued. No provider or live serving-runtime acceptance was exercised during authoring.

## Goals / Non-Goals

**Goals:** attributable native decision metadata, non-interfering private opt-in capture, and a balanced deterministic fixture baseline for later P03 comparison.

**Non-goals:** semantic success judgment, a fix for repeated validation, a merged controller, complete production-session replay, raw diagnostic export, new Jev/provider access, general command attestation, a sandbox, a new service/database/endpoint/CLI command or TypeScript wire contract.

Other P01 candidates and all P03 behavior remain outside this feature. Missing runtime identity or replay inputs remain limitations, not invented observations.

## Decisions

### 1. Separate production metadata from complete offline fixtures

Create a focused pure `crates/jcode-todo-policy/src/continuation.rs`; export its types from the policy crate and existing Rust SDK. Reuse serde and existing dependencies. No new framework or executable.

Two distinct types prevent a production writer from accidentally persisting private replay input:

| Type | Permitted payload | Explicit limitation |
| --- | --- | --- |
| `ContinuationReceiptV1` | Version 1, capture sequence, engine kind, safe/opaque source identities, typed claim counts/status/quality categories, control flags/counters, input-change categories, native event references/outcome categories, process-reported provenance, selected kind and effect metadata, coverage | No raw todo/plan/goal text, generated messages, serialized fingerprints, tool output/arguments, arbitrary error strings or transcript |
| `ContinuationReplayFixtureV1` | Complete reviewed synthetic `TodoSnapshot`, exact engine before-state and fixed clock/eligibility context, expected native decision/messages/effects | Offline fixture only; never produced by automatic capture or accepted by the runtime journal writer |

Runtime capture classifies todo/goal quality as **agent-reported metadata**, not observed proof. Native tool returns remain native execution observations, not test or requirement verdicts. A successful `echo PASS` supplies no semantic verification result.

Use safe native identifiers when available and capture-local opaque tokens for untrusted identifiers. Do not log free-text values inside a supposed fingerprint; record equality/change metadata instead. The new journal serializer accepts only `ContinuationReceiptV1`, not arbitrary JSON or the complete fixture type. A regex secret scrubber is not an adequate substitute for this allowlist.

Production receipt observation coverage can be complete while its replay coverage is explicitly `incomplete: redacted_input`. It must not claim exact replay from omitted input. Complete fixtures support exact offline checks; a production metadata receipt does not silently become one.

Alternative rejected: persisting the exact `TodoSnapshot` and raw fingerprint strings. They contain private user-authored text and could re-log credentials. This privacy correction deliberately limits production replay rather than asserting a safe raw export.

### 2. Observe the actual engine; preserve behavior

Use `ControllerKind::{SdkPolicy,TuiScheduler}` and distinguish controller process-reported `jcode_build_meta::version()`/`git_hash()` from separately observed daemon identity and source revision. An unavailable identity is `unknown` with a typed reason. Repository HEAD, launcher symlinks and a file on disk do not establish the running daemon's build.

The shared-policy adapter observes a read-only before-state and calls the existing `decide` exactly once. Existing `decide` stays pure and usable without tracing. The TUI adapter observes its actual entry/guard/branch result, not a guessed SDK decision. Capture disabled, pending/queued, incomplete-work, gate, final-response and no-follow-up paths. Trace enablement cannot reset or re-arm the controller.

Native event handling contributes only already available session/call-correlated references and typed outcomes. Legacy/missing IDs stay unknown; interleaved events cannot be assigned to the last arbitrary call. Do not add native payload collection or infer cancellation from error prose.

Record requested `TodoEffects` separately from applied effects. A pure decision records application as `not_observed`. An applied result requires an observation from the existing application path; capture must not apply, acknowledge, clear or retry effects itself. No-follow-up may still request effects and is not assumed to mean no state change.

Compare actual in-memory messages, selected kinds, effects, acknowledgements and ordering with capture off/on in tests, without writing the messages to production traces. Logging failures cannot buy an agent turn or change the native outcome.

Alternative rejected: switching the TUI to the shared policy as part of instrumentation. That would mix a behavioral migration into the baseline.

### 3. Default-off, private, bounded local capture

Add `FeatureConfig::continuation_trace: bool`, serde/default false, with its optional `features.continuation_trace` setting documented through existing config-defaults surfaces. Missing setting means false. This diagnostic flag does not change `auto_poke`, permissions or Jev access.

Reuse existing storage helpers and `is_safe_session_id` with a new narrow subdirectory:

```text
<JCODE_HOME>/todos/continuation-traces/<safe-session-id>.json
```

Create the new directory with Unix mode 0700 and files with 0600; use bounded atomic replacement. Do not chmod existing home/todo directories, follow a preexisting symlink, accept a caller-provided path or broaden ACLs. If ownership/privacy cannot be established, capture is unavailable and writes nothing. Initial native capture acceptance is Unix; other platforms stay explicitly unavailable until an equivalent private boundary is verified. Pure replay remains portable.

Cap the **entire serialized trace** at **64 records and 512 KiB**. These are conservative diagnostic defaults, not measured performance claims. Retain truncation/dropped-record metadata when evicting oldest complete records. Record omission metadata for an oversized receipt, within the same budget; if even that cannot be written safely, skip capture. Never pretend retained coverage is a complete history.

Disabled capture performs no trace I/O. An unsafe path, unreadable/malformed journal, serialization error or unwritable location leaves existing files untouched and does not change the decision. Emit at most a bounded typed/redacted diagnostic, not raw state or error contents. No automatic repair, historical-state migration, provider access, export or upload.

Alternative rejected: assuming the existing source checkout/todo directory is private, or introducing a new evidence database. A dedicated small private subdirectory does not alter existing user data.

### 4. Replay complete fixtures against native code, never execute them

`ContinuationReplayFixtureV1` supplies exact sanitized input and before-state. `PolicyReplayStateV1` projects the current nine policy fields: `armed`, `default_on`, `final_response_requested`, `final_response_fingerprint`, `digest_delivered`, `gate_attempts`, `spike_challenged`, `last_ownership_fingerprint`, `last_poke_fingerprint`. A crate-private replay constructor initializes only a fresh isolated policy, never the live engine.

`TuiReplayStateV1` enumerates the decision-relevant reads of the actual TUI scheduling method: pending/queued eligibility, session, control flags, fingerprints, counters and computed timed-review context. Do not serialize all of `App` or substitute the SDK projection. Add branch coverage for every captured guard. Fix clock-dependent context in fixtures; incomplete state cannot be filled with guessed defaults.

Expose `replay_policy_fixture` for SDK-policy fixtures, using the real `FollowUpPolicy::decide`. Keep the TUI adapter in the existing test boundary: `with_temp_jcode_home`/`create_test_app` inject fixture state and call the real scheduler. The pure crate must not depend on TUI. Its public replayer returns `unsupported_engine` for TUI input rather than imitating it; the TUI harness separately verifies its engine.

Replay results are `matched`, `mismatched`, `incomplete`, `unsupported_version`, `unsupported_engine` and `invalid`. Metadata-only receipts are incomplete for exact replay. Validate schema, identity/correlation, coverage and required state. A match describes controller behavior, never completion of the user's goal.

Replay never executes recorded tools/commands, sends provider or follow-up requests, applies native effects, opens filesystem paths named by a fixture or mutates original session data. File I/O for fixtures stays in controlled test locations. Live controllers never import replay state.

Alternative rejected: a separate scripted copy of decision rules or replaying private transcripts through a model. Neither establishes the baseline of the actual code safely.

### 5. A balanced baseline, not a premature improvement claim

Use committed sanitized synthetic fixtures, not real private transcripts or exported task snapshots. This session's repeated-validation behavior is a lead for a derived fixture, not proof of an exact production replay. Include unchanged metadata, real newly required checks, changed state, queued/blocked work, cancellation, unknown/stale identity, interleaved native errors and malformed/version-mismatched/incomplete records.

Run each engine against its own baseline. Report differences instead of normalizing them. Capture off/on must preserve existing behavior and effect counts. Record case counts, limitations and engine identity; quieter output is not automatically better. P01 claims no reduction in turns, tokens, cost or latency. P03 owns future model comparison and deployment.

## Risks / Trade-offs

Metadata capture sacrifices exact production replay to avoid re-logging private text. Missing state stays incomplete, not matched. Native event returns are not semantic proof. Capture I/O can fail or add opt-in overhead; bound it and preserve the default path. Two controller engines may differ; record that without refactoring them. Unknown runtime or platform privacy evidence remains unknown/unavailable.

## Migration and rollback

No existing snapshot/wire format or user data is migrated. The independent trace schema is versioned, and missing configuration remains false. Disabling capture restores the old I/O path without resetting controller state. Removing this feature can leave private diagnostics inert; do not delete historical files, modify permissions or import trace state as rollback.

## Prerequisites and verification boundary

Authoring uses installed local OpenSpec validation. Implementation requires the repository Rust toolchain/dependency cache and coordinated `selfdev` test lock; those builds/tests have not been run by authoring. Initial execution uses isolated test homes, synthetic native events and no credentials, external network, shared-daemon restart or production writes. A later live runtime check must use an isolated socket and identify the actual rebuilt serving process; it is not established by this design.

No new provider, paid route, infrastructure, data migration or external fact is required. Material choices are resolved with conservative defaults above. Code layout may follow existing module patterns but cannot weaken privacy, non-interference or replay boundaries. Structural validity and executable local tests do not grant implementation authority.
