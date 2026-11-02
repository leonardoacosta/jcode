# Tasks

- [ ] Add a judge DM parser for the prefix and single-message rule.
- [ ] Add the per-turn continuation counter and config cap.
- [ ] Mark admitted feedback as advisory in the injected message.
- [ ] Ensure `/review` results never auto-continue.
- [ ] Add tests: valid continue, stop, cap reached, malformed, duplicate.
- [ ] Run `cargo test -p jcode-tui` and `cargo test -p jcode-base` covering the new success and failure scenarios.
- [ ] Run `openspec validate admit-judge-feedback-with-bounds --strict`.
