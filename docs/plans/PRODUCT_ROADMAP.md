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
source contracts linked below. Documentation and source describe intent or current
architecture; neither README figures nor a names-only inventory prove fresh runtime acceptance.

Swarm/todo, review/judge, plan/poke, overnight/bg/schedule, skills/MCP, structured
headless runs, resume, provider diagnostics and self-dev are existing capabilities,
not newly completed roadmap features. Improve their contracts before adding APIs.

**Durable baseline:** no OpenSpec changes are tracked in this checkout's Git index
at roadmap creation. Its existing advisory plans and OpenSpec specs/archives are
local-only material. The observed local `open-user-clicked-links` change has checked
tasks, but final acceptance/closeout is unverified here. It is not a durable P09
binding or completion credit. Other local archives similarly do not complete a phase.
Do not force-add private historical artifacts merely to populate this roadmap.

Future feature IDs become bindings only after their real proposal/task artifacts
are committed and available in a clean checkout. Existing local work can inform
that authoring after its scope, evidence and publication boundaries are reviewed.
All ten phases started Planned; no implementation, cost saving or outcome improvement
was claimed by creating the roadmap. P01 now has one committed proposal,
`record-continuation-evidence`; implementation is archived locally, with final
lifecycle/history and persistence confirmation still pending.

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
| P01 | Trusted core and evidence baseline | None | In progress | [record-continuation-evidence](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/proposal.md) | Baseline archived locally; persistence pending |
| P02 | Discoverable native tools, MCP and skills | P01 | Planned | Not cut | Not established |
| P03 | Jev semantic poke continuation | P01 | Planned | Not cut | Not established |
| P04 | Swarm collaboration and handoffs | P01 | Planned | Not cut | Not established |
| P05 | Durable overnight, scheduling and recovery | P03, P04, P06 | Planned | Not cut | Not established |
| P06 | Provider, auth and usage-budget reliability | P01 | Planned | Not cut | Not established |
| P07 | Memory/context quality and privacy | P01, P06 | Planned | Not cut | Not established |
| P08 | SDK/API and tool integration contracts | P01, P02, P06 | Planned | Not cut | Not established |
| P09 | TUI/onboarding everyday UX | P01, P02, P06 | Planned | Not cut | Not established |
| P10 | Distribution, platforms, performance and diagnostics | P01, P06, P08 | Planned | Not cut | Not established |

### P01: Trusted core and acceptance/evidence baseline

**Outcome:** users can distinguish proposed work, authorized execution and verified
completion across TUI/CLI/daemon sessions without new ceremony for simple tasks.
**Candidate features, uncut titles only:** Restricted planning execution;
Runtime-bound acceptance evidence; Bounded judge/review feedback admission.
**Dependencies:** None. Reuse review/judge semantics; explicitly adopt any local-only
workflow contract before treating it as a committed requirement.
**Real change:** [record-continuation-evidence](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/proposal.md)
([requirements](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/specs/continuation-evidence/spec.md),
[design](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/design.md),
[tasks](../../openspec/changes/archive/2026-10-26-record-continuation-evidence/tasks.md)). This is the
existing OmniRoute/Jcode log-coverage and synthetic offline regression-baseline
slice, not all of P01. It does not add a separate receipt store or require gateway
changes. The native baseline and bounded test-only repairs pass local regression
and strict lint checks and independent implementation review. The [canonical
specification](../../openspec/specs/continuation-evidence/spec.md) is synchronized;
final lifecycle/history and persistence confirmation remain pending. No runtime
improvement or completion of all P01 is claimed.
**Gates/evidence:** actual plan-only actions cannot cross the declared capability
boundary; explicit approval transitions work; cancellation and pending required
checks survive session transitions. Verify scoped revision/check attribution,
stale evidence rejection, duplicate versus escalated feedback, and simple-task
negative controls. Prompt-only read-only instructions are not an OS sandbox.

### P02: Discoverable native tools, MCP and skills provenance

**Outcome:** users know what will run, where it came from and which authority owns
it before invocation, including a remote session.
**Candidate features, uncut titles only:** Typed skill-selector metadata and styling;
Native-tool and MCP capability discovery; Skill/MCP provenance and trust details.
**Dependencies:** P01.
**Gates/evidence:** actual local/remote TUI rows distinguish native commands, native
tools, MCP tools and skills where each is presented; preserve native same-name
precedence, real descriptions, discovery-source details and legacy-peer unknowns.
Test refresh, overrides, symlinks, collisions, control text and widths 0/1/20/40/80.
Retain safe invocation and explicit installation decisions. Discovery-root labels
and the proposed ten-cell label cap still need feature-level design acceptance.

### P03: Jev semantic poke continuation

**Outcome:** continuation responds to actual progress and unfinished authorized
work, not just open task counts or a persuasive final answer.
**Candidate features, uncut titles only:** Structured continuation evidence;
Explicit Jev supervision routing; Shadow-evaluated semantic poke admission.
**Dependencies:** P01. Recommend **poke first, overnight second**.
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
**Candidate features, uncut titles only:** Bounded swarm assignment and recovery
contracts; Scope/base/evidence handoffs; Ownership and blocked-dependency visibility.
**Dependencies:** P01. Discovery and poke need not ship first.
**Gates/evidence:** exercise native todo/swarm delegation with overlapping ownership,
nonconflicting parallel lanes, changed-base revival, stale scope, worker failure,
retry/stop and an attributable complete change range. The coordinator integrates
and verifies worker reports. Preserve root-only spawning, bounded fanout and native
review boundaries; optional high-risk arbitration requires a demonstrated gap.

### P05: Durable overnight, scheduling and recovery

**Outcome:** users can leave bounded authorized work running, stop active work and
resume after interruption with truthful morning/recovery state.
**Candidate features, uncut titles only:** Evidence-aware overnight task admission
and waiting; Responsive overnight cancellation and enforceable limits;
Restart-safe supervised work; Inspectable scheduled-run recovery.
**Dependencies:** P03 decisions, P04 ownership/recovery, P06 budgets. Reuse overnight,
bg and schedule; no parallel daemon. The audit confirms existing persisted records,
not responsive mid-turn cancellation or enforced concurrency from guidance alone.
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
**Candidate features, uncut titles only:** Provider/auth failure recovery contracts;
Usage and budget reconciliation; Provider coverage and diagnostic parity.
**Dependencies:** P01. Extend existing provider-doctor and coverage surfaces.
**Gates/evidence:** real supported flows cover expired/revoked auth, rate limits,
quota exhaustion, unavailable models, cancellation during refresh and interrupted
streams. Verify retry classification, no silent provider/identity change, accounting
across resumed/swarm work and enforcement versus estimated usage. Record supported
provider coverage and unavailable live checks honestly; use the shared usage bar.

### P07: Memory/context quality and privacy

**Outcome:** useful context follows the user without stale retrieval, cross-workspace
leaks or unbounded provider disclosure and prompt growth.
**Candidate features, uncut titles only:** Attributable bounded memory recall;
Context/compaction quality evidence; Memory retention and privacy controls.
**Dependencies:** P01, P06. Reuse [memory architecture](../MEMORY_ARCHITECTURE.md)
and current Jev relevance decisions, not a replacement memory service.
**Gates/evidence:** compare actual tasks with relevant/irrelevant memories, conflicting
or stale facts, workspace boundaries, compaction and long resumed conversations.
Verify retention/deletion behavior, provenance, bounded payloads, redaction and
unavailable-service fallback. Measure task correctness alongside context cost;
more retrieved text or a structural score is not proof of better outcomes.

### P08: SDK/API/tool integration contracts, including browser/voice

**Outcome:** client and tool authors can rely on versioned, cancellable contracts
without gaining authority through transport or an opaque helper model.
**Candidate features, uncut titles only:** Harness/SDK compatibility and event contracts;
Structured headless review; Browser/voice boundary acceptance.
**Dependencies:** P01, P02, P06. Reuse [TypeScript SDK](../../sdk/typescript/README.md),
shared Rust SDK/harness API, existing JSON/NDJSON and [browser handoff](../BROWSER_FAST_AGENT.md).
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
**Candidate features, uncut titles only:** First-run and provider setup clarity;
Session/resume and work-status clarity; Accessible terminal interaction parity.
**Real changes:** Not cut. The local-only link-opening observation above is not a committed phase binding.
**Dependencies:** P01, P02, P06 for phase-wide acceptance, not retroactive changes
to the existing link feature's approved scope.
**Gates/evidence:** exercise first run, auth failure, session selection/resume,
keyboard/mouse navigation, copy/selection, links and error recovery on actual TUI
frames. Include narrow/Unicode terminals, monochrome and existing image/rich-text
fallbacks. Explicit link gestures must preserve opt-outs and agent/harness suppression.

### P10: Distribution/platform support/performance/diagnostics

**Outcome:** supported installs and upgrades are recoverable, platform behavior is
clear and performance/diagnostic claims reproduce on the actual serving build.
**Candidate features, uncut titles only:** Install/update/channel recovery;
Platform and terminal capability acceptance; Reproducible resource diagnostics;
Self-dev build/reload verification.
**Dependencies:** P01, P06, P08. Build on existing install/platform guidance.
**Gates/evidence:** verify declared Linux/macOS/Windows support paths, launcher/channel
identity, failed upgrade/rollback and isolated daemon version checks. Test terminal
capability fallbacks and self-dev reload/session continuity without disturbing the
shared daemon. Reproduce startup/input latency, memory per session and concurrent
load with recorded hardware/workload; redact diagnostic exports. Define targets in
the selected feature, not by borrowing historical README benchmark numbers.

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
