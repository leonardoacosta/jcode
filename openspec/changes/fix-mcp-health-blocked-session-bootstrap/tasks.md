# Tasks

## 1. Cached bootstrap
- [x] 1.1 Add cached-schema/event/shared-pool contention regression and capture its failure on the original registration path.
- [x] 1.2 Publish initial cached status without live health and prove cached proxies/status return before health release with focused tool tests.
- [x] 1.3 Exercise registration-enabled Subscribe and assert SessionId/Done under health contention with a passing integration regression.

## 2. Pool coordination
- [x] 2.1 Add deterministic lock-convoy regressions and capture failures on original begin_connect behavior.
- [x] 2.2 Release connection coordination before map waits, independently snapshot health, and pass convoy, leadership, connected-fast-path, cancellation, notification-order, backoff, and redaction tests.

## 3. Local acceptance and review
- [x] 3.1 Run affected crate tests, formatting checks, cargo check, and selfdev binary build through the development wrapper; record fresh output.
- [x] 3.2 Run the rebuilt binary with isolated home/socket/local MCP fixture; prove SessionId/History while MCP setup is pending and eventual live status afterward, then stop test-owned processes.
- [x] 3.3 Obtain independent review of the final diff, repair actionable findings, rerun affected checks, and record evidence without claiming installed-runtime acceptance or publication.

## Evidence
- Cached-registration original-path regression: `cached registration must return before health is released`; 0 passed, 1 failed. Repaired focused regression: 1 passed, 0 failed.
- Original pool regressions: 14 passed, 2 failed on retained coordination/health guards. Repaired pool suite: 17 passed, 0 failed.
- Registration-enabled Subscribe failed on original health await, then passed after restore. Final two cached-bootstrap regressions: 2 passed, 0 failed; final session lifecycle suite: 40 passed, 0 failed (`/tmp/jcode-bootstrap-client-session-final.log`).
- Independent review found no remaining correctness defects. Fixed test cleanup to await both manager and final registry health publications before dropping temporary environment; reran both regressions and session suite.
- Full tool suite: 63 passed, 5 failed, 1 ignored. Failures: MCP stdio fixture environment, strict-mode tool allowlist, post-tool hook output, tool-description cap, parameter-description cap. No fixes outside repair scope.
- `cargo check -p jcode-base -p jcode-app-core` and selfdev binary build passed through `scripts/dev_cargo.sh`; logs `/tmp/jcode-bootstrap-check.log`, `/tmp/jcode-bootstrap-build.log`.
- Changed pool/test files passed rustfmt; scoped diff check passed. Whole `tool/mod.rs` rustfmt check reports existing module-ordering differences, left untouched.
- Isolated rebuilt-daemon acceptance: cached MCP status plus SessionId/History in 0.004s while local initialize remained gated; connected health arrived after release without reconnect. Test-owned processes stopped. Evidence `/tmp/jcode-bootstrap-7dhlwtzp`; harness `/tmp/jcode-bootstrap-acceptance.py`.
- Original repair acceptance left installed/shared daemon, launcher, config, commits, publication, and archive untouched.
- 2026-11-03 baseline checkpoint: user authorized committing pending work and non-force synchronization before new worktrees. Fresh coordinated task `169918s4ds` passed 41 bash tests, 68 registry tests (1 ignored), the Subscribe contention regression, 17 pool tests and the real stdio integration test. Strict OpenSpec validation and `git diff --check` passed.
- Broader gates remain failed, not hidden by aggregate shell status: task `0156004ab0` exposed configured-model spawn assumptions, project-skill discovery in the nominal empty-skill test, remote-status request-count drift and pre-existing workspace rustfmt differences. Provider suite rerun `169918s4ds` passed 198 tests but failed Azure retry fixture Content-Length parsing and the stalled-response fixture connection reset; the pending provider diff adds only an ignored review driver and was not executed. Full-suite or formatting success is not claimed. No archive or installed-runtime acceptance is added by this checkpoint.
