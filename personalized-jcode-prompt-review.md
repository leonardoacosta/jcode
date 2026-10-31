# Personalized Jcode prompt review

## Scope and review status

This review concerns **Leonardo Acosta’s personalized Jcode setup only**. It is not an upstream Jcode redesign or a proposal to modify other harnesses.

Codex, Claude Code, and Pi are comparison sources—not templates to copy wholesale. The goal is to improve the personal Jcode prompt while preserving its execution strengths.

This is a **static, source-grounded review**. Proposed failure cases are hypotheses supported by instruction text, not demonstrated runtime failures. No prompt or configuration files were changed during the investigation. This document records the findings for review; it does not authorize implementation.

## Initial verdict

**The personalized Jcode prompt has a strong execution core. Its biggest weakness is competing instructions—not missing ambition or insufficient detail.**

Mature it by tightening scope boundaries, removing duplicated policy, and making specialized behavior conditional. Do not replace it wholesale with Codex’s or Claude Code’s prompt.

The important distinction is between:

- **Shipped instructions:** the harness’s default prompt and construction logic.
- **Effective personal instructions:** defaults or replacements, overlays, project policy, skills, tool descriptions, and dynamic context.

Jcode’s local `system-prompt.md` replaces its built-in base. Reviewing only the repository default would therefore miss the personal customization.

## 1. Source trustworthiness

Firecrawl was used for sources not clearly available locally. Source authority, accuracy, version relevance, currency, and purpose were assessed before accepting claims. Search snippets and successful scrape exit codes were not treated as sufficient evidence.

| Source | Trust assessment | What it establishes |
| --- | --- | --- |
| Local Jcode override and overlay | Primary evidence for configured personal instructions | Exact local wording; not proof that an existing session loaded it |
| Jcode prompt assembly source, checkout `d4b2f1c48` | Primary implementation evidence for this checkout | Replacement precedence and additional prompt layers |
| Installed Pi `1.4.0` prompt builder | Primary shipped-package evidence | Default construction, tool-dependent rules, project context, and skill insertion |
| OpenAI Codex repository, tag `rust-v0.145.0` | High trust: official, version-matched source | Default instruction template corresponding to installed CLI `0.145.0`; not proof of the active model-specific prompt |
| Piebald Claude Code extraction, pinned commit `9c0be99c0252a6c0a7a345ec8c5a5e58b8b3679c` | Conditional trust: third-party extraction | Useful candidate fragments, not independently verified against the installed executable |
| Official Claude Code settings documentation | High trust within its scope | Configuration behavior—not authentication of third-party extracted prompt text |

### Local evidence

- Personal base: `/home/nyaptor/.jcode/system-prompt.md`
- Personal overlay: `/home/nyaptor/.jcode/prompt-overlay.md`
- Canonical shared policy: `/home/nyaptor/.agents/AGENTS.md`
- Jcode assembly: `/home/nyaptor/.jcode/crates/jcode-base/src/prompt.rs`
- Assembly documentation: `/home/nyaptor/.jcode/docs/SYSTEM_PROMPT_CONFIG.md`
- Installed Pi builder: `/home/nyaptor/.local/share/pnpm/global/5/node_modules/@earendil-works/pi-coding-agent/dist/core/system-prompt.js`

The overlay points to the canonical shared policy and conditionally loads Jcode source-checkout policy. The assembly source confirms that the base is supplemented by other layers; the base file alone is not the complete effective prompt.

### External sources

- [Version-matched Codex default instructions](https://raw.githubusercontent.com/openai/codex/rust-v0.145.0/codex-rs/protocol/src/prompts/base_instructions/default.md)
- [Pinned Claude Code extraction inventory](https://github.com/Piebald-AI/claude-code-system-prompts/tree/9c0be99c0252a6c0a7a345ec8c5a5e58b8b3679c)
- [Extracted action safety and truthful reporting fragment](https://raw.githubusercontent.com/Piebald-AI/claude-code-system-prompts/9c0be99c0252a6c0a7a345ec8c5a5e58b8b3679c/system-prompts/system-prompt-action-safety-and-truthful-reporting.md)
- [Extracted autonomous operation fragment](https://raw.githubusercontent.com/Piebald-AI/claude-code-system-prompts/9c0be99c0252a6c0a7a345ec8c5a5e58b8b3679c/system-prompts/system-prompt-autonomous-operation-guidelines.md)
- [Official Claude Code settings documentation](https://code.claude.com/docs/en/settings)

### Exclusions and limitations

**Codex cache mismatch.** Local model-cache metadata reports client version `0.164.0`, while the installed CLI reports `0.145.0`. The cache was not accepted as a version-matched prompt baseline.

**Rejected source response.** A requested Codex model-definition source returned a `404: Not Found` body despite the scrape command succeeding. It was rejected as evidence. Successful transport does not establish valid source content.

**Third-party commercial interest.** Piebald promotes its own competing developer product. Its version history and inspectable fragments make it useful, but neither popularity nor freshness makes it authoritative.

**Conditional Claude fragments.** Extracted fragments can be mode-dependent. Their presence in the repository does not prove they are all active together. File metadata identifying when a fragment changed is not proof that the same fragment is loaded in every session of the repository’s latest advertised release.

**No installed-Claude corroboration.** The local Claude executable is under a version-labelled path ending in `2.1.311`; its embedded prompt was not independently extracted or matched to the third-party text.

**No complete live prompt capture.** The investigation did not capture the full effective runtime request for all four harnesses. Findings distinguish inspected text and construction logic from runtime activation.

## 2. Highest-priority Jcode findings

### A. Explicitly protect assessment-only requests

**Observed instruction:**

> Fix problems rather than merely reporting them.

The prompt also prohibits scope expansion, but it does not state the boundary between investigating a problem and being asked to fix it as directly.

**Failure hypothesis:** “Evaluate this configuration” becomes “evaluate and modify this configuration.”

The third-party Claude extraction contains a useful explicit distinction: when the user requests an assessment rather than a change, the assessment is the deliverable. This is a design candidate, not a verified claim about every Claude session.

**Recommendation — adapt:**

> For questions, investigations, evaluations, and reviews, findings are the deliverable unless the user also authorizes changes. Persistence means completing that deliverable, not converting it into implementation.

This preserves autonomy without turning every discovered defect into an editing mandate.

### B. Remove publication pressure from worktree preparation

**Observed instruction:**

> Verify, commit, and push those prerequisites first, then create the new tree from that commit.

The surrounding text preserves publication authorization boundaries, so this is **not unconditional authorization to push**. Nevertheless, it frames commit-and-push as the normal prerequisite for isolation.

**Failure hypothesis:** An agent believes it must publish existing work before safely isolating a task.

**Recommendation — revise:** Require inspection and preservation first. Make commits and pushes conditional on the authorized workflow, rather than prerequisites for creating a worktree.

The canonical shared policy already has more precise authority and Git-preservation rules. The personal base should respect that ownership rather than partially restating it.

### C. Replace mandatory bookkeeping with proportional planning

**Observed instructions:**

> Use the todo tool extensively.

> For multi-step tasks, record the full plan before starting.

**Failure hypothesis:** A small investigation gets excessive task management, or the agent invents a “full” plan before learning enough to make one.

The version-matched Codex default allows skipping plans for straightforward work and warns against padding them with obvious steps.

**Recommendation — adapt:**

> Use a tracked plan when dependencies, uncertainty, duration, or recovery needs justify it. Start with known steps and discovery questions; revise as evidence changes.

Keep truthful progress updates and verification before marking tasks complete.

### D. Consolidate tool-routing policy

**Observed instructions:**

> For web research, prefer Firecrawl.

> Use webfetch for known URLs.

The installed Firecrawl guidance routes known-URL extraction through Firecrawl too. The base also embeds preferences for agentgrep, RTK, diagram formats, and browser tooling.

**Finding:** Tool selection has competing owners. The web instructions are a concrete example.

**Recommendation — simplify:** Keep a short general tool-selection rule in the base. Leave changing product-specific routing to the applicable skill or canonical policy.

Do not add another precedence paragraph beside every tool preference; reduce the duplication instead.

### E. Make critical rules survive long context

**Observed instruction:**

> Read the user's first message carefully...

Mining the first message is useful, but it should not privilege the initial request over later approved scope changes.

**Failure hypothesis:** After compaction, an agent follows the original request and misses a later restriction.

**Recommendation — revise:** Maintain a durable current task contract covering the outcome, authorized effects, non-goals, and unresolved decisions. Update it when the user changes scope. The first message seeds that contract; it does not freeze it.

## 3. What to borrow—and what not to borrow

| Harness | Useful for personalized Jcode | Avoid copying |
| --- | --- | --- |
| Codex | Proportional planning; explicit workspace-instruction handling; bounded validation | Entire default template, CLI-specific mechanics, or communication rituals |
| Claude Code | Clear assessment-versus-implementation boundary; distinction between reversible local work and outward-facing effects | Conditional delegation rules or extracted modules treated as universal defaults |
| Pi | Capability-aware prompt construction; short operational rules; detailed workflows outside the core | Minimalism that removes needed verification, persistence, and ownership rules |
| Personal Jcode | Bounded autonomy; preserving unrelated work; root-cause investigation; honest validation; concise evidence | Duplicate policy ownership and unconditional process language |

**Pi’s strongest lesson is architectural:** its prompt builder adapts rules to selected tools and inserts skills and project context separately. For the personal Jcode setup, this argues for a smaller stable base—not fewer safeguards.

Borrow principles only when they address a demonstrated instruction gap or a testable failure hypothesis. Do not assume wording that works in another harness will behave identically with Jcode’s models, tools, or lifecycle.

## 4. Runtime issue: source and active builds differ

The launcher and shared-server paths resolve to different build directories:

| Surface | Observed identity |
| --- | --- |
| `/home/nyaptor/.jcode/builds/current/jcode` | `/home/nyaptor/.jcode/builds/versions/1ba484fa3-dirty-90b55a2744c6/jcode` |
| `/home/nyaptor/.jcode/builds/shared-server/jcode` | `/home/nyaptor/.jcode/builds/versions/9385b6eeb/jcode` |
| Inspected source checkout | `d4b2f1c48` |

**Consequence:** Source inspection does not establish which assembly behavior an existing Jcode session is running. Symlink identities alone also do not prove the identity of an already-running process.

The daemon was not restarted, and existing sessions were not disturbed. A future runtime check should identify the actual serving binary and inspect a fresh session’s effective prompt without disrupting the shared daemon.

## 5. Recommended direction

**Keep the existing personal base. Sharpen it before expanding it.**

Priority order:

1. Protect assessment-only intent.
2. Remove commit/push pressure from isolation instructions.
3. Make planning proportional.
4. Give shared policy and skills clear ownership.
5. Preserve the current task contract across scope changes and compaction.

### Suggested behavioral checks before adoption

These are proposed checks, not tests already run.

| Scenario | Expected behavior |
| --- | --- |
| Review without edit authorization | Complete evidence-backed findings; do not modify the reviewed target |
| Isolation without push authority | Preserve existing work; do not infer permission to publish; continue independent authorized work |
| Trivial task | Complete directly without unnecessary planning ceremony |
| Conflicting tool guidance | Follow the applicable authoritative route or surface a genuine unresolved conflict; do not silently improvise precedence |
| Late user scope restriction | Update the current task contract and honor the newer restriction, including after compaction |

### No numeric ranking yet

Ranking prompt quality from prose alone would imply behavioral evidence the investigation does not have. Runtime activation, comparative task outcomes, and regression checks remain unverified.

## Evidence identity appendix

Hashes identify the local bytes inspected; they are not assurances of current contents after subsequent edits.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `/home/nyaptor/.jcode/system-prompt.md` | 10,927 | `bad176b5c30c09c3ca8c7137f1f7fa443041bc82481667a5f2faa7478de016b7` |
| `/home/nyaptor/.jcode/prompt-overlay.md` | 803 | `a5260e4f033fc8ee153d61d686b552c101d9e4942c0c083840d7979449df89fa` |
| `/home/nyaptor/.agents/AGENTS.md` | 19,602 | `56cc9e805cbd9dd3c7c6f559fd12d8ed1b87a02fecf3424085f943128dbcc1f2` |
| Installed Pi `dist/core/system-prompt.js` | 8,167 | `ab02924fa078eef81addcf39bc0e43d3663f06d627cb42ead403662a004a9e5c` |

Public-source scrape files were retained separately under `/tmp/prompt-evaluation-public/` during the investigation. That temporary location is not a durable archive; the pinned source links above are the reusable references.
