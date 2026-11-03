# Tasks

- [ ] Expose server git hash and authoritative dirty flag in the protocol handshake.
- [x] Extend `jcode version --json` with source, client, server and verdict fields.
- [x] Update `/version` rendering with one row per field and ellipsized hashes.
- [x] Add tests for matched, mismatched, dirty and unknown verdicts.
- [ ] Run `cargo test -p jcode-tui` and `cargo test -p jcode-base` covering the new success and failure scenarios.
- [x] Run `openspec validate bind-acceptance-evidence-to-runtime --strict`.

## Checkpoint evidence, 2026-11-03

- Base e5b0c7426, authorized first-wave worktree. Runtime helper tests: 5 passed, including unverified-cleanliness unknown, proven stale hash mismatch and safe Git filter/fsmonitor inspection. Protocol suite: 102 passed. Actual disconnect regression: 1 passed; action-first no-wrap version status regression: 1 passed.
- Integrated binary built through `scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode`. Isolated actual daemon public version JSON returned the actual serving hash and executable path, without provider inference or shared-daemon restart. Artifact: `/home/nyaptor/.jcode/scratch/first-wave-runtime-nfgrgpun/version.json`. This predates the reviewed cleanliness qualification and is not final-byte acceptance of that later patch.
- Final repaired binary build passed (`scratch/first-wave-build-final.log`). Actual isolated serving-daemon public CLI acceptance passed at `/home/nyaptor/.jcode/scratch/final-runtime-evidence-99vzngpa/version.json`: server/client hashes agree, executable path names the serving build, unverified server cleanliness is null, and known dirty source yields mismatched. No provider inference/shared runtime mutation occurred.
- Full base gate failed: 2015 passed, 7 failed, 8 ignored. Failures cluster around entitlement and configured provider/catalog assumptions. Full TUI gate failed: 2888 passed, 11 failed, 20 ignored. Selector and version regressions found in that gate are being repaired and rerun; auth/config/temporary-repository failures remain unresolved. Logs: `scratch/first-wave-base-full.log`, `scratch/first-wave-tui-full.log` under primary JCODE_HOME. No full-suite pass claimed.
- Independent review found stale cached build cleanliness, identity retained across disconnect, wrong-host source observation and mismatches hidden by unknown fields. Repairs now invalidate disconnect identity, keep SSH source unknown, evaluate proven mismatch first and explicitly mark cached build cleanliness unverified. Focused post-review helper and disconnect tests passed. Final repaired-diff review remains required.

## Remaining acceptance

## Immutable build execution, 2026-11-03

- [x] Implement authorized opt-in local immutable source compilation without Git/worktree/shared-daemon mutations.
- [x] Verify offline clean/edit/revert, source fingerprint, caller/config forgery, compiler-time mutation and unsafe input rejection.
- [x] Preserve mutable-wrapper identity, changelog and job-sizing regressions.
- [ ] Verify final-source actual Jcode immutable binary with isolated clean/edit/stale-daemon CLI acceptance.
- [ ] Complete independent review and required full gates before any feature completion or archive.

Test evidence: `python3 scripts/test_dev_cargo_cwd.py` passed 7 tests; `scripts/test_dev_cargo_jobs.sh` and `scripts/test_dev_cargo_changelog.sh` passed; `scripts/dev_cargo.sh test --offline -p jcode-build-meta` passed 4 tests. Logs under primary scratch: `immutable-wrapper-final.log`, `immutable-jobs.log`, `immutable-changelog.log`, `immutable-meta-tests.log`. Actual Jcode immutable builds passed before final attestation/PID hardening (`immutable-root-build.log`, `immutable-root-final.log`), so they do not establish final-byte acceptance. Final-source build and runtime evidence remain separate gates.

## Historical mutable-path gap

Real clean-build matching is not established. The dev wrapper now refreshes raw source fingerprints/dirty observations across same-HEAD edits without executing filters/fsmonitor, including hidden index flags. Its deterministic compiler-wrapper regression nevertheless proves an edit→compile→revert can produce a dirty executable while prebuild snapshot is clean. Therefore mutable snapshot provenance remains unverified; no fabricated `matched` result. Genuine clean attestation requires an immutable compiler-input build workflow or enforceable writer exclusion through compilation, a material workflow decision beyond the bounded wrapper change. Runtime capture separately rejects hidden assume-unchanged/skip-worktree cleanliness evidence (6 helper tests pass). No archive, merge, deployment or full feature completion is claimed. Final persistence of the safe partial implementation does not satisfy the missing requirement.
