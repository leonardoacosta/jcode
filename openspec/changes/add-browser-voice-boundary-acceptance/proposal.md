---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P08: SDK/API/tool integration contracts, including browser/voice](../../../docs/plans/PRODUCT_ROADMAP.md#p08-sdkapitool-integration-contracts-including-browservoice)

## Why

Browser and voice boundaries exist but lack acceptance coverage at their edges. The browser tool scopes by `tab_id` and errors when it is missing for `select_tab` (`crates/jcode-app-core/src/tool/browser_request.rs:47-50`, `:262`), but there is no acceptance test that a handoff cannot act outside its tab/goal or that help hand-back (script/text/user_login) and an unavailable browser bridge return explicit states. Voice sends the final transcript to the agent on key release (`crates/jcode-tui/src/tui/app/input_help.rs:127`, `crates/jcode-tui/src/tui/app/voice_input.rs:108`) and reports missing microphone or key (`crates/jcode-voice/src/lib.rs:59-83`); no test confirms Esc cancels without sending or that transcripts are treated as user input data, never tool authority.

## What Changes

- Add browser tests: handoff action on a different tab is refused; each `requested_help` kind returns a typed hand-back; unavailable bridge returns an explicit error, not a hang.
- Add voice tests: Esc discards without sending; missing mic/key surfaces the existing error; transcript is wrapped in `<transcription>` and nothing is executed from it without the normal turn.
- Document the asserted boundaries in `docs/BROWSER_FAST_AGENT.md`.

## Non-scope

- New browser actions or voice providers.
- Desktop voice (consumer boundary only).

## Capabilities

### New Capabilities
- `browser-voice-boundaries`: acceptance contract for browser scope and voice capture.

### Modified Capabilities
- None.

## Impact

Tests in `crates/jcode-app-core/src/tool/browser_tests.rs`, `crates/jcode-tui` voice tests, doc update. Fixes only if tests expose violations.

## Open decisions

None.
