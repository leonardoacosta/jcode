# Task and worktree policy implementation

## Binding and execution scope

Repository: `github.com/leonardoacosta/jcode`, worktree HEAD `e5b0c74262039cdd48a45b155b58d483aa5967d4`. Inspection began 2026-11-03 UTC. This worker implements only this change. User forbids Git/archive mutations, live TOML changes, activation, provider calls, host setup and user-owned worktree creation. Disposable Git fixtures are permitted. Shared files use disjoint coordinated hunks. Graph absent, direct search used.

Execution preflight (`preflight-check-v2-shadow`): local implementation/tests eligible; live activation, Git closeout and archive blocked by explicit scope. G1 2, G2 2, G3 1 (terminal operations excluded here), G4 2, G5 2, G6 1 (executor/restart prerequisites missing), 10/12 = 83.33 shadow score. Score grants no authority. Parent coordinator owns independent review and terminal closeout. No completion claim for an unavailable gate.

## Inventory before edits

- `crates/jcode-base/src/config/config_file.rs`: global `$JCODE_HOME/config.toml`, otherwise `~/.jcode/config.toml`. `load` uses defaults on parse failure; `load_strict` preserves errors. `apply_env_overrides` applies named existing environment variables after file settings. No task/worktree environment overrides exist or are added.
- No project TOML policy loader exists. Repository instructions/setup hints are not user configuration authority. The policy parser explicitly rejects project `[tasks]`, `[worktrees]` or `[setup]` declarations. Unsupported project declarations are not merged, including narrowing declarations. No executable config/hook semantics added.
- Exhaustive Rust search for `worktrees::`, `worktree add`, and Git command argument forms found only `crates/jcode-sdk/src/worktrees.rs` as allocator, with no production callers elsewhere in this checkout. Its SDK public API is the integration boundary.
- `crates/jcode-app-core/src/overnight.rs`: manual `start_overnight_run`, background `spawn_supervisor`/`run_supervisor`, and per-turn `run_turn_monitored`. Existing callers are TUI `/overnight` and server special commands. They carry explicit manual mission/session intent, not per-task persisted authorization. Manual behavior must remain compatible with unattended policy disabled.
- Existing `OvernightManifest` persists run/session/process ownership, cancellation and clock timestamps. It does not persist task authorization, spend reservation, dependency version or atomic attempt ownership. No automatic overnight restart entrypoint exists. Usage percentages/resource projections are not monetary budget enforcement.
- CLI dispatch runs after write-capable startup. A truly read-only command must bypass both startup initialization and dispatch migrations/provider bootstrap.

## Configuration contract

`TasksConfig` adds `[tasks]` with unattended disabled, max concurrency 1, max attempts 3, max runtime 3600 seconds and max spend 1,000,000 micro-USD. These are finite configuration ceilings, not spend authorization or new spend enforcement. All numeric ceilings must be positive. Unknown task/worktree fields are rejected to avoid silently ignored safety settings.

`WorktreesConfig` adds `[worktrees]` with root `~/worktrees`, creator unset, creation authority false and cleanup `manual`. Creator must be explicitly configured and match the trusted request. Only manual cleanup is supported. Existing global loading validates both policies. Malformed policy causes strict reads to fail, while legacy lenient loads fall back to disabled defaults.

`Config::policy_from_toml` reports each field's source as `default` or the exact global file label. No task/worktree environment layer or project widening layer is introduced. `jcode policy` reads this view as JSON before startup side effects. `--admission <JSON>` evaluates an explicitly labeled diagnostic snapshot only, never persists authority or starts work.

## Deterministic admission

A typed `TaskAdmissionSnapshot` records original bounded task scope/authorization, cancellation, dependency/workspace readiness, ownership, existing live attempt, aggregate concurrency and consumed attempts/runtime/spend. Unknown booleans default false. Original task bounds intersect current ceilings using `min`; consumed values subtract with saturation. Broader current config cannot enlarge the original budget. Scope mismatch requires a human. Cancellation/exhaustion blocks. Dependency/workspace/capacity/live-attempt conditions wait. Missing authority or ownership needs a human. Setup always needs separate authority. Optional expired deadline blocks, but there is no required clock window.

This evaluator is not a runtime reservation, filesystem-effects capability, provider spend enforcer or duplicate-attempt lock. A diagnostic `ready` result cannot dispatch by itself.

The existing `OvernightManifest` gains an optional serde-default snapshot. Legacy/manual manifests remain `None`. Native `revalidate_task_admission` additionally checks persisted cancellation/status, process ownership, existing workspace and elapsed runtime. The supervisor checks before provider usage preflight and before every future admitted turn. `ensure_task_dispatch` deliberately refuses snapshot-based dispatch because no authoritative attempt/spend reservation exists. Manual runs remain unchanged. No new ledger, scheduler, auto-resume or inferred authority is introduced.

## Worktree resolver and reuse

The old two-argument SDK `create` now refuses operations because it cannot establish policy or task authority. `create_with_policy` is the single allocator entrypoint. It requires task-level authority or explicit user policy, configured creator match, verified consumers, compatible live owner, literal branch, verified commit ID, repository identity and safe single-component task/unique identifiers.

Repository keys strip URL credentials/query/fragment, support HTTPS/SSH/scp-like origins, and use a deterministic canonical-common-Git-directory hash when origin is unavailable. Ownership metadata also binds the full canonical common Git directory, preventing local-key collision from granting reuse.

Destination: `<root>/<repo-key>/<creator>/<task>--<unique-id>`. All existing path ancestors must be real directories, not symlinks. Traversal is refused. Before creation, invoking HEAD must equal authorized base, branch must not exist, destination must be absent, and safe parent directories are created. Git uses `-b`, never `-B` or force. The allocator passes `-c core.hooksPath=` so checkout cannot execute repository hook commands without separate authority. Before every SDK Git operation, including status/reuse/list, a read-only effective `git config` probe refuses any configured filter driver or unverifiable config. This conservative refusal covers clean/smudge/process commands from `.gitattributes`; filter execution is not granted by creation authority. A disposable sentinel test verifies refusal for all three drivers. Every SDK Git invocation, including the configuration probe, also passes `-c core.fsmonitor=false` to prevent a repository-configured fsmonitor executable from running during status/reuse. A real disposable reuse sentinel failed before the override and passed after it. A failure leaves artifacts for manual inspection, never cleanup.

Ownership lives in the worktree Git administration directory, not tracked/untracked checkout content. Reuse requires matching repository/common directory, creator/task/unique ID/session owner, literal branch, exact base/HEAD freshness, verified consumers and a clean checkout. Locked, detached, bare, prunable, dirty, unowned or incompatible live-owner checkouts are preserved with conflicts. Ownership metadata symlinks are rejected.

Filesystem prechecks cannot provide a fully race-proof hostile-host sandbox around an external Git process. The user-controlled root must remain trusted during allocation. Existing symlink escapes are covered by real disposable Git fixtures. No shell/setup hooks or secret copying are run.

## Requirement mapping and remaining gaps

| Requirement | Implementation/check | State |
| --- | --- | --- |
| Old config/default disabled | config-types defaults and base TOML tests | Implemented |
| Typed validation/provenance | strict load, field-source report, invalid limits and unsupported project tests | Implemented |
| Explainable ready/waiting/blocked/needs-human | typed snapshot evaluator, no clock-window input, cancellation/dependency/budget/owner tests | Implemented contract |
| Current config cannot widen original budget/scope | min intersection, exact scope, native manifest revalidation | Implemented contract |
| Read-only effective policy/admission | CLI early return plus 7 process-level regression tests, including no-write fixture preservation | Verified |
| Actual atomic dispatch/restart and duplicate reservation | Existing manifest seam, fail-closed guard | Incomplete: no executor reservation/spend enforcement or restart entrypoint |
| Original filesystem effects cannot widen on restart | Scope match only; no new effects execution | Incomplete: executor capability/effects binding belongs with evidence-aware execution consumer |
| Global safe allocation | one SDK resolver, authority/creator/path/base checks | Implemented |
| Preserve dirty/live/unowned work and safe matching reuse | Git fixture tests, ownership metadata, no destructive cleanup | Implemented |
| Project cannot grant setup/publication/roots | project parser rejects unsupported layer, admission setup denial | Implemented, no project loader added |
| Review/commit/archive | Parent workflow, explicit no Git/archive mutations for this worker | Not performed |

## Verification evidence

Test-first missing-type/API errors observed for config-types, base config, SDK resolver and native manifest/revalidation. Adversarial ownership metadata symlink fixture failed before the no-symlink ownership check. Native dispatch guard missing-symbol test failed before guard extraction. Full command results and final counts are recorded in `tasks.md` after the final gates. No live providers or user-owned worktrees used.
