# Tasks

- [x] Add `[jev.supervision]` config fields.
- [x] Add resolver returning a supervision client or `NotConfigured`; make supervision selector paths non-panicking.
- [x] Show supervision route status in provider status output.
- [x] Tests: configured route; missing config; `auto` provider; no cross-purpose fallback.
- [x] Run `cargo test -p jcode-base jev`.
- [x] Run `openspec validate route-jev-supervision-explicitly --strict`.

## Execution evidence (2026-11-04)

Implementation base: `6393b322b481453c058c504cec087d982abc74ea` in the existing `main` checkout. Scoped edits: `crates/jcode-base/src/{config.rs,config_tests.rs,jev.rs}`, `src/cli/commands/report_info.rs`, and this change's `design.md`/`tasks.md`. Existing unrelated work and coordinator-owned proposal edits remain untouched. `JevClient::for_supervision` and `JevShadowEvaluator` are preserved. No inference, shadow admission, daemon activation, user config/credential changes, Git mutation, commit, or archive performed.

### RED → GREEN

All Cargo commands used `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh` through coordinated `selfdev test`.

- RED `test -p jcode-base supervision_selector_refuses_other_purposes_without_panicking -- --exact jev::tests::supervision_selector_refuses_other_purposes_without_panicking`: task `358343m6mf`, exit 101, expected panic at supervision's `unreachable!` selector arm.
- Selector GREEN assertion passed 1/1 in task `425625ryiy`, but the wrapper returned exit 97 because other integration binaries matched zero tests. This attempt is not a passing gate; subsequent `--lib` runs remove the unrelated binaries.
- RED `test -p jcode-base --lib jev_supervision_config_preserves_explicit_values_without_defaults`: task `474344uesb`, exit 101, expected missing `Config.jev` compile errors.
- GREEN `test -p jcode-base --lib supervision`: task `54680822zg`, exit 0, 3 passed.
- RED `test -p jcode-base --lib supervision_config`: task `635818lf45`, exit 101, expected missing `NotConfigured` and `from_supervision_config` compile errors.
- GREEN `test -p jcode-base --lib supervision`: task `6714332g3u`, exit 0, 5 passed.
- An initial root `--bin jcode` status-test attempt (`702878io9r`) matched zero tests and returned wrapper exit 97. CLI implementation tests belong to the root library.
- RED `test -p jcode --lib auth_status_supervision_route_is_local_explicit_and_secret_free`: task `7564967g79`, exit 101, expected missing status field/formatter compile errors.
- GREEN `test -p jcode --lib cli::commands::report_info::tests -- --test-threads=1`: task `801221dq6g`, exit 0, 5 passed. Includes absent/valid/auto/unknown routes, plain and JSON serialization, secret-free reasons, terminal-control sanitization, and existing auth lifecycle regression.

### Final local gates

- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-base --lib jev`: task `827720rnwf`, first command passed, 74 passed, 2 live tests ignored. `--lib` avoids the wrapper's zero-test integration-binary guard.
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test -p jcode-base --lib config::`: same task, second command passed, 110 passed.
- `rustfmt --check --edition 2024 --config skip_children=true,reorder_modules=false crates/jcode-base/src/jev.rs crates/jcode-base/src/config.rs crates/jcode-base/src/config_tests.rs src/cli/commands/report_info.rs`: exit 0.
- Scoped `git diff --check`: exit 0.
- `openspec validate route-jev-supervision-explicitly --strict`: exit 0, change valid.
- Combined `clippy -p jcode-base -p jcode --all-targets -- -D warnings` in task `827720rnwf` failed with 10 existing `jcode-app-core` errors outside assigned scope: nested-if lints in interrupts/claims, dead `format_tool_summary`, argument-count lints in lifecycle/comm_session, and redundant borrow in council. No unrelated repair made. Root/base strict combined lint is not claimed passing.
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy -p jcode-base --all-targets -- -D warnings`: task `888105spec`, first command passed. Wrapper/launcher inspection found no lint severity cap.
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh check -p jcode --all-targets`: same task, second command passed, confirming root/exhaustive-config integration. Existing unrelated dead-code/profile warnings remain.
- `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh clippy -p jcode --all-targets --no-deps -- -D warnings`: same task, third command failed on existing `src/cli/debug.rs:30` `items_after_test_module`, outside assigned scope. No owned-file lint reported; root strict lint remains blocked.

## Coordinator final acceptance and review

2026-11-04 UTC: candidate build `962903dm8n` passed without reload or activation.
Actual `target/selfdev/jcode` SHA-256:
`684e149db540e0b08a41e3181e92bac1befbe57493d7903239db7978dea3c811`.
`python3 scratch/p03-route-public.py` passed eight real CLI cases, JSON and plain,
under network-disabled `bwrap` with isolated HOME/data and synthetic keys.
Evidence: `scratch/p03-route-public-lhd_cnoh/results.json`. Cases cover absent,
valid, auto, missing/blank model, invalid provider, selected-key absence with another
provider key present, and control text. Health and entitlement remain `not_checked`.
Earlier build `8966823xua` refused source drift and is not a successful candidate.

Final combined gate `034999xlva` passed policy 78, serial TUI continuation 23,
serial auto-poke 32 and protocol 107 tests. Full native strict validation passed
69/69 (`scratch/p03-wave-openspec-validation.json`). Owned-source rustfmt, Python
19 regressions, strict route validation and whitespace check passed. Unrelated
full formatting and root strict lint failures remain documented above.

Independent reviewer `session_frog_1793813018959_274358bb374dc8f2` returned
`APPROVE` for the complete routing wave, exact tracked diff SHA-256
`33c3fb730f3a6db887086436353030e5b52b8c8422cf01ea0bcd138fdaea1d92`, base
`6393b322b481453c058c504cec087d982abc74ea`, plus both included designs.
No actionable findings. Evaluation remains partial and shadow implementation is
not accepted by this routing approval. Commit/archive/persistence follow this
verified prearchive state and are not inferred from checked tasks.

## Coordinator closeout still required

These checkboxes represent verified task implementation, not full proposal completion. Independent review, public candidate-binary acceptance, final commits, and archival remain coordinator-owned gates. Whole-repository formatting has separately reported unrelated debt; only scoped formatting passed here.

Public acceptance recipe, not yet executed by this worker: build a candidate without installing/reloading it; use an isolated temporary `HOME`, `JCODE_HOME`, and `XDG_CONFIG_HOME` with synthetic provider credentials only. Run candidate `jcode auth status` and `jcode auth status --json` with absent, explicit OpenRouter/model, `auto`, blank/missing model, unknown provider, and selected-provider-missing-credential configs. Assert the separate plain `jev.supervision` row and JSON `jev_supervision` object, expected `configured`/`not_configured`, static refusal reasons, no secret values, and health/entitlement both `not_checked`. No inference request or shared daemon is needed.
