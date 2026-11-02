## 1. Implementation
- [ ] 1.1 Browser tests for out-of-scope tab, each `requested_help` kind, unavailable bridge
- [ ] 1.2 Voice tests for Esc cancel, missing mic/key, `<transcription>` wrapping on send
- [ ] 1.3 Fix any boundary violation the tests expose
- [ ] 1.4 Document asserted boundaries in `docs/BROWSER_FAST_AGENT.md`

## 2. Verification
- [ ] 2.1 `cargo test -p jcode-app-core browser`
- [ ] 2.2 `cargo test -p jcode-tui voice` and `cargo test -p jcode-voice`
- [ ] 2.3 `openspec validate add-browser-voice-boundary-acceptance --strict`
