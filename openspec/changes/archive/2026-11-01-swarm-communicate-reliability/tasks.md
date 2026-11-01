## 1. Deadlines and errors

- [x] 1.1 Add per-action deadline selection in `communicate/transport.rs`; route `spawn`/`launch`/`retry`/`resume` through the extended deadline; unit test the mapping
- [x] 1.2 Introduce a typed request error (timeout / closed / server) in `transport.rs`; update `Failed to ...` wrappers in `communicate.rs` to preserve it; add "may have succeeded, check `status`" text for mutating timeouts
- [x] 1.3 Tests for timeout vs server-error message classification

## 2. Discoverability and await

- [x] 2.1 Build the unknown-action message from `ADVERTISED_SWARM_ACTIONS`; test it lists `launch`, `list_models`, `list_swarms`
- [x] 2.2 Parity test: every `ADVERTISED_SWARM_ACTIONS` entry matches a schema enum value and dispatches without the unknown-action error
- [x] 2.3 Honor `await_members background=false` (remove forced `background = true`); test the blocking deadline branch
- [ ] 2.4 Test `await_swarm_progress` loop: wakes on new ready task, returns on member completion, tolerates refresh failure (deferred: needs a fake CommAwaitMembers socket server; wake predicate already covered by `await_should_wake_for_new_ready` tests)

## 3. Verify

- [x] 3.1 `cargo test -p jcode-app-core communicate`
- [x] 3.2 `openspec validate swarm-communicate-reliability --strict`
