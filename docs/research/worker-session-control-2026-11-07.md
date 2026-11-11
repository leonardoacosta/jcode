# Worker/session control: stop must precede ownership transfer

2026-11-07 · Explore with `improve self`, standard depth · research only.

## Recommendation

Extend the existing daemon's session-control machinery into one durable control authority. Do not introduce a second global manager or a separate process yet. The immediate problem is not absent discovery: stop, execution cancellation, membership recovery, report freshness and file ownership have different authority boundaries.

Route the lifecycle work to Feature under the candidate name `make-worker-stop-durable`. This is a recommendation, not a proposal, approval or implementation contract. Bind assignment/report freshness to the existing `add-swarm-handoff-contracts` change rather than duplicate that outcome. A focused isolated reproduction should be the first evidence gate before committing to the implementation design.

The smallest coherent outcome is: an authorized stop prevents new execution, cancels the current execution, reports whether it actually settled, and cannot be undone by automatic membership registration. Quiescence means the tracked execution and its owned tools have settled, not merely disappeared from membership. Fencing means rejecting operations from a revoked execution. Ownership transfer must wait for settlement. A successful message delivery is not that result.

## Scope and evidence limits

- Repository: `/home/nyaptor/.jcode`, origin `leonardoacosta/jcode`, Rust workspace, TUI/CLI focus. Source HEAD: `fd25ee0c57d71d6119bb7372a9cf7584f150d24e`. Existing tracked source was clean at inspection. Existing untracked application/runtime files and proposals were preserved.
- This run inspected source, graph, existing OpenSpec artifacts, four named incident session snapshots and narrowly filtered daemon-log metadata. It did not inspect or change the original application's source, run its tests, access its database, or resume its workers.
- Incident window: `2026-11-07T00:55:00Z` inclusive through `2026-11-07T01:40:00Z` exclusive. Private session identities are in the supplied `worker-session-management-handoff.md`, lines 38–49. Raw transcripts, arguments, result bodies and private application content are not copied here.
- The native session search returned no match with the bounded working-directory filter. Exact local files existed. The installed forensic extractor supports other transcript shapes, not Jcode JSON snapshots. A bounded read-only query inspected top-level keys, message timestamps, tool call/result IDs and fixed result markers in those four files. This is targeted incident evidence, not a corpus census.
- Observed daemon: PID `599523`, executable `/home/nyaptor/.jcode/builds/versions/6393b322b-dirty-080a894a4d65/jcode`, main socket `/run/user/1000/jcode.sock`. `/proc/599523/exe` agrees with `builds/shared-server/jcode`. `builds/current/jcode` instead resolves to `358294f93-dirty-a87bb1ee5155/jcode`.
- Relevant committed stop, wake, lightweight registration, live-turn, claim, wait, tree and lifecycle files have identical content at HEAD and commit `6393b322b`. This narrows revision drift but does not recover the dirty build's exact source tree or prove the historical binary did not change during the incident.
- Debug `state` inspection was refused: `Debug control is disabled.` No setting was enabled. No fresh stop/cancel reproduction, provider invocation, daemon restart, new worker, build or Rust test run occurred. Findings below distinguish observed incident results from source-level failure paths.
- The daemon still holds `logs/jcode-2026-11-04.log` open. The date-named November 7 log is not its current sink. Logging uses local wall time, whereas session timestamps are UTC. A bounded metadata scan found related request rows, but did not establish correlated per-worker cancellation/quiescence events. Absence is not proof that no cancellation occurred.

## Primary incident observations

All times in this table come from session message timestamps, in UTC.

| Observation | Evidence | What it proves and does not prove |
| --- | --- | --- |
| Rose resume queued repeatedly | Resume results at 00:55:01, 00:58:20, 00:59:29, 01:01:22 and 01:02:39 contain `Status: queued`. | Requests were accepted into a queue path. Not proof of a new work turn. |
| Repeated wait returned identical output | Rose-only waits at 00:58:45 and 01:01:01 returned `All 1 members are done` and `ready`. Result SHA256 matched exactly. | Identical returned report/state, not proof that both waits observed a new assignment. |
| Rose stop returned success | Request 01:06:43.949698192, result 01:06:44.018439273 contains `Stopped agent`. | Tool-level success, not quiescence. |
| Later mutation calls continued in Rose's exact record | 34 direct `apply_patch`/`edit` attempts after that stop, all with matching results, through 01:16:06. | These are later attempts, not merely late delivery of pre-stop tool results. Not all attempts succeeded. |
| At least 10 later edits succeeded according to tool output | Ten results match the edit implementation's `Edited ...: replaced ... occurrence(s)` success format. First 01:12:05.068456540, last 01:15:07.572802881. | Stronger than progress messages: recorded successful file-edit operations. It does not attribute every later application diff or freeze violation to Rose. |
| Cross-swarm messaging worked, stop lookup failed | Cross-swarm messages at 01:26:20 and 01:26:41 returned cross-swarm delivery markers. Directory result at 01:26:27 included `coordinator: none`. Exact-ID forced stop at 01:26:50 returned `Unknown swarm session`. | Confirms the discovery/control mismatch at the tool surface. Delivery is not target acknowledgment. |
| Evergreen stop returned success | Result at 01:25:11.801431159 contains `Stopped agent`. No later direct mutation attempt appears in the selected Evergreen record/window. | Do not claim Evergreen continued writing after stop from this sample. Nested tool work, shell children and other records are not exhaustively reconstructed. |

The sampled Rose snapshot later reports `Active`; Evergreen reports `Closed`. These are persisted snapshot values, not observations that either worker is live now. Do not act on them as current execution state.

## Source findings

### 1. Stop tears down membership without proving execution stopped

**Confirmed source behavior, high confidence.** `crates/jcode-app-core/src/server/comm_session.rs:1077–1265` resolves/authorizes the target, fans out `SessionCloseRequested`, removes the agent map entry, attempts `try_lock`, marks a persisted copy closed if busy, removes swarm membership, then returns a mutation `Done` response. The handler does not call the active-turn cancellation registry, retain an execution join handle, or await quiescence.

`remove_session_entry` at `crates/jcode-app-core/src/server.rs:177–186` only removes a map entry and unregisters an active PID. A tracked live turn owns an `OwnedMutexGuard<Agent>` inside a spawned future until execution finishes (`server/live_turn.rs:102–202`). Removing the map's reference does not drop that owned running future. The TUI close request sets `should_quit` (`crates/jcode-tui/src/tui/app/remote/server_events.rs:1761–1767`), not a server execution barrier. Headless workers have no TUI to quit.

**Impact:** `Stopped agent` can precede later tool operations and ownership transfer. The surviving-future path fits the incident. Exact execution ancestry has not been reproduced or fully correlated in daemon logs.

**Existing mechanism to reuse:** `crates/jcode-app-core/src/turn_cancel_registry.rs:20–84,97–169` tracks actual in-flight turn signals process-globally, including rename support. Both budgeted turn loops register there (`agent/turn_loops.rs:78–101`, `agent/turn_streaming_mpsc.rs:80–106`). `SessionControlHandle::request_cancel` fires those signals (`server/state.rs:742–775`). Client cancellation already signals, waits and falls back to task abort (`server/client_lifecycle.rs:3679–3746`). Descendant cancellation also uses the registry, but publishes `stopped` after signalling rather than awaiting settlement (`server/swarm/tree.rs:78–130`). None alone is a durable no-more-writes guarantee.

### 2. Automatic registration can recreate an ownerless member after stop

**Confirmed source path, incident causation still a hypothesis.** Lightweight control calls `ensure_lightweight_swarm_member` before dispatching Comm requests (`server/client_lightweight_control.rs:306–313`). Registration skips existing members and server-hosted agents, but otherwise accepts an existing startup stub, without checking a durable stop disposition (`:107–144`). It creates a `ready` headless member with `report_back_to_session_id: None` (`:179–202`). The default swarm is `session:<session_id>` (`server/util.rs:415–421`).

After stop removes both map entries, a later Comm request from a surviving execution can satisfy this path. That explains how the same ID could reappear outside its original swarm with no owner. Logs did not confirm the specific registration event for Rose. Do not label identity change or OS orphaning as established fact.

**Impact:** membership deletion cannot be the stop authority. A persisted stop record must survive membership removal and prevent implicit readmission. Legitimate headless first registration and explicit operator resume must remain supported.

### 3. Stop and messaging intentionally use different lookup scopes

**Confirmed source behavior and matching incident observation, high confidence.** `resolve_stop_target_session` filters both exact IDs and aliases to the requester's swarm (`server/comm_session.rs:1275–1328`). `force` is checked only after resolution (`:1116–1153`). Therefore `force=true` cannot make an out-of-swarm target resolvable. The stop request has no `to_swarm` field in its tool construction (`tool/communicate.rs:2833–2849`).

Messaging explicitly resolves a destination swarm before the target ID (`server/client_comm_message.rs:163–240`). Cross-swarm messageability without swarm-stop reachability is possible by design. The defect is unsafe lifecycle loss plus misleading/incomplete control diagnostics, not proof that all cross-swarm callers should gain stop permission.

**Required boundary:** resolve globally by exact stable identity, then authorize separately. Preserve denied access to unrelated sessions. Friendly-name ambiguity must not silently select another swarm's worker. Cross-swarm messaging is not authority to cancel.

### 4. Wait completion is session-status-bound, not assignment-bound

**Confirmed source gap, high confidence.** Wait matching copies `latest_completion_report` and tests status, with no assignment/run generation (`server/comm_await.rs:13–74`). Unknown members can count as done when stopped/completed is among the targets (`:62–64`). A stop that removes membership can therefore satisfy a wait without evidence of execution settlement.

Wait dedup keys contain requester, swarm, member IDs, target statuses and mode, but no work generation (`server/await_members_state.rs:139–162`). If current status still satisfies the condition, `handle_comm_await_members` returns the old persisted final response instead of fresh report content (`server/comm_await.rs:352–387`). Status updates only replace a report when a new report is supplied (`server/swarm/lifecycle.rs:743–776`). Direct wake queues a message when idle acquisition fails (`server/comm_control.rs:2487–2513`). This combination permits a queued new request and an older ready report to coexist.

**Existing owner:** `openspec/changes/add-swarm-handoff-contracts/` covers assignment and report evidence, but currently specifies revision/check/budget fields, not generation-bound waits. Recommend extending that existing scope through Feature. Do not treat more verbose reports as a freshness fix.

### 5. File claims are not stale-execution fencing

**Confirmed source behavior, high confidence.** Removal unregisters a claim worker before settlement (`server/swarm/lifecycle.rs:263–264`). Unregister removes the worker and releases claims (`server/swarm/claims.rs:424–429`). Subsequent `claim_files` for a non-worker returns an empty default guard (`:451–469`). Thus a surviving removed worker no longer receives worker claim conflict checks through this path. Default mode is `Warn`, not `Block` (`:442–449`).

Claims are used by native edit/write/patch/apply_patch. They do not provide universal fencing for arbitrary shell commands, background children, MCP tools or external effects. A process-local cancellation registry is also not durable execution authority. Do not promise absolute write revocation by adding a field to file claims.

**Impact:** releasing claims on a membership event can open a two-writer window. Keep claims until settlement or an explicitly reported unresolved state. Native-tool admission must reject a revoked execution even when it no longer has a member entry. Already-running filesystem operations and detached shell work need an explicit drain/unknown policy.

## Current global capabilities and prior work

The daemon already has process-global live turn cancellation, server session maps, persisted swarm records, persisted mutation requests, persisted waits, spawn specifications and worker revival. `crates/jcode-swarm-core/src/lib.rs:203–219` persists identity, swarm, owner and latest report, but not a run generation or durable stop barrier. The SDK already exposes session listing/global event discovery and session cancellation (`crates/jcode-sdk/src/client.rs:722–747,912–917,1664–1711`). That discovery follows one parent client's API session list. It is not evidence of a cross-machine or multi-daemon authority, and the cancel call's successful response is not proof of quiescence.

Existing owners checked:

| Artifact | Scope boundary |
| --- | --- |
| `add-swarm-handoff-contracts` | Active assignment/report evidence work. Reuse for run-bound reports and waits. |
| `surface-worker-stall-and-block-reasons` | Active presentation/stall work. Consume reliable lifecycle state, do not own it. |
| Archived `recover-swarm-workers-after-reload` and `swarm-state-persistence` spec | Existing persisted recovery. Extend, do not replace with a parallel registry. |
| Archived `add-swarm-file-claims` and `swarm-file-claims` spec | Existing configurable native-tool conflict control. Not universal sandboxing. |
| `enforce-overnight-cancellation-and-limits`, `reconcile-overnight-runs-after-restart` | Active overnight supervisor scopes. Future consumers of common worker controls, not justification to rewrite overnight here. |

Documentation drift matters: `docs/SWARM_ARCHITECTURE.md:39–44` describes promoting children to roots without a fallback. Current `server/swarm/lifecycle.rs:374–394` deliberately preserves a dangling owner edge instead. The recovery spec says restorable workers resume, but `server/swarm_persistence.rs:377–425` maps running to crashed and ready to stopped. Worker revival (`server/worker_revive.rs:50–155`) reconstructs eligible agents on explicit drive. Recorded contracts are not proof of automatic recovery behavior.

No tracker issue was supplied or verified. This exploration creates no issue and makes no remote link or state change. Existing OpenSpec files remain authoritative for their work.

## Priority and change risk

Effort is a source-review estimate, not a delivery commitment. These slices share dependencies and should not be implemented as independent competing controls.

| Priority | Candidate | Effort | Change risk | Evidence confidence |
| --- | --- | --- | --- | --- |
| 1 | Durable stop barrier plus active-turn settlement | L | High: admission, tools and recovery race | High source confidence, no fresh reproduction |
| 2 | Prevent automatic ownerless readmission after stop | M | Medium: preserve legitimate headless first registration | High source confidence, incident mechanism unproved |
| 3 | Exact global lookup with separate ownership authorization | M | High: avoid widening cancel permissions | High source and incident confidence |
| 4 | Generation-bound reports and waits in existing handoff change | M | Medium: persistence and protocol compatibility | High source confidence |
| 5 | Keep claim protection until settlement; reject revoked native calls | M | High: pending IO and shell limitations | High source confidence |

## Adversarial comparison

| Alternative | Why it is insufficient or preferable | Disposition |
| --- | --- | --- |
| Coordinator policy only | Waiting for acknowledgment and hashes reduces risk, but a busy/removed worker can still execute. Messaging cannot establish quiescence. | Necessary operational safeguard, not the fix. |
| Global lookup only | Makes Rose findable, but repeating the same teardown still does not stop the captured future. Broadening force could affect unrelated sessions. | Reject as complete solution. |
| Fire cancel then return success | Reuses existing signals, but has a race with new admission and cannot prove an in-flight tool or child process settled. Restart can lose cancellation intent. | Useful primitive, insufficient contract. |
| New durable manager process/database | Could centralize state, but adds another authority, migration and split-brain surface while existing machinery already covers discovery, persistence and cancellation. | Defer unless the isolated evidence proves current boundaries cannot support the contract. |
| Extend daemon controls with a durable stop barrier and observable settlement | Addresses membership recreation, active-turn cancellation, repeat requests and safe ownership transfer without replacing session storage. | Recommended candidate, not implemented. |

Concrete counterexamples the design must survive: stop races with wake; a worker holds the agent lock while stop deletes membership; a later Comm request re-registers the same ID; a pending write completes after its async caller is cancelled; old ready report satisfies a new wait; daemon crashes between stop persistence and cancellation; another client reconnects with a stale owner edge; an unrelated session has the same friendly name.

## Bounded next investigation

Investigation ID: `worker-stop-quiescence`, revision 1. Decision authority: user. Investigator: a later explicitly scoped run. This run performed source review only and selected no implementation candidate for execution.

Question: can existing daemon controls enforce stop admission and settlement without another manager? Baseline establishment is required because no fresh isolated runtime result exists. Freeze the tested binary digest, socket, private state root, two test-session IDs, mutation marker sequence and acceptance checks before running candidates.

Recommended bound: one isolated daemon, no credentials or network/model calls, two deterministic fake-provider workers, one unique private fixture directory, at most eight scenarios. No shared daemon or original application access. If the project test harness cannot isolate state, tool effects or provider calls, stop and report that prerequisite instead of using production sessions.

Observe: admitted run ID; stop request/result; active turns/tools before and after; sequence of marker mutations; durable owner/stop state; wait report generation. Do not invent a latency target. Use bounded test deadlines and report timeout as unresolved, never `stopped`.

Selection criteria, frozen before candidate results: no new execution from pre-stop queued work; no owned mutation after confirmed settlement; preserved stop across restart; correct owner denial; fresh waits distinguish runs. A supported explicit resume creates a new run and cannot silently revive an old one. Detached external work must remain visible as unresolved rather than counted as settled.

Stop after one baseline pass plus one targeted repeat for a concrete discrepancy. Preserve sanitized results and exact test commands in the canonical research/design artifact. Clean up only owned fixtures through supported controls. No result authorizes implementation, publication or shared-runtime activation.

## Feature route, if selected later

Candidate outcome: durable worker stop and safe ownership transfer within one daemon. Use existing session IDs. Add execution-incarnation/run identity only at the boundaries that need it. Persist stop intent before teardown, prevent stale admission, reuse cancellation, await tracked settlement, then release claims and publish terminal state. A bounded failure must return a precise pending/unresolved result, not `Stopped agent`.

Candidate state contract: runnable → stop requested → draining → stopped, with an unresolved result when settlement cannot be proved. Reported work completion is separate from execution stopped. Explicit resume/reassignment must select a new generation. Keep this model advisory until Feature resolves compatibility and source tests.

In-scope paths: existing server Comm handlers, lightweight registration, live-turn/control registry, persisted member/mutation/wait state, native tool admission, protocol/tool diagnostics and their tests. Coordinate wait/report schema work with `add-swarm-handoff-contracts`. Out of scope: new service/database, multi-host control, arbitrary OS-process killing, credentials/permissions, worktrees, Desktop, application fixes, overnight rewrite, universal shell sandboxing and automatic recovery of cancelled work.

Acceptance scenarios for the later canonical proposal:

| Scenario | Observable acceptance |
| --- | --- |
| Stop idle, provider-busy and native-tool-busy worker | Distinguish requested/draining/stopped. Confirm zero live tracked work before terminal success; no later fixture mutation. |
| Stop vs queued wake, duplicate stop and stale Comm registration | Same exact identity remains inspectable; no ownerless automatic readmission or duplicate execution. |
| Authorized owner targets worker outside original membership | Exact global resolution followed by preserved authorization. Unrelated callers receive denied access, not accidental cancellation. |
| New assignment after earlier ready/completed report | Old report and old persisted wait response cannot satisfy the new run's wait. Unknown membership is not quiescence. |
| Restart between stop intent and final settlement | Reconcile durable intent without silently rerunning cancelled work. Preserve owner and generation bindings. |
| Native mutation or shell child outlives cancellation | Drain tracked work or report unresolved. Do not release ownership based only on a removed map entry. State shell/background limits explicitly. |
| Owner disconnects, reconnects or changes session attachment | Reconcile ownership without giving a stale execution new write admission or control over unrelated sessions. |
| Two clients race control requests | Deterministic durable outcome, at most one admitted generation, correlated request and settlement evidence. |

Mechanical checks should include the relevant `jcode-app-core` control/await/cancellation/claim tests, `jcode-swarm-core` serialization tests, protocol compatibility and strict OpenSpec validation once a proposal exists. Those checks are proposed, not run here. Runtime acceptance must use the actual isolated binary/socket and public control interfaces, not just direct helper calls.

## Decisions and completion boundary

Recommended scope decision: global means all sessions on the existing daemon, with owner checks preserved. Defer cross-daemon/host authority and universal arbitrary-process write fencing. A request for those guarantees materially expands the design and needs a separate decision before authoring.

The active Explore workflow ends with research and recommendation. Formal proposal drafting requires selecting Feature and its authoring preflight. This note does not mark the candidate Ready or authorize Apply.

No source edits, proposal/spec/task changes, issue updates, builds, tests, implementation commits, pushes, daemon restarts or old-worker controls occurred. The report is local and uncommitted. Commit is deferred because this assessment is not a commit workflow and checkout-only policy says to use a separate branch while root policy says preserve the primary checkout. The checkout remains `main`; local `origin/HEAD` points to `dev`, so the default is not safely established for publication.

Presentation is blocked: `ropen` is not installed/on PATH and `ropen-preview` is not discoverable. No preview server, alternate transport or external publication was started. The local Markdown remains available at this path.

## Follow-up: public-interface observations and proposal request

2026-11-07T17:25–17:26Z. Append-only follow-up, not a replacement for the earlier evidence limits.

The real `swarm` tool was exercised against the serving daemon, without a synthetic harness or provider request:

- `action=list_swarms` returned Rose's exact ID under its own `session:<id>` swarm, one member, `coordinator: none`. This is current directory membership, not evidence of a current model turn or live writer.
- `action=status` for Rose's exact ID returned `is not in the same swarm as requester`. No stop, wake, message, force action or ownership mutation was attempted.
- `action=status` for this session's exact ID succeeded with `Lifecycle: running`, `Activity: busy (swarm)` and `Role: coordinator`.

This closes the read-only discovery/status observation loop: the actual public surface confirms global visibility plus swarm-local status access, while the in-swarm control observation succeeds. It does not validate stop semantics or show that the product has improved. No implementation exists. Historical call/result evidence, not a new stop experiment, remains the evidence for post-stop editing. Fresh cancellation acceptance needs isolated execution and cannot be substituted by these read-only checks.

The user then explicitly requested help authoring an OpenSpec proposal for Jcode session/swarm management, without implementation. Loaded `feature`, `preflight-check`, its `preflight-check-v2-shadow` rubric, roadmap guidance and native authoring instructions. `openspec list --json` resolves the existing root to `/home/nyaptor/.jcode`; schema is `spec-driven` with proposal/specs/design/tasks. P04's existing outcome is attributable parallel work without clobbering shared work. No new change has been scaffolded.

Pre-draft binding: source HEAD remains `fd25ee0c57d71d6119bb7372a9cf7584f150d24e`; tracked files unchanged; this research note is the only new authored path. Existing unrelated untracked files remain untouched. The report digest before this append was `1f91003fa499617db72239f96da43943eea4bbe20a29f697db39febb599cabe2`. User authority covers local proposal authoring, not implementation, shared-runtime activation, external publication or changes to unrelated sessions.

Pre-draft eligibility: local source inspection and research updates are eligible. Formal drafting is blocked on the material architecture/scope choice below. Runtime stop acceptance is unverified; shared debug control is disabled and no isolated runtime is prepared. Proposal strict validation is available after scaffolding. The canonical dependency validator is `/home/nyaptor/dev/personal/.agents/scripts/openspec/validate_dependencies.py`; default `python3` failed because `yaml` is missing, but existing `/usr/bin/python3` successfully imports PyYAML. No package installation is needed. Commit/publication closeout remains blocked by checkout-policy/default uncertainty; preview remains unavailable. No readiness or implementation authority follows from schema discovery.

Advisory shadow quality, pre-draft, all six groups assessed: G1 scope/authority 1 (proposal request explicit, global scope not selected); G2 verification route 1 (public read paths observed, cancellation acceptance unavailable); G3 delivery policy 1 (local artifacts permitted, conflicting branch instructions/default reference and preview unavailable); G4 runtime ownership 1 (existing daemon observed, intended global boundary unresolved); G5 preservation 2 (only owned research changes, existing files preserved); G6 dependencies/completion 1 (existing owners and native schema known, scope-dependent task/terminal mapping not authored). Total 7/12 = 58.33%, advisory only. This is not a numeric admission gate. No rewriting can replace the unresolved scope decision or runtime evidence.

Decision needed before formal drafting: use the existing daemon as the durable authority for all its sessions, or require coordination across daemon processes/hosts. Recommended: existing-daemon scope, preserving owner authorization, durable stop/settlement, stale-run rejection and generation-bound waits. Cross-daemon scope adds discovery, trust and split-brain contracts not established by this incident. A lookup-only/policy-only patch is smaller but cannot prevent the verified post-stop edit path. The selected answer unlocks one focused native proposal/design/spec/task set and reciprocal P04 binding. No existing handoff-contract proposal will be rewritten without an explicit scoped decision about its ownership.

## Scope decision and authoring, 2026-11-07T17:39Z onward

The user selected **existing-daemon scope**, repeated explicitly, and requested parallel swarms. That resolves the material architecture boundary: extend the current daemon, not a manager spanning daemons/hosts. Local proposal/design/spec/tasks and reciprocal P04 roadmap links are selected. Source implementation, shared daemon activation and remote publication remain excluded.

The source HEAD and research digest were rechecked before scaffolding. Two new unrelated untracked Nomi handoffs appeared and were preserved without reading their contents. Native `openspec context --json` and `list --json` confirmed the repository root; `openspec new change make-worker-stop-durable` created the schema metadata. Proposal capability inventory was checked against native main specs before writing.

Pre-draft update under `preflight-check-v2-shadow`: local artifact authoring eligible, implementation not requested. G1 2 (scope/authority selected), G2 1 (public discovery/status observed and existing e2e/binary paths located, stop acceptance not run), G3 1 (local drafting available, branch/default and preview limitations unchanged), G4 2 (existing daemon and owner-preserving control boundary explicit), G5 2 (exact disjoint artifact paths, other dirty paths preserved), G6 2 (native schema, existing owner boundaries and completion operations mapped). Advisory total 10/12 = 83.33%. Targeted investigation found the actual `e2e` Cargo target, real spawned-binary reload tests and real daemon/bridge test paths. This repairs command uncertainty but does not claim cancellation acceptance. No permission, provider, dependency-installation or branch change is needed for authoring.

Ownership split: Owl owns only the four new delta spec files, Mosquito owns only `design.md`, coordinator owns proposal/tasks/research and narrow P04 roadmap references. Default configured worker model/placement retained. Worker outputs require coordinator inspection and actual native validation before handoff. This is planning parallelism, not a new reproduction or proof of safe worker shutdown.

Cross-change decision: `add-swarm-handoff-contracts` continues to own revision/check/budget enrichment. `make-worker-stop-durable` owns the execution-generation primitive and generation-bound waits needed for stop/reassignment without that enrichment. No existing proposal is rewritten. Overlapping future schema changes must serialize and revalidate, not create a fabricated semantic dependency. Archived recovery/claims/communicate work remains baseline; new dependency metadata is `depends_on: []`.

## Authoring result and handoff preflight, 2026-11-07T18:03Z

The existing-daemon decision was repeated at 18:02Z and remains unchanged. Native change: `openspec/changes/make-worker-stop-durable/`. Artifacts: scaffold metadata, proposal, design, tasks and four capability deltas. Thirteen requirements and 44 scenarios map to 21 unchecked tasks. P04 references were added only to its phase index, real-change list and exact reliability-slice binding. No source code, maintained main specs or unrelated proposal changed.

Both parallel artifact workers failed at the configured provider before making any tool call or writing any file. Fresh status showed failed/idle and summaries showed no tool calls. The coordinator cancelled their assignments through non-waking notifications, took over the exact files and authored locally. No worker contribution, independent review or safe-stop reproduction is claimed. No automatic provider/model override or worker respawn occurred.

Coordinator self-review repaired four concrete issues before final validation: ambiguous lost-session stop-barrier phrasing; archive task's potential self-dependency; active versus queued execution generation identity and generation reuse; and precision about scripted inference versus actual public-binary control acceptance. Stop now revokes active plus already-accepted pending generations, while resume allocates above every previous identity. Synthetic provider input remains explicitly labeled, not claimed as live-provider acceptance.

Observed final authoring gates at 18:02:59Z, all successful:

- `openspec validate make-worker-stop-durable --strict --no-interactive --json`: valid, zero issues.
- `/usr/bin/python3 /home/nyaptor/dev/personal/.agents/scripts/openspec/validate_dependencies.py --root /home/nyaptor/.jcode make-worker-stop-durable --json`: valid, zero issues, selected closure size 1.
- `openspec status --change make-worker-stop-durable --json`: proposal/specs/design/tasks all present, planning complete. Its `isComplete` is artifact presence, not implementation completion.
- `openspec instructions apply --change make-worker-stop-durable --json`: actual native consumer parses 21 tasks, zero complete, 21 remaining, structural state ready. This command only read instructions. Apply was not executed.
- Requirement/scenario/task audit: 13 requirement titles mapped, 44 WHEN/THEN scenarios, 21 unique unchecked task IDs, exact modified headers matched existing specs, local links resolved, no placeholders, no em dashes or trailing blank lines. This is supplementary structural evidence, not product behavior acceptance.
- `git diff --check`: passed. Existing main capability specs and adjacent `add-swarm-handoff-contracts` each passed native strict validation before authoring. Cargo metadata confirmed root `e2e` target; Rust tests were not run because no implementation was requested.

Final artifact SHA256 bindings, same source HEAD `fd25ee0c57d71d6119bb7372a9cf7584f150d24e`:

| Artifact, relative to the change | SHA256 |
| --- | --- |
| `.openspec.yaml` | `cb72bd9217510d5461ddb5384ca197195a83da2d8dfbd237178f7ed662291c68` |
| `proposal.md` | `9e894ab1cc936de2268d550e72f0aca6a871b1e1821659e7811f964ccc1ca38b` |
| `design.md` | `93fb0899b82492adc43741188cd014694814db79718f4fd0c88fa559f36e9fd7` |
| `tasks.md` | `9ea2cc62b2fa7bec1bb352a1b972413ceb096ea6233d3ae67134dd779c59497b` |
| `specs/worker-session-control/spec.md` | `527771f37f314d5c53ecdfe056ca8e4c035aa13f264b7d694a38e09a2b9e8627` |
| `specs/swarm-state-persistence/spec.md` | `35830574c317f0ed55c8e93f4344df8d9a27403bad52f32b15a4e638714431c8` |
| `specs/swarm-communicate-reliability/spec.md` | `ceeeaaefe09664bc0455bfc2875018aecaf8736af8c96e710b40751bc416a27b` |
| `specs/swarm-file-claims/spec.md` | `38c0fce11eaa209bc5eb6ccc1fdd972b227f016cd5e7f46616454b170b80039d` |

Handoff binding: phase `handoff`, rubric `preflight-check-v2-shadow`, this repository/host and the user-selected existing-daemon scope. Exact modified targets are this change, this research note and four added/two replaced P04 roadmap lines; unrelated untracked Nomi handoffs/runtime files remain preserved. Every requirement maps to tasks and public acceptance; tools/environment/authority and terminal operations are explicit in `tasks.md`.

| Operation | Eligibility and evidence | Resolver / remaining condition |
| --- | --- | --- |
| Local proposal inspection/review | Eligible. Native scaffold, strict validation, dependency check and task consumer passed. | User can review the saved proposal and design. |
| Local implementation tasks 1–6 | Not currently requested. Structurally planned, not operationally accepted. | Later explicit Apply request, then fresh task 1.1/JIT preflight and isolated baseline 1.2. |
| Runtime acceptance 7.1 | Unverified, not executed. Shared debug control is disabled. Isolated real-binary command is a task 1.3 deliverable. | Prove private state/socket isolation, exercise actual public controls and native mutations, then record results. No production worker control. |
| Independent final implementation review 8.1 | Unverified route availability after provider failures; remains mandatory after fresh gates. | A permitted independent reviewer must run on the final diff. Failed authoring workers are not a review. |
| Commit/publication closeout | Blocked only for dependent persistence. Configured identity exists, checkout `main` differs from local `origin/HEAD=dev`, checkout-only policy says own branch while root policy says preserve primary. No push/PR authority or linked issue. | Resolve exact branch/default policy before committing, do not switch/reset/stage unrelated work. External publication requires scoped authority. |
| Archive 8.3 | Not eligible in authoring. Implementation, acceptance, final review and commits absent. | Later Apply terminal gates, in the order already encoded. |
| Tailnet artifact preview | Blocked: `ropen` unavailable, `ropen-preview` skill missing. | No alternate server/transport or installation attempted. Local files preserved. |

Advisory handoff quality: G1 2, G2 1 (actual control acceptance and independent review route unverified), G3 1 (persistence/preview policy limits), G4 2, G5 2, G6 2. Total 10/12 = 83.33%, uncalibrated shadow assessment, not execution permission. Remaining weak points are genuine operation prerequisites, not prose defects. No number makes them pass. Source/artifact edits, changed ownership, different binary/socket, provider access or policy changes invalidate the relevant JIT observations.

Result: proposal artifacts are valid and available for review; runtime improvement is neither implemented nor accepted. No commits, push, PR, archive, shared reload or old-worker stop occurred. Local persistence exists as uncommitted artifacts. Full workflow closeout is limited by the recorded policy and presentation constraints, rather than reported complete by artifact presence alone.
