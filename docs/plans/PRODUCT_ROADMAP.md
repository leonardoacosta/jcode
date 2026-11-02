# Jcode product roadmap

## Goal and authority

Make Jcode a trusted daily coding agent that can continue useful work, collaborate,
and recover without losing user control or claiming unverified success.
This is an expandable strategic map, not an implementation plan or execution ledger.
The [documentation index](../README.md) links this roadmap. OpenSpec proposal,
design and task artifacts remain the only feature execution authority.

**Product scope:** this repository's TUI, CLI/headless runs, daemon/session runtime,
native tools, MCP, skills, providers/auth, memory/context, shared Rust SDK/harness API
and TypeScript SDK. Browser, voice, self-development and platform integrations are
included at their owned boundaries. Desktop lives in a separate repository:
shared contracts may support it, but this roadmap makes no Desktop delivery or
other cross-repository implementation commitments.

**Non-goals:** implement features here; create all proposal stubs; replace native
OpenSpec tasks with roadmap checkboxes; invent issue IDs, estimates or dates;
add a second scheduler/orchestrator; migrate data or expand permissions implicitly.
Priority and phase grouping below are proposed, not committed release promises.

## Evidence scope and existing baseline

Sources: the tracked [product README](../../README.md), [documentation index](../README.md),
[native-tooling audit](../research/native-tooling-harness-audit-2026-10-26.md) and the
source contracts linked below. Additional advisory evidence: [Sandcastle domain mining](../research/sandcastle-mining-2026-11-02/README.md) and [task/worktree configuration research](../research/sandcastle-task-worktree-config-2026-11-02.md). These source observations do not establish implementation or acceptance. Documentation and source describe intent or current
architecture; neither README figures nor a names-only inventory prove fresh runtime acceptance.

Swarm/todo, review/judge, plan/poke, overnight/bg/schedule, skills/MCP, structured
headless runs, resume, provider diagnostics and self-dev are existing capabilities,
not newly completed roadmap features. Improve their contracts before adding APIs.

**Durable baseline:** no OpenSpec changes are tracked in this checkout's Git index
at roadmap creation. Its existing advisory plans and OpenSpec specs/archives are
local-only material. Archived changes, including `open-user-clicked-links` (P09),
are now bound below as delivered slices; an archived slice does not complete a phase.
Do not force-add private historical artifacts merely to populate this roadmap.

Future feature IDs become bindings only after their real proposal/task artifacts
are committed and available in a clean checkout. Existing local work can inform
that authoring after its scope, evidence and publication boundaries are reviewed.
All ten phases started Planned; no implementation, cost saving or outcome improvement
was claimed by creating the roadmap. P01 now has one committed proposal,
`record-continuation-evidence`; its implemented baseline and canonical capability
are archived and committed locally. Broader P01 completion and remote publication
are not claimed. Later archived slices also move P04, P07, P08, P09 and P10 to
In progress; none of them establishes a phase outcome.

## Cross-cutting acceptance contract

Every feature carries these constraints from the first phase, not as late cleanup:

- **Security, privacy, control:** external content is data, never authority; preserve
  consent, least privilege, secret redaction, workspace separation, explicit stop
  and user decisions. Scheduling, memory, judge feedback and session identity do
  not authorize external actions. State what crosses a provider/service boundary.
- **Truthful evidence:** name the source revision, actual serving binary/daemon and
  public acceptance surface. Distinguish passed, failed, blocked, cancelled,
  inconclusive and budget-exhausted. Compilation, task ticks and archive presence
  cannot substitute for observed behavior and native lifecycle closeout.
- **Compatibility and recovery:** test local/remote and old/new peers where relevant,
  malformed input, interruption, restart, stale state and rollback. Preserve existing
  user data, task ownership and cancellation. Caps never synthesize success.
- **Terminal correctness:** compact chrome stays one logical row per visual row.
  Budget Unicode display cells, ellipsize rather than wrap, sanitize controls,
  retain monochrome cues and test narrow widths, CJK and combining text. Reuse the
  info widget's limit-bar renderer for usage, not a competing convention.
- **Responsiveness and resources:** preserve measured interactive latency, memory
  and concurrency baselines from the first feature. Compare matched workloads on
  recorded hardware; define feature-specific budgets before claiming improvement.

## Phase index: derived rollup only

All phases start **Planned**. No phase-wide end-to-end completion is established.
Dependencies are acceptance prerequisites between capability areas, not permission
or an instruction to block an already-authorized bounded feature until every
other candidate ships. Independent slices may proceed concurrently.

| Stable ID | Phase | Depends on | Status | Real OpenSpec changes | Phase evidence |
| --- | --- | --- | --- | --- | --- |
| P01 | Trusted core and evidence baseline | None | In progress | [admit-judge-feedback-with-bounds](../../openspec/changes/admit-judge-feedback-with-bounds/proposal.md), [bind-acceptance-evidence-to-runtime](../../openspec/changes/bind-acceptance-evidence-to-runtime/proposal.md), [restrict-plan-command-execution](../../openspec/changes/restrict-plan-command-execution/proposal.md) (cut, not implemented), [record-continuation-evidence](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/proposal.md), [hot-file-test-coverage](../../openspec/changes/archive/2026-11-02-hot-file-test-coverage/proposal.md), [improve-workflow-suite](../../openspec/changes/archive/2026-10-20-improve-workflow-suite/proposal.md) (archived); supporting, no completion credit: [evaluate-personalized-jcode-prompt](../../openspec/changes/archive/2026-10-27-evaluate-personalized-jcode-prompt/proposal.md) (archived), [automate-personal-prompt-experiments](../../openspec/changes/archive/2026-10-28-automate-personal-prompt-experiments/proposal.md) (archived), [extend-personal-prompt-experiment-adapters](../../openspec/changes/archive/2026-10-28-extend-personal-prompt-experiment-adapters/proposal.md) (archived), [app-core-clippy-clean](../../openspec/changes/archive/2026-11-01-app-core-clippy-clean/proposal.md) (archived) | Baseline archived; remote scope unresolved |
| P02 | Discoverable native tools, MCP and skills | P01 | Planned | [add-typed-skill-selector-metadata](../../openspec/changes/add-typed-skill-selector-metadata/proposal.md), [discover-native-and-mcp-capabilities](../../openspec/changes/discover-native-and-mcp-capabilities/proposal.md), [show-skill-and-mcp-provenance](../../openspec/changes/show-skill-and-mcp-provenance/proposal.md) (cut, not implemented), [mcp-failure-backoff](../../openspec/changes/archive/2026-11-02-mcp-failure-backoff/proposal.md), [mcp-management-experience](../../openspec/changes/archive/2026-11-02-mcp-management-experience/proposal.md), [lint-skill-name-and-folder](../../openspec/changes/archive/2026-11-02-lint-skill-name-and-folder/proposal.md), [tool-timeout-and-patch-recovery](../../openspec/changes/archive/2026-11-02-tool-timeout-and-patch-recovery/proposal.md), [evaluate-code-search-strategies](../../openspec/changes/evaluate-code-search-strategies/proposal.md); supporting, no completion credit: [add-lifecycle-hooks](../../openspec/changes/add-lifecycle-hooks/proposal.md), [improve-workflow-suite](../../openspec/changes/archive/2026-10-20-improve-workflow-suite/proposal.md) (archived) | Not established |
| P03 | Jev semantic poke continuation | P01 | Planned | [add-structured-continuation-evidence](../../openspec/changes/add-structured-continuation-evidence/proposal.md), [admit-semantic-poke-in-shadow](../../openspec/changes/admit-semantic-poke-in-shadow/proposal.md), [route-jev-supervision-explicitly](../../openspec/changes/route-jev-supervision-explicitly/proposal.md) (cut, not implemented) | Not established |
| P04 | Swarm collaboration and handoffs | P01 | In progress | [add-swarm-handoff-contracts](../../openspec/changes/add-swarm-handoff-contracts/proposal.md) (cut, not implemented), [report-agent-lifecycle-to-herdr](../../openspec/changes/report-agent-lifecycle-to-herdr/proposal.md), [surface-worker-stall-and-block-reasons](../../openspec/changes/surface-worker-stall-and-block-reasons/proposal.md), [focus-worker-herdr-pane](../../openspec/changes/focus-worker-herdr-pane/proposal.md) (cut, not implemented), [return-full-swarm-reports](../../openspec/changes/archive/2026-11-02-return-full-swarm-reports/proposal.md), [queue-messages-to-busy-workers](../../openspec/changes/archive/2026-11-02-queue-messages-to-busy-workers/proposal.md), [show-resolved-worker-model](../../openspec/changes/archive/2026-11-02-show-resolved-worker-model/proposal.md), [add-swarm-file-claims](../../openspec/changes/archive/2026-11-02-add-swarm-file-claims/proposal.md), [recover-swarm-workers-after-reload](../../openspec/changes/archive/2026-11-02-recover-swarm-workers-after-reload/proposal.md), [add-council-tool](../../openspec/changes/archive/2026-11-02-add-council-tool/proposal.md); [swarm-communicate-reliability](../../openspec/changes/archive/2026-11-01-swarm-communicate-reliability/proposal.md) (archived), [swarm-module-split](../../openspec/changes/archive/2026-11-01-swarm-module-split/proposal.md) (archived), [swarm-state-async-safety](../../openspec/changes/archive/2026-11-01-swarm-state-async-safety/proposal.md) (archived), [swarm-widget-cell-width](../../openspec/changes/archive/2026-11-01-swarm-widget-cell-width/proposal.md) (archived); supporting, no completion credit: [add-role-model-slots](../../openspec/changes/archive/2026-11-02-add-role-model-slots/proposal.md) | Archived slices delivered; phase outcome not established. |
| P05 | Durable overnight, scheduling and recovery | P03, P04, P06 | Planned | [configure-task-and-worktree-policy](../../openspec/changes/configure-task-and-worktree-policy/proposal.md) (proposed, not implemented), [admit-overnight-tasks-on-evidence](../../openspec/changes/admit-overnight-tasks-on-evidence/proposal.md), [enforce-overnight-cancellation-and-limits](../../openspec/changes/enforce-overnight-cancellation-and-limits/proposal.md), [reconcile-overnight-runs-after-restart](../../openspec/changes/reconcile-overnight-runs-after-restart/proposal.md), [recover-missed-scheduled-runs](../../openspec/changes/recover-missed-scheduled-runs/proposal.md) (cut, not implemented), [reattach-background-tasks-after-reload](../../openspec/changes/reattach-background-tasks-after-reload/proposal.md) (cut, not implemented), [shared-server-crash-resilience](../../openspec/changes/archive/2026-11-02-shared-server-crash-resilience/proposal.md); supporting, no completion credit: [recover-swarm-workers-after-reload](../../openspec/changes/archive/2026-11-02-recover-swarm-workers-after-reload/proposal.md), [repair-jcode-herdr-session-binding](../../openspec/changes/archive/2026-10-21-repair-jcode-herdr-session-binding/proposal.md) (archived) | Not established |
| P06 | Provider, auth and usage-budget reliability | P01 | Planned | [classify-provider-auth-failures](../../openspec/changes/classify-provider-auth-failures/proposal.md), [reconcile-usage-budget-accounting](../../openspec/changes/reconcile-usage-budget-accounting/proposal.md), [report-provider-doctor-coverage](../../openspec/changes/report-provider-doctor-coverage/proposal.md) (cut, not implemented), [fallback-on-transient-model-errors](../../openspec/changes/fallback-on-transient-model-errors/proposal.md), [add-role-model-slots](../../openspec/changes/archive/2026-11-02-add-role-model-slots/proposal.md) | Not established |
| P07 | Memory/context quality and privacy | P01, P06 | In progress | [add-attributable-memory-recall](../../openspec/changes/add-attributable-memory-recall/proposal.md), [add-compaction-quality-evidence](../../openspec/changes/add-compaction-quality-evidence/proposal.md), [add-memory-retention-controls](../../openspec/changes/add-memory-retention-controls/proposal.md) (cut, not implemented), [native-instruction-file-loading](../../openspec/changes/archive/2026-11-02-native-instruction-file-loading/proposal.md), [workspace-aware-context](../../openspec/changes/workspace-aware-context/proposal.md), [add-prompt-budget-ratchet](../../openspec/changes/add-prompt-budget-ratchet/proposal.md), [session-store-retention](../../openspec/changes/archive/2026-11-02-session-store-retention/proposal.md), [evaluate-personalized-jcode-prompt](../../openspec/changes/archive/2026-10-27-evaluate-personalized-jcode-prompt/proposal.md) (archived), [automate-personal-prompt-experiments](../../openspec/changes/archive/2026-10-28-automate-personal-prompt-experiments/proposal.md) (archived), [extend-personal-prompt-experiment-adapters](../../openspec/changes/archive/2026-10-28-extend-personal-prompt-experiment-adapters/proposal.md) (archived) | Archived slices delivered; phase outcome not established. |
| P08 | SDK/API and tool integration contracts | P01, P02, P06 | In progress | [add-browser-voice-boundary-acceptance](../../openspec/changes/add-browser-voice-boundary-acceptance/proposal.md), [add-harness-sdk-contract-tests](../../openspec/changes/add-harness-sdk-contract-tests/proposal.md), [add-structured-headless-review](../../openspec/changes/add-structured-headless-review/proposal.md) (cut, not implemented), [add-lifecycle-hooks](../../openspec/changes/add-lifecycle-hooks/proposal.md), [repair-jcode-herdr-session-binding](../../openspec/changes/archive/2026-10-21-repair-jcode-herdr-session-binding/proposal.md) (archived) | Archived slices delivered; phase outcome not established. |
| P09 | TUI/onboarding everyday UX | P01, P02, P06 | In progress | [improve-model-picker-browsing](../../openspec/changes/improve-model-picker-browsing/proposal.md) (proposed, not implemented), [add-terminal-accessibility-fallbacks](../../openspec/changes/add-terminal-accessibility-fallbacks/proposal.md), [clarify-first-run-provider-setup](../../openspec/changes/clarify-first-run-provider-setup/proposal.md) (cut, not implemented), [restore-resume-picker-redesign](../../openspec/changes/restore-resume-picker-redesign/proposal.md) (cut, not implemented), [remote-client-link-copy](../../openspec/changes/remote-client-link-copy/proposal.md) (proposed, not implemented), [open-user-clicked-links](../../openspec/changes/archive/2026-11-01-open-user-clicked-links/proposal.md) (archived), [preserve-failed-prompts](../../openspec/changes/archive/2026-10-30-preserve-failed-prompts/proposal.md) (archived); supporting, no completion credit: [show-resolved-worker-model](../../openspec/changes/archive/2026-11-02-show-resolved-worker-model/proposal.md), [swarm-widget-cell-width](../../openspec/changes/archive/2026-11-01-swarm-widget-cell-width/proposal.md) (archived) | Archived slices delivered; phase outcome not established. |
| P10 | Distribution, platforms, performance and diagnostics | P01, P06, P08 | In progress | [record-resource-diagnostics](../../openspec/changes/record-resource-diagnostics/proposal.md), [report-install-channel-recovery](../../openspec/changes/report-install-channel-recovery/proposal.md), [verify-selfdev-reload-identity](../../openspec/changes/verify-selfdev-reload-identity/proposal.md), [verify-terminal-capability-matrix](../../openspec/changes/verify-terminal-capability-matrix/proposal.md) (cut, not implemented), [app-core-clippy-clean](../../openspec/changes/archive/2026-11-01-app-core-clippy-clean/proposal.md) (archived); supporting, no completion credit: [shared-server-crash-resilience](../../openspec/changes/archive/2026-11-02-shared-server-crash-resilience/proposal.md), [swarm-state-async-safety](../../openspec/changes/archive/2026-11-01-swarm-state-async-safety/proposal.md) (archived) | Archived slices delivered; phase outcome not established. |

### P01: Trusted core and acceptance/evidence baseline

**Outcome:** users can distinguish proposed work, authorized execution and verified
completion across TUI/CLI/daemon sessions without new ceremony for simple tasks.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** None. Reuse review/judge semantics; explicitly adopt any local-only
workflow contract before treating it as a committed requirement.
**Real changes:** [admit-judge-feedback-with-bounds](../../openspec/changes/admit-judge-feedback-with-bounds/proposal.md), [bind-acceptance-evidence-to-runtime](../../openspec/changes/bind-acceptance-evidence-to-runtime/proposal.md), [restrict-plan-command-execution](../../openspec/changes/restrict-plan-command-execution/proposal.md) (cut, not implemented); [hot-file-test-coverage](../../openspec/changes/archive/2026-11-02-hot-file-test-coverage/proposal.md) (cut, not implemented); [improve-workflow-suite](../../openspec/changes/archive/2026-10-20-improve-workflow-suite/proposal.md) (archived); supporting, no completion credit: [evaluate-personalized-jcode-prompt](../../openspec/changes/archive/2026-10-27-evaluate-personalized-jcode-prompt/proposal.md) (archived), [automate-personal-prompt-experiments](../../openspec/changes/archive/2026-10-28-automate-personal-prompt-experiments/proposal.md) (archived), [extend-personal-prompt-experiment-adapters](../../openspec/changes/archive/2026-10-28-extend-personal-prompt-experiment-adapters/proposal.md) (archived), [app-core-clippy-clean](../../openspec/changes/archive/2026-11-01-app-core-clippy-clean/proposal.md) (archived); [record-continuation-evidence](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/proposal.md)
([requirements](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/specs/continuation-evidence/spec.md),
[design](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/design.md),
[tasks](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/tasks.md)). This is the
existing OmniRoute/Jcode log-coverage and synthetic offline regression-baseline
slice, not all of P01. It does not add a separate receipt store or require gateway
changes. The native baseline and bounded test-only repairs pass local regression
and strict lint checks and independent implementation review. The [canonical
specification](../../openspec/specs/continuation-evidence/spec.md) is synchronized
and the archive is committed locally. Broader P01 work and remote publication
remain outside the completion claim; no runtime improvement is asserted.
**Gates/evidence:** actual plan-only actions cannot cross the declared capability
boundary; explicit approval transitions work; cancellation and pending required
checks survive session transitions. Verify scoped revision/check attribution,
stale evidence rejection, duplicate versus escalated feedback, and simple-task
negative controls. Prompt-only read-only instructions are not an OS sandbox.

### P02: Discoverable native tools, MCP and skills provenance

**Outcome:** users know what will run, where it came from and which authority owns
it before invocation, including a remote session.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** P01.
**Real changes:** [add-typed-skill-selector-metadata](../../openspec/changes/add-typed-skill-selector-metadata/proposal.md), [discover-native-and-mcp-capabilities](../../openspec/changes/discover-native-and-mcp-capabilities/proposal.md), [show-skill-and-mcp-provenance](../../openspec/changes/show-skill-and-mcp-provenance/proposal.md) (cut, not implemented); [mcp-failure-backoff](../../openspec/changes/archive/2026-11-02-mcp-failure-backoff/proposal.md), [mcp-management-experience](../../openspec/changes/archive/2026-11-02-mcp-management-experience/proposal.md), [lint-skill-name-and-folder](../../openspec/changes/archive/2026-11-02-lint-skill-name-and-folder/proposal.md), [tool-timeout-and-patch-recovery](../../openspec/changes/archive/2026-11-02-tool-timeout-and-patch-recovery/proposal.md), [evaluate-code-search-strategies](../../openspec/changes/evaluate-code-search-strategies/proposal.md). Cut, not implemented; no outcome claimed; supporting, no completion credit: [add-lifecycle-hooks](../../openspec/changes/add-lifecycle-hooks/proposal.md), [improve-workflow-suite](../../openspec/changes/archive/2026-10-20-improve-workflow-suite/proposal.md) (archived).
**Gates/evidence:** actual local/remote TUI rows distinguish native commands, native
tools, MCP tools and skills where each is presented; preserve native same-name
precedence, real descriptions, discovery-source details and legacy-peer unknowns.
Test refresh, overrides, symlinks, collisions, control text and widths 0/1/20/40/80.
Retain safe invocation and explicit installation decisions. Discovery-root labels
and the proposed ten-cell label cap still need feature-level design acceptance.

### P03: Jev semantic poke continuation

**Outcome:** continuation responds to actual progress and unfinished authorized
work, not just open task counts or a persuasive final answer.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** P01. Recommend **poke first, overnight second**.
**Real changes:** [add-structured-continuation-evidence](../../openspec/changes/add-structured-continuation-evidence/proposal.md), [admit-semantic-poke-in-shadow](../../openspec/changes/admit-semantic-poke-in-shadow/proposal.md), [route-jev-supervision-explicitly](../../openspec/changes/route-jev-supervision-explicitly/proposal.md) (cut, not implemented).
**Gates/evidence:** retain the pure deterministic [FollowUpPolicy](../../crates/jcode-todo-policy/src/followup.rs)
with its existing caps/fingerprints. Native code owns authorization, budgets,
stop/cancellation and eligibility; Jev interprets bounded structured actual
check/progress evidence, not self-reported success or arbitrary tool authority.
The current [Jev client](../../crates/jcode-base/src/jev.rs) has Memory/Browser/Voice
purposes only. Supervision needs explicit purpose/capability/provider routing,
not an assumed existing route or silent reuse of another purpose's credentials.
Replay/shadow **this roadmap-authoring session** before enabling decisions, using
redacted evidence and an authorized inference path. Include real waits, blocked
work, changed evidence, premature stopping and required-check negative controls.
Compare matched inputs: both **fewer redundant turns and no missed required checks**
are necessary. Record turn/call cost, latency and uncertain decisions separately.
Failure, malformed output and timeout must never create loops, bypass checks or
grant extra authority. No turn, token, latency or cost savings are claimed yet.

### P04: Swarm collaboration and handoffs

**Outcome:** parallel workers receive executable scope, expose ownership and return
attributable evidence without losing dependencies or clobbering shared work.
**Candidate features, uncut titles only:** None; all candidates cut. Bounded swarm assignment and recovery contracts covered by recover-swarm-workers-after-reload; ownership and blocked-dependency visibility covered by add-swarm-file-claims and surface-worker-stall-and-block-reasons.
**Dependencies:** P01. Discovery and poke need not ship first.
**Real changes:** [add-swarm-handoff-contracts](../../openspec/changes/add-swarm-handoff-contracts/proposal.md) (cut, not implemented); [report-agent-lifecycle-to-herdr](../../openspec/changes/report-agent-lifecycle-to-herdr/proposal.md), [surface-worker-stall-and-block-reasons](../../openspec/changes/surface-worker-stall-and-block-reasons/proposal.md), [focus-worker-herdr-pane](../../openspec/changes/focus-worker-herdr-pane/proposal.md) (cut, not implemented); [return-full-swarm-reports](../../openspec/changes/archive/2026-11-02-return-full-swarm-reports/proposal.md), [queue-messages-to-busy-workers](../../openspec/changes/archive/2026-11-02-queue-messages-to-busy-workers/proposal.md), [show-resolved-worker-model](../../openspec/changes/archive/2026-11-02-show-resolved-worker-model/proposal.md), [add-swarm-file-claims](../../openspec/changes/archive/2026-11-02-add-swarm-file-claims/proposal.md), [recover-swarm-workers-after-reload](../../openspec/changes/archive/2026-11-02-recover-swarm-workers-after-reload/proposal.md), [add-council-tool](../../openspec/changes/archive/2026-11-02-add-council-tool/proposal.md) (cut, not implemented); [swarm-communicate-reliability](../../openspec/changes/archive/2026-11-01-swarm-communicate-reliability/proposal.md) (archived), [swarm-module-split](../../openspec/changes/archive/2026-11-01-swarm-module-split/proposal.md) (archived), [swarm-state-async-safety](../../openspec/changes/archive/2026-11-01-swarm-state-async-safety/proposal.md) (archived), [swarm-widget-cell-width](../../openspec/changes/archive/2026-11-01-swarm-widget-cell-width/proposal.md) (archived). Archived slices are delivered evidence only; no phase outcome claimed; no outcome claimed; supporting, no completion credit: [add-role-model-slots](../../openspec/changes/archive/2026-11-02-add-role-model-slots/proposal.md).
**Gates/evidence:** exercise native todo/swarm delegation with overlapping ownership,
nonconflicting parallel lanes, changed-base revival, stale scope, worker failure,
retry/stop and an attributable complete change range. The coordinator integrates
and verifies worker reports. Preserve root-only spawning, bounded fanout and native
review boundaries; high-risk council arbitration is a required
outcome, delivered by add-council-tool.

### P05: Durable overnight, scheduling and recovery

**Outcome:** users can leave bounded authorized work running, stop active work and
resume after interruption with truthful morning/recovery state.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** P03 decisions, P04 ownership/recovery, P06 budgets. Reuse overnight,
bg and schedule; no parallel daemon. The audit confirms existing persisted records,
not responsive mid-turn cancellation or enforced concurrency from guidance alone.
**Real changes:** [configure-task-and-worktree-policy](../../openspec/changes/configure-task-and-worktree-policy/proposal.md) (proposed, not implemented), [admit-overnight-tasks-on-evidence](../../openspec/changes/admit-overnight-tasks-on-evidence/proposal.md), [enforce-overnight-cancellation-and-limits](../../openspec/changes/enforce-overnight-cancellation-and-limits/proposal.md), [reconcile-overnight-runs-after-restart](../../openspec/changes/reconcile-overnight-runs-after-restart/proposal.md), [recover-missed-scheduled-runs](../../openspec/changes/recover-missed-scheduled-runs/proposal.md) (cut, not implemented); [reattach-background-tasks-after-reload](../../openspec/changes/reattach-background-tasks-after-reload/proposal.md) (cut, not implemented); [shared-server-crash-resilience](../../openspec/changes/archive/2026-11-02-shared-server-crash-resilience/proposal.md). Cut, not implemented; no outcome claimed; supporting, no completion credit: [recover-swarm-workers-after-reload](../../openspec/changes/archive/2026-11-02-recover-swarm-workers-after-reload/proposal.md), [repair-jcode-herdr-session-binding](../../openspec/changes/archive/2026-10-21-repair-jcode-herdr-session-binding/proposal.md) (archived).
**Gates/evidence:** cancel a deliberately active long turn and child/background work
on an isolated real runtime; observe no later launches, consistent terminal state
and restart-safe reconciliation. Test time/token/cost/concurrency caps, offline or
expired auth, crash/restart, missed/overlapping schedules and user-stop precedence.
Adopt P03 semantic supervision only after its replay gate. If no eligible authorized
work remains, wait for a real worker/event or the report boundary rather than invent
new tasks or recheck unchanged receipts. Resume only on a valid new signal.
Recurring schedules are conditional on a concrete recurring-work requirement and
separate activation consent.

### P06: Provider/auth/usage-budget reliability

**Outcome:** supported provider/model paths authenticate predictably, explain failures
and stop at declared spending/resource boundaries without leaking credentials.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** P01. Extend existing provider-doctor and coverage surfaces.
**Real changes:** [classify-provider-auth-failures](../../openspec/changes/classify-provider-auth-failures/proposal.md), [reconcile-usage-budget-accounting](../../openspec/changes/reconcile-usage-budget-accounting/proposal.md), [report-provider-doctor-coverage](../../openspec/changes/report-provider-doctor-coverage/proposal.md) (cut, not implemented); [fallback-on-transient-model-errors](../../openspec/changes/fallback-on-transient-model-errors/proposal.md), [add-role-model-slots](../../openspec/changes/archive/2026-11-02-add-role-model-slots/proposal.md). Cut, not implemented; no outcome claimed.
**Gates/evidence:** real supported flows cover expired/revoked auth, rate limits,
quota exhaustion, unavailable models, cancellation during refresh and interrupted
streams. Verify retry classification, no silent provider/identity change, accounting
across resumed/swarm work and enforcement versus estimated usage. Record supported
provider coverage and unavailable live checks honestly; use the shared usage bar.

### P07: Memory/context quality and privacy

**Outcome:** useful context follows the user without stale retrieval, cross-workspace
leaks or unbounded provider disclosure and prompt growth.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** P01, P06. Reuse [memory architecture](../MEMORY_ARCHITECTURE.md)
and current Jev relevance decisions, not a replacement memory service.
**Real changes:** [add-attributable-memory-recall](../../openspec/changes/add-attributable-memory-recall/proposal.md), [add-compaction-quality-evidence](../../openspec/changes/add-compaction-quality-evidence/proposal.md), [add-memory-retention-controls](../../openspec/changes/add-memory-retention-controls/proposal.md) (cut, not implemented); [native-instruction-file-loading](../../openspec/changes/archive/2026-11-02-native-instruction-file-loading/proposal.md), [workspace-aware-context](../../openspec/changes/workspace-aware-context/proposal.md), [add-prompt-budget-ratchet](../../openspec/changes/add-prompt-budget-ratchet/proposal.md), [session-store-retention](../../openspec/changes/archive/2026-11-02-session-store-retention/proposal.md) (cut, not implemented); [evaluate-personalized-jcode-prompt](../../openspec/changes/archive/2026-10-27-evaluate-personalized-jcode-prompt/proposal.md) (archived), [automate-personal-prompt-experiments](../../openspec/changes/archive/2026-10-28-automate-personal-prompt-experiments/proposal.md) (archived), [extend-personal-prompt-experiment-adapters](../../openspec/changes/archive/2026-10-28-extend-personal-prompt-experiment-adapters/proposal.md) (archived). Archived slices are delivered evidence only; no phase outcome claimed; no outcome claimed.
**Gates/evidence:** compare actual tasks with relevant/irrelevant memories, conflicting
or stale facts, workspace boundaries, compaction and long resumed conversations.
Verify retention/deletion behavior, provenance, bounded payloads, redaction and
unavailable-service fallback. Measure task correctness alongside context cost;
more retrieved text or a structural score is not proof of better outcomes.

### P08: SDK/API/tool integration contracts, including browser/voice

**Outcome:** client and tool authors can rely on versioned, cancellable contracts
without gaining authority through transport or an opaque helper model.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** P01, P02, P06. Reuse [TypeScript SDK](../../sdk/typescript/README.md),
shared Rust SDK/harness API, existing JSON/NDJSON and [browser handoff](../BROWSER_FAST_AGENT.md).
**Real changes:** [add-browser-voice-boundary-acceptance](../../openspec/changes/add-browser-voice-boundary-acceptance/proposal.md), [add-harness-sdk-contract-tests](../../openspec/changes/add-harness-sdk-contract-tests/proposal.md), [add-structured-headless-review](../../openspec/changes/add-structured-headless-review/proposal.md) (cut, not implemented); [add-lifecycle-hooks](../../openspec/changes/add-lifecycle-hooks/proposal.md) (cut, not implemented); [repair-jcode-herdr-session-binding](../../openspec/changes/archive/2026-10-21-repair-jcode-herdr-session-binding/proposal.md) (archived). Archived slice is delivered evidence only; no phase outcome claimed; no outcome claimed.
**Gates/evidence:** actual CLI and Rust/TypeScript clients cover version drift,
reconnect, streaming completion/error ordering, malformed messages and cancellation.
Headless review binds verdicts to revision/check evidence and handles fail,
inconclusive and budget/cancel outcomes without free-text PASS parsing.
Browser tab/goal/action scope and voice capture/intent remain explicit, bounded and
consent-safe; test help hand-back and unavailable services. Desktop is a consumer
compatibility boundary here, not an owned application delivery.

### P09: TUI/onboarding everyday UX

**Outcome:** new and daily users can configure, discover, resume and inspect work
without misleading chrome, invisible failures or surprising external launches.
**Candidate features, uncut titles only:** None; all candidates cut. Session/resume and work-status clarity covered by restore-resume-picker-redesign and surface-worker-stall-and-block-reasons.
**Real changes:** [improve-model-picker-browsing](../../openspec/changes/improve-model-picker-browsing/proposal.md) (proposed, not implemented), [add-terminal-accessibility-fallbacks](../../openspec/changes/add-terminal-accessibility-fallbacks/proposal.md), [clarify-first-run-provider-setup](../../openspec/changes/clarify-first-run-provider-setup/proposal.md) (cut, not implemented); [restore-resume-picker-redesign](../../openspec/changes/restore-resume-picker-redesign/proposal.md) (cut, not implemented); [remote-client-link-copy](../../openspec/changes/remote-client-link-copy/proposal.md) (proposed, not implemented); [open-user-clicked-links](../../openspec/changes/archive/2026-11-01-open-user-clicked-links/proposal.md) (archived), [preserve-failed-prompts](../../openspec/changes/archive/2026-10-30-preserve-failed-prompts/proposal.md) (archived). Archived slices are delivered evidence only; no phase outcome claimed. Supporting, no completion credit: [show-resolved-worker-model](../../openspec/changes/archive/2026-11-02-show-resolved-worker-model/proposal.md), [swarm-widget-cell-width](../../openspec/changes/archive/2026-11-01-swarm-widget-cell-width/proposal.md) (archived).
**Dependencies:** P01, P02, P06 for phase-wide acceptance, not retroactive changes
to the existing link feature's approved scope.
**Gates/evidence:** exercise first run, auth failure, session selection/resume,
keyboard/mouse navigation, copy/selection, links and error recovery on actual TUI
frames. Include narrow/Unicode terminals, monochrome and existing image/rich-text
fallbacks. Explicit link gestures must preserve opt-outs and agent/harness suppression.

### P10: Distribution/platform support/performance/diagnostics

**Outcome:** supported installs and upgrades are recoverable, platform behavior is
clear and performance/diagnostic claims reproduce on the actual serving build.
**Candidate features, uncut titles only:** None; all candidates cut.
**Dependencies:** P01, P06, P08. Build on existing install/platform guidance.
**Real changes:** [record-resource-diagnostics](../../openspec/changes/record-resource-diagnostics/proposal.md), [report-install-channel-recovery](../../openspec/changes/report-install-channel-recovery/proposal.md), [verify-selfdev-reload-identity](../../openspec/changes/verify-selfdev-reload-identity/proposal.md), [verify-terminal-capability-matrix](../../openspec/changes/verify-terminal-capability-matrix/proposal.md) (cut, not implemented); [app-core-clippy-clean](../../openspec/changes/archive/2026-11-01-app-core-clippy-clean/proposal.md) (archived). Archived slice is delivered evidence only; no phase outcome claimed. Supporting, no completion credit: [shared-server-crash-resilience](../../openspec/changes/archive/2026-11-02-shared-server-crash-resilience/proposal.md), [swarm-state-async-safety](../../openspec/changes/archive/2026-11-01-swarm-state-async-safety/proposal.md) (archived).
**Gates/evidence:** verify declared Linux/macOS/Windows support paths, launcher/channel
identity, failed upgrade/rollback and isolated daemon version checks. Test terminal
capability fallbacks and self-dev reload/session continuity without disturbing the
shared daemon. Reproduce startup/input latency, memory per session and concurrent
load with recorded hardware/workload; redact diagnostic exports. Define targets in
the selected feature, not by borrowing historical README benchmark numbers.

## Factory research: existing phase refinements

The [2026-10-29 adversarial placement review](../research/factory-harness-knowledge-placement-2026-10-29.md)
accounts for nineteen shipped Droid CLI references, native overlap and opposing
cases. This adds research input within the existing documentation framework;
phase IDs/status, feature bindings and execution authority remain unchanged.

**Pre-cutover delivery policy:** Skull is an independent, divergent implementation
in a custom fork of Jcode upstream. While Jcode remains the owner's working harness,
pursue useful general harness improvements here as well as evaluating their fit
for Skull; do not reserve their benefit for a future cutover. Each implementation
owns its scope, integration and acceptance. No ongoing baseline inheritance,
automatic synchronization, feature parity, shared runtime or cutover date is implied.
Skull-specific capabilities remain owned by Skull.

| Existing phase | Refinement / acceptance focus | Boundary |
| --- | --- | --- |
| P01 / P04 | Approved obligation → scoped assignment → attributable handoff → independent revision-bound acceptance. Reject stale-owner reports and worker-supplied PASS as acceptance. | Reuse native swarm/todo/review; no second ledger or universal extra reviewer. Product work identity is not a harness session ID. |
| P05 / P06 | Keep occurrence identity, versioned instructions, checkpoints, notes, output and usage coverage distinct. Unreadable identity holds; ambiguous send/crash reconciles; missing usage stays unknown. | Existing Jcode automation only; no parallel scheduler, imported vendor limits or activation authority. Preserve existing P05 prerequisites. |
| P07 | Compare anchored/bounded compaction and indexed pull-on-demand against native context/memory on obligation retention, stale swaps, cancellation, scope, correction and total task cost. | Preserve source/history under actual retention rules. Retired retrieval visibility is not evidence deletion; missing read accounting is not disuse. No new memory engine selected. |
| P08, if a consumer is selected | Carry product work/attempt/configuration references, bounded evidence, cancellation and usage coverage through a versioned adapter. | Semantic research seam only; no new API or required integration. Consumers retain their own authority and durable state. |

Skull owns its Linear mission/attempt/outcome/decision state and fenced acceptance;
Nomi owns planned jobs, source lifecycle, knowledge review and recurring K01 catch-up.
Sharing these patterns selects no common daemon, database, corpus or scheduler.
Nomi K01 does not inherit Jcode P05's semantic-poke/overnight dependency chain;
Skull's one-shot/manual-rearm intake policy remains distinct. Consumer integration,
if later justified, requires its own compatibility, permission and recovery checks.
Static Factory code and bundled instructions pass none of these gates. Exact
workloads, versions, independent evaluation, useful gain/non-regression, limits,
provider/egress authority, target and cleanup remain future feature-level bindings.

## Phase lifecycle

### Cut real feature IDs and maintain the map

Stable phase IDs P01–P10 are never reused or silently renumbered. New phases append
new IDs. A feature is exactly one real `openspec/changes/<slug>/` change: its actual
kebab-case slug is the ID, not a made-up issue number. Several feature IDs may serve
one phase; choose one primary phase and record supporting phases without duplicating
tasks or completion credit. Uncut titles above are options, not reservations.

Worked procedure, using existing semantic workflows rather than invented CLI flags:

1. Select one bounded candidate, for example P03's *Structured continuation evidence*.
   Use `explore` to trace existing policy/callers, overlap, consent and unknowns.
   Agree the owned outcome, acceptance owner and dependencies before calling it ready.
2. Use `feature` / native OpenSpec authoring to create or update the actual change.
   It owns proposal, spec delta, design when needed and executable `tasks.md`.
   The authoring workflow establishes the real slug; no example slug is pre-created here.
3. After the real proposal/task artifacts are committed, replace the relevant
   “Not cut” with their actual links, or append another real ID. Check that a clean
   checkout includes them; a local ignored directory is not a durable binding.
   Keep other candidate titles visibly uncut.
   Scope/compatibility/security decisions and expected checks stay in those artifacts.
4. Authorized `apply` implements that feature. Native verify and fresh final-diff
   gates precede independent review; repair findings and rerun affected gates/review.
   Reconcile native tasks, commit verified scope, archive only after required gates
   and commits, then confirm archive/history and required persistence. Publication
   still requires its own applicable authority. Roadmap editing authorizes none of this.
5. Derive phase status and evidence from that closeout, never from chat, worker
   reports, checked tasks or an archive directory alone. After archival, update
   links to the actual archived proposal/tasks while retaining the original ID.
   The index carries links and a concise outcome, not copied implementation steps
   or task-level statuses.

**Readiness and completion:** a cut feature is not automatically ready. Its native
artifacts must resolve required decisions, prerequisites and executable acceptance.
Use Planned, In progress, Blocked or Complete only when the underlying record
supports that rollup. A phase is Complete only for a nonempty, explicitly accepted
scope whose owned outcomes and feature closeouts all pass, including integration
acceptance. Uncut required outcomes prevent completion. Deferred, rejected or
superseded archives are dispositions, not delivered outcomes or completion credit.

**Future evidence fields:** when evidence exists, add accepted phase scope/revision,
acceptance owner and verifier, real change IDs, serving artifact/revision, exact
acceptance commands/results and limitations, independent review reference,
commit/archive/persistence references, and the outcome decision. Link the canonical
record instead of copying logs. Until then “Not established” remains truthful.

**Expansion and priority:** record material scope changes with previous scope,
reason, acceptance owner and affected feature links; preserve past completion
boundaries and explicitly reopen a phase if its accepted scope grows. Do not erase
history by dropping failed gates or parking required work. Proposed near-term order
is P01 plus P02, then P03 before P05 semantic automation; P04/P06 can progress in
parallel. Final product priority, read-only enforcement strength and which optional
candidates to cut remain feature-level decisions, not commitments made by this map.
