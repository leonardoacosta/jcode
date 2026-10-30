# Proposal

## Why

Jcode can consume a draft before submission validation succeeds, leaving only an error instead of a recoverable user message. A failed submission must preserve the complete prompt and surface it beside the error in a copyable block, without relying on prompt history or asking the user to retype it.

## What Changes

- Capture the full user-authored prompt, with pasted text expanded, before input, paste storage or recovery state can be discarded in local and daemon-connected TUI submission paths.
- Show a clearly labeled **Failed prompt** block with the error. Its complete text must remain inspectable and copyable, including multiline and large input, without truncation, paste placeholders or injected skill/system instructions.
- Restore an empty composer for editing without overwriting a newer draft. Keep the failed prompt independently accessible when the composer is occupied or clipboard copying fails.
- Associate recovery with the correct submission. Handle repeated error events without duplicate recovery records or new automatic resubmissions, and distinguish rejection before dispatch from an uncertain remote outcome.
- Preserve existing secret-input and history exclusions. Reuse current recovery/display/copy machinery rather than add a persistent prompt journal.

## Capabilities

### New Capabilities

- `failed-prompt-recovery`: Lossless, safely rendered, copyable recovery of user-authored prompts after interactive submission or turn failures.

### Modified Capabilities

None.

## Impact

Primary surfaces are `crates/jcode-tui/src/tui/app/input.rs`, existing failed-input handling in `model_context.rs`, remote submission/error handling, and the shared transcript/copy path. Tests must cover paste expansion, validation rejection, provider/transport errors, concurrent drafts, duplicate events and privacy exclusions.

No new dependencies, durable storage or protocol changes are planned. This covers the interactive TUI in local and daemon-connected modes, not the separate Desktop repository or new headless output formats. Crash/restart recovery, historical-prompt reconstruction and retry-policy redesign are out of scope.

This deliverable is planning only. The harness fix remains unimplemented, with all implementation tasks unchecked. See [design](design.md), [requirements](specs/failed-prompt-recovery/spec.md), [tasks](tasks.md) and [source evidence](research.md).
