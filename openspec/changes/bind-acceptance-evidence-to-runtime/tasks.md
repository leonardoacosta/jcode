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

Real clean-build matching is not established: cached build metadata can lag source changes, so the production cleanliness assertion is unavailable. The helper supports authoritative observations but production must not claim matched until final artifact/source cleanliness is actually attested. Client dirty is still a cached build observation, not final-byte attestation. No archive, merge, deployment or full feature completion is claimed. Final fresh gates, independent re-review, scoped commit and required persistence remain open.
