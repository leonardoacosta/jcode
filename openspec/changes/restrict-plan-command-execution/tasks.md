# Tasks

- [ ] Add a plan-only flag to the turn request and carry it from `commands_plan.rs` to the server.
- [ ] Classify native tools as read-only or mutating and enforce denial in tool dispatch.
- [ ] Add the read-only bash allowlist per the resolved open decision.
- [ ] Surface an unenforced notice when the server does not acknowledge the flag.
- [ ] Add tests: read allowed, edit denied with no file change, restriction cleared after completion and cancel, old-server notice.
- [ ] Run `cargo test -p jcode-tui` and `cargo test -p jcode-base` covering the new success and failure scenarios.
- [ ] Run `openspec validate restrict-plan-command-execution --strict`.
