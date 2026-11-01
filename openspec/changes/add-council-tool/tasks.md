## 1. Runtime

- [ ] 1.1 Verify swarm spawn/message/await/stop entry points in `crates/jcode-app-core/src/tool/communicate.rs` and dependency changes landed
- [ ] 1.2 Add tool allowlist option to swarm spawn; enforce in executor
- [ ] 1.3 Add `council.rs` with registry, caps, barrier, vote parsing, early stop

## 2. Tool surface

- [ ] 2.1 Implement `open`/`round`/`say`/`log`/`close`/`run` and register tool
- [ ] 2.2 Verdict call returning PASS|PARTIAL, tally, dissent; optional `council` model slot
- [ ] 2.3 Persist transcript JSON per round under session dir

## 3. Lifecycle

- [ ] 3.1 Stop seats on close/abort/error via drop guard
- [ ] 3.2 Stop owner-less council seats in reload recovery

## 4. Verification

- [ ] 4.1 Tests: seat-count rejection, call cap, timeout naming, early stop, split-vote verdict, edit rejected
- [ ] 4.2 Runtime check: 3-seat run leaves `git status` unchanged, calls <= 7, no live seats after
