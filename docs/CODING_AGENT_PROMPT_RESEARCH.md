# Coding-agent prompt and swarm research

Research date: 2026-09-24.
Local baseline: `fa773a4a8f1052224994151e8d26721b6dad3b43`.
Status: research and proposed revisions only. No prompt changes or new evaluations.

## Executive summary

The public evidence favors targeted behavioral instructions over a larger universal workflow. Explicit action wording can fix a narrow failure that general advice leaves untouched. Persistence, tool-grounding, and planning reminders helped GPT-4.1, but their bundled result does not establish each section's contribution. A revised repository-context study found no statistically significant accuracy change after removing testing, overview, or tooling categories, although some removals reduced cost.

Swarm performance depends heavily on whether work is genuinely parallelizable. Published architectural gains do not establish that a particular delegation sentence improves coding success. For Jcode, prioritize grounding in inspected files, bounded persistence, clear worker assignments, and coordinator verification. Retain autonomy, avoid mandatory verbose planning, and keep repository-specific instructions out of the universal prompt.

Recommendations in this report are adaptations, not demonstrated improvements to Jcode. Strong causal evidence for a minor behavior does not automatically make it the highest product priority.

## Scope and method

The request was to research publicly evaluated coding-agent prompt components and swarm guidance, then recommend additions or revisions to Jcode's existing prompts. The focus is section-level evidence, not running evaluations or treating architecture comparisons as prompt ablations.

This document records the completed synthesis. [Research notes and source ledger](CODING_AGENT_PROMPT_RESEARCH_NOTES.md) preserve earlier leads, unverified findings, retrieval limitations, and follow-up questions.

Evidence classifications:

| Class | Meaning |
|---|---|
| Direct narrow A/B | Changes a sentence and measures one specific behavior |
| Section removal | Removes a category from a larger instruction file |
| Prompt variant | Compares prompts that may change several things together |
| Bundle comparison | Evaluates multiple instructions together |
| Architecture comparison | Changes agent topology or orchestration, not an isolated prompt section |
| Operational evidence | Engineering observations without an isolated controlled comparison |
| Proposal | A suggested adaptation whose performance has not been measured |

## Jcode baseline and customization boundaries

Baseline sources were read locally when recording this report:

- [Base system prompt](../crates/jcode-base/src/prompt/system_prompt.md)
- [Swarm prompt](../crates/jcode-base/src/prompt/swarm_prompt.md)
- [Prompt configuration](SYSTEM_PROMPT_CONFIG.md)

The effective research session had richer instructions than the checked-in base, including craftsmanship, grounding, and verification. An omission from the base file is not necessarily an omission from the effective prompt. Inspect overrides before applying recommendations.

| Area | Checked-in coverage | Implication |
|---|---|---|
| Persistence | “Persist to completing a task” and “keep iterating” | Retain; avoid more persistence slogans |
| Autonomy | Complete related work and avoid unnecessary questions | Clarify scope and blocking conditions rather than intensifying autonomy |
| Safety | Hesitate for destructive actions; never reset passwords | Retain independently of benchmark effects |
| Grounding | No explicit inspect-rather-than-guess rule in the base file | Candidate addition if absent from effective instructions |
| Verification | Iterate, but no explicit evidence requirement for completion | Effective session already supplied it; consolidate rather than duplicate |
| Planning | “Use the todo tool extensively” | Consider task-dependent planning instead of universal bookkeeping |
| Swarm routing | Model/effort recommendations and fallback behavior | Reviewed evidence does not validate the specific assignments |
| Swarm structure | Labels; root-only spawning in normal/light mode; recursion reserved for deep mode | Retain current constraints |
| Worker assignment | No explicit objective/scope/output/acceptance contract | Add a compact assignment contract |
| Swarm integration | No explicit evidence-checking or integration responsibility | Add coordinator responsibility without requiring full duplicate execution |

At the recorded baseline, swarm routing recommends Fable 5 as the default and for design/investigation/review, GPT-5.5 at low effort for implementation, and GPT-5.5 at no effort for bulk reading. These are configurable policy choices, not evaluated conclusions of this research. The effective session's tool guidance can differ from the checked-in file.

Prompt layers, according to the configuration documentation:

1. Built-in base prompt, overridable by a file.
2. Capability modules.
3. Self-dev guidance, when applicable.
4. Project and global AGENTS.md.
5. Project and global prompt overlays.
6. Project and global preferred-tools guidance.
7. Memory and active skill prompt, dynamic and not cached.

Project `./.jcode/system-prompt.md` takes precedence over global `~/.jcode/system-prompt.md`. The first non-empty override wins. Replacement affects only the base layer. Both project and global overlays are included when present. File changes apply to new sessions, not existing ones. Embedded base changes require a rebuild. Swarm guidance has analogous project/global overrides; newly created workers load the latest guidance, while existing workers retain their captured prompt.

## Evaluated prompt interventions

| Section or instruction | Reported effect | Class | Jcode implication | Limits |
|---|---|---|---|---|
| Persistence + tool-grounding + planning reminders [S1] | Internal GPT-4.1 SWE-bench Verified score increased “close to 20%” | Bundle | Retain persistence and explicitly discourage guessing about repository contents | Not attributable to one section; source does not clearly distinguish relative percent from percentage points |
| Explicit planning between tool calls [S1] | Separately reported “4%” pass-rate increase for GPT-4.1 | Vendor-reported targeted intervention | Planning can help a non-reasoning agent | Limited experimental detail and ambiguous units; not justification for verbose reasoning with every model |
| Delete stale history comments [S3] | Opus baseline 0/16 to 15/16 with explicit deletion wording | Direct narrow A/B | Concrete actions can outperform abstract style advice | Not evidence of functional correctness or overall code quality |
| Describe comments as current-state documentation [S3] | “Comments describe the code as it is, not how it changed” scored 0% on both reported models | Direct narrow A/B | Writing good new text and removing stale text are different tasks | Does not establish ineffectiveness for other comment tasks |
| Defensive programming and edge cases [S4] | Test pass 19.3% to 26.7%; task success unchanged at 10% | Prompt variant | Consider a focused relevant-failure-path reminder | +7.4 percentage points in diff-based tests, no solved-task improvement |
| Comprehensive checklist [S4] | Task success 10% baseline to 4% | Prompt variant | Combining sensible instructions need not improve results | Does not isolate which instruction caused regression |
| Generated repository context [S5] | No significant resolution benefit; average cost +20% on SWE-bench and +23% on CTXbench | Whole-file comparison | Avoid duplicating discoverable documentation | Does not justify deleting necessary conventions or safety rules |
| Testing / overview / tooling categories [S5] | No significant accuracy effect from removing each category in GPT-5.2 ablation | Section removal | Categories need to earn their induced work | Not proof of zero effect or transfer to all models and repositories |

### GPT-4.1 versus reasoning-model guidance

OpenAI's GPT-4.1 guide concerns a non-reasoning model. Its reminder bundle covers persistence, consulting tools instead of guessing, and optional explicit planning. It also reports a separate planning improvement. Neither result establishes a universally best coding prompt. [S1]

OpenAI's reasoning-model guidance recommends simple, direct instructions and says explicit chain-of-thought prompting is unnecessary and may hinder performance. A short task plan is different from narrating reasoning before every tool call. Jcode can request the former without requiring the latter. [S2]

### Sentence-level comment experiment

| Wording | Haiku 4.5 | Opus 4.8 |
|---|---:|---:|
| No guidance | 0–13% | 0% |
| “Comments describe the code as it is, not how it changed.” | 0% | 0% |
| “Code comments describe the current code, never its history. When you edit a line, remove any nearby comment that just narrates a past change.” | 75% | 67% |
| “When you change code, delete any comment that only records its history.” | 50% | ~94% |

The assistant-ui README reports small Haiku samples, generally eight trials. Opus baseline and final candidate were confirmed across sixteen trials. A separate Sonnet instance judged results. The final Opus result was 15/16. These model labels and results are those recorded in the research synthesis; the linked README is mutable. [S3]

The transferable hypothesis is instruction shape: **when condition X occurs, perform observable action Y**. “Be rigorous” is less actionable than “Before reporting a fix as verified, run the relevant check and inspect its result.” The latter is a proposal, not a measured equivalent of the comment experiment.

The comment rule is optional. It has unusually direct evidence but targets a narrower problem than unsupported completion claims or poor delegation.

### SWE-Bench Mobile prompt variants

The study compared twelve prompt strategies using Claude Code with GLM 4.6 on fifty iOS tasks. The codebase was a large production Swift/Objective-C application. Its evaluation loads the patch as text and uses task-specific pytest checks to inspect diff text with pattern matching. These results do not establish application compilation or execution. [S4]

| Variant | Task success | Test pass | Interpretation |
|---|---:|---:|---|
| Baseline | 10% | 19.3% | Reference |
| Defensive programming | 10% | 26.7% | More partial test success, no additional solved tasks |
| Chain of thought | 10% | 21.8% | Same solved-task rate as baseline |
| Figma emphasis | 8% | Not recorded here | UI emphasis did not consistently match task needs |
| Test driven | 6% | Not recorded here | Prompt asks what tests would verify implementation; not a controlled evaluation of actual TDD |
| Comprehensive | 4% | Not recorded here | Worse than baseline, with cause not isolated |

Selected Appendix C wording:

**Baseline:** “You are an iOS developer. Read the PRD carefully and implement the required changes. Generate a unified diff patch that can be applied to the codebase.”

**Defensive programming:** “You are a senior iOS engineer known for writing robust, production-ready code. Implement the feature with a focus on defensive programming and edge case handling. Don’t just implement the happy path.” It then names empty/nil/invalid data, extreme text lengths and screen sizes, slow networks/timeouts/concurrency, first-time/offline/low-memory scenarios, and graceful handling without crashes.

**Comprehensive:** “You are a senior iOS engineer. Before implementing: (1) Analyze the PRD thoroughly (2) Identify all affected files (3) Plan your implementation strategy (4) Consider edge cases (5) Review the Figma design (6) Check for existing patterns (7) Implement with tests in mind (8) Validate against requirements Generate a complete, production-ready patch.”

The paper suggests the comprehensive checklist overwhelmed the model. That is an interpretation, not a separately established causal mechanism. The prompts also vary role framing and other wording, so this is not a clean isolated edge-case-section ablation.

### Repository-context study, revised June 2026

Use version 2 of *Evaluating AGENTS.md*, dated 2026-06-23, rather than silently mixing earlier headlines with revised results. It evaluates four agent/model pairings across SWE-bench Lite (300 Python tasks) and CTXbench (138 tasks from twelve repositories). Main experiments sample once per agent/task. CTXbench includes generated and manually reviewed task descriptions and tests. [S5]

The study finds instructions are followed and cause more exploration, testing, and specialized tool use, without statistically significant resolution improvements versus no context. Developer-written files outperform generated ones in the reported comparison (p=0.038), but their difference from no context is not significant (p=0.21). Generated-file comparisons against no context have p=0.87 on SWE-bench and p=0.37 on CTXbench.

Appendix B removes Overview, Tooling, or Testing categories from generated context files using GPT-5.4, then evaluates the modified files with GPT-5.2. These are not results pooled over every model in the main study.

| Condition | CTXbench accuracy | CTXbench cost/task | SWE-bench accuracy | SWE-bench cost/task |
|---|---:|---:|---:|---:|
| Full generated file | 68.12% | $0.4715 | 54.36% | $0.3272 |
| Without testing | 66.67% | $0.3730 | 57.72% | $0.2756 |
| Without overview | 62.32% | $0.4027 | 54.20% | $0.3018 |
| Without tooling | 63.77% | $0.4815 | 53.69% | $0.2715 |

| Removal | CTXbench accuracy p | CTXbench cost p | SWE-bench accuracy p | SWE-bench cost p |
|---|---:|---:|---:|---:|
| Testing | 0.80 | 0.023 | 0.099 | 0.0035 |
| Overview | 0.15 | 0.24 | 0.73 | 0.10 |
| Tooling | 0.31 | 0.97 | 0.85 | 0.0012 |

No accuracy difference was statistically significant. Removing testing significantly reduced cost on both benchmarks. Removing tooling significantly reduced cost on SWE-bench. Accuracy uses McNemar's test against the full-file condition; cost uses a permutation test. [S5, Table 7]

Other relevant findings:

- Repository overviews did not meaningfully reduce steps until interaction with a file in the reference patch.
- Some GPT-5.1 mini traces repeatedly searched for and read context files already included in context.
- Named tools were used more often when mentioned. This supports instruction following, not improved task resolution.
- The study found no simple relationship between file length and resolution or cost. Unnecessary induced work matters more than word count alone.
- Removing other documentation made generated context files more useful. The paper reports an average 2.7% improvement in that setting, without clear unit disambiguation here. Claude Code was excluded from that ablation for cost reasons.
- Python-only scope, benchmark construction, and task-resolution metrics limit transfer to other languages and nonfunctional requirements such as security.

## Swarm evidence

| Source and intervention | Finding | Evidence class | Jcode implication |
|---|---|---|---|
| Anthropic Research: Opus 4 lead, Sonnet 4 workers [S6] | 90.2% improvement over single-agent Opus 4 on internal research evaluation | Whole-system | Independent parallel research can benefit; not a coding prompt effect |
| Anthropic assignment specificity [S6] | Vague tasks led to duplicated searches, gaps, and misunderstood scope | Operational | Include objectives, output format, source/tool guidance, boundaries |
| Anthropic effort allocation [S6] | Early systems spawned excessive workers and searched indefinitely | Operational | Add budgets and stopping conditions, not fixed research-specific worker counts for all coding |
| Anthropic parallel execution [S6] | Workers plus parallel tools cut complex research time by up to 90% | Combined operational intervention | Parallelize independent operations; no universal latency guarantee |
| Scaling Agent Systems, 180 configurations [S7] | All tested multi-agent variants degraded sequential planning by 39–70% relative to single-agent baseline | Architecture | Keep tightly dependent work together |
| Scaling Agent Systems, finance [S7] | Centralized coordination improved financial reasoning by 80.9% relative to baseline | Architecture | Benefits depend on decomposability; not a repository-coding evaluation |

Anthropic warns that coding often has fewer truly parallelizable subtasks than research. Its token-cost comparison is approximately 4× chat for agents and 15× chat for multi-agent systems, not 15× a single agent. [S6]

The scaling study's benchmarks are Finance-Agent, BrowseComp-Plus, PlanCraft, and Workbench. It compares single-agent, independent, centralized, decentralized, and hybrid systems across three model families with standardized prompts, tools, and budgets. Its reported 17.2× versus 4.4× error-amplification measures, roughly 45% empirical capability threshold, and 87% held-out architecture-selection accuracy are study-specific findings, not portable Jcode thresholds or guarantees. [S7]

| Proposed swarm section | Existing coverage | Recommendation | Support |
|---|---|---|---|
| When to delegate | Hierarchy rules, limited task selection | Delegate independent bounded work; keep tight dependencies together | Architecture-informed, wording untested |
| Assignment contract | Not explicit | Objective, scope/exclusions, inputs, output, completion check, stopping condition | Operational support |
| Shared-file ownership | Base acknowledges concurrent agents | Assign non-overlapping edits where practical; coordinate interfaces | Engineering proposal for shared checkout |
| Coordinator verification | Not explicit | Inspect consequential claims and integrated result | Architecture-informed proposal |
| Stop or reassign | Not explicit | Stop duplicate work, repeated no-progress attempts, fulfilled assignments | Operational support |
| Model routing | Detailed | Retain configurable policy without claiming benchmark validation | No direct supporting evaluation identified |
| Recursion | Restricted to deep mode | Retain | No reviewed evidence warrants default recursion |

The most valuable proposed change is better worker assignments, not more role names or more agents.

## Prioritized recommendations

Priorities combine operational relevance and evidence, not measured Jcode gains.

| Priority | Action | Placement | Rationale |
|---|---|---|---|
| P1 | Inspect rather than guess | Base or existing grounding section | Addresses repository hallucination; part of evaluated GPT-4.1 bundle |
| P1 | Bound persistence by completion and blockers | Existing autonomy section | Preserve initiative without endless retries or false completion |
| P1 | Add worker-assignment contract | Swarm prompt | Current file emphasizes routing and hierarchy rather than task quality |
| P1 | Assign evidence-checking and integration to coordinator | Swarm prompt | Worker completion is not proof of overall completion |
| P2 | Consolidate verification | Existing verification section or base if absent | Preserve correctness policy without duplication |
| P2 | Make planning/todos proportional | Revise extensive-todo rule | No reviewed evidence validates universal bookkeeping |
| P2 | Add relevant failure-path reminder | Coding section | Narrow mobile evidence; transfer uncertain |
| P3 | Delete stale history comments if recurring | Repository convention or cleanup skill | Direct but narrow evidence |
| Avoid | Mandatory extensive narration before every tool call | Universal prompt | GPT-4.1 result does not generalize to reasoning models |
| Avoid | Large generic repository overviews | Universal prompt/generated overlay | No reliable navigation or resolution benefit in revised context study |
| Avoid | Reviewer/debater/manager for every task | Swarm prompt | Evidence opposes unconditional multi-agent scaling |

## Proposed wording

These snippets are untested adaptations. Replace or consolidate overlapping rules rather than automatically append all of them.

### Grounding

```markdown
When repository facts are uncertain, inspect the relevant files and tool
results before deciding. Do not invent file contents, APIs, or test results.
```

Closest to the tool-grounding reminder in OpenAI's evaluated bundle. [S1]

### Bounded persistence

```markdown
Continue until the requested outcome is verified or a concrete blocker
prevents progress. If repeated attempts add no evidence, change approach.
When blocked, report what remains unverified and what is needed next.
```

Stopping-policy proposal, not independently benchmarked wording.

### Planning and verification

```markdown
Use a short plan for multi-step work. Skip formal planning for trivial tasks.

Before claiming a change works, run the relevant available checks and inspect
their results. Distinguish verified behavior from checks that failed,
were skipped, or could not run.
```

The research session already had similar verification guidance. Consolidation is more appropriate than another copy.

### Relevant failure paths

```markdown
Check the failure paths relevant to the change, not only the happy path.
Follow existing error-handling conventions. Avoid unrelated defensive code.
```

The final sentence prevents speculative validation and unrelated refactoring. The adapted wording has not been evaluated. [S4]

### Swarm task selection and handoff

```markdown
Delegate independent, bounded work when it is worth the coordination cost.
Keep tightly dependent steps together.

Give each worker an objective, scope and exclusions, relevant context,
expected output, completion check, and stopping condition. Assign edit
ownership when workers share files.

Require findings or changes with evidence, validation results, and unresolved
questions. The coordinator checks consequential claims, resolves conflicts,
and verifies the integrated result before declaring completion.

Stop or reassign duplicate work and repeated attempts that make no progress.
```

Retain existing labels, root-only spawning, and deep-mode recursion constraints. [S6, S7]

## Counterarguments and risks

| Tempting conclusion | Why it goes too far | Better interpretation |
|---|---|---|
| Shorter is always better | No simple length-performance relationship; longer wording helped a smaller model in one experiment | Remove redundant or counterproductive work, not words indiscriminately |
| Remove verification because testing sections did not help | Generated testing sections were ablated while agent instructions remained; resolution is not every quality property | Keep concise verification, avoid redundant rituals |
| Defensive programming is the best prompt | One model/harness pair, fifty tasks, one iOS codebase, diff checks | Candidate reminder, not universal winner |
| The checklist overwhelmed the model | Mechanism was not isolated | Variant performed worse; exact cause uncertain |
| A second agent guarantees independent review | Shared assumptions can propagate unsupported claims | Require evidence and counterexamples, not agreement |
| Native concurrency removes need for ownership | Harness concurrency does not resolve semantic conflicts | Coordinate shared interfaces and overlapping edits |
| More exploration means more rigor | More exploration/testing did not reliably improve success | Tie further work to a missing requirement or uncertainty |

## Open questions

The reviewed evidence does not establish the best section order or total prompt length for Jcode; gains from these adaptations on current Jcode models; optimal coding worker count or effort budget; superiority of current model-routing assignments over inheritance; whether universal independent review pays for itself; or whether proposed changes work better together than separately.

No verified AgentCN-specific section ablation or Jcode prompt-section comparison was identified. This is a bounded search finding, not proof of absence. Earlier research leads remain in the companion notes rather than being silently upgraded into evidence.

## Sources

| ID | Source | Use |
|---|---|---|
| S1 | [OpenAI GPT-4.1 guide](https://cookbook.openai.com/examples/gpt4-1_prompting_guide) | Reminder bundle and planning intervention |
| S2 | [OpenAI reasoning guidance](https://developers.openai.com/api/docs/guides/reasoning-best-practices.md) | Limits on transferring explicit-planning guidance |
| S3 | [assistant-ui evals README](https://github.com/assistant-ui/assistant-ui/blob/main/evals/README.md) | Narrow sentence-level A/B; mutable source |
| S4 | [SWE-Bench Mobile](https://arxiv.org/html/2602.09540) | Prompt variants, Appendix C wording, diff-based evaluation |
| S5 | [Evaluating AGENTS.md v2](https://arxiv.org/html/2602.11988v2) | Whole-file and category-removal comparisons |
| S6 | [Anthropic multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system) | Delegation, effort allocation, parallelism, failure modes |
| S7 | [Scaling Agent Systems v1](https://arxiv.org/html/2512.08296v1) | Task-dependent architectural effects |
| J1 | [Jcode base prompt](../crates/jcode-base/src/prompt/system_prompt.md) | Local baseline |
| J2 | [Jcode swarm prompt](../crates/jcode-base/src/prompt/swarm_prompt.md) | Local baseline |
| J3 | [Prompt configuration](SYSTEM_PROMPT_CONFIG.md) | Layers and loading boundaries |

## Rerun inputs

```text
topic: Publicly evaluated coding-agent prompt sections and Jcode revisions
requested depth: extensive, with section-level evidence prioritized
output: cited Markdown matrices, recommendations, and proposed wording
constraints: public sources only; no new evaluations; no prompt modifications
baseline: fa773a4a8f1052224994151e8d26721b6dad3b43
coverage: focused synthesis, not exhaustive absence-of-evidence claims
```
