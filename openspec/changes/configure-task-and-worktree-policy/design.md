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

The existing `OvernightManifest` gains optional serde-default snapshot and `OvernightTaskAttempt` ownership/decision records. Legacy/manual manifests remain `None`. Native `revalidate_task_admission` additionally checks persisted cancellation/status, process ownership, existing workspace and elapsed runtime. The supervisor checks before provider usage preflight and before every future admitted turn. Manual runs remain unchanged.

Explicit `start_task_attempt` and `restart_task_attempt` APIs now require the exact existing trusted overnight runtime path and matching safe run identity, reject manifest/run/candidate symlinks, reread strict current global policy and intersect requested/original ceilings. Stable OS-lock sidecars serialize capacity observation and manifest transactions; a held per-run attempt lock supplies live ownership across processes, not a PID. Claims increment a nonrefundable attempt and persist ownership, canonical workspace, counters and the exact blocked dispatch decision in the existing manifest. Restart never scans to activate work, preserves original authorization/counters and refuses duplicate live owners, cancellation, scope/workspace drift, exhausted limits and unknown legacy authority. All existing manifest save callers share a monotonic locked merge, so stale saves cannot refund counters, change original authority/workspace/runtime origin or clear cancellation. Lock sidecars carry no planning ledger. Every authority/accounting manifest read uses a dedicated nonrecovering `std::fs::read` plus JSON parser, including validation, locked current/merge, capacity scan, supervisor load/latest and cancellation. Corrupt primary state never restores an older `.bak`, refunds attempts or clears cancellation. Real corrupt-primary/current-N and corrupt-candidate fixtures verify errors with primary and backup bytes unchanged. Generic storage recovery behavior is not changed.

Positive provider dispatch still fails closed with `budget-unverifiable: provider lacks authoritative pre-send monetary bound`. Existing `ProviderCallBudget` enforces completion/retry counts, not monetary reservations. Session token totals are post-request. Pricing tables and per-credential local spend are estimates or aggregate after-the-fact values, not a per-task pre-send billing upper bound. No provider/session effect runs through the new APIs. Resolving this precise external dependency requires a provider transport reservation/upper-bound contract covering retries and side calls, not more authorization or live configuration activation. No token ceilings or adjacent cancellation executor are introduced.

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
| Atomic start/restart and duplicate reservation | Stable global/manifest/attempt OS locks, original bounds, public trusted-root APIs, process/race fixtures | Implemented fail-closed native admission; positive dispatch blocked on monetary transport contract |
| Original filesystem effects cannot widen on restart | Exact original scope/authorization/workspace retained; no effects executed while monetary bound unknown | Native guard implemented, positive effects executor remains blocked |
| Global safe allocation | one SDK resolver, authority/creator/path/base checks | Implemented |
| Preserve dirty/live/unowned work and safe matching reuse | Git fixture tests, ownership metadata, no destructive cleanup | Implemented |
| Project cannot grant setup/publication/roots | project parser rejects unsupported layer, admission setup denial | Implemented, no project loader added |
| Review/commit/archive | Parent workflow, explicit no Git/archive mutations for this worker | Not performed |

## Verification evidence

Test-first missing-type/API errors observed for config-types, base config, SDK resolver and native manifest/revalidation. Adversarial ownership metadata symlink fixture failed before the no-symlink ownership check. Native dispatch guard missing-symbol test failed before guard extraction. Full command results and final counts are recorded in `tasks.md` after the final gates. No live providers or user-owned worktrees used.

## Authorized provider monetary expansion (2026-11-03 UTC)

The user approved expanding the provider contract, without live config changes, paid inference, Git/archive/publication, new access, or deployment. This replaces the earlier missing-interface dependency, not the unsupported-vendor finding. Exact file ownership was agreed with the coordinator before code: provider-core `lib.rs`, `provider_call_budget.rs`, new `provider_monetary_budget.rs`, `Cargo.toml` and its one-line lock dependency; OpenRouter `openrouter_provider_impl.rs` and `openrouter_wire_tests.rs`. The coordinator owns native `overnight.rs`/`overnight/admission.rs`, proposal/spec changes, and final native evidence. This design/tasks update belongs to the provider worker. No separate proposal or execution ledger is introduced.

### Monetary authority and persistence

`Provider::monetary_capability()` defaults to `None`. Pricing tables, user config, credential labels, model prose, and post-response usage cannot construct an authoritative capability. No existing live provider opts in. A capability has private fields, no deserializer, and binds the exact account/task/original bound and immutable route/model plus request/response byte and per-send charge ceilings. Only the explicit `OfflineBoundedProvider` can mint the current capability. It is not registered as a selectable provider, has no endpoint or credential, and provides synthetic accounting, not a real invoice or upstream guarantee. A future paid adapter requires its own audited transport guarantee and code change.

`MonetaryBudget::create/open` require the exact task ID and original integer micro-USD cap. The account lives under the existing trusted run namespace. The strict append-only journal records the original binding, monotonically narrowing total ceilings, nonrefundable reservations, and settlements. A stable locked journal inode serializes threads/processes; no-follow opens and ancestor checks reject symlink aliases, journal hard links reject, malformed/partial/overflow/duplicate settlement state fails closed, and reservation writes sync before provider execution. No backup recovery or refunds exist. Dropped/cancelled/crashed/failed operations retain the full reservation. A settlement cannot exceed its individual reservation. Admission counts all reservations, including unsettled ones, not only known actual spend. Narrowing is durable and reopening cannot restore a larger allowance.

Every synthetic send checks a 65,536-byte serialized request ceiling, reserves one synthetic micro-USD, produces a fixed response below 128 bytes, and settles exactly one synthetic unit. The adapter permanently owns the account, including `fork`, so raw task spawns cannot erase it. Explicit child processes reopen the same exact path/task/original binding. Nested monetary scope cannot substitute another account. The existing `provider_call_budget::inherit/spawn` also carries monetary context to tool/provider workers. The existing council count permit is still consumed for synthetic requests, with no additional public count bypass. Existing raw transport hooks (`record_call`, `reject_unmetered`) deny monetary-scoped unsupported routes before sends. OpenRouter also denies at provider entry, before auth refresh/request preparation, because its generic retry heuristics otherwise delay a permanent refusal. No generic retry policy is changed.

### Acceptance mapping

| Requirement | Concrete check | Result |
| --- | --- | --- |
| Enforceable pre-send bound, not estimated price | Explicit offline Provider completes, reserves and settles before exposing stream | Passed, synthetic only |
| Original task/cap binding and durable narrowing | Reopen, wrong task/original cap, attempted recreate, narrowed restart tests | Passed |
| Retry/compaction/child aggregate | Foreground, forked compaction, raw spawned retry, later follow-up exhaustion; nested account substitution rejected | Passed |
| Concurrent processes cannot overspend | Six real child processes reopen one account, exactly eight reserved/actual units | Passed |
| Crash/unknown settlement cannot refund | Aborted real task, dropped reservation, invalid over-settlement, reopened exhausted account | Passed |
| Corrupt accounting cannot recover/refund | Partial/invalid/overflow journals and symlink aliases reject with bytes unchanged | Passed |
| Unknown actual API route rejects before HTTP | Real OpenRouter `complete` against loopback listener, zero accepted TCP connections | Passed |
| Actual shared transport hook rejects | Real reqwest request through shared timeout helper, zero accepted TCP connections | Passed |
| Existing call-budget behavior preserved | Full provider-core suite and actual HTTP retry exhaustion regression | Passed |
| Native original-authority integration | Coordinator-owned `task_monetary_account`, bounded start/restart, consuming `complete_with_provider` | Coordinator validates final native diff |

The positive offline fixture proves the concrete contract and bounded native operation, not paid-vendor execution or an unattended arbitrary-tools executor. Unsupported live routes remain denied. No launch/default/config changes follow from these tests. Required review and terminal closeout remain outside this worker's authority.
