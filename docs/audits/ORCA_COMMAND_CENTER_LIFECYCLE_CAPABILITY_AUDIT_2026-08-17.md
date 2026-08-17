# Orca lifecycle capability audit for Jcode Command Center

Date: 2026-08-17 UTC  
Scope: Command Center start, retry, cancel, idempotency, project/worktree identity, receipts, reconnect, reconciliation, and failure attribution  
Installed Orca inspected: `1.4.176` at `/home/nyaptor/.local/opt/orca/1.4.176/resources/bin/orca-ide`  
Research tooling: Firecrawl CLI `1.19.27`, official Orca documentation, GitHub release metadata, public `stablyai/orca` repository evidence, and non-mutating installed-CLI probes

## Executive conclusion

The prior statement that installed Orca `1.4.176` exposes only `orchestration run-create` is no longer accurate for the current installed binary. Its machine-readable command registry exposes 223 commands, including a substantial supervised lifecycle surface:

- `run-create`, `run-use`, `run-current`, `run-list`, and `run-show`
- `task-create`, `task-list`, and `task-update`
- `worker-start`, `worker-show`, `worker-read`, `worker-stop`, `worker-abandon`, `worker-release`, `worker-retain`, and `worker-list`
- exact-attempt retry through `worker-start --retry-of <dispatch_id>`
- unknown-result recovery through `--retry-request <id>` on orchestration mutations
- FIFO mailbox replay through `check`, with explicit Delivery acknowledgement

This materially narrows the Orca-side gap, but it does **not** fully satisfy the current Command Center mutation contract. The installed surface is Dispatch-oriented, while the durable initiative requires project/context-scoped Run start, exact prior-Run retry, exact linked-Run cancellation, caller-correlated idempotency, stable versioned output schemas, and accepted plus terminal Run receipts. Orca `1.4.176` has useful primitives for a compatibility adapter, not the complete advertised contract.

A newer public release is proven to exist. GitHub marks `v1.4.184` as the latest public release, published 2026-08-17 00:14 UTC. This audit does not assume upgrading closes the Command Center gaps. The installed binary remained `1.4.176` throughout inspection, and no external repository or installed package was modified.

## OpenSpec contract evaluated

The comparison uses these repository requirements:

- [`optimize-orca-command-center-orchestration` specification](../../openspec/changes/optimize-orca-command-center-orchestration/specs/command-center-orca-orchestration/spec.md)
- [`optimize-orca-command-center-orchestration` design](../../openspec/changes/optimize-orca-command-center-orchestration/design.md)
- [`add-solidstart-command-center-vertical-slice` task 4.5](../../openspec/changes/add-solidstart-command-center-vertical-slice/tasks.md)

The decisive requirements are:

1. preserve canonical Orca project, Run, Task, Dispatch, worktree, terminal, correlation, and idempotency identities separately;
2. record a durable Jcode command envelope before mutation;
3. settle Jcode state only from verified correlated receipts;
4. recover after a crash without duplicating mutation;
5. advertise only version-verified capabilities;
6. start a project/context-scoped execution;
7. retry one exact prior attempt with distinct causality;
8. cancel one exact nonterminal linked execution without claiming success before terminal evidence;
9. retain partial cleanup and failure attribution without fabricating release.

## Evidence method

### Firecrawl collection

Firecrawl CLI `1.19.27` was authenticated through its stored credential and had 3,765 of 5,000 credits before research. Installed global, `search`, `scrape`, and `research search-github` help were inspected before use.

Firecrawl was used for:

- discovery of the canonical public repository and release surface;
- official documentation discovery for the Orca CLI and orchestration guide;
- indexed GitHub history for durable receipts, retry recovery, reconnect, worker lifecycle, and identity scoping;
- current release-page discovery.

Raw Firecrawl JSON and local Orca JSON were retained only in a mode-`0700` private scratch directory with mode-`0600` files. This report contains only scrubbed assertions and schema key names. Local repository IDs, worktree paths, Run IDs, Task IDs, Dispatch IDs, terminal handles, and message contents were not copied into the repository.

### Installed CLI inspection

No lifecycle mutation was executed. The installed behavior was inspected through:

- `orca --help`
- `orca agent-context --json`
- command-specific `--help`
- `orca skills get orchestration --full`
- read-only `repo list`, `project list`, `worktree current`, `run-list`, `run-show`, `run-current`, `worker-list`, `worker-show`, and `worker-read`
- one synthetic read-only unknown-Dispatch lookup to verify typed rejection shape

The installed executable path proves version `1.4.176`. `orca agent-context --json` itself reports command schema version `1` and command count `223`, but does not report the Orca application version.

## Source inventory

| Source | Date or version | Evidence used |
|---|---:|---|
| [Official Orca CLI overview](https://www.onorca.dev/docs/cli/overview) | accessed 2026-08-17, unversioned | CLI ships with the app; worktree-current and orchestration entry points |
| [Official Orca orchestration guide](https://www.onorca.dev/docs/cli/orchestration) | accessed 2026-08-17, unversioned | Run/Task/Dispatch model, retry-of, worker-stop/release, Delivery ack/replay, terminal outcomes |
| [Official changelog](https://www.onorca.dev/changelog) | accessed 2026-08-17 | `1.4.149` introduced the Run + worker-start loop; `1.4.176` added per-worker model and effort overrides |
| [Release `v1.4.176`](https://github.com/stablyai/orca/releases/tag/v1.4.176) | published 2026-08-07 07:06 UTC | installed release provenance and release notes |
| [Release `v1.4.184`](https://github.com/stablyai/orca/releases/tag/v1.4.184) | published 2026-08-17 00:14 UTC | proof that a newer public release exists; no compatibility inference |
| [PR #9925](https://github.com/stablyai/orca/pull/9925) | merged 2026-07-27 | durable Runs/inboxes, mutation receipts, exact Dispatch capabilities, stop/abandon, reconnect and duplicate suppression |
| [PR #11432](https://github.com/stablyai/orca/pull/11432) | repository evidence accessed 2026-08-17 | completed receipt retention, unresolved receipt preservation, ledger fail-closed behavior, Run pagination |
| [PR #14586](https://github.com/stablyai/orca/pull/14586) | merged 2026-08-15 | same-request replay after unknown results and `failedStage`/`residualResources` recovery guidance on current main |
| [PR #301](https://github.com/stablyai/orca/pull/301) | merged 2026-04-05 | `worktree current` and `active`/`current` worktree selector semantics |
| [PR #9449](https://github.com/stablyai/orca/pull/9449) | open when accessed 2026-08-17 | runtime-global listing defaults and explicit statement that orchestration Tasks do not persist stable repo/worktree ownership |
| [Issue #13005](https://github.com/stablyai/orca/issues/13005) | closed 2026-08-09; reproduced on `1.4.175` | low-level Dispatches lacked the capability identity required by stop/abandon; composed `worker-start` was the positive control |
| [Issue #13298](https://github.com/stablyai/orca/issues/13298) | open when accessed 2026-08-17 | current lifecycle-attribution risk around gate creation and active Dispatch completion |

Official docs are current and unversioned. They are evidence of the documented current product surface, not proof that every documented detail is implemented identically in `1.4.176`. Installed help and read-only probes are the authority for the installed command surface.

## Installed `1.4.176` surface

### Command registry and capability discovery

`orca agent-context --json` returned:

- command schema version: `1`
- command count: `223`
- per-command fields: command path, aliases, argument mode, summary, usage, flags, positional arguments, examples, and notes

It does **not** expose:

- application version;
- semantic capability versions;
- JSON response schemas;
- receipt schema versions;
- lifecycle transition schemas;
- a declared Command Center profile;
- a machine-readable statement that a multi-command composition is atomic.

The registry is enough to discover that a verb and flag exist. It is not enough to verify a stable output contract for a durable adapter.

### Start

Available composition:

1. `orchestration run-create --objective ...`
2. `orchestration task-create --spec ... --run <run>`
3. `orchestration worker-start --task <task> --worktree ... --agent ...`

Important installed semantics:

- a Run is a durable namespace and coordinator inbox;
- a Run never schedules or places workers;
- `worker-start` is the composed supervised start primitive;
- placement can be `current`, an exact existing worktree, `new-child`, or `new-top-level`;
- exact repository selection is available for new top-level worktrees;
- `worker-start` exits `0` only for `ready`;
- failed or `outcome_unknown` starts exit nonzero and include stage, failed stage when available, setup state, effects, residual resources, and recovery commands.

Gap: there is no single project/context-scoped `start_initiative_run` operation. `run-create` does not require or retain canonical project/worktree identity. Jcode would have to make a durable multi-step composition and reconcile partial completion across Run, Task, worktree, terminal, and Dispatch creation.

### Retry

Installed retry is attempt-oriented:

```text
orca orchestration worker-start --task <task_id> --retry-of <dispatch_id> \
  --worktree <explicit-placement> --agent <agent> --json
```

Documented and installed semantics:

- `--retry-of` links the replacement attempt;
- placement is intentionally not inherited;
- the caller must repeat server, worktree, and agent or terminal choices;
- a replacement gets a distinct Dispatch identity;
- Tasks circuit-break after repeated failures.

Gap: there is no `run-retry`, and a Run has no terminal outcome that can be retried as one unit. The current Command Center initiative asks for exact prior-Run retry. Orca offers exact prior-Dispatch retry. Adopting it requires an explicit contract change or a Jcode mapping that defines the linked run's retry target as one Dispatch attempt.

### Cancel and cleanup

Installed exact-target controls are:

- `worker-stop --dispatch <id>`: fence and stop one supervised agent terminal;
- `worker-abandon --dispatch <id>`: fence without claiming the process stopped and retain possibly-live resources;
- `worker-release --dispatch <id>`: post-completion cleanup of one settled worker terminal after archiving output;
- `worker-retain --dispatch <id>`: durable exception that keeps a settled terminal live;
- `worker-list`: separate terminal-resource accounting from Task state.

Truthful boundaries are explicit:

- stop does not delete the worktree, setup terminal, configured tabs, or unrelated processes;
- abandon performs no process or filesystem action;
- release is not cancellation;
- release can report retained, release-pending, release-unknown, already-released, or released states;
- release of unknown ownership must not be replaced with broad terminal closure.

Gap: there is no exact whole-Run cancel verb and no Run terminal cancellation receipt. `worker-stop` targets a Dispatch and may leave other Run resources live. Command Center's `cancel_linked_run` contract therefore cannot be implemented literally without either narrowing its semantics to exact Dispatch cancellation or composing and reconciling every resource owned by the linked Run.

### Idempotency and unknown outcomes

Orca has substantial internal idempotency support:

- orchestration mutations advertise `--retry-request`;
- installed help says it is for exact recovery after an unknown mutation result;
- public repository evidence describes durable mutation receipts, in-flight joining, completed receipt replay, and explicit unknown outcomes;
- completed receipts are retained for 30 days or bounded to 10,000 rows;
- unresolved receipts are not evicted;
- if unresolved receipts fill the ledger, new mutations fail closed rather than discarding safety evidence;
- public current-main evidence shows same-request recovery after restart without recreating a worker.

Gaps for Jcode:

1. The documented first call does not accept a separate caller-owned Command Center idempotency key. `--retry-request` is described as recovery using the Orca request ID supplied after an unknown result.
2. If Jcode crashes after starting the CLI but before durably storing the returned Orca request ID, the Command Center envelope alone cannot query the Orca receipt ledger by its own key.
3. The command registry does not publish receipt request/response schemas.
4. The Run/Task/worker composition uses several mutations, each with its own acceptance boundary. There is no advertised transaction spanning the composition.

Orca's ledger is a strong component of a bridge. It does not replace the Jcode durable command-envelope and receipt store required by OpenSpec.

### Canonical project and worktree identity

Installed read-only JSON exposes strong identity primitives:

- `repo list`: registered repository ID, path, display name, kind, upstream, and remote identity;
- `project list`: durable project ID, source repository IDs, provider/remote identity, and timestamps;
- `worktree current`: exact worktree ID, repo ID, host ID, instance ID, canonical path, branch, Git state, and lineage;
- `worker-show`: Run ID, Task ID, Dispatch ID, worker worktree ID, agent terminal handle, process/runtime identity, and observation status.

The current working directory resolved successfully to one managed worktree without printing its local values into this report.

Gaps:

- installed Run rows contain Run ID, objective, coordinator identity, home database, timestamps, and legacy marker, but no project, repo, or worktree field;
- the public repository's still-open PR #9449 states that orchestration Task rows do not persist stable repo or worktree ownership;
- several listing commands default to runtime-global scope;
- remote `current` and `new-child` are intentionally invalid because they are ambiguous across servers.

Command Center can preserve canonical identity in its own envelope and verify worker worktree identity after start, but Orca does not currently enforce project/worktree scope at the Run record itself.

### Accepted, rejected, and terminal receipts

Installed JSON uses a consistent top-level transport envelope:

- accepted/read result: `ok: true` with `result`;
- rejected read: `ok: false` with typed `error.code` and `error.message`;
- a synthetic unknown-Dispatch lookup returned nonzero with `dispatch_not_found`.

Lifecycle evidence is richer below that wrapper:

- start acceptance: worker state `ready` plus created/reused effects and setup receipt;
- start rejection or uncertainty: failed or `outcome_unknown` plus stage and residual resources;
- terminal worker report: `worker_done` requires exact Task and Dispatch IDs and outcome `succeeded` or `failed`;
- active Dispatch capability and runtime-attested sender identity are completion authority;
- `worker-show` exposes Dispatch status, failure count, last failure, heartbeat, capability state, worker stage, effects, last error, residual resources, worktree ID, and observation status;
- release has explicit terminal-resource states independent from Task status.

Gaps:

- Run itself has no accepted-to-terminal lifecycle state;
- no versioned output schema is published through `agent-context`;
- no single receipt correlates the full Run/Task/worktree/terminal/Dispatch composition;
- Jcode cannot settle `cancel_linked_run` from a Run terminal cancellation receipt because that receipt type does not exist.

### Reconnect and mailbox replay

Available reconnect primitives are meaningful:

- `run-use` binds a coordinator terminal to an existing Run;
- takeover preserves existing worker assignments;
- Runs and inboxes are durable;
- `check` returns the oldest unacknowledged FIFO Delivery;
- the same Delivery is replayed until explicitly acknowledged;
- `peek` and `all` do not consume mail;
- `run-list` uses stable cursor pagination;
- public PR #9925 records disconnect/reconnect and duplicate-suppression acceptance.

A read-only `run-current` from this Jcode shell returned typed `no_active_sender_terminal`, showing that binding is terminal-context scoped rather than silently inferred.

Gaps relative to Command Center replay:

- Orca replay is scoped by Run/mailbox consumer, not by Jcode authenticated principal and initiative;
- there is no Command Center event sequence cursor or authorized snapshot contract;
- there is no cross-system atomic acknowledgement that both Orca Delivery processing and Jcode durable settlement completed;
- Jcode must retain its own authorization-scoped ordered evidence and snapshot fallback.

### Reconciliation and failure attribution

The installed and documented surface supports useful reconciliation:

- `run-list` and `run-show` recover durable Run records;
- `task-list` and `dispatch-show` recover Task/Dispatch context;
- `worker-show` combines Dispatch, worker, terminal, terminal-resource, and observation state;
- `worker-list` accounts for active, reclaimable, retained, release-pending, release-unknown, and released resources;
- worker starts expose partial effects and residual resources;
- unknown starts are not automatically retried;
- explicit stop or abandon resolves an uncertain Dispatch without claiming more than observed;
- failed worker completion is a terminal outcome, not prose-only status.

Known public risks should remain visible:

- issue #13005 showed that low-level `dispatch` rows without worker capability identity could not use stop/abandon, while `worker-start` Dispatches worked. It was closed after the installed `1.4.176` release date, so a Command Center adapter should use the composed `worker-start` path and test the exact installed build.
- open issue #13298 reports that `gate-create` can complete an unrelated active Dispatch and revoke its capability. Command Center should not treat gate and worker lifecycle composition as safe without an installed-version acceptance test.

## Requirement comparison

| Command Center requirement | Installed `1.4.176` evidence | Status | Exact gap |
|---|---|---|---|
| Project/context-scoped start | Run + Task + `worker-start`; exact repo/worktree placement is available | Partial | Run creation neither requires nor stores project/worktree identity; composition is not atomic |
| Exact retry | `worker-start --retry-of <dispatch>` creates a linked replacement attempt | Partial | Retry targets Dispatch, not prior Run; explicit placement must be reconstructed |
| Exact cancel | `worker-stop` and `worker-abandon` target one Dispatch | Partial | No whole-Run cancel or Run terminal cancellation receipt |
| Caller idempotency | durable Orca mutation receipts and `--retry-request` unknown-result replay | Partial | no documented caller-owned first-attempt idempotency key or Jcode-key receipt lookup |
| Crash-safe recovery | durable Runs, inboxes, mutation receipts, request replay, read-side reconciliation | Partial | Jcode still lacks durable per-step envelope/receipt persistence and can crash before capturing Orca request identity |
| Canonical identity | repo/project/worktree IDs and worker worktree ID are available | Partial to strong | Run and Task records are not stably project/worktree scoped |
| Accepted/rejected receipts | `ok/result`, typed errors, ready/failed/outcome-unknown worker receipt | Partial to strong | output schema is not versioned in `agent-context`; no composition-level accepted receipt |
| Terminal receipt | exact `worker_done` succeeded/failed and resource release states | Partial | terminal state exists for Dispatch/Task/worker resources, not Run cancellation or whole execution |
| Reconnect | `run-use`, durable Run/mailbox, FIFO Delivery replay until ack | Strong primitive | no Jcode principal/initiative authorization scope or cross-system settlement ack |
| Reconciliation | run/task/dispatch/worker observation plus residual-resource evidence | Strong primitive | no query by Jcode correlation/idempotency key and no whole-Run cleanup receipt |
| Failure attribution | stage, failed stage, effects, residual resources, last error/failure, recovery commands | Strong primitive | current open lifecycle defect risk and no schema-version guarantee |
| Runtime capability discovery | 223-command registry with flags and notes | Partial | no app version, semantic capability version, response schema, or Command Center capability profile |

## Why task 4.5 remains blocked

The blocker is narrower than previously recorded but still real.

It is **not** accurate to say Orca `1.4.176` lacks retry or cancel mechanics entirely. It has exact Dispatch retry, stop, abandon, release, durable receipt replay, and strong worker failure evidence.

Task 4.5 remains blocked under its current wording because the Jcode public commands are Run-oriented and require a versioned adapter contract that the installed registry does not publish:

- `start_initiative_run` expects one project/context-scoped accepted operation;
- `retry_linked_run` expects an exact prior linked execution target, currently described as a Run;
- `cancel_linked_run` expects exact nonterminal Run cancellation and terminal proof;
- capability negotiation must be safe from schema drift;
- Jcode must persist crash-safe correlation and per-step receipts.

Implementing the adapter by guessing that Run equals Dispatch would violate the existing fail-closed design. Implementing it as a multi-step composition without durable per-step reconciliation would violate the crash-safety requirement.

## Concrete unblock options

### Option A: approve a Dispatch-oriented compatibility contract for `1.4.176`

Change the Command Center contract explicitly:

- `start_initiative_run` means create a Jcode-correlated Orca Run, Task, and one initial Dispatch;
- `retry_linked_run` means retry one exact prior Dispatch using `--retry-of` with fully explicit placement;
- `cancel_linked_run` means stop one exact active Dispatch, not cancel a Run namespace;
- a Run remains a grouping namespace and never carries terminal outcome authority.

Required Jcode work:

1. add a durable multi-step operation store before invoking Orca;
2. persist each Orca request ID, Run ID, Task ID, Dispatch ID, worktree ID, terminal handle, and receipt;
3. reconcile after every process restart with `run-show`, `task-list`, `dispatch-show`, and `worker-show` before replaying a mutation;
4. pin a tested `1.4.176` response fixture set and reject any unrecognized schema;
5. run isolated live acceptance for ready, rejected, outcome-unknown, retry, stop, abandon, release-pending, and release-unknown paths;
6. prohibit low-level `dispatch` for supervised workers because its control capability differs from `worker-start`.

Trade-off: no Orca release dependency, but Jcode assumes responsibility for a compatibility profile not formally published by Orca.

### Option B: request a minimal upstream capability and receipt profile, not necessarily new verbs

Orca could expose a machine-readable profile containing:

- Orca application version and orchestration schema version;
- input and output schema identifiers for Run, Task, Dispatch, worker start, stop, abandon, release, and mailbox Delivery;
- explicit statement that `worker-start --retry-of` is the supported attempt retry contract;
- caller-supplied first-attempt idempotency key or a preflight request-ID reservation;
- receipt lookup by request ID;
- declared project/worktree identity fields and transition authority;
- stable accepted, rejected, unknown, and terminal state enums.

Jcode could then compose existing primitives without inventing a new Run scheduler. This is the smallest upstream change that preserves fail-closed capability negotiation.

### Option C: request the original Run-oriented upstream contract

Add dedicated operations for:

- project/context-scoped Run start;
- exact prior-Run retry with distinct attempt identity;
- exact Run cancellation with nonterminal precondition and terminal receipt;
- whole-operation accepted and terminal receipts;
- correlation and idempotency keys supplied before the first mutation;
- resource-by-resource cleanup evidence.

This best matches the current initiative wording but duplicates some existing Dispatch mechanics and has the largest upstream scope.

### Option D: evaluate `1.4.184` in isolation before choosing A, B, or C

A newer public release is available, but upgrade is not itself an unblock. In a disposable profile:

1. capture `agent-context --json` and command-specific help;
2. compare command and flag fingerprints with `1.4.176`;
3. execute isolated mutation acceptance and record exact response schemas;
4. verify whether PR #14586's retry guidance and PR #14105's settlement fixes are in the binary;
5. rerun the known gate and low-level Dispatch failure cases;
6. make no Jcode contract change until evidence shows which gaps actually closed.

### Recommended decision

Choose Option B if Orca maintainers can provide a small versioned schema profile quickly. Choose Option A only if Jcode maintainers explicitly accept Dispatch-oriented semantics and the additional durable composition store. Keep the current fail-closed adapter otherwise.

Decision recorded 2026-08-17: Jcode maintainers selected Option A. The authoritative
Command Center design and protocol now define the installed `1.4.176` compatibility profile
as Run-grouping plus exact-Dispatch attempt semantics. This decision removes the semantic
choice blocker. It does not enable mutations by itself: the adapter remains fail closed until
the durable composition store, pinned schema fixtures, recovery logic, and isolated lifecycle
acceptance listed in task 4.5 are complete.

Do not keep the initiative blocked merely on “a version newer than `1.4.176`.” Version number alone is not the missing contract, and `1.4.184` already exists. The blocker should be stated as a missing verified semantic and receipt profile, or an unapproved Dispatch-oriented compatibility mapping.

## Evidence validation and retention

Validation performed on 2026-08-17:

- Firecrawl searches returned the canonical `stablyai/orca` repository, official documentation, release surface, and cited repository records.
- Official documentation URLs were fetched successfully.
- GitHub release metadata proved `v1.4.176` publication and `v1.4.184` latest-release status.
- GitHub API metadata confirmed PR merge/open states used above.
- Installed `agent-context` JSON parsed successfully and reported schema `1`, count `223`.
- Every installed read-only JSON probe parsed successfully except `worker-read` for a settled record whose archived output was unavailable; that typed nonzero result was not used to infer lifecycle support.
- Repository-local citation paths exist.
- No external repository, Orca installation, Orca runtime state, or managed worktree was modified.

The private scratch evidence is transient and is deleted after report validation. This committed report is the durable, scrubbed evidence artifact.
