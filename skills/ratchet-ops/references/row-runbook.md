# Ratchet Row Runbook

> Per-row remediation map for `scripts/bin/validate-cc --tier 3`. Row source of truth is the
> `POLICY_CHECKS` array + `_chk_*` functions in `scripts/bin/validate-cc`; narrative history is
> `rules/TOOLING.md` § Config Ratchet Lane. If this table and the script disagree, the script
> wins — update this file.

## Blocking rows

| Row id | Asserts | Fix pattern |
| --- | --- | --- |
| `model-pin` | Every invocable command carries a `model:` frontmatter pin | Add `model: opus\|sonnet\|haiku` to the offender's frontmatter; tier by role (orchestrator/standard/trivial). Excluded: `references/`, README, `archive/`, contract docs |
| `verification-frontmatter` | Every agent's `skills:` list includes `verification-before-completion` | Add it to the agent frontmatter; policy is ALL agents, including analysts |
| `reference-readme-surfacing` | `commands/**/references/*` + READMEs carry `disable-model-invocation: true` | Add the frontmatter key — these are not invocable commands and must not surface as such |
| `preprocessor-adoption` (`>= 20`) | preprocessor injection sites have not regressed (59 as of 2026-07-07) | A drop means someone flattened injection blocks into prose — restore the bang-fenced blocks; do not pad with fake ones |
| `capability-epic-drift` | 0 closed capability epics, 0 dup-titled capabilities | `bd reopen $EPIC_ID --reason "Capability epic revived — premature close"`; merge dup children to one epic, mark loser `[MERGED INTO ...]` |
| `ledger-mutation-loud` | improvement-ledger mutation subcommands exit non-zero on invalid payload | Restore the validation/exit path in `scripts/bin/improvement-ledger` — mutations must fail loud |
| `matcher-coverage` | Every write-capable agent (Write/Edit in allowed-tools) is matched by the completion-verification matcher | Extend the matcher regex in settings.json (currently `...-engineer\|ui-*\|db-*\|api-*\|e2e-*\|tdd-*\|test-writer\|claude`) or add a documented `KNOWN_GAPS` entry + bead |
| `matcher-blind` | No `"matcher": ""` on PostToolUse / Notification / PostToolUseFailure | Tighten to a pipe-separated tool list. Empty matchers are CORRECT on SessionStart/Stop/PostCompact/SubagentStart — do not "fix" those |
| `hook-liveness` | Declared critical hooks have fired within their window | If matched dispatches exist but fires = 0, the event/matcher is dead — migrate the hook to a proven-live event (SubagentStop -> SubagentStart precedent) rather than deleting the declaration |
| `cost-coverage` | Every model ID seen in 30d of transcripts resolves to a non-null rate | Add the rate row to `scripts/lib/cost-rates.sh` (single source of truth). Date-suffixed IDs are stripped before lookup; `<...>` sentinels excluded |
| `worktree-cd-ban` | No bare `cd` into `.worktrees` in command/skill sources | Rewrite as `git -C` or `( cd ... && ... )` subshell |
| `session-primer-single-ready` (`-eq 1`) | session-primer invokes `bd ready --json` exactly once | Hoist to a single `READY_JSON=$(bd ready --json ...)` capture; both consumers jq the variable (~1.8s/call saved) |
| `stale-implement-outcomes` | No live `implement` ledger row >14d with `outcome=null` or `ref=commit-pending` | Per row: `improvement-ledger outcome <id> --metric <m> --value <v>` after re-measuring, or patch `ref` to the real SHA. This is a debt-backlog row — work it down, never bulk-null it |
| `dangling-skill-ref` | Every skill citation (agent frontmatter, command `Skill()` calls, PATTERNS.md routing column) resolves to a real skill/command/agent | Fix the citation or restore the moved entity; `plugin:skill`-form names are exempt |
| `fleet-rot` | 0 stale fleet copies + dead hooks + dead CLAUDE.md `/cmd` refs (`scripts/bin/fleet-rot --json`) | `stale_copies`: re-sync the project-local shadow from canonical (or add to `KNOWN_CUSTOM_COMMANDS` if genuinely hand-customized — verify via multi-commit git history). `dead_hooks`: restore the script or unwire the settings entry. `dead_claude_md_refs`: repoint prose to a live command or `docs/archive/commands/`. Known gap: expected non-zero until the non-oo shadow sweep lands |
| `skill-descriptions-budget` (`<= 20480`) | Summed `description:` bytes across **in-house** `skills/*/SKILL.md` (vendored-external skills — provenance marker `source: ~/.agents/skills@<date>` — excluded; their total is `skill_descriptions_vendored` in the context-floor INFO row) | Reclassify explicit-only skills (every invocation is a `Skill()` call / `@skill` directive / agent frontmatter — confirm by grep) and trim to <=200-char one-liners. NEVER trim auto-triggered domain skills; NEVER trim vendored descriptions (re-vendor wipes trims). Worktree false-low trap resolved 2026-07-13 (vendor-external-skill-symlinks — zero symlinks remain) |
| `unify-metrics-lane-sites` | metrics-enqueue call sites present (ratchet-metrics-enqueue + its ratchet.service wiring) | Restore the dropped enqueue call. Two sites, not three — the `rtk-local` site was dropped 2026-08-06 with the shim itself, same as radar-summary's in 2026-07-16 |
| `deferred-dialect` | No `[DEFERRED]/[SKIP]/[BLOCKED]/**DEFERRED**` on checkbox lines in `openspec/changes/**/tasks.md` | Replace with beads escalation + plain prose `blocked: <reason>`. Backtick-quoted examples are exempt; `/archive/` skipped |
| `skill-quality-floor` | Latest `docs/skill-scores/skill-judge-batch-*.json` shows no unexempted score < 99/120 | Remediate the skill per its `weakest_dimension` (D5 = split to references/, D3 = real NEVER section, D1 = delete generic-doc content) then re-score via `skill-judge-batch ingest`; exemptions only via `SKILL_QUALITY_FLOOR_EXEMPT` + rationale |

## hook-contract check (not a POLICY_CHECKS row)

Walks every `scripts/hooks/*.sh` `# requires-settings: <key>=<value>` header and asserts the
key holds on that script's settings.json hook entry.

Fix = add the key to the hook OBJECT in settings.json (e.g. `"continueOnBlock": true` beside
`type`/`command`/`timeout`), never by deleting the header — the header is the contract.

## INFO rows (signal, never fail)

| Row | Reading it |
| --- | --- |
| `context-floor` | Always-resident byte floor by component + `dollars_per_1k_turns`; `floor_warn` at >10% growth vs prior run -> `[Ratchet] WARN` primer line. A legit new skill can trip it — attribute the delta to a component before reacting |
| `closure-gate` | Count of complete-but-unarchived specs (cc's own zombies). Enforcement half is `openspec-status --closure-check` inside /apply |
| `agent-dispatch-census` | `zero_dispatch` (defined, never used in 90d = floor cost) + `orphans` (dispatched but undefined = mis-typed name). Archive-vs-keep is a per-agent user decision |
| `orphan-skill` | Skills with zero citations across the three scanned sources. Large count is EXPECTED (auto-trigger skills carry no citation); it is a prune signal, not an error |
| `guard-installs` | Repos with all 3 pre-commit guard templates wired (`guard-install --check`). Extend via `guard-install <repo>`; insert BEFORE the beads-managed block marker, never by substring-matching `bd hooks run` |
| `mcp-zero-invocations` | User-global MCP servers with no `toolUsage` entry inside 90d. Dead weight vs break-glass is an operator call |
| `memory-lint` | Memory-file hygiene findings — repoint or delete stale entries |

## Trap Index

- **Worktree measurement trap — RESOLVED 2026-07-13**: `vendor-external-skill-symlinks`
  (169a7e8e) removed every `skills/` symlink, so byte-count rows now measure identically from
  any checkout. Only relevant again if `skill-symlink-liveness` reports a new external symlink.
- **Snapshot staleness**: `ratchet-last-run.json` is nightly; always reproduce live before and
  after a fix.
- **`[Ratchet] STALE`**: lane has not run >48h — check the systemd user timer
  (`scripts/install/ratchet-timer/`), do not chase rows.
- **Grep-audit trap**: rule #2 (exit-0) applies to runtime failures in `--json` mode only;
  arg-parse `exit 1` and GATE scripts (`openspec-status --closure-check`) are compliant.
