# Herdr + Cursor Mac integration spike

Date: 2026-09-23. This is an authorized disposable feasibility test, not a shipped extension or formal feature implementation.

## Accepted user decisions

The user approved project/server identity, remote-host placement, separation from task/debug/Cursor automation, release-only disposal, and the distinction between client persistence and reboot recovery. The user replaced the initial takeover recommendation: **IDE-created sessions always take over; sessions created outside the IDE are observe-only.** This supersedes the earlier exploration's explicit-takeover default. Durable creation provenance is required before automatic routing can safely ship. Absent/unknown provenance should observe, not infer IDE origin from labels or directory paths. IDE-origin needs to be scoped to a server/terminal identity and survive loss of the extension's cache. Clarify whether all compatible IDE windows may take over the same IDE-created session, and avoid automatic reconnect loops that repeatedly steal ownership from each other.

## Environment and isolation

- Authorized host: `ssh mac`, macOS 27.0, Apple Silicon.
- Cursor 3.21.18, commit c4730f7d93d787d9ab120af715999f0345ee5bc0.
- Actual Cursor extension host reported VS Code API version 1.128.0.
- Installed Herdr 0.9.1.
- Used Cursor's real `--extensionDevelopmentPath` / `--extensionTestsPath` runner, with the installed editor binary. Context7 confirmed the supported extension test-host pattern.
- Used fresh task-owned user-data and extensions directories. No personal profile was copied or modified. No extension was installed into the user's editor.
- Disabled IDE terminal persistence in the test profile. Thus revival cannot explain the observed persistence.
- Created one named disposable Herdr server, `ide-spike-20260923`, with a separate config. The explicit `--session` selector determines its actual sockets under `~/.config/herdr/sessions/ide-spike-20260923/`, overriding the attempted scratch socket paths. Observed this in server startup output before using it.
- Original Cursor process PID 14663 remained running throughout and after cleanup.
- SSH does not inherit HERDR_ENV. No default/focused remote Herdr state was inspected or controlled. Operations targeted only the explicitly created disposable named session from the authorized test harness.

## Results

Three complete phases ran without failed assertions. See `events.jsonl` for machine-readable receipts and `tests.js` for the disposable harness.

| Check | Result | Evidence and limitation |
| --- | --- | --- |
| Native Cursor terminal runs direct attach | Pass | Created with real vscode.window.createTerminal, Herdr executable and terminal attach arguments |
| Closing terminal preserves shell/child | Pass | Terminal.dispose invoked the IDE terminal disposal path. Shell PID 8903 and child PID 9851 remained alive, heartbeat advanced. Toolbar mouse click was not separately tested |
| Reattach same shell | Pass | Shell wrote $$ after reattach, still 8903 |
| Full isolated Cursor process exit | Pass | Test host finished and its main process 5672 exited. Original personal Cursor remained running |
| Progress while test Cursor absent | Pass | Child 9851 advanced heartbeat from 60 to 79 before a new test Cursor instance launched |
| Reopen and reconnect same process | Pass | New Cursor instance attached shell 8903 and child 9851, heartbeat 83 |
| IDE-origin automatic takeover mechanism | Pass | New direct attachment used --takeover. Old attachment PID 93728 exited, replacement 93992 accepted input, shell/child survived |
| Read-only observation while controlled | Pass | Observe emitted frames, did not displace controller. Attempted input written to observer stdin did not create the test file |
| Native terminal resize | Pass | Maximizing Cursor panel changed Herdr shell stty size from 16x61 to 50x61 |
| Ctrl-C | Pass | Input byte 0x03 interrupted foreground sleep, shell and background heartbeat remained alive |
| Attachment crash | Pass | SIGKILL of only the attachment process preserved shell/child and continuing heartbeat. Reattach again returned shell PID 8903 |
| One workspace / three tabs | Pass | Initial workspace root tab plus two tab.create calls produced exactly three tabs in w1 |
| Native Pseudoterminal observer | Pass, bounded | JSON observe frames decoded into a real native Cursor Pseudoterminal. Input callback ignored attempted input. This is transport/adapter evidence, not end-to-end durable origin classification |
| Quoted text input | Pass | Terminal.sendText produced the exact expected literal file text. OS clipboard and bracketed-paste UI not tested |
| Alternate-screen command returns | Pass, bounded | ANSI enter/clear/leave sequence and command completion succeeded. Visual output was not inspected |
| Shell integration | Not available in tested path | terminal.shellIntegration was false at multiple checkpoints. No command-boundary/exit-status integration was established |
| Visual screenshot | Blocked | screencapture exited 1: could not create image from display. Accessibility exposed the test window title and 1000x800 geometry, but that is not image verification |

## What this proves

Herdr can own live processes independently of the actual Cursor process on this Mac. A thin extension can attach them in real native terminal tabs, detach on terminal disposal, reconnect to the same shell and preserve background work. The writable and observe-only mechanisms needed by the approved policy both work in this environment. The observer path can also feed a native terminal rather than requiring a webview.

This is not a terminal emulation mock. The process persistence, IDE terminal API and real editor-exit boundaries were exercised. The extension entry is deliberately empty, with test code using the native API. It is not a profile provider/sidebar implementation and does not implement durable provenance, automatic view restoration or project workspace deduplication.

## Remaining gates

- Actual screenshot-based visual inspection, mouse selection/wheel, clipboard, native scrollback/search, links, screen reader behavior and Unicode glyph fidelity.
- A real agent TUI, not just alternate-screen escape sequences.
- Cursor agent-runner compatibility remains explicitly excluded. Ordinary interactive terminal success must not be presented as sandbox/agent integration.
- VS Code proper was not tested. Neither was Remote SSH extension placement, WSL, Windows or another Mac.
- Window reload and extension-host crash were not exercised. Full isolated editor exit and attachment SIGKILL were exercised.
- Production origin tagging and server-qualified identity. The observe fixture was created by the test harness, not independently discovered from another real terminal application. Do not claim origin classification is implemented.
- Multi-window simultaneous workspace creation, cancellation/retry idempotency, tab restoration and reconnect/takeover races.
- Behavior when Herdr server dies/restarts or host reboots.
- Task-owned profile disabled workspace trust solely for a known disposable workspace. Production trust handling remains a requirement.

## Recommendation after testing

Proceed to feature authoring for a thin extension. Persistence is no longer merely a source-based expectation: it passed in real Cursor. Use native direct-attach terminals for IDE-owned writable sessions and a native Pseudoterminal observer adapter for externally created sessions, subject to the remaining visual/scrollback checks. The observed absence of shell integration means v1 should promise persistent interactive terminals, not full ordinary-shell feature parity. Prefer consistent transport only if a follow-up test establishes a meaningful benefit, not for abstraction symmetry.

Do not create a custom webview terminal to solve an unmeasured fidelity gap. Do not automatically reconnect a displaced IDE client using takeover in a loop. Store immutable origin separately from current owner/attachment state.

## Cleanup and evidence

After the final Cursor instance exited, heartbeat was independently verified to advance again. Then only the task-owned Herdr server PID 84379 was terminated. Its shell and child exited as expected. Final receipt: serverAlive=false, shellAlive=false, childAlive=false, taskCursorAlive=false, originalCursorAlive=true.

No personal IDE settings, existing Herdr workspaces or source repositories were changed. No credentials, authentication profiles or browser CDP endpoints were accessed. No remote plugins or dependencies were installed. Disposable scratch files and the named test session's logs remain for reproducibility, but no test processes remain.

Local retained evidence:
- `/home/nyaptor/.jcode/research/herdr-cursor-spike-20260923/report.md`
- `/home/nyaptor/.jcode/research/herdr-cursor-spike-20260923/events.jsonl`
- `/home/nyaptor/.jcode/research/herdr-cursor-spike-20260923/tests.js`

Remote test files:
- `/Users/leonardoacosta/.jcode/scratch/herdr-cursor-spike-20260923/`
