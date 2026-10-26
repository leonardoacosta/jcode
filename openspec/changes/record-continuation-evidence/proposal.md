# Record continuation evidence

## Why

Unnecessary repeated validation should be diagnosed from existing evidence before adding another recording layer. P01 needs a source-grounded baseline for continuation decisions before later work can evaluate them. Jcode already emits selected `AUTO_POKE_DECISION` reasons and a `queue_continuation` event, but log coverage, correlation, provenance, and deployed behavior are not established by those source observations. OmniRoute observes routed inference traffic only; native early-return stop/wait paths may involve no inference. Missing calls or log entries do not prove that an event did not occur.

Roadmap owner: [P01: trusted core and acceptance/evidence baseline](../../../docs/plans/PRODUCT_ROADMAP.md#p01-trusted-core-and-acceptanceevidence-baseline). This change defines one evidence slice, not P01 completion, loop reduction, or a behavior fix.

## What Changes

- Assess current Jcode logging and OmniRoute visibility before proposing additional logging. Distinguish source-verified capability from deployed properties assessed as `verified`, `unknown`, `unavailable`, or demonstrably `missing`. Source-pinned OmniRoute v3.8.52 docs/source do not verify deployed capabilities, capture, or retention. Existing OmniRoute log privacy is not automatically certified.
- Reuse current Jcode logging and existing OmniRoute logs where relevant. Do not introduce a receipt store, capture flag, trace path, typed production receipt, public replay schema, SDK/export API, or automatic raw-log export/import.
- Add bounded metadata fields or reason events to existing logging only if the assessment demonstrates a concrete, reproducible evidence gap. Do not presume new fields are needed. Tests must demonstrate the gap and cover any authorized logging change.
- Preserve offline, reviewed synthetic regression fixtures using the actual `FollowUpPolicy::decide` and TUI `schedule_auto_poke_followup_if_needed` test entry points. Keep the pure policy and TUI scheduler distinct. Do not copy decision rules into a replay engine or promise production-trace replay.
- Treat `ToolDone` success, `PASS` text, and confidence labels alone as insufficient proof of acceptance. Unknown or missing correlation and provenance remain unknown. Fixtures and docs use reviewed synthetic data, never raw transcripts, tool arguments, outputs, message bodies, or credentials from user sessions.
- Keep baseline tests independent of gateway availability and live log access. Record outcomes and limits in the existing test-module docs, not another executable ledger. `tasks.md` remains the sole executable ledger.

## Capabilities

### New Capabilities

- `continuation-evidence`: A source-grounded baseline and offline synthetic regression evidence for existing continuation decisions, without a new production capture or replay architecture.

### Modified Capabilities

None.

## Impact

Planning and validation focus on existing logging and tests, including [`input.rs`](../../../crates/jcode-tui/src/tui/app/input.rs), [`followup.rs`](../../../crates/jcode-todo-policy/src/followup.rs), and existing TUI scheduler tests. Inspect current coverage before changing logging. Any narrowly justified logging adjustment remains part of existing logging; it must not add persisted receipt data, raw content, or an export path.

Actual behavior changes have a separate gate. This design does not authorize implementing a behavior fix, claim that current behavior is correct, or make gateway deployment changes or a new journal prerequisite. Design is not implementation approval.
