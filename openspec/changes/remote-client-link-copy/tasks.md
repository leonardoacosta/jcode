## 1. Copy routing (fix 1)
- [x] 1.1 Add `is_remote_session()` and the `JCODE_CLIPBOARD` override in `helpers.rs`, as a pure function over environment input
- [x] 1.2 Make `copy_to_clipboard` emit only OSC 52 when the session is remote. Keep the local order unchanged
- [x] 1.3 Unit tests for every remote-detection scenario (environment-injected, no OS clipboard)

## 2. Link picker (fix 2)
- [x] 2.1 Collect link destinations from rendered Markdown in the transcript
- [x] 2.2 Add the `/links` picker and a copy-last-link key that route through `copy_to_clipboard`
- [x] 2.3 Tests: a wrapped long URL copies exactly; with no links, show a notice and do not copy

## 3. Mouse toggle (fix 3)
- [x] 3.1 Add the `/mouse` command and key that call Enable or Disable `MouseCapture` and keep the state across redraws and resume
- [x] 3.2 Test that the state flips and the notice appears

## 4. Remote link open (fix 4)
- [x] 4.1 In `navigation.rs`, gate on `is_remote_session()` instead of `HERDR_ENV`. Always copy through OSC 52 and use a non-wrapping notice
- [x] 4.2 Test that a remote session with no host copies and does not launch a browser

## 5. Matrix (fix 5)
- [x] 5.1 Add `docs/remote-clipboard.md` with a Ghostty/Tern/Moshi × SSH/mosh matrix and the OSC 52 probe command
- [ ] 5.2 Manual: run the probe and `/links` copy from each available client and record the results (blocked on physical devices)

## 6. Verify
- [x] 6.1 `cargo test -p jcode-tui` and `openspec validate remote-client-link-copy --strict`
