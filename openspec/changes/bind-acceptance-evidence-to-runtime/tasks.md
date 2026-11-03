# Tasks

- [x] Expose server git hash and authoritative dirty flag in the protocol handshake.
- [x] Extend `jcode version --json` with source, client, server and verdict fields.
- [x] Update `/version` rendering with one row per field and ellipsized hashes.
- [x] Add tests for matched, mismatched, dirty and unknown verdicts.
- [x] Run required TUI and base coverage with the documented isolated-process topology below.
- [x] Run `openspec validate bind-acceptance-evidence-to-runtime --strict`.

## Immutable build execution

- [x] Implement authorized opt-in local immutable source compilation without Git/worktree/shared-daemon mutations.
- [x] Verify offline clean/edit/revert, source fingerprint, caller/config forgery, compiler-time mutation and unsafe input rejection.
- [x] Preserve mutable-wrapper identity, changelog and job-sizing regressions.
- [x] Verify actual Jcode immutable binary with isolated clean/edit/stale-daemon public CLI acceptance.
- [x] Complete fresh gates, independent final review and implementation commit before native archive.

## Final evidence, 2026-11-03

Implementation: `f232d2448`, reviewed range `36c5e6d61..f232d2448`. Independent reviewer zebra found no remaining actionable findings after sealed metadata environment precedence and requested-runtime narrowing repairs. No provider inference, shared daemon restart, live config change or deployment occurred.

- `python3 scripts/test_dev_cargo_cwd.py`: 7 passed, including immutable source mutation/remount/proc-alias/userns attacks and Cargo environment forgery. Job-sizing/changelog regressions passed. Build-meta tests: 4 passed. Logs: `scratch/immutable-wrapper-final.log`, `immutable-jobs.log`, `immutable-changelog.log`, `immutable-meta-tests.log` under primary JCODE_HOME.
- Actual clean immutable Jcode build: `JCODE_IMMUTABLE_BUILD=1 CARGO_TARGET_DIR=/home/nyaptor/.jcode/target scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode` passed. Log `scratch/immutable-clean-acceptance-build.log`; source fingerprint `86dc6537767d964ed1cdf55f8bc2c3e27a6d16d6c14c458e954d961c873b8416`. Compiler inputs enforced read-only in private Linux namespaces; mutable builds remain unverified.
- Actual isolated daemon/client: `scratch/immutable-clean-runtime-k2cx3_fj/{matched,edited,restored}.json` proves clean source/client/server `f232d2448` matched, source edit mismatched, restored source matched. Exact serving binary path is reported.
- Actual new client versus old serving daemon: `scratch/immutable-stale-daemon-2wl4elti/version.json` proves client `f232d2448` versus server `36c5e6d61` mismatched. Owned processes stopped after each probe.
- Complete TUI suite: 2,907 passed, 0 failed, 20 ignored (`scratch/expanded-final-jcode-tui.log`).
- Monolithic base suite failed with 2,019 passed, 3 failed, 8 ignored (`scratch/expanded-final-jcode-base.log`). Investigation identified the existing proxy-mutating fixture as an isolation boundary. Full equivalent coverage then passed with only that fixture in a separate fresh process: remainder 2,021 passed, 0 failed, 8 ignored, 1 filtered; isolated proxy test 1 passed. Logs `scratch/expanded-base-proxy-isolated-{rest,proxy}.log`. No assertion removed or skipped without separate execution. This is a split-process gate pass, not a monolithic pass.
- Root all-target check, selected leaf all-target strict Clippy and diff checks passed. Runtime helper regression suite: 6 passed, including hidden-index flags and safe Git filter/fsmonitor handling.

## Historical gap resolved

Mutable prebuild snapshots could not attest final compiler inputs, as a deterministic edit/compile/revert regression proved. The user authorized an immutable execution path; that path now passes actual clean/stale serving acceptance. Mutable/default builds correctly remain unknown rather than inheriting the immutable claim. Source attestation is not reproducible-toolchain or cryptographic executable authenticity. Unsupported namespace enforcement fails closed. Archive and persistence are native lifecycle operations following this verified task state, not inferred from checked boxes.
