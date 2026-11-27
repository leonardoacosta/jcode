# Handoff: durable global worker/session management

## Fresh-session objective

The user requests a fresh Jcode session to use the **`improve` skill/lens** to investigate worker-management failures and identify a sound OpenSpec proposal, ideally for a **global, durable session manager**.

Start with investigation and recommendations. A global manager is a desired direction, not a settled design or proven missing component. Discover and reuse current runtime capabilities before proposing a replacement. This handoff does not authorize implementation, daemon restart, deployment, publication, or termination of unrelated sessions.

Suggested opening prompt:

> Load `improve` and investigate the worker-control incident in this handoff. Trace current global session identity, swarm membership, execution cancellation, message delivery, recovery and file ownership. Distinguish confirmed defects from hypotheses. Recommend the smallest coherent design, ideally a global durable session manager or an extension of existing machinery. Then use the native feature/OpenSpec workflow to author a proposal with concrete acceptance scenarios and unresolved decisions. Do not implement or restart the shared daemon.

## Suggested skills

Use the native Skill tool to load, as relevant:

1. **`improve`**: primary evidence-led improvement lens requested by the user.
2. **`systematic-debugging`**: investigate the incident before asserting a root cause.
3. **`jcode-orchestration`**: understand native swarm ownership and lifecycle contracts.
4. **`runtime-evidence`**: bind observations to the actual binary, daemon, socket and session.
5. **`preflight-check`**: required authoring/handoff readiness assessment before formal proposal drafting.
6. **`feature`** and **`openspec-explore`**: select/reuse an existing change or author a native OpenSpec proposal.
7. **`state-machine`**: model lifecycle, cancellation, ownership transfer and recovery explicitly.
8. **`jcode-handoff`**: if producing a later implementation brief.

Skills grant no extra execution or external-action authority.

## Scope and policies

- Requested destination and intended fresh-session repository: `~/.jcode`, verified as a Git checkout rooted at `/home/nyaptor/.jcode` when this document was written.
- Read `~/.agents/AGENTS.md`, then applicable repository instructions. `~/.jcode/.agents/AGENTS.md` applies to this verified source checkout only.
- Product focus is Jcode TUI/CLI unless the user explicitly requests Desktop. Desktop is a separate repository.
- Repo instructions require `selfdev` and `debug_socket` for self-development when available. Discover actual tools; do not invent tool access.
- Confirm the active runtime binary/daemon/socket before runtime conclusions. A successful local build does not mean the shared daemon runs it. See repository policy for isolated-socket testing.
- Do not manually kill/restart the bridge or shared daemon. Do not mutate unrelated workers, credentials, permissions or worktrees.
- No Jcode source code was inspected or changed for this incident in this conversation. Bundled docs were read. This file is a planning handoff, not a source-level diagnosis.

## Incident evidence and identifiers

Use session/tool history and logs as primary evidence, not this narrative alone. Times below are UTC on **2026-11-07**.

| Role | Session ID |
| --- | --- |
| Root coordinator, friendly name `goat` | `session_goat_1793813250757_f2dcdb11727df7d3` |
| Original implementer, `rose` | `session_rose_1794010960955_2b4e71ae94e17ac7` |
| Replacement implementer, `evergreen` | `session_evergreen_1794013629578_d6a18a81be2f00f6` |
| Independent reviewer, `seedling` | `session_seedling_1794012832442_2d8d6c71fd852fda` |

Root swarm: `session:session_goat_1793813250757_f2dcdb11727df7d3`.
Later cross-swarm route: `session:session_rose_1794010960955_2b4e71ae94e17ac7`.

### Observed sequence

1. The coordinator assigned Rose a bounded implementation in a shared checkout. The work needed exact-byte review and preservation of unrelated dirty hunks.
2. Rose repeatedly acknowledged repair instructions while local `status` showed `ready`/idle and `await_members` delivered earlier completion reports. Multiple `resume` calls returned `Status: queued`. Some later acknowledgments did not establish that the expected work turn had run.
3. At approximately **01:06:44**, `swarm stop` for Rose returned **`Stopped agent`**. The coordinator transferred exclusive implementation ownership to Evergreen.
4. Evergreen implemented repairs and declared a post-gate hash freeze. At about **01:23**, Seedling independently detected that `storage.py` changed after that freeze and stopped review. A fresh coordinator test run failed a stale SQL-block assertion. Ownership and candidate bindings were no longer reliable.
5. At approximately **01:25:11**, the coordinator stopped Evergreen. Later source/research changes still appeared.
6. At **01:26:11** and subsequently, messages arrived from Rose through the separate cross-swarm route, describing ongoing repairs/tests and plans to update research. `list_swarms` showed that swarm with **`coordinator: none`**, **one member**. Rose was no longer in the root's normal local worker list.
7. Exact-ID stop attempts, including `force: true`, returned **`Unknown swarm session`**. Supplying the cross-swarm target did not make the stop operation resolve it. Cross-swarm `message` calls reported successful delivery.
8. Repeated cross-swarm stop/interrupt messages were followed by additional progress reports rather than an explicit stopped/no-more-writes acknowledgment. This could involve queued/stale delivery, live execution, identity changes or another lifecycle issue. Delivery success was not proof of receipt or compliance.
9. At **01:32:37**, after the user asked to course-correct Rose, the coordinator sent a new **read-only handoff assignment**, cancelling its obsolete implementation plan and requesting current activity, last edits and explicit ownership release.
10. A fresh coordinator focused test run completed at **01:32:53**: **203 run, 191 passed, 12 skipped**. All seven before/after hashes matched throughout that run. This established temporary byte stability, not reliable writer shutdown.
11. At the last direct control check around **01:35**, Rose's separate swarm was still listed and no explicit handoff/shutdown acknowledgment had arrived. Seedling remained paused. No feed-stage commit or completion checkbox was made.

The long pause before this handoff means **all live state is stale**. Inspect current sessions and repository state afresh. Do not assume Rose still runs, that a queued message is new, or that old hashes are current.

### Evidence boundaries and corrections

- “Worker registry” was coordinator shorthand for session/swarm membership and ownership records, not a verified source component name.
- “Registry and routing disagreed” describes tool-level symptoms, not an established internal diagnosis.
- A session lifecycle/reparenting/cancellation defect is a hypothesis. No implementation trace or daemon-log correlation has proved it.
- Late cross-swarm messages and file changes were observed, but attribution of each individual edit to Rose versus Evergreen was not fully reconstructed. Do not assert authorship from timing alone.
- `coordinator: none` is an observed directory field, not proof that an OS process was orphaned.
- A successful message tool result is not acknowledgment by the target. A successful stop result should be investigated for its precise documented semantics: membership removal, cancellation request, quiescence, or actual termination.
- The coordinator contributed to the incident by transferring ownership after a stop response without verifying execution quiescence, and by issuing repeated resume/message requests that may have complicated recovery. Evaluate agent policy and runtime design separately.
- Jcode maintainer feedback was queued about stopped workers continuing through another swarm route, unresolved stop targets and stale completion reports. This was agent-observed feedback, not a confirmed maintainer diagnosis.

## Existing artifacts to consult, not duplicate

### Jcode architecture and policy

Read current source-checkout versions and compare them with bundled documentation:

- `docs/SWARM_ARCHITECTURE.md`: lifecycle states, `report_back_to_session_id`, ownership, stop permissions, reparenting, resume identity updates, notification delivery and cross-swarm DMs.
- `docs/SERVER_ARCHITECTURE.md`: server-hosted headless sessions and daemon/socket ownership.
- `docs/RUNTIME_COMMANDS.md`: actual supported runtime controls.
- `~/.jcode/.agents/AGENTS.md`: build/reload/runtime-evidence and product boundaries.

Bundled architecture docs state that membership survives daemon recovery, parent edges determine stop authority, cross-swarm DMs route by swarm/session identity, and notifications are queued as soft interrupts. They also state idle/completed agents do not resume merely because notifications arrive. Treat those as documented contracts to verify, not proof of current behavior. The architecture document contains design/status material, so inspect actual implementation and tests.

### Original application work, outside Jcode

The incident happened in **`/home/nyaptor/dev/personal/.agents`**, not the consumer installation `~/.agents`.

- Authoritative proposal: `openspec/changes/consume-nomi-evidence/`.
- Authoritative workflow state: its `tasks.md`.
- Append-only evidence narrative: its `research.md`, including freeze failures and recovery attempts. Some attribution is provisional as noted above.
- Last known committed HEAD: **`7829a30`**, conservative saved-report invalidation slice, independently approved. Prior commit **`c13148d`** enabled bounded staged per-work Nomi commits.
- The later feed-staging seven-path working diff was **uncommitted and not independently approved** at last observation. Preserve it. Do not repair, commit, archive or resume that application task as part of Jcode proposal investigation without separate scope.
- Preserve unrelated Brown/report/journal/resource/graph changes. Do not reset, clean, blanket stage or revert anything.
- Original application tests have a destructive opt-in: **never set `LINEAR_REPORT_TEST_POSTGRES`**. It can drop schema. No application DB/provider/schema actions are part of this handoff.

Useful existing scratch evidence, if still present:

- `~/.jcode/scratch/nomi-feed-completion-baseline/`: initial HEAD/working copies, hashes, patches and later freeze files.
- `~/.jcode/scratch/nomi-feed-repair-baseline`: replacement-worker baseline artifact; inspect its actual type before use.
- `~/.jcode/scratch/nomi-feed-control-check/{before,after}.sha256`: seven-file equality across the 01:32 focused run.

The application's dirty Graft graph must not be refreshed as a side effect of investigation. Prior Graft invocations unexpectedly wrote it. For that application checkout, read existing cards only. This is not permission to skip applicable navigation policy in the separate Jcode repository; assess tooling side effects there independently.

## Investigation questions for the improve lens

1. **Identity and lookup:** Is there one authoritative global session ID across stop/resume/reparent/reload? How do local swarm tools and cross-swarm tools resolve the same ID? Can a session be messageable but not controllable?
2. **Stop semantics:** What exactly does `Stopped agent` guarantee? Are in-flight model turns, tools, queued wakes, timers and notification-triggered continuations cancelled? Can a pending resume recreate a stopped worker? Is quiescence observable and durable?
3. **Durable ownership:** Where are owner/report-back edges persisted? Are updates atomic with membership removal and execution state? What happens if no coordinator exists?
4. **Message semantics:** Are notifications ordered, deduplicated and bound to a session incarnation? Can delayed acknowledgments be distinguished from fresh replies? Can interrupts actually preempt, or only queue?
5. **Completion freshness:** How does `await_members` choose a report? Does it bind to the current assignment/run generation, or return an earlier ready/completed result after resume?
6. **Write authority:** Are exact-file claims advisory or enforced? Does stop/reassignment revoke a former worker's write authority? Could ownership epochs or fencing tokens prevent stale workers from writing after transfer?
7. **Global management:** Which global session discovery/control capabilities already exist? Is a durable manager missing, or are current lookup/cancellation/reconciliation paths inconsistent?
8. **Recovery:** Can the system reconcile persisted sessions, live executions, swarm directories, claims and queued events after a crash or rename? What safe operator controls exist when a target is not locally resolvable?
9. **Observability:** Can one trace a command from coordinator request through acknowledgment, execution cancellation and final quiescence using correlated IDs, without exposing prompts or private source?
10. **Agent workflow:** Which safeguards should the coordinator enforce even with a correct runtime, such as stop acknowledgment plus quiescence before transferring ownership, bounded retries, and no concurrent repair writers?

## Candidate proposal outcomes to evaluate

These are options, not selected requirements:

- Extend the existing global session directory into a durable authoritative manager, rather than introduce a parallel registry.
- Separate stable session identity from execution-incarnation and assignment/run IDs.
- Define an explicit lifecycle state machine with stop-requested, draining/quiescent and stopped semantics if the current model needs them.
- Make stop/reassignment idempotent and durable across reloads, with reconciliation and clear operator feedback.
- Route control and messaging through consistent global identity resolution, while preserving ownership/permission checks.
- Bind waits/completion reports to a specific assignment or run generation.
- Fence stale workers' writes after ownership transfer, if enforcement is feasible at the actual tool boundary.
- Provide a session-scoped global inspect/cancel/handoff surface. Do not equate this with permission to kill arbitrary processes.

Assess minimum scope, migration/compatibility cost, failure modes and whether each outcome solves observed behavior. Avoid an oversized manager framework where a small correction to existing machinery is sufficient.

## Acceptance scenarios worth specifying

A proposal should map requirements to observable tests, including:

- Stop an idle worker, a model-in-flight worker and a tool-in-flight worker; verify exact guarantees and no later queued resume/write.
- Stop then resume/reassign, including concurrent or repeated requests; an old incarnation cannot regain ownership silently.
- A cross-swarm messageable session is globally inspectable/controllable by its authorized owner, or the control failure explains the precise boundary.
- Coordinator disappears; child state and ownership reconcile without untracked live writers.
- Daemon restart/recovery preserves cancellation, ownership and report-generation bindings.
- An old completion report cannot satisfy a wait for a newer assignment.
- A stale worker attempts a write after ownership transfer; verify enforcement or an explicit documented limitation and safe recovery path.
- Two clients issue conflicting control requests; deterministic outcomes, no duplicate live turns.
- Stale notifications arrive after stop; no accidental new execution.
- Operator sees whether a stop was requested, acknowledged or actually quiescent, with bounded failure reporting.

Use actual runtime interfaces and isolated test sockets where authorized. Source inspection, mocked state transitions and successful compilation alone do not prove cancellation or ownership behavior.

## Next-session sequence

1. Read policies and load `improve`.
2. Inspect current Jcode checkout identity, dirty state, relevant graph/context and existing OpenSpec changes/issues. Preserve unrelated work.
3. Reconstruct a bounded incident timeline from the identified sessions and runtime logs. Verify the actual runtime build/socket. Redact private content.
4. Trace the current mechanisms and reproduce the smallest failure safely, without touching the original application checkout or unrelated sessions.
5. Separate confirmed defects, missing evidence and coordinator-policy failures. Compare minimal fixes with a durable global-manager design.
6. Present a concise recommendation and resolve material design questions. Use `preflight-check` before formal proposal drafting.
7. Author/update one native OpenSpec change with scope, state model, identity/ownership contracts, recovery semantics, acceptance tests, rollout and compatibility limits. Do not claim readiness from structural validation alone.

## Completion of this handoff

This document records context for a fresh investigation and proposal. It does not establish the root cause, approve the uncommitted application slice, or authorize implementation of a session manager.
