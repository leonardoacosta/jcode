# Question pane and overnight advice

Status: approved by the user on 2026-09-24, with implicit enablement for new overnight runs and prompt-driven opt-out. Implementation has not started.

## Why
The question renderer clears the entire TUI and returns before rendering the conversation. Its fixed-size box flattens option labels and descriptions into one paragraph. Users lose context while making a decision.

## What Changes
- Replace the composer with a content-sized question pane while preserving transcript, status and the user's draft.
- Offer asynchronous, explicitly labeled Jev recommendations through the configured Jev MCP capability. Failure never blocks manual answers or produces an invented recommendation.
- Add an implicitly enabled overnight policy for new runs for low-risk, delegated decisions after five minutes without user interaction. Advice is not permission. Unknown or sensitive decisions remain human-only.
- Persist provenance for automatic answers and surface it in tool results, transcript, overnight logs and deterministic end-of-run review.

## Capabilities
### New Capabilities
- `question-composer-pane`: integrated, scrollable TUI question and review surface.
- `question-recommendations`: optional bounded Jev advice with visible provenance and failure states.
- `overnight-question-decisions`: explicitly delegated, audited timeout decisions.

## Impact
TUI layout/input, question protocol and server lifecycle, configured MCP dispatch, overnight manifests/events/report rendering. No new database. Old manifests remain manual-only until a new run starts. Wire extensions require negotiated capability support.

## Non-goals
Desktop UI, generic permission auto-approval, secrets collection, changing retention, auto-enabling overnight from the clock, worker question forwarding, headless question exposure, durable pending-channel resurrection, multi-select automatic answers, broad MCP repair, unrelated existing dirty changes.

## Dependencies and approval
Builds on the existing native question implementation from `native-tool-interaction-and-routing`. That proposal's checklist is stale relative to working source. Verify baseline behavior before implementation. This change deliberately supersedes its human-only response rule ONLY for explicit, policy-eligible automatic answers with non-human provenance. Existing manual contracts stay intact.

Approved automation defaults: enabled for new overnight runs unless the user opts out in the launch prompt, a later direct instruction, or the disable control, single-choice batches only, minimum Jev probability 0.90 and winner margin 0.20, unknown risk blocks automation. These are conservative policy choices, not measured calibration guarantees.
