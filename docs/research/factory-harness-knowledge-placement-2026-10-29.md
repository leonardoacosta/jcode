# Factory patterns: Jcode, Skull and Nomi placement

2026-10-29 · adversarial swarm review · planning/reference only.

## Conclusion and evidence boundary

The first group primarily informs reusable work/continuity contracts and Skull's
mission workflow. The second primarily informs state and knowledge lifecycle,
including Nomi's K01. Neither group belongs exclusively to one layer. Share narrow
mechanisms and evidence conventions where a measured gap warrants it; retain one
authority for each product's work, permissions, schedules and knowledge.

Target: the nineteen references in the owner's Factory Droid review request,
compared with the three current roadmaps. Claim under challenge: mission patterns
belong in Jcode/Skull, while automation/memory patterns should serve both products.
Attack surface: documented ownership, native source, persistence and consumer seams.
Evidence allowed: read-only source/artifact review and local documentation edits.
Excluded: implementation, target execution, inference, installations, source upload,
service changes, deployment and OpenSpec proposal/spec/task/config changes.
Success means an attributable owner and existing delivery lane for every reference,
with concrete counterexamples and future checks; it does not mean runtime acceptance.
This architecture review uses the adversarial claim/evidence contract, rather than
claiming a merge verdict, product release audit or negative-test run.

The [exact-build audit][audit] retains the publisher download/checksum, custom Bun
extraction, byte ranges and actual REA Evidence IDs. Artifact: Linux x64 Droid
`0.251.0`, SHA256
`b878737e849b1d484a4894421b8f0345ad7257ab529de32aafded3e1ba200929`.
Publisher-checksum agreement is not independent authentication. REA 6.5.0 parsed
14 selected JavaScript files, not the entire application; graph coverage is partial.
Code branches, prompt construction and bundled guidance are separate evidence types;
none proves runtime reachability, availability, enforcement or useful gain.
FM/FK labels below identify this review's rows, not REA Evidence IDs.

Three read-only reviewers examined harness reuse, Skull authority and Nomi knowledge
seams. The coordinator checked their conclusions against the existing roadmaps and
native documentation/source. No separate model/provider benchmark or runtime trial
was performed. [Starting repository snapshot][snapshot] records heads, dirty state
and file hashes; unrelated active Jcode work is outside this review.

## Existing framework and native baseline

Jcode already has a [documentation index](../README.md), a
[P01–P10 roadmap](../plans/PRODUCT_ROADMAP.md), architecture docs, research and audits.
Extend that framework with this reference; do not create a second roadmap, source
registry or task ledger. The roadmap's actual feature bindings remain authoritative.

Jcode's [swarm guide](../SWARM_ARCHITECTURE.md#completion-report-policy) already
describes worker reports containing outcome, validation and blockers. Its
[mission record](../../crates/jcode-app-core/src/mission.rs:41) stores a session ID,
objective, intent, success criteria, validation plan and checkpoints. Those fields
are source evidence, not proof of complete dispatch/acceptance enforcement or a
portable product mission protocol. [Memory architecture](../MEMORY_ARCHITECTURE.md#storage-and-lifetime)
already documents scoped project/global records, supersession, local search and
scope-checked queued recall. Its optional remote recall has its own disclosure
boundary. Do not describe these existing capabilities as missing Factory features.

Skull's [roadmap][skull-roadmap] and [domain contracts][skull-domains] already own
mission/attempt recovery, independent acceptance, context and knowledge lanes.
Nomi's [roadmap][nomi-roadmap] and [knowledge architecture][nomi-architecture] own
the planned foundation jobs, mail/calendar and required K01 library → current-work
recommendation → owner-controlled catch-up. Nomi is still a synthetic desktop spike;
its planned backend/job contracts are not implemented by these references.

## Ownership and integration

**Owner clarification — independent fork and pre-cutover delivery:** Skull is a
fully independent, divergent implementation in its custom fork of Jcode upstream.
Fork ancestry establishes no ongoing Jcode baseline, feature-parity obligation,
automatic synchronization or shared runtime dependency. Until the owner cuts over
to Skull, useful general harness improvements should also be pursued in the active
Jcode harness rather than reserved for Skull. Assess and adapt each implementation
against its own contracts; Jcode delivery neither delivers nor accepts the Skull
counterpart. Skull-specific work remains in Skull. This establishes no cutover date,
migration plan or implementation authority.

| Concern | Authority / smallest reuse | Boundary |
| --- | --- | --- |
| Execution context, worker report, compaction and usage | Independently owned native mechanisms: improve active Jcode P01/P04/P06/P07 and Skull's fork where each benefits. P08 consumer contracts remain optional. | Reuse is a per-change design/code decision, not ongoing inheritance. Product IDs may be carried as references; a harness-local mission/session record cannot accept or replace a product mission/job. No shared runtime or database selected. |
| Mission, attempt, outcome, decision and allowed effects | Skull's canonical Linear state, fenced attempt and permitted local journal/checkpoint. | Worker reports are inputs to acceptance, not acceptance authority. Nomi recommendations cannot create missions, rearm watches or launch workers without the relevant grant. |
| Source library, preserved human knowledge, jobs and catch-up | Nomi's planned workflow owner; Recon preservation discipline; native retrieval baseline, conditional Supermemory projection. | No second corpus/scheduler. Company Brain contributes patterns. Jcode memory cannot become the sole source library or silently ingest mailbox/private-world data. |
| Consumer seam, if eventually needed | Consumer supplies scoped work/attempt/configuration references, permitted input, evidence pointers and limits; runtime returns attributable status, partial results, usage coverage and cancellation state. | Proposed semantic seam only: no wire schema/SDK or hard cross-repository dependency selected. Unsupported semantics must hold or remain visibly unavailable, not silently degrade. |

These boundaries keep jobs independently recoverable, not products fragmented.
Nomi's ordinary library/jobs/mail/calendar must not require Jcode's daemon or inherit
P05 → P03 semantic-poke prerequisites. Skull does not depend on Nomi's knowledge
backend or Jcode runtime for its local mission. If Nomi selects an optional Jcode executor later,
its invocation needs version/capability, permissions, cancellation and recovery
acceptance; lose that executor without losing the owning product's durable state.
Sharing a pattern is not selecting that executor.

## First group: work, acceptance and compaction

All seven owner-supplied references are accounted for below. Placements are design
recommendations within existing lanes, not new feature IDs or completion credit.

| ID / exact shipped reference | Jcode placement | Skull / Nomi application and limit |
| --- | --- | --- |
| FM01 — [worker prompt builder][fm01], line 77; prompt construction | P04: bounded assignment tied to obligations and checks. | Skull M05/M06/M08: bind assertions to mission/intent/attempt revision. Nomi K01: bound a research stage by source scope and useful-output criteria; do not invent a coding milestone for every capture. |
| FM02 — [mission guidance][fm02]; bundled instructions | P01/P04: keep approved obligations, work and validation aligned when scope changes. | Skull M05/M06/V02: approval changes invalidate stale checks. Nomi: revise the research question/configuration explicitly; changing evidence does not authorize new actions. Guidance is not enforcement. |
| FM03 — [EndFeatureRun][fm03], line 226; code | P01/P04: attributable active-worker completion and structured handoff, then separate verification. | Skull R03–R07/M05/M08: fence/check the active attempt and retain uncertainty. Nomi foundation/K01: job completion, useful knowledge and external delivery remain distinct. Caller-supplied `validatorsPassed` is not independent evidence. |
| FM04 — [manual compaction][fm04], line 2; code | P07: retain task/ownership identity across permitted context changes. | Skull W01/W02: Linear identity survives a permitted session replacement; ownership/fencing governs continuation. Nomi: occurrence identity survives summary replacement. Reject copying Factory's rule that owner-session identity forbids a Missions 2.0 new-session handoff. |
| FM05 — [compaction implementation][fm05], line 43; code | P07: anchored prior summary + delta is a comparison candidate against native compaction. | Skull K/W05/B03/B04; Nomi K01 working context: retain unfinished obligations, source/artifact pointers and configuration revisions. Reject stale replacements; preserve original-history fallback. No quality or concurrency guarantee established. |
| FM06 — [agent compaction][fm06]; code | P06/P07: bounded input and attributable summary usage, including discarded work. | Skull O02/W05 and Nomi whole-workflow accounting include retries, re-fetches and failed summaries. No vendor byte/token settings or cost gain transfers. |
| FM07 — [Mission Ledger][fm07], line 26; bundled instructions | P01/P04: approved promise, revisable plan, progress, validation and independent review stay distinct. | Skull M05/M06/M08/V02: represent those views through existing Linear/OpenSpec authorities, not another ledger. Nomi K01: separate requested outcome, evidence, review and action; no mission platform required. |

## Second group: automation and knowledge lifecycle

| ID / exact shipped reference | Jcode placement | Skull / Nomi application and limit |
| --- | --- | --- |
| FK01 — [automation persistence][fk01], line 2; code | P05: inspect native run identity/checkpoint persistence before adding mechanisms. | Skull R12/W02/I03 and Nomi foundation/K01 retain their own durable records. Temp-file/rename is not a demonstrated transaction, concurrency or power-loss guarantee. |
| FK02 — [creation path][fk02], line 4; code | P05/P07: distinguish versioned instructions/configuration, run state, notes and reports. | Skull M06/M07/R07 and Nomi jobs bind actual configuration/source revisions. A generated memory/report cannot silently rewrite future instructions or widen grants. Do not copy Factory's directory layout. |
| FK03 — [identity resolution][fk03], line 4; code | P05: unreadable existing identity stays unresolved. | Skull holds uncertain ownership; Nomi holds/reconciles the occurrence rather than silently registering a second job. Missing state is not proof of a fresh run. |
| FK04 — [guard messages][fk04], line 2; prompt text | P01/P05: separate daemon-owned state from agent-authored output. | Skull R07 and Nomi API/worker enforcement must actually restrict writes/children. A warning string or filesystem convention is not confinement. |
| FK05 — [workstream memory code][fk05], line 37; code | P07: compare bounded index/pull and immutable note identity with existing scoped memory. | Skull K/W06–W09/B08 and Nomi K01: source-backed reusable notes, distinct recent-run context. Retirement replaces the note body; preserve originals/corrections independently under their retention/access rules. Do not transplant lock-expiry rules or vendor limits. |
| FK06 — [memory contract][fk06], line 485; bundled instructions | P07: situation-specific retrieval cues and explicit scope, not a global profile shared by products. | Skull separates curated knowledge from task truth; Nomi separates reusable lessons from events/human notes. Native files/wiki stay the baseline; no selected memory engine. |
| FK07 — [memory-read accounting][fk07], line 10; code | P06/P07: keep result delivery separate from successful usage recording. | Skull O01–O03 and Nomi run accounting preserve coverage/unknown values. Missing reads do not prove disuse or authorize deletion; an accounting gap must not silently relax an enabled hard budget. |
| FK08 — [workstream listener][fk08], line 119; bundled instructions | P05/P08: stable event/dispatch identity and reconciliation, if needed by an actual consumer. | Skull I03/I04 keeps one-shot/manual rearm; Nomi K01 owns recurring catch-up. Send → record → cursor leaves an unverified crash window; receiver idempotence/reconciliation is needed before any duplicate-safe delivery claim. |
| FK09 — [overlap guidance][fk09], line 66; bundled instructions | P04/P05: source/external identity helps distinguish repeat work from new evidence. | Skull intake attaches evidence without duplicate mission/owner launch; Nomi links new revisions to the same topic/recommendation. Similar wording is neither a reliable dedupe key nor permission to merge account grants. |
| FK10 — [shared context][fk10], line 20; bundled instructions | P02/P07/P08: explicit audience/scope/version and untrusted-content boundary. | Skull M07/W09 and Nomi source/retrieval seams filter access before retrieval, invalidate revoked context and preserve citations. Shared UI/search does not merge Personal/Priceless/Brown or account authority. Server enforcement remains unverified. |
| FK11 — [Wiki skill][fk11], line 28; bundled instructions | No new Jcode Wiki feature justified; P07 may reuse revision/freshness principles. | Primarily Nomi K01/Recon: source-change-driven derived refresh, human edits protected, correction/revocation propagated. Skull W06 can consume scoped knowledge without owning the Wiki. Incremental instructions do not prove correct regeneration. |
| FK12 — [activation guidance][fk12], line 9; bundled instructions | P05: reviewed run/configuration is separate from schedule activation. | Nomi owns sources/cadence/location/quiet hours and recurring opt-in; Skull preserves I03 and its distinct maintenance policies. Successful test output grants neither scheduling nor external-write authority. |

## Swarm cross-examination and disposition

These are counterexamples to the proposed placement, not newly reproduced defects.

| Challenge | Strongest objection / trigger | Revised position |
| --- | --- | --- |
| Put the entire first group in the harness? | Copy session-as-mission identity, then recover a Skull mission in a new session: either continuation breaks or a second mission is created. | Jcode carries context/assignment evidence; the consuming product owns stable work identity and authorized attempt transitions. Adapt FM04's invariant, reject its product-specific identity restriction. |
| Share all of the second group as infrastructure? | Nomi catch-up inherits a Jcode schedule while Nomi jobs also schedule it, or it waits for semantic poke to ship. | Reuse principles/adapters only where needed. One Nomi occurrence owner; no mandatory shared daemon or Jcode phase dependency. Preserve Skull's different one-shot policy. |
| Unify memory to avoid fragmentation? | A private mailbox note enters a global harness memory, then another world retrieves it; retiring it destroys its only original. | One Nomi experience does not merge grants or evidence/derived-state authorities. Scoped projections, original preservation and explicit export/revocation handling come first. |
| Trust completion flags and guards? | Worker reports success on stale checks, or an agent bypasses a daemon-owned-state warning. | Independent revision-bound checks and actual effect enforcement remain required; report/guard text supplies no authority. |
| Are these nineteen missing features? | Jcode already has swarm reports, persisted missions and scoped memory; adding counterparts duplicates ownership without fixing a demonstrated gap. | Refine existing P01/P04/P05/P06/P07/P08 and Skull/Nomi gates. No port, service, framework, provider or code-copy selection follows. |

## Future discriminating checks and delivery placement

These are acceptance cases for later scoped feature authoring, not executable tasks,
passing results or additional dependencies on currently accepted slices.

| Existing owner / lane | Trigger → required observation | Evidence still needed |
| --- | --- | --- |
| Jcode P01/P04; Skull M05/R03–R07 | Worker completes with stale check revision or lost ownership → handoff remains attributable, acceptance/new effects are refused, unresolved work stays visible. | Exact source/runtime/caller tracing; independent check attribution; overlapping-owner and stale-result schedules. |
| Jcode P07; Skull K/W05; Nomi K01 | User changes a constraint/configuration while a summary is pending, then cancels/restarts → stale swap cannot erase the change, work identity or unknown effects; permitted original context can be rehydrated. | Native foreground baseline first; independent recall/artifact/continuation/decision cases, full cost and crash/cancel evidence before selecting async compaction. |
| Jcode P05/P08 where consumed; Skull D/I04; Nomi foundation/K01 | Duplicate trigger, unreadable identity, crash after accepted send but before local record/cursor, or manual/scheduled overlap → one owning logical occurrence/effect, or explicit reconciliation hold without blind resend. | Actual receiver semantics, concurrency/commit points and target-specific idempotence; no exactly-once provider guarantee inferred. |
| Jcode P07; Skull K/B08; Nomi K01 | Retire/correct/revoke a source, lose a read metric, or switch account/world with queued recall → no unauthorized/stale injection, no inference of disuse from missing metrics, history preserved only under actual retention policy. | Native retrieval comparison, source/revision mapping, access enforcement, deletion/export/restore and measured usefulness. |
| Nomi K01 and optional executor integration | Incremental refresh meets a human edit, an injected instruction, changed sources or unavailable Jcode/index/inference → preserve notes/library/core mail, surface conflicts/coverage and abstain where necessary; no new schedule or write is activated. | Foundation/provider prerequisites, K01 delivery binding, protected usefulness evaluation, activation and outage/restart acceptance. |

Jcode: refine existing phase candidates, with P01/P04 handoff/evidence work and
P06 accounting available independently of a new memory mechanism; P07 owns context
comparisons; P05 owns Jcode automation only; P08 is conditional consumer compatibility.
Keep phase status and real change IDs unchanged.

Skull: preserve accepted G1 local isolation and exact constrained Linux local-fixture
G2 acceptance. G3 independent/human/terminal gates and broad live Linear reliability
remain open. K and D keep their own gates; advanced context does not become a G3
prerequisite. Q-C01/Q-S01/Q-J01/Q-I01 remain **NOT RUN** with every existing family gate.

Nomi: refine foundation jobs and K01, required on Arch before platform expansion.
Do not defer the approved knowledge catch-up into optional mailbox briefings. K01's
implementation binding remains unassigned. No Jcode dependency or new backend is
selected. Recon owns preservation discipline; Supermemory remains conditional and
derived; Company Brain remains pattern input rather than another mandatory runtime.

No owner-only decision blocks this placement/documentation pass. Later authoring
must establish actual versions/adapters, workloads, independent evaluator/holdouts,
useful-gain/non-regression criteria, limits and usage coverage, source/provider/egress
authority, targets, retention/export/restore and cleanup. Existing authority is not
silently widened or withdrawn. Missing runtime facts remain gates, not failures or
outcomes. No automatic promotion, campaign, implementation or deployment is authorized.

[audit]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/REPORT.md
[snapshot]: /home/nyaptor/.local/share/jcode/artifacts/factory-pattern-placement-20261029/repository-before.json
[skull-roadmap]: /home/nyaptor/dev/personal/skull-chat/openspec/ROADMAP.md
[skull-domains]: /home/nyaptor/dev/personal/skull-chat/docs/skull/DOMAINS.md
[nomi-roadmap]: /home/nyaptor/dev/personal/nomi/openspec/ROADMAP.md
[nomi-architecture]: /home/nyaptor/dev/personal/nomi/docs/knowledge-architecture.md
[fm01]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-jyhc8gnc.js:77
[fm02]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-5k86y7jv.js
[fm03]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-h6z5g89d.js:226
[fm04]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-2vsgb20a.js:2
[fm05]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-k74ns6rx.js:43
[fm06]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-3fmb2skn.js
[fm07]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/decoded/SKILL-1y4qj4cw.md:26
[fk01]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-p7w58m2c.js:2
[fk02]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-p7w58m2c.js:4
[fk03]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-y4pbkxq4.js:4
[fk04]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-6v720gm0.js:2
[fk05]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-xhyj8csq.js:37
[fk06]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/decoded/SKILL-11c53hbr.md:485
[fk07]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/embedded/chunk-m4eqc2gx.js:10
[fk08]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/decoded/SKILL-9ynn2e96.md:119
[fk09]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/decoded/SKILL-0060hxp9.md:66
[fk10]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/decoded/SKILL-7qj8y4j5.md:20
[fk11]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/decoded/SKILL-5wg3z30z.md:28
[fk12]: /home/nyaptor/.local/share/jcode/artifacts/factory-droid-0.251.0-audit-20261029/decoded/activation-c6keytdd.md:9
