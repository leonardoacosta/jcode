## 1. Implementation
- [ ] 1.1 Browser tests for out-of-scope tab, each `requested_help` kind, unavailable bridge
- [ ] 1.2 Voice tests for Esc cancel, missing mic/key, `<transcription>` wrapping on send
- [ ] 1.3 Repair only demonstrated violations of the listed tab/goal/help/capture boundaries, with a failing regression first; verify through actual browser-tool dispatch/bridge and voice composer event paths in isolated fixtures. Do not use this task for new browser actions, new voice providers, live user login/microphone capture or broader permissions. If a fix requires those effects, record the blocker for separate scope rather than expanding this change.
- [ ] 1.4 Document asserted boundaries in `docs/BROWSER_FAST_AGENT.md`

## 2. Verification
- [ ] 2.1 `cargo test -p jcode-app-core browser`
- [ ] 2.2 `cargo test -p jcode-tui voice` and `cargo test -p jcode-voice`
- [ ] 2.3 `openspec validate add-browser-voice-boundary-acceptance --strict`

### Visible overnight JIT, 2026-11-09 21:22 UTC

Current HEAD7abc9a0da, main preserved (origin HEAD dev mismatch blocks branch-dependent closeout), existing proposal empty dependencies. Run overnight_1794256623697_10169366891028959966 authorizes bounded local test work, not real login/mic/provider calls or reviewers. Coordinator serialized ownership of clean-start app/tests/voice_input.rs and app-core/tool/browser_tests.rs only, docs/BROWSER_FAST_AGENT.md if needed. Scope tests/composer paths, no new capture/network APIs. G1=2,G2=1(no real mic/login/reviewer permitted),G3=1(default mismatch/publication excluded),G4=2(private fixture runtime),G5=2(clean exact targets),G6=1(Esc recording fixture access incomplete):9/12 shadow, not authority. Existing browser_fast mocks cover script/text/login/scope but do not count as live browser acceptance. Missing-key real composer path and actual BrowserTool status dispatch can be exercised offline. Full proposal closeout stays blocked.

### Offline acceptance subset, 2026-11-09 21:29 UTC

- Actual BrowserTool::execute status in fresh private HOME/JCODE_HOME returns ready=false/not-installed within5seconds and does not create browser install. Exact child-test path initially wrong, explicit `1 passed` assertion caught zero-match and was corrected; passing result runs the intended code. No real browser launch, setup, credential, or model call.
- Added actual composer key missing-key regression: fresh child strips NARI_API_KEY/private config, Ctrl+Space reports existing Nari help, voice_input=None, pending/processing=false, typed draft/cursor unchanged. Child asserts nonzero test count. Added transcript `/clear` regression proving wrapped user data through normal submit, not slash-command dispatch.
- Actual normal-binary Alacritty/private-X11 agent-desktop Ctrl+Space repeats missing-key effect, draft `keep my typed draft` unchanged, processing=false; screenshot/result retained current run validation/task-004-voice-*. Binary4724b2ade4c4fb564424aee293940443052750b1bfa07a4de26b6a53675374d1. No capture started, no provider request.
- Focused suites98browser pass7ignored,12TUIvoice pass,70voice pass. Strict named spec/diffcheck pass; documentation names exact checks and limits. Existing browser_fast mocks cover scope/script/text/login but are not live handoff acceptance. Active-recording Esc/missing-recorder fixture remains unimplemented: NariRecording owns private worker/capture fields across crate boundary, supported start binds actual provider URL; no new fixture-only production API, microphone or real service call introduced. Full native tasks/review/commit/archive remain pending.
