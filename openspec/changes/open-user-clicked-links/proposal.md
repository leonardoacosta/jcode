## Why

Markdown links are currently rendered with visible destinations but do not provide a deliberate mouse action to open them. The existing Linux system opener supports URLs and local paths, including valid `file:///` targets, but the generic open-tool suppression prevents launching external applications in harness-driven sessions. Users need a safe, explicit way to open rendered links without enabling agent-initiated or test-driven launches.

## What Changes

- Add a user-initiated TUI action for opening a rendered Markdown link under the pointer.
- Route supported HTTP(S) and local `file:///` destinations through the existing system opener (`xdg-open` on Linux, existing platform equivalents elsewhere).
- Preserve open-tool and test-harness suppression for agent-initiated or automatic opens; keep explicit `NO_BROWSER` and `JCODE_NO_BROWSER` opt-outs effective.
- Retain link destination metadata through rendering so the action can resolve the destination reliably, including wrapped and scrolled message content.
- Report opener failures in the TUI without losing normal mouse selection behavior for non-link clicks.

## Capabilities

### New Capabilities
- `tui-link-opening`: Explicitly open supported rendered Markdown links from a user click in the TUI, including local file URLs.

### Modified Capabilities
- None.

## Impact

- TUI Markdown parsing/rendering and message layout metadata.
- TUI mouse-event handling and user-visible error reporting.
- Existing platform open helper and suppression policy, with the explicit user gesture kept separate from agent tool calls.
- Tests for link target retention, click hit-testing, local file URL handling, opt-outs, and suppression of non-user-initiated opens.
