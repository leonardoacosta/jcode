# Record continuation evidence

## Why

Repeated validation can consume agent turns after an outcome already works, but task labels and model-written confidence fields do not establish which controller, inputs or native observations caused a follow-up. P01 needs an observable, replayable baseline before P03 can judge whether Jev improves continuation decisions.

Roadmap owner: [P01: trusted core and acceptance/evidence baseline](../../../docs/plans/PRODUCT_ROADMAP.md#p01-trusted-core-and-acceptanceevidence-baseline). This change covers one evidence slice, not completion of P01.

## What Changes

- Add a versioned native continuation receipt that separates typed agent-claim metadata, observed tool-event references, controller metadata, runtime provenance, selected follow-up kind and effects. Production receipts exclude user-authored free text by construction.
- Add opt-in, owner-private, bounded local trace capture through `features.continuation_trace`, defaulting to false. Reuse existing session/storage helpers with a new private trace subdirectory, not a new database or service. Do not store raw todo/plan/goal text, generated messages, serialized fingerprints, tool outputs, command arguments, credentials or complete session transcripts.
- Record the actual TUI scheduler and shared Rust policy as distinct controller engines. Preserve their existing decisions, messages, caps, ordering and effects; do not silently replace the TUI implementation with the SDK policy or assert parity.
- Add offline replay of complete, sanitized synthetic fixtures against the real policy and TUI test entry points, not a second implementation of their rules. Metadata-only production receipts are explicitly incomplete for exact replay; no automatic raw-trace export is included.
- Add sanitized, in-repository regression fixtures for repeated validation, changed evidence, legitimate missing checks, blocked/queued work, stale identity and malformed/truncated traces. Record baseline outcomes without claiming a semantic fix or improvement.
- Expose the pure receipt/replay types to Rust SDK consumers. No new harness endpoint, TypeScript wire contract, provider call or user-facing command is needed in this slice.

## Capabilities

### New Capabilities

- `continuation-evidence`: Native, provenance-aware continuation receipts, private opt-in capture and non-executing replay of current controller decisions.

### Modified Capabilities

None. Existing local workflow specifications provide research context but do not define this native recording contract or become committed dependencies through this proposal.

## Impact

Planned implementation surfaces:

- `crates/jcode-todo-policy/src/followup.rs` and a focused `continuation.rs`: read-only state projection, receipt types, validation and replay adapter.
- `crates/jcode-todo-policy/src/lib.rs` and focused tests: public exports and deterministic baseline cases.
- `crates/jcode-base/src/todo.rs` and existing pure store helpers: bounded private per-session trace read/write, without rewriting existing todo/goal files.
- `crates/jcode-config-types/src/lib.rs` and existing config defaults/tests: default-off diagnostic feature switch.
- `crates/jcode-tui/src/tui/app/input.rs`, native event handling and existing app/test helpers: capture the actual scheduler boundary and available native event references.
- `crates/jcode-sdk/src/lib.rs`: export the pure contract, retaining existing API compatibility.

No live Jev inference, decision/admission changes, autonomous retries, new permissions, planning sandbox, overnight recovery, scheduler activation, provider/account changes, automatic export, historical-state migration or remote publication is included. The isolated verification lane needs no provider credentials. Implementation remains a separate execution request; native artifact validity does not authorize running the feature.
