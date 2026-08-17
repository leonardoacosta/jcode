---
name: cc-reference
description: "CC workflow reference — routing, rollback, teams cost model, worktree cleanup, thinking budget. Explicit-only."
user-invocable: false
allowed-tools: Read, Glob, Grep, Bash
---

# Reference

> Loaded on-demand for command routing, team decisions, and inventory lookups. Not in system prompt.
> For core rules: `rules/CORE.md` | For code patterns: `rules/PATTERNS.md`

## Command Reference

### Stacked Slash-Skill Invocation (CC v2.1.199+)

Typing multiple leading slash-skills in one prompt loads ALL of them (up to 5), not just the
first: `/ponytail /review check this diff` loads both skills before the prompt body runs.
Human-typed CLI input only — agent-side loading stays programmatic via the `Skill` tool
(unaffected; multiple `Skill()` calls in one turn already worked). Useful for combining a
behavior modifier (`/ponytail`) with a task skill (`/review`, `/commit`) in a single line.

### Global Flags

| Flag | Commands | Meaning |
| ---- | -------- | ------- |
| `--quick` | `/feature` | Skip Phase 1 (Discovery) only — refinement + spec creation still run |
| `--no-sync` | `/feature` | Skip beads sync after spec creation |
| `--changed` | `/audit:code` | Scope to `git diff --name-only HEAD` (unstaged + staged vs HEAD) |
| `--context=<path>` | `/feature` | Skip Phase 1 Discovery, load pre-gathered context.md instead |
| `--area <name>` | `/workflow:evolve` | Filter to specific area (features, skills, mcp, hooks, agents, claudemd, memory, performance) |
| `--save` | `/workflow:evolve` | Persist report to `~/.claude/docs/audit/evolve/` |
| `--all` | `/bootstrap:strategy` (CC-meta scoped) | Run full pipeline (all 7 artifacts sequentially) |
| `--dry-run` | `/apply`, `/apply:all`, `/bootstrap:roadmap` (CC-meta scoped) | Show plan without executing |
| `--continue` | `/apply`, `/apply:all` | Resume from checkpoint state file |
| `--incremental` | `/feature` | Create artifacts one at a time with review pause |
| `--skip-gates` | `/archive` | Skip validation gates entirely |
| `--foreground` | `/archive` | Block on quality gates instead of background execution |
| `--no-commit` | `/archive` | Skip the Stop prompt hook commit |
| `--no-roadmap` | `/archive` | Skip roadmap update in the Stop prompt hook |
| `--discard-state` | `/p2p` | Discard saved state and start from Step 0 |
| `--no-tag` | `/p2p` | Skip release tagging after successful deploy |
| `--squash` | `/p2p` | Use squash merge instead of merge commit |
| `--skip-review` | `/apply:all` | Skip Phase 5 local review gate (emergency hotfixes) |
| `--skip-browser` | `/apply:all` | Skip Phase 9 browser smoke verification (manual override) |

### Rollback Procedures

#### `/apply` Failure Recovery

| Scenario | Command | Effect |
| -------- | ------- | ------ |
| Gate fails, want to retry | `/apply $SPEC` | Resumes from first incomplete task |
| Reset current batch | `git checkout -- .` | Discards uncommitted changes, then `/apply $SPEC` |
| Full abort | `git reset --hard origin/$BRANCH` | Returns to pre-apply state |

### Workflow Anti-Patterns (NEVER — and why)

Hard-won failure modes. Each is a thing that looks safe and silently corrupts state or burns
budget — the WHY is the non-obvious part you only learn after it bites you.

| NEVER | Why it bites |
|-------|--------------|
| **NEVER `git reset --hard` to abort a `/apply` while the orchestrator is still running** | `/apply` runs in the main checkout (shared-tree model, no worktree); the per-repo apply lock releases via an EXIT trap when the process exits, NOT when you reset the tree. Reset mid-run and the orchestrator resumes from `tasks.md` `[x]` markers on top of your reset — a half-reverted, half-re-applied tree. Stop the run first, then discard; resume with `/apply <spec> --continue`. |
| **NEVER launch a second `/apply` in the same repo expecting it to "just queue"** | The apply lock (`apply_lock_acquire`, `scripts/lib/merge-slot-helpers.sh`) is single-flight, not a queue: a second invocation while the lock is held prints the holder (session/spec/age) and STOPs immediately. Wait for the holder to finish (a crashed holder's stale lock is reclaimed automatically via a dead-pid `kill -0` takeover) — never delete the lock dir by hand to force past it. |
| **NEVER spin up an agent team "just to organize" parallel work** | A team is `Nx` tokens for N teammates PLUS idle cost — teammates burn tokens while waiting on each other. Most tasks are I/O-bound (LSP, tests, builds), so parallelizing doesn't shorten wall-clock; you pay N times for the same elapsed time. Use `Task` with `run_in_background: true` instead — same parallelism, 1x idle cost. |
| **NEVER broadcast to a team when one teammate needs the message** | Broadcast is `Nx` message deliveries. A 3-teammate team with 5 broadcasts = 15 deliveries billed. Direct-address the one teammate that needs it. |
| **NEVER delete a leftover `/apply` worktree with `rm -rf .worktrees/<id>` instead of `git worktree remove`** | `/apply` no longer creates worktrees (session-worktree model retired 2026-07-17); this applies only to *draining* leftover `.worktrees/*` minted before the cutover. `rm -rf` leaves the worktree registered in `.git/worktrees/` (git still thinks it exists) AND orphans the CC session-memory dir under `~/.claude/projects/`. Use `git worktree remove` (or `wt reap`) then `prune-project-memory`. Drain-vs-delete tooling status: `openspec/changes/retire-session-worktrees/design.md`. |
| **NEVER run `bd sync` before a commit to "make sure beads is saved"** | `bd sync` does not exist as a command in bd 1.0.3 (confirmed via `bd --help` and a direct runtime error — see § bd Sync-Equivalent Commands below). The pre-commit hook already runs `bd hooks run pre-commit`, which flushes the JSONL. Use `bd export -o .beads/issues.jsonl` only if you need to force a flush outside the commit path (the stop hook does this), `bd import` only after a `git pull` — never invent a bare `bd sync` call. |
| **NEVER assume `/apply` archived the spec because the tasks are all `[x]`** | Checkbox state is not archive state. `[x]` means a task ran; archival happens in Phase 4 and can be skipped (`--skip-gates`) or fail silently if gates error in background mode. Verify the spec moved to `openspec/changes/archive/` before claiming done. |

### Command Routing

| Situation | Command | Why |
| --------- | ------- | --- |
| New feature, unclear requirements | `/feature` | Full discovery + refinement |
| New feature, clear requirements | `/feature --quick` | Skip discovery, keep refinement |
| Execute approved spec | `/apply $SPEC` | Batch execution + archive + push |
| Execute multiple specs | `/apply:all` | Consolidated mega-batch execution |
| CI/build failures | `gh run view` / `gh pr checks` | No dedicated ci:gh command was ever built — direct GitHub CLI diagnosis (see `monitor:triage`) |
| Push to production, monitor CI + deploy | `/p2p` | PR from dev→main, CI/review wait, merge, deploy monitor |
| Quick exploration | `/recon` | Context scan (local / DESIGN.md / CLAUDE.md / external-repo audit), outputs docs/recon/{name}.md (+ .html verdict for external repos) |
| Bootstrap single project | `/bootstrap:init` (**CC-meta scoped**) | Config scaffolding (CLAUDE.md, settings, beads) |
| Project scoping | `/bootstrap:scope` (**CC-meta scoped**) | Interrogate requirements, lock scope |
| User stories | `/bootstrap:user-stories` (**CC-meta scoped**) | Personas, flows, wireframes from scope-lock |
| Financial projections | `/bootstrap:financials` (**CC-meta scoped**) | Revenue model, unit economics |
| Design system | `/bootstrap:design` (**CC-meta scoped**) | Brand board, tokens, palette via gepetto (brownfield refresh: `/advise:design`) |
| PRD generation | `/bootstrap:prd` (**CC-meta scoped**) | Accumulate locked artifacts into PRD |
| Spec pipeline | `/bootstrap:roadmap` (**CC-meta scoped**) | PRD-to-specs with conflict analysis (openspec-native) |
| Full planning pipeline | `/bootstrap:strategy` (**CC-meta scoped**) | Maturity dashboard + streaming DAG |
| Execute one spec (or ad-hoc 2+) | `/apply $SPEC ...` | Inline tasks.md execution; 2+ names auto-build a wave plan and delegate to `/apply:all` |
| Execute all specs (consolidated) | `/apply:all` | Wave-based phase-aligned mega-batches with checkpoint, 58-70% token savings |
| Project audit (init) | `/audit:init` | Bootstrap domain discovery + persona management |
| Project audit (code) | `/audit:code` | Standards compliance + health scoring |
| CC practices audit | `/workflow:evolve` | Web-search latest patterns, score setup |
| Product analytics overview | `/monitor:posthog` | Trends, errors, experiments, flags, surveys |
| Don't know which diagnostic tool | `/monitor:triage` | Routes to right monitor command |
| Validate project config health | `/workflow:check` | Config scoring + remediation |
| Run all quality gates | `unified-gate-runner` (via `/apply`, `/apply:all`, `/m2m`) | Typecheck, build, test, lint — no standalone command |
| Fix type errors | `scripts/bin/tsc-report` | TypeScript compiler fix loop — no standalone `/test:fix-types` command (archived 2026-07-04, zero-use) |
| Run E2E tests | `/test:e2e` | Against deployed environment (see decision tree in command) |
| Local pre-merge review | `/m2m` | Quality gates + code-review + architecture review before merge (cc's `/review` command archived 2026-07-04; native CC `/review` is a fast single-pass — use `/code-review <level> <pr#>` for multi-agent) |

> Inline flag docs also live in each command's `.md` file for local context.
> Commands with `execution:` frontmatter declare blocking/background behavior canonically.

## Agent Teams

Teams are **experimental** and **expensive**. The `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` flag is
enabled for testing, but teams should NOT be used in production workflows until cost/benefit
improves.

**Default:** Use Task tool with `run_in_background: true` for parallel work. Use sequential agents
for multi-step tasks.

### Cost Model

| Pattern           | Cost         | Explanation                                |
| ----------------- | ------------ | ------------------------------------------ |
| Single agent      | 1x tokens    | One conversation thread                    |
| N teammates       | Nx tokens    | Each teammate is a separate Claude session |
| Broadcast message | Nx messages  | Same message delivered to every teammate   |
| Team coordination | Nx idle cost | Teammates consume tokens even when waiting |

**Example:** 3-teammate team with 5 broadcast messages = 15 message deliveries + 3x idle overhead.

### When Teams ARE Warranted

- Truly independent parallel workstreams (e.g., frontend engineer + backend engineer + test writer
  all coding simultaneously)
- Tasks that would take >30min sequentially but can complete in <10min with parallelization
- Multi-project coordination (one teammate per project, coordinating shared changes)
- Real-time collaboration on large-scale refactors

**Key criterion:** Parallelization must save MORE cost/time than team overhead adds.

### When Teams Are NOT Warranted

- Sequential task execution (use `/apply` batches or chained Task calls instead)
- Simple parallel tasks (Task tool with `run_in_background: true` is cheaper)
- Research/discovery (Explore agents return findings in one response)
- Single-project implementation (one engineer agent with clear spec is faster)
- "Just to organize work" (use task lists without teammates)

**Reality check:** Most tasks are I/O-bound (waiting for LSP, tests, builds), not CPU-bound.
Parallelizing doesn't help.

### Quick Reference

| Scenario                         | Use                                                |
| -------------------------------- | -------------------------------------------------- |
| Single spec implementation       | One engineer agent                                 |
| Multiple independent tasks       | Task tool with `run_in_background`                 |
| Research across codebase         | Explore agent                                      |
| Multi-step refactor              | Sequential agents (analyst -> architect -> engineer) |
| **Future:** Multi-project deploy | Team with project-specific teammates               |

**Bottom line:** Teams exist for testing, not production. Use simpler patterns until cost/benefit
improves.

## bd Sync-Equivalent Commands (Rare)

`bd sync` does not exist as a command in bd 1.0.3 (confirmed via `bd --help`'s full command
tree — zero `sync` entries in any group — and via a direct runtime error, `unknown command
"sync" for "bd"`, hit during real usage 2026-07-08). Never invent a bare `bd sync` call (see
Workflow Anti-Patterns above and `rules/BEADS.md` § JSONL Git-Merge Conflicts for the incident).
The real per-purpose equivalents:

| Command | When |
| --- | --- |
| `bd import` | After `git pull`, to import remote beads changes. Reads `.beads/issues.jsonl` by default (upsert semantics — new issues created, existing ones updated); pass a path or `-` for stdin. |
| `bd dolt status` | Check sync/connectivity status (read-only) — reachability, server version, database, vs. `bd status`'s issue-count stats. |
| `bd export -o .beads/issues.jsonl` | Force JSONL export without git ops (the session-stop hook, `scripts/bin/session-closer`, uses exactly this — see its header comment for the full incident writeup). |

**Audit:** `git log --oneline | grep "bd sync"` should return 0 results going forward.

> **Re-verified against bd 1.1.0 (2026-07-19)**: `bd sync --help` still returns
> `Error: unknown command "sync" for "bd"` (exit 1) — same error shape, not a 1.0.3-only gap.
> The three per-purpose equivalents above are unchanged and remain the correct replacement set.
> See `docs/reference/bd-1.1.0-baseline.md` § `bd sync --help` for the pasted runtime output.

## Session Worktrees (Retired 2026-07-17)

The per-session git-worktree `/apply` model — `.worktrees/<session-id>/` provisioning,
`EnterWorktree` entry, merge-back, and the `wt` CLI as the primary `/apply` mechanism — was
**retired 2026-07-17** by the `retire-session-worktrees` proposal. `/apply` and `/apply:all` now
run **directly in the main checkout** on the current branch, serialized by a single-flight per-repo
apply lock (see § How Work Ships in `CLAUDE.md`, `commands/apply.md` § Phase 0a: Apply Lock, and
the Workflow Anti-Patterns above).

The `wt` CLI, `scripts/lib/worktree-helpers.sh`, and `prune-project-memory` stay operational **only
for draining** `.worktrees/*` dirs minted before the cutover — not for normal `/apply`. If you need
the historical worktree-lifecycle reference (`wt` subcommands, memory-cleanup policy, skip-worktree
git-index flags) during the drain window, see `openspec/changes/retire-session-worktrees/design.md`
(drain-vs-delete tooling status). The former `references/worktrees.md` is now a dated tombstone.

## Extended Thinking

### Quick Reference

| Scenario                       | Extended Thinking | Rationale                                            |
| ------------------------------ | ----------------- | ---------------------------------------------------- |
| Multi-step architecture design | Always            | Complex dependencies, tradeoffs require deliberation |
| Security/architecture reviews  | Always            | Missing edge cases = vulnerabilities                 |
| Complex refactoring strategy   | Often             | Multiple interdependent changes need planning        |
| Schema migration planning      | Often             | Data integrity, rollback paths critical              |
| Simple CRUD operations         | Waste             | Pattern is mechanical, no decisions to make          |
| File moves/renames             | Waste             | Trivial operations don't benefit from thinking       |
| Boilerplate generation         | Waste             | Following templates, not reasoning                   |
| Straightforward bug fixes      | Rarely            | Unless root cause is non-obvious                     |

**CLI context:** `alwaysThinkingEnabled: true` in settings.json. Thinking is automatically applied
when beneficial. Don't request it redundantly.

### 75% Context Warning

When the context window reaches ~75% full, display this warning and act on it:

```
WARNING: 75% Context ────────────────────────────
  Approaching context limit. Output quality degrades
  past this point. Consider: /clear, compacting, or
  wrapping up the current task before continuing.
─────────────────────────────────────────────────
```

**Actions to take at 75%:**
- Finish the current atomic task, then stop
- Suggest `/clear` if starting a new unrelated task
- If mid-spec: commit progress, start fresh
- Do NOT start new large tasks — quality degrades past this point

### Decision Tree

```
Is this a new problem with unclear solution?
├─ Yes -> Use extended thinking
│   ├─ Multiple valid approaches? -> Higher budget
│   ├─ Security/correctness critical? -> Higher budget
│   └─ Standard complexity? -> Default budget
└─ No -> Skip extended thinking
    └─ Is it following an established pattern?
        ├─ Yes -> Execute mechanically
        └─ No -> Reconsider (might need thinking)
```

## Named Failure Modes

25 documented incident-class failure rows (moved from `CLAUDE.md` § 3) — "what a weaker model
gets wrong here," each row a real recurrence with the binding rule that prevents it.
Read `references/named-failure-modes.md`.

## Quality Bar Per Deliverable

Checkable (not adjective) completion criteria per deliverable type — code change, script, hook,
skill, command, agent, doc, spec/bead/commit (moved from `CLAUDE.md` § 4).
Read `references/quality-bar.md`.

## Agent Naming Decision Record

Why `-auditor`/`-validator`/`tdd-*` agents were NOT renamed to the strict 5-suffix set
(advisor-plans 032 vs 033, Leo's 2026-07-11 call) — moved from `CLAUDE.md` § 8, which keeps the
live naming rule inline.
Read `references/agent-naming-decision-record.md`.

## Advisory Ledger Format

**Load `references/advisory-ledger.md` WHEN** authoring or auditing an `advisor-plans/` ledger —
the row schema, the exact status vocabulary (`TODO | IN PROGRESS | DONE (ref) | BLOCKED (reason) |
REJECTED (rationale)`), and the `advisor-plans/` vs `plans/` role split. Demoted from
`rules/CORE.md` § File Placement (2026-07-25); `CLAUDE.md` § 2 already treats this surface as
historical rather than the primary path, so it does not earn per-turn residency.
**Do NOT load** for ordinary spec/bead work — the funnel runs through `openspec/changes/`.

## Beads Ops

**Load `references/beads-ops.md` WHEN** touching `.beads/*` git state, resolving a proposal's
feature-bead approval state, working with `/feature` order codes, changing an admission posture
in `wave-extend-scan`, or reading the sanctioned bead-title prefix table. Carries the JSONL
merge-driver contract, the git-tracked-file table, the Approval Signal model (including why
`triage-list-drafts` degrades permissively and `wave-extend-scan` fail-closed), Order Codes, the
apply-lock contract, Funnel Convergence provenance, the capability-epic drift walkthrough,
session-closer's three sync steps, and Hierarchy's detection-order / landing-pad / dormancy
subsections.
**Do NOT load** for a routine bead mint, a priority call, or the MICRO threshold — `rules/BEADS.md`
keeps Bead Hygiene, the Priority Model, and the ceremony floor resident by design.

Three commands carry a MANDATORY load trigger for this file at their decision point rather than a
trailing citation (`rebase-auto-load-ceiling` 3.1–3.3): `commands/triage.md` § Resolve the feature
bead, `commands/feature.md` § Funnel-Pressure Warning + the order-code allocation block, and
`scripts/bin/wave-extend-scan`'s header contract.

## Worktrees (tombstone + drain-window blocks)

**Load `references/worktrees.md` WHEN** a repo still has `.worktrees/*` on disk and you are
draining them — it carries the drain-window caveat and the `nv-nhm2j` pnpm install-race footgun
(demoted from `rules/CORE.md` by `prune-core-stale-and-rescope-narrow`), plus `wt check-symlinks`
as the detection.
**Do NOT load** for current `/apply` behaviour — it runs in the main checkout under a
single-flight lock (`commands/apply.md` § Phase 0a).

Note the file is a **retirement tombstone**, not a live mechanism reference: worktrees were
retired 2026-07-17. The historical `wt` lifecycle detail (subcommands, memory-cleanup policy,
skip-worktree git-index flags) is one hop further on, in
`openspec/changes/archive/2026-07-17-retire-session-worktrees/design.md`.

## Questioning Standards

**Load `references/questioning-standards.md` WHEN** running a `/feature`, `/bootstrap:*`, discovery,
or any flow that clarifies requirements with the user. The one-line rule: use the
**AskUserQuestion tool** for all 2+ option clarifications (never freeform), ask as a senior
expert accountable for the outcome (2-4 focused questions per message), and NEVER use vague
closers ("Anything else?"). Full philosophy, good-vs-bad question table, and stop conditions are
in the reference.

## Skills Inventory

> The per-skill enumeration is intentionally NOT hardcoded here — it drifts the moment a skill is
> added or removed (this section was stale by ~58 skills before it was cut). Treat the live
> filesystem as the source of truth.

**Regenerate the live inventory on demand:**

```bash
# Count + list installed skills (filesystem is authoritative)
ls -d ~/.claude/skills/*/ | wc -l           # total skill dirs
ls -l ~/.claude/skills/ | grep '\->'        # symlinked (canonical) vs in-house (real dirs)
```

The Skill tool's system-reminder catalogue (auto-injected each session) is the other live
source — it lists every loadable skill with its description.

### Installation Rule

```bash
# ALWAYS use -a flag to target only .claude/ (avoids .agents/.crush/.goose sprawl)
npx skills add <repo> -a claude-code

# After install, run /reload-skills (CC v2.1.152+) to pick it up in the current session
```

### Directory Layout (structure, not counts)

| Path | Purpose |
| ---- | ------- |
| `~/.agents/skills/` | Canonical skill installs (symlinked into `~/.claude/skills/`) |
| `~/.claude/skills/` | Per-session skills: symlinks to `~/.agents/skills/` + in-house real dirs |

Project-level installs live in a project's own `.claude/skills/` and override global defaults.
In-house skills (`wayfinder`, `orchestrator-patterns`, `simplify`, `browser-benchmark`,
`system-architect`, etc.) are real dirs in `~/.claude/skills/`, not symlinks.

## CC-Meta Scoped Commands

This repo (`~/dev/claude`) is symlinked as `~/.claude`, so files at `cc/{commands,skills,agents,rules}/` load globally for every CC session — regardless of CWD. Anything at `cc/.claude/` only loads when CWD=`~/dev/claude`. The overlay scopes **CC-meta tooling** (audits/configures Claude Code itself) to the cc workspace, keeping non-cc projects clean.

| Layer                      | Path                                            | Loads when               |
| -------------------------- | ----------------------------------------------- | ------------------------ |
| Global (project-applied)   | `cc/commands/`, `cc/skills/`, `cc/agents/` etc. | Every CC session         |
| CC-Meta overlay (cc-only)  | `cc/.claude/commands/`, `cc/.claude/agents/`    | CWD=`~/dev/claude` only      |

**Workflow command split:**

- **Global** (any project): `/handoff`, `/project:housekeep`
- **CC-Meta** (cc only): `/workflow:check`, `/workflow:evolve`, `/workflow:retrospect`, `/workflow:improve`, `/workflow:explain`
  (`/workflow:onboard`, `/workflow:upgrade`, `/workflow:local` archived 2026-07-04, no direct
  successor; external-repo auditing moved to the global `/recon <github-url>`)

> **Improvement ledger spine:** `/workflow:improve`, `/workflow:evolve`, and `/workflow:retrospect`
> are all **producers** on one append-only improvement ledger (`scripts/bin/improvement-ledger`).
> It adds the before/after **outcome loop** the run-state series lacked — `improvement-ledger
> outcome <id> --metric --value` re-measures a baseline to prove whether an improvement helped.
> Full reference: `docs/improvement-ledger.md`.

**CC-Meta agents** (cc only): `cc-feature-analyst`, `cc-practices-analyst`.

Skills (`cc-tooling`, `cc-reference`, `cc-practices-current`, `skill-creator`, `skill-judge`, etc.) and rules (`CORE.md`, `BEADS.md`, `TOOLING.md`, `PATTERNS.md`) remain global — they govern every session and are referenced by agents that may run in any project context.

When adding a new CC-meta primitive, place it under `cc/.claude/`, not `cc/commands/` or `cc/agents/`, to keep the global namespace clean.

## Spec Lifecycle

> `/feature` → `/apply` → PR → merge

`/apply` handles execution, archiving, beads close, and push in one command.

- `/apply` archives the spec in Phase 4
- Always verify validation gates pass before marking a spec complete
- Never mark tasks complete in tasks.md unless the actual work has been executed and verified

### Apply Batch Order

| Batch | Agent Types | Gate |
| ----- | ----------- | ---- |
| DB | `db-engineer` | `tsc --noEmit` |
| API | `api-engineer`, `types-engineer` | `pnpm build` |
| UI | `ui-engineer` | `pnpm build` |
| E2E | `tdd-test-writer`, `e2e-engineer` | `lint && test` |

After each task: mark `- [x]` in tasks.md + `bd close $BEADS_ID`. Gate failure → retry 3x →
escalate. `[deferred]` / `[user]` tasks: auto-skipped.

### Multi-Spec Execution

| Criterion | `/apply` (single / ad-hoc) | `/apply:all` (consolidated) |
|-----------|----------------------------|------------------------------|
| Specs per run | 1 inline (2+ delegates to `/apply:all`) | 2-8 per wave |
| Tasks per wave | Any | ≤40 |
| Token budget | Unlimited | Constrained |
| Isolation needed | No (shared main checkout; apply lock serializes) | No (phase-aligned mega-batch) |

**`/apply:all` 7-phase wave pipeline:**

```
Phase 1: DB batch → typecheck gate        (existing)
Phase 2: API batch → build gate           (existing)
Phase 3: UI batch → build gate            (existing)
Phase 4: E2E batch → lint+test gate       (existing)
Phase 5: local review gate (inline Simplify + MUST rules) (blocking — skippable with --skip-review)
Phase 6: commit + push                    (existing)
Phase 7: deploy monitor                   (auto-detected Vercel/git-hook — last wave skips)
```

Wave N+1 begins only after Phase 7 passes (or is skipped for the last wave).
