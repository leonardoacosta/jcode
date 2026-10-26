# Native tooling improvements: harness audit and proposal domains

Date: 2026-10-26. Status: research, not an implementation plan or an OpenSpec proposal.

## Intent and boundaries

Identify useful improvements to Jcode's existing native workflows by auditing OMP and empryo independently. Assess native-command versus skill identification, skill descriptions, and inline source-directory labels in the TUI selector. Recommend bounded domains for subsequent Feature/OpenSpec work.

This investigation does not implement features, create proposals, publish findings, change issue state, or authorize automatic execution. There is no linked issue. Existing MCP edits and `swarm-prompt.md` are outside scope.

Evidence baseline: Jcode checkout `b7dbb0287d862ca7e77e40a5522c5c29f9466e76`, inspected 2026-10-26. The session banner identifies a different running build, so source findings must not be described as observations of that binary. Checkout remains on `main`; local `origin/HEAD` points to `dev`. No branch switch or remote action is needed for research.

Two separate native swarm workers completed OMP and empryo audits. The coordinator rechecked the strongest claims against source, installed CLI help and primary documentation. No model-backed harness task runs or UI acceptance tests were performed; empryo version/help probes were run.

## Terms and falsification

A **native command** is a Jcode-dispatched slash command, such as `/review`. A **native tool** is a model-callable API, such as `swarm` or `todo`. A **skill** is instruction content loaded from `SKILL.md`. An MCP tool is a fourth surface, not a skill. `skill_manage` is itself a native tool even when it loads a third-party skill.

The selector mixes native commands and skill invocations. That is the first identification surface to fix. Transcript-wide recoloring is a separate extension, not a prerequisite. These findings would be disproved by a current selector that already preserves and renders command kind, real skill description, and source metadata, or by equivalent lifecycle controls in the full execution path. Source below establishes the selector gap, not every proposed workflow gap.

## Existing Jcode inventory

Do not propose adding these capabilities as though they were absent.

| Capability | Existing surface | Improvement boundary |
| --- | --- | --- |
| Swarm workers | `swarm` tool: spawn, message, report, status, wait, stop, resume, retry | Assignment contracts, ownership visibility, bounded recovery |
| Plan-driven dispatch | `todo` lanes/phases plus `swarm.launch` | Keep one authoritative task list and expose blocked dependencies |
| Swarm settings | `/swarm`, `/swarm-prompt` | Explain native behavior versus loaded orchestration skills |
| Code review | `/review`, `/autoreview` | Scope, capability enforcement, evidence and feedback routing |
| Outcome judging | `/judge`, `/autojudge` | Distinct intent/completeness assessment, bounded feedback admission |
| Planning | `/plan` with a rendered plan card | Read-only execution guarantees and explicit transition to execution |
| Continuation | `/poke [on|off|status]` | No-progress and blocked-state handling, not another retry tool |
| Overnight supervision | `/overnight` start/status/log/review/cancel | Active-turn cancellation, hard bounds, inspectable recovery |
| Background processes | `bash` background mode plus `bg` | Cancellation and reliable completion/progress delivery |
| Deferred work | `schedule` | Future eligibility versus already-running work |
| Durable initiative views | `/initiatives`, legacy `/goals` | Existing overview is not proof of a model-callable initiative API |
| Verification and recovery | `/test`, `/fix`, `/improve`, `/refactor` | Reuse their evidence/recovery boundaries rather than create parallel loops |
| Skill management | `/skills`, `skill_manage`, slash skill invocation | Discoverability, provenance, authoritative remote metadata |
| Scriptable runs | `jcode run --json`, `jcode run --ndjson` | Reuse existing JSON/event transport for any headless review contract |

Inventory evidence: `crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs:102-158`; `crates/jcode-app-core/src/tool/mod.rs:475-532`; `crates/jcode-app-core/src/tool/communicate.rs:1683-1700,2935-2941`; `crates/jcode-tui/src/tui/app/commands.rs:55-79`; `src/cli/args.rs:193-205`; `src/cli/commands.rs:2678-2702,3142-3195,3339`.

The model-facing initiative tool is intentionally unregistered (`tool/mod.rs:28-31,490-492`). Do not restore it, expose a new task-graph API, or migrate saved initiative data merely to make a catalog symmetrical. The live tool schema overrides older skill examples.

## Confirmed selector feasibility

**Yes: different native/skill colors, real descriptions, and inline source labels are feasible.** The obstacle is metadata discarded before rendering, not terminal color support.

| Finding | Evidence | Consequence |
| --- | --- | --- |
| Skills already retain name, description and path | `crates/jcode-base/src/skill.rs:14-25,483-504,517-542` | Local descriptions do not need new parsing or per-frame filesystem reads |
| Candidate generation replaces descriptions with `Activate skill` | `crates/jcode-tui/src/tui/app/state_ui_input_helpers.rs:375-410` | Fix the shared candidate model, not the displayed string alone |
| Candidate/cache/trait use `(String, &'static str)` | `crates/jcode-tui/src/tui/app.rs:775-802`; `crates/jcode-tui/src/tui/mod.rs:851-858` | Dynamic description/source/kind must cross the existing shared boundary |
| Both row variants use the same color logic | `crates/jcode-tui/src/tui/ui_input/command_suggestions.rs:3-65` | Kind-aware styling must cover single and multiple results, selection and fuzzy matches |
| Native commands win same-name collisions | `state_ui_input_helpers.rs:393-410` | `/review` and `/plan` must not be mislabeled as same-named skills |
| Remote sessions transport names only | `crates/jcode-protocol/src/wire/events.rs:591-593`; `crates/jcode-tui/src/tui/app.rs:1362` | A local-only fix would leave the remote selector incomplete or misleading |
| Existing skill report explicitly distinguishes local metadata from remote names | `crates/jcode-tui/src/tui/app/state_ui.rs:1776-1812` | Reuse the same authoritative metadata rather than infer remote provenance locally |
| Existing refresh hooks invalidate candidates | `crates/jcode-tui/src/tui/app/state_ui_runtime.rs:37-55`; `remote/server_events.rs:2122-2123` | Preserve caching and invalidate on metadata changes, including same-name edits |
| Styled display-cell ellipsis already exists | `crates/jcode-tui-render/src/lib.rs:112-161` | Reuse it, do not introduce character-count clipping or a dependency |

### Recommended row contract

```text
/review   [native]    Launch a one-shot headed review session
/explore  [.agents]  Research, investigate or evaluate a prompt…
/omp      [.agents]  Use whenever the user asks about Oh My Pi…
```

Order: command/skill name, source label, description. Native and skill names use distinct semantic colors from the existing palette. Retain a text cue so monochrome users do not depend on color. Selection/fuzzy-match styling must not erase the distinction. Final hues need visual verification, not new theme configuration by default.

Interpret “parent directory” as the meaningful **discovery-root owner**, such as `.agents`, `.jcode`, or a plugin package. The immediate parent of `SKILL.md` is usually the skill name; displaying it would repeat the first column. The directory named `skills` alone is equally uninformative. This is a recommended interpretation, not a settled user decision.

Use at most **10 terminal display cells** for the source label, including an ellipsis when needed, rather than ten bytes or Unicode scalar values. Show the full discovery path/scope in an existing details surface. Keep loaded location distinct from canonical symlink ownership. Do not resolve paths on every frame, silently relabel imported copies as their original author, or expose remote absolute home paths.

Use the authoritative server's metadata for remote skills. Preserve the existing names-only wire field for older peers and add optional metadata with a truthful unknown fallback. Do not infer a remote skill's description from an unrelated same-named local skill. Global/project roots and truncated prefixes can collide, so details must expose the full source and scope.

Width budget: fit the whole row to the actual overlay width; shorten description first, then source, then the name if unavoidable. Reserve existing scroll indicators. Sanitize metadata to one plain-text line. Never wrap UI chrome, emit raw control sequences, or change invocation/Enter behavior. Reuse `REGISTERED_COMMANDS` and `SkillRegistry`, not a second hand-maintained command catalog.

Acceptance checks: native/skill same-name collision; local and remote discovery; legacy names-only peer; same-name metadata refresh; global/project overrides; symlinked sources; two source labels sharing ten-cell prefixes; selected/unselected/single-result/fuzzy rows; descriptions with newlines/control characters; widths 0, 1, 20, 40 and 80; CJK/combining text; one logical row per visual row. Exercise actual tester frames before claiming UI completion.

## Confirmed control-loop observations

`/plan` currently builds a plan-only instruction and starts or queues a synthetic user turn (`commands_plan.rs:27-70`). That establishes a prompt contract, not by itself a capability-enforced read-only mode. Determine the full executor permissions before claiming a security defect or promising enforced planning.

Review/judge startup text requests analysis-only behavior (`commands_review.rs:8-23`). The judge deliberately receives a user-visible conversation mirror, excluding deep tool output and hidden context. Preserve that independence while letting it verify repo/test claims. Review and judge are not synonyms: judge checks alignment/completeness and missed follow-through (`commands_review.rs:590-616`), review targets changes. Child auto-feedback is disabled on preparation (`:710-736`). Do not add recursive judging.

Auto-poke already stops for classified non-retryable errors and clears queued follow-ups (`commands.rs:188-209`). Its incomplete-task selection excludes only completed/cancelled tasks (`:2746-2762`). A useful extension must distinguish blocked work and lack of progress without falsely treating ordinary waiting as failure.

Overnight already persists manifests, logs, resource samples and morning-report phases. However, `cancel_latest_run` records `CancelRequested` (`crates/jcode-app-core/src/overnight.rs:764-783`), the supervisor checks that state before/after a coordinator turn (`:266-281,306-324`), and `run_turn_monitored` only waits for the turn and logs resource/long-turn events (`:384-430`). The inspected path does not interrupt an active long-running turn on cancellation. This is a source-confirmed boundary, not a runtime reproduction. `max_agents_guidance` is named guidance, not proof of an enforced concurrency cap (`crates/jcode-overnight-core/src/lib.rs:47-81`).

## Harness audit findings

### OMP: source-confirmed feedback controls, documentation-led orchestration ideas

Auditor: native swarm worker `omp audit`. Installed `@oh-my-pi/pi-coding-agent` version **18.12.5**, repository metadata `https://github.com/can1357/oh-my-pi`, package directory `packages/coding-agent`. The coordinator directly checked package metadata, feedback admission and severity-routing source, and plan auto-approval help. Broader subagent claims come from the installed OMP skill's documentation snapshot, indexed 2026-08-25, not runtime verification of version 18.12.5.

Paths beginning `OMP/` below mean the installed package root, not a Jcode file.

| Candidate | Evidence and Jcode comparison | Disposition |
| --- | --- | --- |
| Feedback deduplication with explicit suppression reasons | `OMP/src/advisor/emission-guard.ts:100-130,180-210,279-323` admits or suppresses empty/noisy/duplicate/rate-limited notes and supports severity escalation | High-value evaluation candidate, conditional on tracing equivalent Jcode feedback controls |
| Severity-aware queued versus interrupting feedback | `OMP/src/advisor/advise-tool.ts:46-73` escapes advisory content, adds provenance/age, queues nits, and treats concerns/blockers as interrupting | Reuse explicit delivery semantics, not unconditional interruption of running side effects |
| Review permission profiles and diff scope | OMP docs `https://omp.sh/docs/advisor` and `https://omp.sh/docs/review`, installed reference `coding-and-integrations.md:39-45,82-88`; Jcode already has independent review/judge sessions | Investigate executor capability restrictions and structured evidence, not another reviewer persona |
| Assignment ownership and verification contracts | OMP docs `https://omp.sh/docs/subagents`, installed reference `coding-and-integrations.md:74-80,90-94`; Jcode already has todo lanes, launch and worker recovery | Improve native dispatch inputs/visibility only where absent; keep per-task instructions in skills |
| Revalidate revived workers against changed scope/base | Same OMP reference `:78-80,94`; Jcode already contains worker revival | Conditional recovery enhancement, not a new resume primitive |
| Bounded fanout and runtime with visible shared-tree risk | OMP subagent docs disclose permissive concurrency/runtime/isolation defaults | Use as a failure-case checklist, not defaults to copy |

Do **not** port OMP `--plan-yolo` auto-approval (`OMP/src/commands/launch-help.ts:36-40`). It changes from planning to implementation on the model's first resolve call. Do not copy the advisor's unlimited blocker budget (`emission-guard.ts:290-295`), automatically abort non-idempotent tools for every concern, or create new overnight/background/poke APIs that duplicate Jcode.

The feedback proposal must preserve the difference between an unchanged duplicate and a real severity escalation, distinguish suppression causes, and allow a finding to reappear against a genuinely changed revision. Advisory text remains data, not authorization. Whether Jcode lacks equivalent admission controls remains unknown after this bounded pass.

### empryo: useful advertised headless contracts, unverified implementation provenance

Auditor: native swarm worker `empryo audit`. The coordinator repeated `empryo --version` and `empryo --headless --help`, both exiting successfully. The executable reports **3.11.0-beta**, while `.runtime-version.tui` says **3.7.2-beta**. No verified source revision or build manifest links those versions. This is a provenance gap, not proof the executable is incorrect. No binary upload, source reconstruction, model call, or routine execution was needed.

Primary local evidence is the advertised CLI contract, captured outside Git at `scratch/empryo-headless-help.txt:5-6,15-17,23-26`. Flags in help do not prove they work. The coordinator also fetched the current workflow and routines pages below on 2026-10-26. Those pages are unversioned claims, not implementation evidence for 3.11.0-beta.

| Candidate | Evidence and overlap | Disposition |
| --- | --- | --- |
| Headless review with machine-readable verdict/exit status | Local help advertises `--review [instructions]`, exit code equal to verdict; Jcode has native review sessions and existing run JSON/NDJSON | Strong evaluation-domain candidate: expose the existing reviewer, reuse transport, define failure/inconclusive semantics |
| Bounded coder/reviewer repair operation | Local help advertises `--loop` and `--max-iterations`; Jcode already has review, judge, improve and refactor flows | Trace existing loops first; reuse their state instead of adding another execution framework |
| Optional parallel review and arbitration gate | `https://empryo.com/docs/agents/workflows` describes separate roles, parallel reviews, approval and bounded retries | Conditional high-risk orchestration extension; the page explicitly says its example is a workflow-design request, not a fixed built-in preset |
| Request-local file/line locator | Local help advertises `--marionette` and locator lanes; `https://empryo.com/docs/concepts/marionette` is a documentation lead, not coordinator-verified behavior | Park until a measured latency/relevance problem justifies it; reuse `agentgrep` and optional indexes before a new pre-pass |
| Repository-scoped recurring routines | Local help advertises `--routines`; `https://empryo.com/docs/tools/routines` describes `/schedule`, cron/interval prompts, pause/stop/remove and app/runner lifecycle | Optional scheduler extension only after a real recurring-work requirement; do not add a parallel daemon |
| JSON and JSONL output | Local help advertises `--json` and `--events` | **Already present in Jcode:** `run --json` and `run --ndjson` (`src/cli/args.rs:193-205`, implementation `src/cli/commands.rs:2678-2702,3142-3195,3339`) |

The last item was not confirmed in the worker's initial Jcode search. Coordinator revalidation found it and removed “add JSONL output” from the improvement list. This is why an auditor summary is not sufficient evidence of a gap.

A routine's existence must not grant authority to publish, deploy, send messages or perform destructive actions. Reaching a repair-loop cap is unresolved/budget-exhausted, not PASS. Help also advertises automatically installing a missing skill for `--skill`; do not copy implicit installation into Jcode's selector.

## Adversarial conclusions

| Challenge | Finding | Constraint on the next feature |
| --- | --- | --- |
| Are these missing tools, or existing tools with weak discoverability? | Most named capabilities already exist; Jcode also already has JSON/NDJSON runs | Improve shared contracts and lifecycle boundaries, not names or duplicate APIs |
| Does distinct color prove origin? | No: selection currently homogenizes colors, same-name skills lose to native commands, remote metadata is absent | Carry typed identity and authoritative provenance; retain a non-color cue |
| Does a parent-path label prove authorship? | No: imported copies and symlink projections separate load location from canonical ownership | Label discovery source honestly; disclose full scope/path separately |
| Does a prompt saying read-only enforce it? | Not by itself; launch helpers show instructions, not an established security boundary | Trace executor permissions; distinguish a restricted tool profile from an OS sandbox |
| Does an available cancel command stop active work? | The inspected overnight monitor does not observe cancellation during an active turn | Add and test actual cancellation propagation before claiming responsive stop |
| Does every change need parallel reviewers and an arbiter? | No; existing workflow specs already require proportionate, risk-triggered evidence | Ordinary low-risk work must not gain universal review stages or extra model calls |
| Does a judge verdict establish success? | No; source revision, performed checks, conflicting evidence and unfinished work matter | Record evidence and inconclusive/budget/cancel outcomes; a cap must never synthesize PASS |
| Do recurring schedules imply unattended authority? | No; a scheduled prompt is still subject to the user's original action boundaries | Keep external-action approval separate from scheduling and resume |

## Recommended domain boundaries

Domains are capability ownership boundaries, not one proposal per file or per command. The following feature names are advisory slices, not created proposals, task state, or readiness claims.

| Domain | Owns | Candidate Feature/OpenSpec slices | Main integration boundary |
| --- | --- | --- | --- |
| **Discovery and provenance** | Selector identity, descriptions, source labels, compatible remote metadata | **Typed skill-selector metadata and styling**: one end-to-end feature covering local/remote rows, color and width-safe source/description display | `SkillRegistry` → wire/session snapshot → command candidates/caches/`TuiState` → overlay renderer |
| **Task orchestration and collaboration** | Worker assignments, dependency eligibility, ownership, recovery, optional high-risk review gates | **Bounded swarm assignment and recovery contracts**; later **Optional review/arbitration gates**, only if a concrete gap survives tracing | `todo` lanes/phases ↔ `swarm.launch` ↔ worker/session recovery; do not create a second plan ledger |
| **Planning and continuation control** | Plan-only execution boundaries, approval transitions, task continuation eligibility, no-progress stop | **Restricted planning execution** and **Progress-aware poke** are separate features because permissions and continuation are different state machines; locator pre-pass remains parked research | TUI commands ↔ executor policy ↔ native task state; existing `/improve` and `/refactor` loops must be reconciled |
| **Evaluation and evidence** | Review versus judge semantics, findings/verdict schema, evidence scope, feedback admission/delivery | **Structured headless review** using existing run output; **Bounded judge/review feedback** after confirming missing admission controls | Review/judge sessions ↔ scoped source revision and checks ↔ queued/interrupt delivery ↔ existing CLI JSON/NDJSON |
| **Durable execution and recovery** | Overnight active turns, background cancellation, persisted limits, restart/recovery, scheduling | **Responsive overnight cancellation and enforceable limits** first; **Recurring scheduled work** only after an explicit recurring-use requirement | Overnight supervisor ↔ agent/tool cancellation ↔ `bg`/`schedule` ↔ persisted run manifest |

Keep presentation and server metadata in the same discovery feature unless a separate compatibility rollout is genuinely needed. Splitting by local-versus-remote implementation files alone would make it too easy to claim completion while the requested UI still lies in remote sessions.

Do not bundle all five domains into an omnibus proposal. Do not split `/review` and `/judge` into unrelated frameworks: they share evidence/delivery infrastructure but retain different evaluation purposes. Place the optional locator under planning investigation, not a new indexing service; place recurring routines under the existing scheduler, not a second daemon.

## Priority, dependencies and observable acceptance

| Order | Bounded outcome | Why now / dependency | Evidence required before claiming complete |
| --- | --- | --- | --- |
| **1** | Typed selector metadata, distinct native/skill styling, source before real description | Directly requested, source-confirmed gap; no other workflow redesign is required | Actual TUI frames plus model/renderer tests covering the selector cases above; remote legacy/new peer round trips; invocation/collision behavior unchanged |
| **2** | Overnight cancellation reaches active work; limits are enforced rather than merely logged | Source-confirmed active-turn boundary; cancellation semantics must precede more unattended automation | Start a deliberately long task on an isolated tester/socket, cancel mid-turn, observe stopped coordinator and child work, consistent terminal state, restart-safe status and no further launches |
| **3** | Structured, scoped headless review | Reuse existing review and JSON/NDJSON rather than invent transport; settle permission and verdict contract first | Real CLI fixture with known passing/failing/inconclusive findings, deterministic terminal status/exit mapping, stale-revision rejection, clean machine output and cancellation |
| **4** | Swarm recovery/ownership and planning/poke improvements | Trace current contracts first; separate confirmed gaps from existing prompt/skill policy | Changed-base revival, overlapping ownership, unmet dependency, stopped/blocked task, genuine wait, no-progress loop, explicit user-stop and simple-task negative controls |
| **Conditional** | Feedback admission, repair/arbitration gates, locator, recurring routines | Need confirmed Jcode gaps, measured benefit, or a concrete user use case | Dedupe versus escalation tests; explicit caps and unresolved outcomes; no added ceremony for simple tasks; locator relevance/latency comparison; scheduler restart/missed-run/overlap tests as applicable |

The feedback and orchestration slices can proceed independently of selector implementation once their gaps are established. Durable overnight automation must consume actual cancellation/budget controls, not rely on prose or assume that resource samples enforce limits. A repair loop should consume the same review result contract instead of parsing free-form PASS text.

## Guidance for subsequent OpenSpec authoring

For each selected feature, use native `proposal.md`, capability/spec delta, `design.md` when needed, and `tasks.md`. Keep this research advisory. Do not duplicate authoritative tasks here or infer issue readiness from this report.

Each proposal should name its owned runtime state, existing surfaces it extends, non-goals, compatibility/approval boundaries, observable acceptance and negative controls. Separate execution success, cancellation, failure, inconclusive evidence and exhausted limits. State source-versus-runtime limits explicitly. Include local/remote integration where relevant, not only a renderer unit test.

Reconcile with `openspec/changes/archive/2026-10-20-improve-workflow-suite/proposal.md:9-15` and the existing `workflow-evidence-and-control` and `workflow-behavior-evaluation` specs. Those already own proportionate evidence, executable-task completeness, causal recovery, selective review and behavioral evaluation. New proposals should address native runtime/UI gaps, not restate the instruction suite or invent another generic eval framework. Use existing behavioral fixtures for regression/negative controls where relevant; native executor/UI changes also need their own actual integration checks.

## Decisions and remaining unknowns

No unresolved decision blocks completion of this research. No Feature/implementation or external publication was authorized by this Explore pass.

- **Source-label interpretation:** recommend the meaningful discovery-root owner, capped at ten display cells, with full loaded path/scope in details. Canonical author/repository provenance is a separate field if later needed. This interpretation is proposed, not an accepted UI design.
- **Read-only strength:** future planning/review features must state whether they promise restricted native tool capabilities, a shell policy, or a hard filesystem sandbox. These are not equivalent. Do not silently expand the task into sandbox infrastructure or call prompt-only restrictions a security guarantee.
- **Unverified harness/runtime claims:** empryo's implementation provenance is unknown; OMP subagent documentation is older than the installed package; neither harness's model-backed behaviors were exercised. Current Jcode feedback admission, complete recovery contracts, repair-loop reuse and headless-review semantics need targeted tracing before related proposals can claim a confirmed gap.
- **Parked ideas:** no demonstrated need yet for an always-on locator, recurring routines, universal arbiters, automatic plan approval, new workflow DSL, restored initiative tool or replacement orchestration framework. Reopen only for a concrete requirement or measured failure.

## Verification and closeout evidence

Performed: two separate native swarm audits; direct Jcode source/caller/protocol inspection; direct OMP package/advisor/launch-help inspection; successful empryo version/help probes; primary workflow/routines documentation reads; overlap checks against archived/current OpenSpec; coordinator rejection of the duplicate JSONL proposal.

Not performed: model-backed harness task execution, changed-code tests, UI tester frames, release build, new feature implementation, OpenSpec proposal creation, issue mutations or publication. These are research findings, not a claim that any proposed behavior now exists.

Artifact checks passed: required-section and unfinished-draft checks, balanced Markdown fences, 21 fully qualified repository citation path/line-range checks, and `git diff --no-index --check /dev/null docs/research/native-tooling-harness-audit-2026-10-26.md`. Citation-range checks establish that locations exist, not that every behavioral inference is proven.

Raw worker reports and CLI output remain in ignored scratch storage. The durable artifact contains findings, provenance limits, source anchors and proposal-domain guidance, not copied external implementation code or private session/configuration content.
