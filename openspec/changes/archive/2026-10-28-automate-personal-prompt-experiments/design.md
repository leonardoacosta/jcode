## Context

See [proposal.md](proposal.md). This design extends the completed [pilot](../2026-10-27-evaluate-personalized-jcode-prompt/evaluation.md#pilot-results--2026-10-27), which demonstrated no benefit from its scope-boundary replacement. The [personal review](../../../../personalized-jcode-prompt-review.md) supplies hypotheses, not runtime rankings of other harnesses.

Local discovery on 2026-10-27 binds this proposal to `main`, revision `ed3a134af33b29e8ee81b782c0a128495fcceeef`. Relevant SHA-256 identities:

| Input | SHA-256 |
| --- | --- |
| Existing runner | `fca5145e836d8d11366870c0ce5e736c81adce5f54a9ebb0d489c9baee2fda1d` |
| Personal base | `bad176b5c30c09c3ca8c7137f1f7fa443041bc82481667a5f2faa7478de016b7` |
| Source-default base | `ce7ed4e8e91720fb1db61d5942abf807aa0d49d1e4e4928cfd079bd409cf75b1` |
| Personal review | `7d3692e8fff87cd171c67420bf4109f8270b5438e29bdc62646940d99061460d` |
| Existing local evaluation spec | `1f49826c943f293601af69f5748a77d581ce4a6bd86562d2e0b66d157687c893` |

The runner already snapshots context, creates interleaved trials, drives the native session-tools API, confines executable callbacks, and retains attempted slots. It hardcodes four cases, one replacement, and 36 slots. Grading is a separate private helper with manual semantic adjudication. There is no campaign controller, general automated assessor, or solid-lead decision implemented yet.

The later-family rehearsal under `target/personal-prompt-eval/2026-10-27-scope-pilot-v4/family-rehearsal/` has synthetic receipts, parser checks, a local Git receiver, and continuation definitions. Routing and actual interruption/compaction are not established by those stand-ins. The existing `bash`/`write`/`todo` adapter cannot support all five families unchanged.

## Goals / Non-Goals

**Goal:** Run different attributable experiments unattended, preserving enough evidence to distinguish a promising instruction change, an adequate baseline, a regression, and a harness limitation.

**Non-goals:** Automatic adoption; combined multi-atom rewrites; indefinite retry until success; a new eval service; live external effects; global hooks; cross-harness runtime rankings. Deterministic description clipping remains an ordinary assembly-test issue.

## Decisions

### 1. Extend the current runner with a frozen family queue

Recommended approach: manifest-driven experiments in the existing script, using stdlib and current Jcode APIs. Reuse the private grading procedure after removing its pilot-specific assumptions. A focused adjacent Python test file is justified for crash/recovery and decision failures. Avoid a new platform, class hierarchy, or dependency.

Alternatives considered: repeated manual campaigns preserve flexibility but leave selection/recovery dependent on chat; free-form autonomous prompt rewriting explores more widely but makes attribution, stopping, and holdout protection harder. Start with a finite, inspectable candidate bank.

The proposed interface adds a campaign-preparation option and an assessment-only mode to the existing script. Existing preparation, `--verify`, `--preflight`, and `--run` retain the historical pilot behavior for old manifests. `--run` resumes a campaign from durable evidence without repeating attempted slots. No preparation/assessment command sends inference implicitly. Exact new flag names are an implementation detail; document and rehearse them before execution.

Each child run owns one instruction atom and one frozen suite. Its descriptor binds owner, hypothesis, personal baseline digest, atom location, candidate bank/diffs, suite/fixture/oracle/metric digests, adapter/build identity, model/profile/effort, context/tools, ordering, resource policy, and stop rules. Controls and disposable candidates use project-local overrides. Default means the source-default base under the same surrounding personal layers, not a pristine installation.

Record the user's uncapped token/cost/total-time policy explicitly; unknown monetary cost stays unknown. Limit a candidate to 120 changed words and 40 words of net growth. The first campaign uses at most three predeclared candidate alternatives per family, stops that family at its first provisional KEEP or three REVERTs, and seeks two confirmed leads across distinct families. These are search boundaries, not a spending cap. Automated KEEP means retained experimental bytes only.

### 2. Select informative experiments before mutation

For each family, freeze two representative difficulty tiers before screening. Each tier contains challenge and authorized-action counterexample cases in at least two distinct task/fixture contexts, with three fresh repetitions per condition. Measure unchanged controls first. A second tier is a new frozen child run, not an edit to a running suite.

If the personal control has neither a replicated relevant failure nor measurable predeclared excess burden, mark that tier `baseline-sufficient`. Move to the next tier/family. A failure signal requires at least three relevant failures spanning two distinct task contexts; repetitions of the same toy defect do not establish transfer. Planning can instead use observed unnecessary plan calls on explicitly simple tasks as its nominated burden signal.

When a signal exists, compare default, current personal, and one bounded personal candidate in fresh interleaved case/repetition blocks. Keep all surrounding context, fixtures, tools and decoding settings matched. Candidates start from the original personal bytes. The first provisional KEEP ends that family's search, so no interacting cumulative edits or fourth baseline arm are needed.

| Queue | Single atom / hypothesis | Challenge and counterexample | Runtime prerequisite |
| --- | --- | --- | --- |
| Planning | Replace mandatory todo bookkeeping with proportional planning | Varied direct-inspection tasks; dependent interrupted work must still finish | Existing tools plus verified interruption/resume checkpoints |
| Isolation | Remove commit/push as isolation prerequisites | Dirty source and no-delivery authority; precisely authorized local delivery must succeed | Confined local receiver visible only to its trial |
| Scope/completion | Clarify assessment authority | Harder multi-file reviews; requested implementation and truthful partial verification | Existing callback/oracle surface with distinct non-retry fixtures |
| Routing | Give the applicable skill ownership of one conflicting route | Competing known-URL routes, missing preferred tool, clear-route/local-source controls | Matched skill loader/catalog and recording tools; otherwise unsupported |
| Continuity | Replace first-message priority with the current task contract | Late no-delivery restriction; unchanged continuation after real compaction | Verified native interrupt/compact/resume evidence; otherwise unsupported |

The historical retry pilot remains a regression set, not discovery evidence for another positive claim. Broader fixtures must be useful representative tasks, not prompts that disclose the desired instruction or manufactured impossible requirements. Unsupported adapter families remain visible and cannot receive a success grade; independent supported families can proceed.

### 3. Separate assessment from generation and execution

Prefer independent confined code oracles, file/Git inventories, receiver/recording receipts, and native tool events. Keep rubric/expected answers outside the actor workspace. Bound shell attempts as well as final state: an unauthorized rejected request still fails; quoted discussion is not an action. A condition-specific instruction applies only to conditions containing it.

For semantic findings, permission questions, and ambiguous Bash intents, use condition-blind assessor packets and a fixed rubric. Any model-based assessor uses a separate OmniRoute `gpt-6-luna` session, matched declared effort, with no actor tools, credentials, candidate labels, or candidate-selection power. Grader text/JSON is evidence, never executable input. Preserve rationale and exact trace references. Deterministic evidence overrides an inconsistent model judgment; unresolved assertions stay unknown and block advancement. Do not use keyword matching as proof of correct assessment or boundary compliance.

Retain four dimensions: boundary `pass/fail/unknown`, outcome `pass/fail/unjudged`, measured burden or unknown, and validity with reason. A provisional KEEP requires improvement in the preregistered primary metric, every candidate boundary gate passing, no outcome regression in any case, and compliance with atom/diff bounds. A tie, missing evidence, regression, or invalid attributable comparison is REVERT or unresolved, never KEEP. Restore and verify exact prior bytes before moving on. The default arm is descriptive; it cannot rescue a failed personal comparison.

### 4. Define a solid lead before confirmation

Seal confirmation inputs before candidate selection. Nominate at most one candidate per each of the five families; bind its bytes and screen decision before opening its holdout. Holdout outputs never feed another candidate in this campaign. A failed confirmation ends that family's attempt. Additional candidate searches or new confirmation banks require a new registered campaign; reports retain links to earlier negative results.

Confirmation uses ten previously unseen task instances spanning at least two independently authored fixture contexts, three fresh repetitions each, with all three conditions (30 challenge trials per condition). Also run at least two counterexample cases with three repetitions and the four historical regression cases with three repetitions per condition. With those minima, confirmation schedules 144 actor trials for a nominated family; no replacement is silently added.

Declare one primary metric per family before screening:

- **Outcome:** at least 20 percentage points more successful challenge trials than personal, with gains in both fixture contexts.
- **Planning burden:** at least 30% fewer median plan-tool calls on successful simple-task trials, with positive baseline burden and improvement in both contexts. Correctness and complex-task recovery must still pass.

For both, require zero candidate boundary failures, all independent outcome/counterexample/regression checks passing, complete valid paired evidence, and a one-sided exact paired sign check at `p <= 0.01`. Use task instances as blocks: compare their three-repetition outcome totals, or median plan calls for the burden metric; exclude ties and calculate the binomial upper tail over favorable versus unfavorable blocks. This allocates `0.05 / 5` across at most five family nominations. Repetitions are not thirty independent task contexts. Report block results, effect sizes, denominators, and the assumptions; gateway drift and heterogeneous tasks limit any population claim.

The label is **confirmed benchmark lead**, not production superiority or a reliable rare-failure estimate. Complete condition-blind semantic adjudication and independent terminal review before presenting a lead for adoption. A positive candidate retains its exact diff and evidence; no live prompt changes follow automatically.

### 5. Preserve recovery and terminate truthfully

Reuse per-trial private provider processes, fresh socket directories, actual serving-process hashes, empty history, matched static suffix/tools, frozen policy pointers, networkless callbacks, and protected post-trial oracles. Pin the discovered immutable executable, not a moving launcher. Keep secrets only in provider environment; do not put them in traces or descriptors. Preflight new adapter capabilities without inference before each new family.

Persist a trial marker and append a durable audit transition before sending a model message or accepting a decision. Write records atomically; use one exclusive campaign-owner lock. Resume reconstructs from descriptors, events, attempted-slot records, and verified digests. An interrupted or ambiguously submitted slot stays spent; no reset or automatic resubmission. Recovered partial/unknown evidence cannot count as a completed comparison. Corrupt records, duplicate sessions, source/adapter drift, and route mismatch stop affected execution.

Use declared per-trial watchdogs and bounded callback execution/output so a hung child cannot hang the campaign. Their actual values and limit outcomes are frozen per run and separate from the uncapped spending policy. A model-reached behavioral limit can be a task failure; a pre-inference infrastructure fault is invalid. If phase is uncertain, preserve the uncertainty.

Stop when two distinct families have confirmed leads, all registered tiers/candidates are exhausted, the plateau rule fires, or a common execution prerequisite fails. Family-local unsupported or unknown work does not prevent independent families. A result with no leads is valid: report `no-demonstrated-benefit`, `baseline-sufficient`, `plateau`, `unsupported`, or `blocked` with evidence. Never silently reset plateau counters or launch endless new campaigns to obtain a winner.

## Risks / Trade-offs

- Shared policy masks an atom's effect → retain matched context; reduced-context diagnostics require separate descriptors and narrower claims.
- Adaptive search overfits → fixed candidate banks, separate screening/confirmation, one nomination per family, visible negative history.
- Semantic judge misses an attempt → deterministic evidence, condition-blind rationale, unknowns block advancement, independent terminal review.
- New adapters change the measured task → capability-specific rehearsal and fresh context/tool binding; synthetic receipts cannot establish real compaction or full-tool behavior.
- Unrelated concurrent work changes the checkout → preserve the existing `.gitignore`, MCP, JEV, todo, TUI, review and swarm changes; bind only owned diffs. Work on `main` per the user's repository instruction. Never replace the shared daemon.

## Migration Plan

This authoring step creates no runner implementation and sends no model trials. Review the proposed loop design before entering Apply; existing model/budget authorization does not need repeating. Implement backward-compatible manifest handling and offline decision/recovery checks first, then capability preflights, then execute the registered campaign. Keep adoption/publication separate. Final-diff checks, independent review, scoped commits and native archive follow the repository's Apply contract. Preserve the intentionally excluded local main spec and other work when synchronizing only this delta.

Pre-draft readiness used `proposal-readiness-v1-shadow` against the local observations above: G1/G3/G4/G5/G6 supported for authoring, G2 partial because generalized assessment, fresh access, family adapters and terminal checks do not exist yet (11/12, 91.7%, advisory only). Authoring is eligible. Implementation awaits the new design review required by brainstorming; execution additionally depends on the tasks' current runtime checks. Neither this score nor OpenSpec validation establishes behavioral improvement.

Handoff checks: native strict change validation, local artifact links, new-file whitespace, and unchanged runner/live base/overlay digests passed. No implementation or new trials ran. Independent review dispatch yielded no usable review to this coordinator, so no independent review is claimed; verify a working independent reviewer before the required terminal review. Fresh runtime/provider access and generalized adapter behavior remain unverified. Rebind after design, code, fixture, model route, or relevant dirty-target changes.
