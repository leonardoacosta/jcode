# Continuation evidence design

## Context

See [proposal.md](proposal.md) for intent. This is a bounded planning design, not implementation approval.

Source inspection confirms `AUTO_POKE_DECISION` logging for `idle/no_todos`, `idle/unchanged_ownership_assessment`, `idle/unchanged_todos`, and `queue_continuation` in [`input.rs`](../../../crates/jcode-tui/src/tui/app/input.rs). The shared pure policy entry point is [`FollowUpPolicy::decide`](../../../crates/jcode-todo-policy/src/followup.rs). These source facts do not establish complete log coverage, runtime/deployed behavior, or whether events correlate across components.

| Layer | Source-verified fact | Not established here |
| --- | --- | --- |
| Native decisions | The selected TUI events above exist | Complete branch coverage or observed daemon behavior |
| Native log context | [`LogContext`](../../../crates/jcode-logging/src/lib.rs) supports optional server/session/provider/model; [`logging.rs`](../../../crates/jcode-base/src/logging.rs) is its facade | Context propagation at each decision or a reliable cross-gateway join |
| Gateway logging | [Pinned OmniRoute request-log endpoint](https://raw.githubusercontent.com/diegosouzapw/OmniRoute/dbe703a0000b303cd7b1cf5879cb8740e5bfce71/src/app/api/usage/request-logs/route.ts) reads existing gateway logs | Deployed version, route coverage, enabled capture, retention or privacy |

OmniRoute sees routed inference traffic only. A native early-return stop/wait may produce no inference request, so absence from OmniRoute is not evidence of absence of a native decision. Source-pinned OmniRoute v3.8.52 documentation/source are not verification of the deployed version, configuration, capture, or retention. Existing OmniRoute log privacy remains unassessed here, not certified.

## Goals and non-goals

**Goals:** establish what current logs and native tests do and do not evidence; retain offline, reviewed synthetic regressions using the real policy and TUI test entry points; expose concrete evidence gaps without overstating confidence.

**Non-goals:** a new receipt store or trace path; a capture flag; typed production receipts or public replay schemas; SDK/export APIs; automatic raw-log export/import; production-trace replay; behavior changes; a claim of reduced loops, P01 completion, or phase completion.

## Decisions

### 1. Assess existing evidence before proposing logging

Inventory relevant current Jcode logging, its scope, identifiers, correlation limits, and provenance. Separately assess what deployed OmniRoute access can verify, if available. Distinguish **source-verified** capability from deployed properties classified as `verified`, `unknown`, `unavailable`, or demonstrably `missing`, with the basis for each. Source references establish available code paths, not observed execution; a source-pinned OmniRoute v3.8.52 reference does not establish deployed capability, configuration, capture, or retention. Do not infer non-occurrence from absent calls or records.

Prefer existing Jcode logging and existing OmniRoute logs where they actually apply. Only consider a bounded metadata field or reason event in existing logging after identifying a concrete, reproducible evidence gap. Any such proposal must specify the observation the new metadata would distinguish and tests that reproduce the gap. No specific field is presumed necessary. Do not increase body capture or retention, enable disabled logging, or certify existing log privacy by assumption.

### 2. Treat claims, native events, and acceptance as distinct

Existing fields and reason strings are evidence only for what their source path actually logs. A `ToolDone` success, `PASS` text, or confidence label does not establish acceptance. Correlation and provenance not present in evidence remain unknown. Do not reconstruct native stop/wait paths from inference traffic, or assign uncorrelated events to a decision.

Fixtures and documentation use reviewed synthetic data, never raw transcripts, tool arguments, tool outputs, message bodies, or credentials from user sessions. Synthetic expected messages may test native behavior without copying private content. Baseline test recipes must not require a gateway, live log access, provider calls, or deployed service changes.

### 3. Use real entry points with synthetic regression fixtures

Keep reviewed fixtures synthetic and offline. Policy regressions call the actual `FollowUpPolicy::decide`; TUI scheduling regressions use the actual `schedule_auto_poke_followup_if_needed` test entry point. Reuse current test infrastructure and decision rules rather than copying them into a replay engine. Keep pure policy and TUI scheduler evidence distinct; do not claim parity.

Fixtures freeze current behavior for this evidence slice, not correctness or improvement. Cover relevant unchanged and changed evidence, genuine missing checks, and blocked/queued paths as supported by the existing entry points. Record observed outcomes, test recipes, and limits in existing test-module documentation. Do not create another executable ledger; `tasks.md` is the sole executable ledger.

### 4. Keep implementation and deployment gates separate

This plan does not authorize implementation or a behavior fix. Gate future behavior changes separately from this evidence slice. Do not make a new journal or gateway deployment change a prerequisite. During separately authorized execution, only a specific, reproducible evidence gap may activate the optional metadata-only existing-log branch in `tasks.md`, with focused tests. Sufficient or inconclusive coverage means no logging addition. No new artifact surface is needed.

## Risks and limits

Current source logging may not expose enough scope, correlation, or provenance to establish an event. OmniRoute cannot observe native paths that do not route inference, and the deployed gateway configuration and retention are unknown unless independently verified. Synthetic fixtures test the actual local decision entry points, not live production traces or goal acceptance. These limits must remain explicit rather than filled with inferred data.

## Validation boundary

The coordinator verifies the current test recipes and schema validation. This design adds no test dependency on a gateway or live logs. Local validation of these planning files does not establish runtime behavior, deployed configuration, log retention/privacy, or phase completion.
