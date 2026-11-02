# Tasks

## 1. Claim store

- [x] 1.1 Add a canonical-path to session claim map in swarm server state, persisted with swarm state.
- [x] 1.2 Add `swarm.file_claims` config: `off`, `warn` (default), `block`.

## 2. Enforcement

- [x] 2.1 Claim on first write by edit/write/patch tools.
- [x] 2.2 On conflicting write, return a warning naming the holder, or refuse in `block` mode.
- [x] 2.3 Release claims on finish, stop, and orphan detection (via `recover-swarm-workers-after-reload`).
- [x] 2.4 Show active claims in `swarm status`.

## 3. Verification

- [x] 3.1 Unit tests: claim, conflict warn, conflict block, release on stop and orphan.
- [x] 3.2 Test claims survive server reload.
- [x] 3.3 Run `openspec validate add-swarm-file-claims --strict`.
- [x] 3.4 Exercise real tool registry calls in isolated processes for default/off/warn/block, edit/write/both patch formats, coordinator bypass, blocked multi-file/move/delete preflight, status snapshots, resumed workers, and persisted orphan recovery.

## 4. Independent review safety repairs

- [x] 4.1 Resolve dangling symlink targets before claiming and verify real alias/target writes cannot bypass block mode.
- [x] 4.2 Serialize cross-process claim updates, refresh persisted state under the lock, and verify only one concurrent writer wins.
- [x] 4.3 Roll back failed/cancelled reservations and blocked batch preflight without releasing claims for successful mutations or earlier ownership.
- [x] 4.4 Release claims for actual `orphaned` recovery with a persisted worker spec and missing session.
- [x] 4.5 Independently re-review the final safety repair after fresh gates.
- [x] 4.6 Preserve overlapping same-session reservations with generation-aware rollback, retaining successful ownership and other in-flight reservations.

### Safety verification evidence (2026-11-02)

All Cargo checks below ran through `selfdev test`, with `&&` chaining and preserved exit status. Final gate task: `732566q89c`, exit 0. The earlier red run (`884009askd`, exit 101) reproduced two concurrent process winners and a failed Codex patch retaining a claim.

- `cargo test -p jcode-app-core --lib server::swarm:: -- --nocapture`: 43 passed, including 10 claim tests. These cover real registry calls in four isolated config modes, dangling/relative symlink identity, cycle rejection, failed writes/edits/patches, blocked batches, partial success, cancellation with continuing filesystem work, stale-store rollback, corrupt-state refusal, concurrent processes, status, and actual orphan recovery.
- `cargo test -p jcode-app-core --lib server::swarm_persistence:: -- --nocapture`: 25 passed.
- `cargo test -p jcode-app-core --lib tool::<module>::` for `edit`, `write`, `apply_patch`, and `patch`: 20, 6, 22, and 7 passed respectively.
- `cargo test -p jcode-base --lib config::tests`: 87 passed. `cargo test -p jcode-protocol --lib`: 100 passed.
- `cargo test -p jcode-app-core --lib server::swarm::claims::claim_tests::file_claim_processes_share_one_owner -- --exact`: five additional independent runs passed. `git diff --check` and `openspec validate add-swarm-file-claims --strict` passed.

Recovery verification reran the complete commands above under `selfdev test`: task `147887hy1p` exited 0 with the same 310 passing tests, strict validation, and diff check. Task `206873pxur` exited 0 after five fresh real-process race runs, using `|| exit $?` to preserve failure status. These gates did not cover overlapping reservations and are superseded by the next verification below. The final independent re-review is recorded after the overlap repair below.

Sauropod then found that an older same-session reservation's rollback could erase a newer successful write's ownership. Red task `471028599y` exited 101 with both the persisted-store regression and a real registry write bypass. The repair records unique reservation tokens for provisional claims, makes successful ownership permanent until lifecycle release, and rolls back only the caller's token. Committed ownership retains the legacy string-map representation. Regression coverage includes newer success/older rollback, previous success/later rollback, both all-failed drop orders with surviving in-flight reservations, and stale generations after lifecycle release. Initial focused green task `544345j7wp` passed 11 claim tests. Final locked gate `619199lka0` exited 0 with 44 swarm tests (11 claim tests), 25 persistence tests, 55 mutation-tool tests, 87 configuration tests, and 100 protocol tests, totaling 311. Five additional real-process races, strict OpenSpec validation, and `git diff --check` passed in the same `&&`-chained gate. Sauropod completed final independent re-review at `2026-11-02T17:16:47Z` with explicit `FINAL READY` and no remaining blockers. Its independent locked gate `7122577rbd` passed 11 claim tests, 25 persistence tests, strict OpenSpec validation, and diff check. Reviewed `claims.rs` SHA256 `8b280a76906d17fc81d65e4f561f4e9b5cc1064eed2c754f982cbb528470cd9a` matches the final source. No implementation edits followed these gates. Commit-range review remains required before any archive, which is outside this worker's scope.
