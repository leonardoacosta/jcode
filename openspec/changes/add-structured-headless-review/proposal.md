---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P08: SDK/API/tool integration contracts, including browser/voice](../../../docs/plans/PRODUCT_ROADMAP.md#p08-sdkapitool-integration-contracts-including-browservoice)

## Why

Autoreview returns a free-text DM: the reviewer is told to summarize findings "or `No issues found`" (`crates/jcode-tui/src/tui/app/commands_review.rs:479-505`). There is no machine-readable verdict, no binding to the reviewed revision or checks, and no distinct outcome for inconclusive, cancelled or budget-exhausted reviews, so headless callers must parse prose to tell pass from fail. The roadmap gate forbids free-text PASS parsing.

## What Changes

- Define a review verdict JSON schema: `verdict` (`pass`|`fail`|`inconclusive`|`cancelled`|`budget_exhausted`), `revision` (git HEAD + dirty flag), `checks` run with exit status, and `findings` (severity, path, message).
- Add a headless entry (`jcode review --json`) that runs one reviewer session read-only and prints exactly one verdict object; exit code maps from verdict.
- Have autoreview request the same structured output and display it; invalid or missing JSON yields `inconclusive`.

## Non-scope

- Judge CONTINUE/STOP admission (admit-judge-feedback-with-bounds).
- Auto-fixing findings.

## Capabilities

### New Capabilities
- `structured-review-verdicts`: typed, revision-bound review results.

### Modified Capabilities
- None.

## Impact

`src/cli/args.rs`, `crates/jcode-tui/src/tui/app/commands_review.rs`, a new verdict type. Reviewer remains read-only per existing guardrails.

## Resolved defaults

- Exit codes: distinct per verdict or pass=0/else=1? Selected default: distinct (0 pass, 1 fail, 2 inconclusive, 3 cancelled/budget), because CI must distinguish a real failure from an unavailable reviewer.
