# CC Adoption Signals

_Last refreshed: 2026-07-21 — sources: docs sha256:4f0de0cf8e, gh v2.1.217, npm 2.1.217 (stable 2.1.206)_

This is the file `/workflow:evolve` consumes when scoring gaps. Every entry should be gradeable: a concrete `Check` whose exit code tells you whether the user already has the thing, plus a concrete `Action` if they don't.

## orchestration (v2.1.215-217 delta)

### concurrent-subagent-cap-v2.1.217

- **What**: A cap on concurrently-running subagents, default 20, override via `CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS`. Distinct from the existing `CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION` (lifetime count, cc has this set to 100) — this new knob throttles how many can run AT ONCE.
- **Why**: cc's `Workflow` tool already self-caps parallel `agent()` calls at `min(16, cpu cores - 2)` internally, but plain `Agent` tool fan-outs (e.g. `/audit:journeys` per-journey dispatch, `/apply:all` wave batches, ultracode-triggered ad-hoc parallel `Agent` blocks) are not routed through `Workflow` and could hit the new default-20 concurrency ceiling with no override set.
- **Check**: `grep -c 'CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS' ~/.claude/settings.json 2>/dev/null | grep -qv '^0$'`
- **Action**: Determine whether any cc-orchestrated fan-out (non-Workflow) realistically issues >20 concurrent `Agent` calls in one message; if so, set `CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS` in `~/.claude/settings.json` env block to match, or confirm 20 is already comfortably above cc's largest plain-Agent fan-out.
- **Introduced**: `v2.1.217`, 2026-07-21

### nested-subagent-spawn-depth-v2.1.217

- **What**: Subagents no longer spawn nested subagents by default. Override via `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` to allow deeper nesting.
- **Why**: `codebase-health-orchestrator` is itself dispatched as a subagent (`Agent({subagent_type: "codebase-health-orchestrator"})`) and its own contract is to fan out to `architecture-reviewer`/`security-reviewer`/`service-auditor` in parallel from inside that dispatch — a nested-spawn pattern this default change could silently disable (the orchestrator's own nested `Agent` calls would fail or no-op) unless `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` is set. This is a regression-risk signal, not just an FYI — worth confirming behavior against the real agent before assuming it still works.
- **Check**: `grep -c 'CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH' ~/.claude/settings.json 2>/dev/null | grep -qv '^0$'`
- **Action**: Dispatch `codebase-health-orchestrator` (or any other agent known to spawn further subagents internally) and confirm its nested fan-out still completes; if it silently no-ops or errors, set `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH=2` (or higher) in settings.json env.
- **Introduced**: `v2.1.217`, 2026-07-21

## beads (seed)

### beads-1.1.0-formula-system

- **What**: bd 1.1.0 introduces a formula/molecule workflow system — formulas (multi-step scripted workflows), molecules (composable formula building blocks), gates (conditional checkpoints), wisps (lightweight ephemeral state), and swarm (multi-agent coordination primitives) layered on top of the existing issue-tracking core.
- **Why**: cc's bd usage today is issue-tracking only (epics/features/tasks, `bd ready`, dependency graph). The 2026-07-20 spikes evaluated whether any primitive could replace hand-rolled cc orchestration machinery (wave-state, apply-lock, orchestrator-patterns).
- **Check**: bd v1.1.0 (2026-07-04) confirmed current as of 2026-08-06 — no bd release since the spikes; all re-check triggers below remain unmet.
- **Action**: Verdict table (currency-checked 2026-08-06):

| Primitive | Verdict | Evidence | Re-check trigger |
|-----------|---------|----------|------------------|
| wisps | **ADOPTED** (2026-07-30, `adopt-wisps-for-operational-beads`) | `recon://legacy/bd-native-integrations-5c22d1ab0a1c/legacy-bd-native-integrations-5c22d1ab0a1cdd27` § B | n/a — shipped |
| formulas/molecules | **NO-GO** (`54c8cff3`) | `recon://legacy/bd-formulas-molecules-166d9f6fcc0d/legacy-bd-formulas-molecules-166d9f6fcc0d3826` | Re-evaluate if bd ships (a) an incremental molecule-update primitive (e.g. `bd mol sync <mol-id> --formula <name>` that diffs a live molecule against its source formula and appends only new/changed steps without touching existing progress) or (b) epic-typed molecule roots / `bd epic status` awareness of `issue_type=molecule`. Neither existed in bd 1.1.0 (`8e4e59d39`) at spike time. |
| gates | **NO-GO** (`77120213`) | `recon://legacy/bd-human-gates-99570597607b/legacy-bd-human-gates-99570597607b0acb` | Re-evaluate if bd ships either (a) a `close`/`update`/`batch` refusal specific to `issue_type=gate` targets (i.e. gates stop being ordinary closeable issues and require the dedicated `bd gate resolve` verb, which itself would still need a second fix — see next point), or (b) a cryptographically or session-verified actor credential on `bd gate resolve` that is actually recorded and checkable (today's `--actor` string is unenforced and unrecorded). Neither existed in bd 1.1.0 (`8e4e59d39`) at spike time. |
| swarm | **evaluated — no resource-exclusion axis** | `recon://legacy/bd-formulas-molecules-166d9f6fcc0d/legacy-bd-formulas-molecules-166d9f6fcc0d3826` § `bd swarm` | Orthogonal to spec-sync — coordinates parallel agent dispatch on epics, not a spec-sync replacement candidate. |
| apply-lock | **KEEP hand-rolled** | cc `scripts/lib/apply-lock.sh` | No bd primitive evaluated as a drop-in replacement for cross-process apply serialization. |

- **Introduced**: bd `v1.1.0`, source: https://beads.gascity.com/workflows/index.md

## openspec (seed)

### openspec-1.x-opsx-workflow

- **What**: OpenSpec's `opsx` artifact-guided workflow ships as part of a breaking 1.0 release.
- **Why**: cc's OpenSpec usage (`/feature`, `/apply`, `openspec validate --strict`) is built against the pre-1.0 CLI contract; a breaking 1.0 release plus a new `opsx` artifact-guided workflow could change spec-format expectations, CLI flags, or the validate/archive lifecycle cc's `/feature`/`/apply` commands depend on. Needs research into what actually broke and whether `opsx` is a workflow cc should adopt.
- **Check**: (no local setting — capability/breaking-change question; needs research into the 1.0 CHANGELOG diff against cc's current `openspec validate --strict` / archive usage)
- **Action**: pending research
- **Introduced**: OpenSpec `v1.x`, source: https://github.com/Fission-AI/OpenSpec/blob/main/CHANGELOG.md

## orchestration (v2.1.212-214 delta)

### subagent-spawn-cap-v2.1.212

- **What**: A per-session cap on subagent spawns, default 200, override via `CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION`; `/clear` resets the budget.
- **Why**: cc's biggest fan-outs (`/apply:all` mega-waves, `/audit:waves`, ultracode dynamic workflows spawning "tens-to-hundreds of agents") could realistically approach or exceed 200 spawns in one session — needs a concrete count check against our largest documented runs, not a guess.
- **Check**: (no local setting — capacity question; needs research into our largest historical fan-out counts)
- **Action**: pending research
- **Introduced**: `v2.1.212`, 2026-07

### fork-background-session-v2.1.212

- **What**: `/fork` copies the current conversation into a new background session (its own row in `claude agents`) while the original keeps working; the in-session-subagent behavior previously named `/fork` is now `/subtask`.
- **Why**: cc's workflow leans on worktree-isolated or hand-rolled parallel investigation patterns (background `Agent` dispatch, `wt`-CLI worktrees). A native "branch this exact conversation into its own session" primitive could replace some of that hand-rolled machinery for the "investigate this without losing my current context" case.
- **Check**: (no local setting — capability/workflow-fit question)
- **Action**: pending research
- **Introduced**: `v2.1.212`, 2026-07

### mcp-auto-background-v2.1.212

- **What**: MCP tool calls running longer than 2 minutes now move to the background automatically so the session stays usable; threshold configurable/disableable via `CLAUDE_CODE_MCP_AUTO_BACKGROUND_MS`.
- **Why**: cc runs several MCP servers with potentially slow calls (gsheets batch operations, fallow analysis, context7 doc fetches) — worth knowing whether any of our documented MCP usage patterns rely on a call blocking synchronously past 2 minutes in a way auto-backgrounding would change.
- **Check**: (no local setting — behavior-fit question)
- **Action**: pending research
- **Introduced**: `v2.1.212`, 2026-07

## hooks (v2.1.212-214 delta)

### hook-exit2-schema-validation-fix-v2.1.214

- **What**: Fixed hooks with exit code 2 not blocking as documented when the hook's stdout JSON fails schema validation — previously such a hook silently failed open instead of blocking per its exit code.
- **Why**: cc runs 25+ hooks; any hook that exits 2 with malformed/schema-invalid JSON was silently non-blocking before this fix and will start blocking correctly now — a real behavior change for any hook that (accidentally or by loose convention) emits invalid JSON on its exit-2 path.
- **Check**: (no automated check — needs per-hook JSON-shape audit against exit-2 paths)
- **Action**: pending research
- **Introduced**: `v2.1.214`, 2026-07

## memory (v2.1.212-214 delta)

### memory-frontmatter-modified-timestamp-v2.1.214

- **What**: An ISO `modified` timestamp field was added to memory file frontmatter.
- **Why**: Unclear yet whether this is a harness-auto-stamped field on CC's own native memory mechanism, or something our bespoke `~/.claude/projects/*/memory/*.md` topic-file convention should start writing manually (0 of 90 current memory files carry a `modified:` field, confirmed 2026-07-18). Needs research to determine which memory system this applies to before deciding whether cc's memory-writing convention needs a change.
- **Check**: `grep -l '^modified:' ~/.claude/projects/"$(echo "$HOME/dev/claude" | tr '/' '-')"/memory/*.md 2>/dev/null | wc -l` (0 = not yet adopted, if applicable)
- **Action**: pending research
- **Introduced**: `v2.1.214`, 2026-07

### memory-frontmatter-hash-truncation-fix-v2.1.214

- **What**: Fixed memory frontmatter values being silently truncated at an inline `#`.
- **Why**: A real historical footgun, now fixed. Swept all 90 current cc memory file frontmatter blocks for a `#` in the `description:` field — zero hits, so no evidence any existing memory was corrupted by this bug.
- **Check**: `grep -l '^description:.*#' ~/.claude/projects/"$(echo "$HOME/dev/claude" | tr '/' '-')"/memory/*.md 2>/dev/null` (empty = clean, verified 2026-07-18)
- **Action**: VERIFIED CLEAN 2026-07-18 — no action needed, informational only.
- **Introduced**: `v2.1.214`, 2026-07

## general (v2.1.212-214 delta)

### endconversation-tool-v2.1.214

- **What**: The `EndConversation` tool lets Claude end a session in cases of sustained user abuse or jailbreak attempts (mirrors claude.ai behavior since 2025).
- **Why**: This ships with its own harness-level usage guidance surfaced generically (deferred-tool description), not something cc's own config needs to author guidance for.
- **Check**: (none — built-in tool, no local config surface)
- **Action**: Informational only — no cc-side action.
- **Introduced**: `v2.1.214`, 2026-07

### pkill-self-match-fix-v2.1.214

- **What**: Fixed the Bash tool killing the Claude session when a `pkill -f` pattern accidentally matched the CLI's own process (Linux).
- **Why**: cc's own `scripts/state/failures/*.jsonl` logs show repeated exit-code-144 failures on `pkill -f "next dev --turbo"`-style commands in the `harness` project (2026-06 through 2026-07) — plausibly this exact bug, though not confirmed with certainty (exit 144 could also be an unrelated process-group kill). No cc-side action needed either way; noting as a bug fix that may explain a recurring failure pattern.
- **Check**: (none — upstream bug fix, informational)
- **Action**: Informational — if the `harness` project's pkill-related dev-server-kill failures recur after upgrading past v2.1.214, the bug is not what was hitting us; if they stop, this was likely the cause.
- **Introduced**: `v2.1.214`, 2026-07

### task-tool-mode-deprecated-v2.1.212

- **What**: Task tool's `mode` parameter deprecated (now ignored); subagents inherit the parent session's permission mode by default.
- **Why**: Swept `commands/`, `skills/`, `agents/` for `mode:`/`"mode":` usage that could be an actual Task/Agent-tool call parameter — every hit found was cc's own unrelated JSON schema field (audit findings `mode: static/live`, wave-state `mode: consolidated`, fallow config `mode: mild`), not a real Task-tool `mode` param.
- **Check**: `grep -rn '"mode"' ~/.claude/commands/ ~/.claude/skills/ ~/.claude/agents/ 2>/dev/null` then manually confirm any hit is unrelated (verified 2026-07-18, all 12 hits unrelated).
- **Action**: VERIFIED CLEAN 2026-07-18 — no action needed, informational only.
- **Introduced**: `v2.1.212`, 2026-07

## settings (v2.1.202-211 delta)

### permission-rule-prefix-hygiene-v2.1.210

- **What**: CC now warns at startup about `Write(path)`, `NotebookEdit(path)`, and `Glob(path)` permission rules — the supported prefixes are `Edit(path)` for write-class tools and `Read(path)` for read-class tools.
- **Why**: Non-canonical rule prefixes may silently fail to match the tools they intend to govern; the startup warning signals they are unsupported spellings.
- **Check**: `! grep -qE '"(Write|NotebookEdit|Glob)\(' ~/.claude/settings.json ~/.claude/settings.local.json 2>/dev/null`
- **Action**: Rewrite any `Write(...)`/`NotebookEdit(...)`/`Glob(...)` permission rules to `Edit(...)`/`Read(...)` equivalents in settings files (user + project scopes).
- **Introduced**: `v2.1.210`, 2026-07

### automode-user-settings-only-v2.1.207

- **What**: `autoMode` is no longer read from repo-resident `.claude/settings.local.json`; only `~/.claude/settings.json` (user scope) is honored. Same release stopped reading `pluginConfigs` from project `.claude/settings.json`.
- **Why**: A repo-level `autoMode` value is now a silent no-op — a setup that relied on per-repo auto-mode opt-in/out has silently changed behavior.
- **Check**: `! grep -l '"autoMode"' ~/dev/claude/.claude/settings.local.json ~/dev/claude/.claude/settings.json 2>/dev/null | grep -q .`
- **Action**: Move any repo-level `autoMode` key to `~/.claude/settings.json`; sweep fleet repos for repo-resident `autoMode`/`pluginConfigs` keys and relocate.
- **Introduced**: `v2.1.207`, 2026-07

### forward-subagent-text-v2.1.211

- **What**: `--forward-subagent-text` flag / `CLAUDE_CODE_FORWARD_SUBAGENT_TEXT` env var includes subagent text and thinking in stream-json output.
- **Why**: Headless/`-p` orchestration runs currently lose subagent narrative — this makes subagent reasoning minable for telemetry, session-forensics, and eval harnesses.
- **Check**: `grep -rqn 'FORWARD_SUBAGENT_TEXT\|forward-subagent-text' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ ~/.claude/commands/ 2>/dev/null`
- **Action**: Evaluate adding the env var to headless eval/forensics harnesses that parse stream-json and want subagent visibility; skip if no consumer exists.
- **Introduced**: `v2.1.211`, 2026-07

## otel (v2.1.202-211 delta)

### workflow-otel-attributes-v2.1.202

- **What**: Telemetry emitted by workflow-spawned agents now carries `workflow.run_id` and `workflow.name` OTel attributes, so a workflow run's activity can be reconstructed from OTel data.
- **Why**: cc exports OTel to Grafana (homelab Alloy pipeline); workflow runs were previously unattributable in dashboards — these attributes enable per-run cost/activity reconstruction.
- **Action**: Manual — check Grafana for `workflow.run_id`/`workflow.name` attributes on recent workflow-spawned agent telemetry; add a dashboard panel or session-forensics query keyed on them if workflow usage warrants.
- **Introduced**: `v2.1.202`, 2026-07

## commands (v2.1.202-211 delta)

### code-review-effort-pr-form-v2.1.202

- **What**: `/review <pr>` reverted to a fast single-pass review; `/code-review <level> <pr#>` is the multi-agent effort-tiered form. `/code-review` findings quality further improved on claude-opus-4-8 (v2.1.206).
- **Why**: Any cc doc/command that describes `/review` as the multi-agent path is stale; review-routing guidance (e.g. /m2m, session guidance) should point at the right command per intent.
- **Check**: `! grep -rn 'review.*multi-agent\|multi-agent.*review' ~/.claude/commands/m2m.md 2>/dev/null | grep -q '/review '`
- **Action**: Audit cc docs/commands referencing `/review` vs `/code-review` semantics and align with the v2.1.202 split.
- **Introduced**: `v2.1.202`, 2026-07

## general (v2.1.202-211 delta)

### doctor-full-checkup-v2.1.205

- **What**: `/doctor` is now a full setup checkup that can diagnose and fix issues (`/checkup` alias); v2.1.206 added a check proposing CLAUDE.md trims for content Claude could derive from the codebase.
- **Why**: cc runs a bespoke ratchet lane (validate-cc) for config health — `/doctor` now overlaps as an upstream-maintained checkup, and its CLAUDE.md-trim check aligns directly with cc's context-floor cost discipline.
- **Action**: Manual — run `/doctor` once on current CLI; compare findings against validate-cc Tier 3 rows; note any check worth adopting or any false positives against cc's deliberate config choices.
- **Introduced**: `v2.1.205`, 2026-07

## worktree (v2.1.202-211 delta)

### enterworktree-confirm-outside-v2.1.206

- **What**: The `EnterWorktree` tool now asks for confirmation before entering a git worktree outside the project's `.claude/worktrees/` directory. Separately, v2.1.211 made "always allow" permission rules save at the repo root so approvals persist across worktrees.
- **Why**: cc's /apply convention uses `<repo>/.worktrees/<session-id>/` — outside the blessed `.claude/worktrees/` path — so any flow using the EnterWorktree tool against /apply worktrees now hits a confirmation prompt (friction in autonomous runs).
- **Check**: `! grep -rqn 'EnterWorktree' ~/.claude/commands/ ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null`
- **Action**: If any cc command/script drives the EnterWorktree tool, either relocate the worktree convention to `.claude/worktrees/` or accept the prompt; if none do (orchestrator uses git CLI directly), record as no-op.
- **Introduced**: `v2.1.206`, 2026-07


## general (v2.1.200-201 delta)

### askuserquestion-no-autocontinue-v2.1.200

- **What**: `AskUserQuestion` dialogs no longer auto-continue by default after an idle timeout. Opt into the old behavior via `/config` (idle timeout setting).
- **Why**: Autonomous/unattended workflows that call `AskUserQuestion` expecting auto-continue will now stall indefinitely until a human responds. The cc workflow:evolve Phase 5 gate uses `AskUserQuestion`-style interaction — unattended evolve runs must not rely on auto-continue.
- **Check**: `grep -rn 'AskUserQuestion' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null | grep -v '#' | wc -l` (>0 = uses exist; review each for auto-continue assumption)
- **Action**: Audit any command or skill that calls `AskUserQuestion` in an unattended context and ensure it does not assume the dialog will resolve automatically. For truly unattended runs, replace with a default/defer decision path.
- **Introduced**: `v2.1.200`, 2026-07

### permission-mode-manual-rename-v2.1.200

- **What**: The permission mode named `"default"` is now called `"Manual"` across the CLI, `--help`, VS Code, and JetBrains. Both `--permission-mode manual` and `"defaultMode": "manual"` are accepted alongside the legacy `"default"` string.
- **Why**: cc uses `"defaultMode": "auto"` — not "default" — so no breakage. But any documentation, scripts, or settings files that reference `"defaultMode": "default"` (not `"auto"`) contain an ambiguous legacy value that should be migrated to `"manual"` for clarity.
- **Check**: `grep -rn '"defaultMode".*"default"' ~/.claude/settings.json ~/.claude/settings.local.json 2>/dev/null | wc -l` (0 = clean)
- **Action**: If any settings file has `"defaultMode": "default"`, rename to `"defaultMode": "manual"`. cc's `"defaultMode": "auto"` is unaffected.
- **Introduced**: `v2.1.200`, 2026-07

## general (v2.1.196-199 delta)

### sonnet-5-default-v2.1.197

- **What**: Claude Sonnet 5 is the new CC default model (2026-06-30): near-Opus agentic quality, native 1M context, new tokenizer (~1.0-1.35x tokens), promo $2/$10 per Mtok through 2026-08-31 (then $3/$15).
- **Why**: Every `model: sonnet` tier alias silently resolves to it. Cost tables keyed by exact model string (scripts/lib/cost-rates.sh) nulled cost for all default-model sessions.
- **Check**: `grep -q 'claude-sonnet-5' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/lib/cost-rates.sh` (exit 0 = rates present)
- **Action**: DONE 2026-07-02 — promo rates added (2/10/4/0.20), runtime-verified. Revert to 3/15/6/0.30 after Aug 31: beads cc-vd8wf.
- **Introduced**: `v2.1.197`, 2026-06

### rules-symlink-loading-fix-v2.1.198

- **What**: `.claude/rules/` conditional (glob-frontmatter) rules now load when the target file is reached via a symlinked path.
- **Why**: ~/.claude -> ~/dev/claude is exactly this topology; the fix makes glob-scoped rules viable here for the first time.
- **Check**: `grep -rlE '^globs:' ~/dev/claude/rules/*.md | wc -l` (0 = no conditional rules in use)
- **Action**: SKIP 2026-07-02 — zero conditional rules exist; rules load via CLAUDE.md tables or Skill(). Re-check on adoption of glob-scoped rules.
- **Introduced**: `v2.1.198`, 2026-06

## agents (v2.1.196-199 delta)

### subagents-background-by-default-v2.1.198

- **What**: Subagents run in the background by default (orchestrator keeps working, notified on completion). `TaskOutput` deprecated in favor of `Read` on the task output file. Subagents cut off by rate limit/API error now return partial work / report the error to the parent.
- **Why**: Doc guidance teaching explicit `run_in_background: true` + TaskOutput-callback polling is stale.
- **Check**: `! grep -q 'TaskOutput callback' ~/.claude/skills/orchestrator-patterns/SKILL.md` (exit 0 = doc updated)
- **Action**: DONE 2026-07-02 — orchestrator-patterns SKILL.md updated at 3 sites (background-by-default + Read-output-file).
- **Introduced**: `v2.1.198`, 2026-06

### explore-inherits-model-v2.1.198

- **What**: Built-in Explore agent inherits the session model (capped opus) instead of haiku; subagents + compaction inherit extended-thinking config.
- **Why**: Only affects setups without explicit model pins.
- **Check**: `grep -q 'model: haiku' ~/.claude/agents/utility/explore.md` (exit 0 = our pin shadows the default)
- **Action**: SKIP 2026-07-02 — deliberate haiku pins on explore + 5 sibling analysts; no thinking config to inherit.
- **Introduced**: `v2.1.198`, 2026-06

## hooks (v2.1.196-199 delta)

### agent-notification-hooks-v2.1.198

- **What**: `claude agents` background sessions fire the Notification hook with typed sub-events `agent_needs_input` / `agent_completed`.
- **Why**: Push instead of dashboard-polling for detached sessions.
- **Check**: `grep -q 'agent_needs_input' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/telemetry.sh` (exit 0 = routed)
- **Action**: DONE 2026-07-02 (user override; analyst rec was skip) — telemetry.sh handle_notification routes both sub-types to nx TTS, bypassing throttle + low-effort suppression. Field name defensively parsed; empirically unverified until a detached session runs.
- **Introduced**: `v2.1.198`, 2026-06

### hook-stderr-exit2-shown-v2.1.199

- **What**: SessionStart/Setup/SubagentStart hooks exiting 2 no longer hide stderr — shown in transcript. Exit 2 has no blocking semantics on these events.
- **Why**: Real hook failures previously vanished (fail-open exit 0 + swallowed stderr).
- **Check**: `grep -q 'PRIMER_FAIL' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/session-primer` (exit 0 = failures surfaced)
- **Action**: DONE 2026-07-02 (user override) — session-primer's 7 sections emit named stderr + exit 2 on failure. session-init (expected-optional failures), skill-list-dedup (JSON contract), titler (no failure paths) deliberately unchanged.
- **Introduced**: `v2.1.199`, 2026-07

## skills (v2.1.196-199 delta)

### stacked-skill-invocation-v2.1.199

- **What**: `/skill-a /skill-b do X` loads all leading skills (up to 5), not just the first. Human-typed CLI input only.
- **Why**: Enables behavior-modifier + task-skill combos in one line.
- **Check**: `grep -q 'Stacked Slash-Skill' ~/.claude/skills/cc-reference/SKILL.md` (exit 0 = documented)
- **Action**: DONE 2026-07-02 (user override) — documented in cc-reference § Command Reference.
- **Introduced**: `v2.1.199`, 2026-07

## settings (v2.1.196-199 delta)

### retry-watchdog-300-v2.1.199

- **What**: `CLAUDE_CODE_RETRY_WATCHDOG` default retry count raised to 300; `CLAUDE_CODE_MAX_RETRIES` cap of 15 lifted. Transient non-usage 429s auto-retry with backoff.
- **Why**: Long multi-agent sessions survive provider hiccups.
- **Check**: `jq -e '.env.CLAUDE_CODE_RETRY_WATCHDOG == "1"' ~/.claude/settings.json` (exit 0 = opted in)
- **Action**: SKIP 2026-07-02 — already adopted (settings.json); new defaults ride the upgrade.
- **Introduced**: `v2.1.199`, 2026-07

### stream-idle-watchdog-default-v2.1.196

- **What**: Streaming idle watchdog (aborts + retries after 5min of no stream events) now default-on for ALL providers. `CLAUDE_ENABLE_STREAM_WATCHDOG=0` disables.
- **Why**: Only extends coverage to Bedrock/Vertex/Foundry/gateway routes.
- **Check**: `! grep -qE 'BEDROCK|VERTEX|FOUNDRY|ANTHROPIC_BASE_URL' ~/.claude/settings.json` (exit 0 = direct API, already covered)
- **Action**: SKIP 2026-07-02 — direct Anthropic API only; was already default-on since v2.1.163.
- **Introduced**: `v2.1.196`, 2026-06

## hooks (v2.1.184–195 delta)

### hook-matcher-exact-match-v2.1.195

- **What**: Hook matchers with hyphenated identifiers (`code-reviewer`, `mcp__brave-search`) now **exact-match** instead of accidentally substring-matching (v2.1.195); match all tools of a hyphenated MCP server with `mcp__brave-search__.*`. Separately, comma-separated matchers (`"Bash,PowerShell"`) that silently never fired are a confirmed footgun — pipe form (`Bash|PowerShell`) is required (v2.1.191 fix).
- **Why**: A silent matcher-semantics shift. cc's Task matcher `*-engineer|ui-*|db-*|api-*|e2e-*` uses glob `*` (not substring) and all cc matchers are pipe-form — likely unaffected — but engineer-agent telemetry/preamble hooks firing on named subagent spawns must be re-verified post-change.
- **Check**: `! jq -r '.hooks | .. | objects | select(has("matcher")) | .matcher' ~/.claude/settings.json 2>/dev/null | grep -qE ','` (exit 0 = no comma matchers = clean)
- **Action**: VERIFIED 2026-06-27 (on installed CLI 2.1.177, which is < 2.1.195 — see ceiling). (1) Matchers comma-free; the ONLY hyphenated matcher in settings.json is wildcard-form `*-engineer|ui-*|db-*|api-*|e2e-*` (SubagentStop verification-prompt) — ZERO bare-hyphenated matchers, so nothing of the exact form v2.1.195 changes exists. (2) Live `db-engineer` spawn (agent af2e0671) produced an `agent-audit.jsonl` record with `child_type:"db-engineer"` — the hyphenated type flows through the telemetry/matcher path correctly; `PostToolUse=Task` matcher has no hyphen so is categorically immune. (3) v2.1.195 changes only bare hyphenated identifiers (accidental-substring -> exact); it does NOT remove wildcard support — Anthropic's own migration example `mcp__brave-search__.*` endorses wildcard form. Conclusion: NOT expected to break. RESIDUAL (defer): one confirmatory `*-engineer` re-spawn after the local binary updates to >=2.1.195 closes the empirical gap on the glob match outcome. Side-note: a direct orchestrator->engineer spawn trips an `orchestrator_direct_actor_spawn` audit classifier (expected).
- **Introduced**: `v2.1.195` (exact-match) / `v2.1.191` (comma fix), 2026-06

## mcp (v2.1.184–195 delta)

### claude-mcp-login-cli-v2.1.186

- **What**: `claude mcp login <name>` / `claude mcp logout <name>` authenticate an MCP server from the CLI without opening the interactive `/mcp` menu; `--no-browser` redirects the auth URL to stdin for completing over SSH. MCP OAuth discovery/token requests also retry transient errors and headless envs skip the browser popup (v2.1.191).
- **Why**: cc's remote OAuth MCP servers (Slack/Figma/PostHog/Sentry/Stripe/Vercel via claude.ai) are a known auth-pain point (memory `mcp_auth.md`). CLI login + headless `--no-browser` makes re-auth scriptable and SSH-friendly instead of requiring an interactive `/mcp` round-trip.
- **Check**: `command -v claude >/dev/null && claude mcp login --help >/dev/null 2>&1` (exit 0 = subcommand exists)
- **Action**: Document `claude mcp login/logout` in `cc-tooling` § MCP. Evaluate a re-auth helper script for the OAuth servers using `--no-browser` so headless/cron sessions can refresh tokens without the interactive menu.
- **Introduced**: `v2.1.186`, 2026-06

### mcp-tool-idle-timeout-v2.1.187

- **What**: Remote MCP tool calls that hang with no response for 5 minutes now abort with an error instead of blocking indefinitely; override the window via `CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`. MCP capability discovery (`tools/list` etc.) retries transient network errors with backoff (v2.1.191).
- **Why**: cc runs 12 MCP servers including remote OAuth ones; an indefinitely-hanging tool call could freeze a long autonomous run. The native 5-min abort is a safety net — informational, but the override is worth knowing for slow servers.
- **Check**: `jq -e '.env.CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT' ~/.claude/settings.json 2>/dev/null`
- **Action**: Inform-only — the 5-min default abort is a strict improvement. Set the env override only if a legitimately-slow MCP server trips it.
- **Introduced**: `v2.1.187`, 2026-06

## settings (v2.1.184–195 delta)

### respond-to-bash-commands-v2.1.186

- **What**: `!` bash commands now **trigger Claude to respond to the output automatically**; set `"respondToBashCommands": false` in settings.json to keep the previous context-only (silent) behavior.
- **Why**: cc's session guidance tells the user to type `! <command>` for interactive logins (e.g. `gcloud auth login`) so output lands in the conversation. With the new default, every `! <cmd>` now also spends a model turn responding — changes the cost/UX of the bash-mode escape hatch. cc silently inherited the new default (`respondToBashCommands` absent).
- **Check**: `grep -q 'respondToBashCommands' ~/.claude/settings.json` (exit 0 = explicitly set; non-zero = on the new auto-respond default)
- **Action**: Decide: keep auto-respond (richer, but a model turn per `!`), or set `"respondToBashCommands": false` to restore silent context-only injection for the login/paste-output pattern cc documents.
- **Introduced**: `v2.1.186`, 2026-06

### automode-classifyallshell-v2.1.193

- **What**: `autoMode.classifyAllShell` routes **all** Bash/PowerShell commands through the auto-mode classifier, instead of only arbitrary-code-execution patterns. Auto-mode denial reasons now surface in the transcript, denial toast, and `/permissions` recent denials.
- **Why**: cc runs `permissions.defaultMode: auto` with a custom `autoMode.hard_deny` ($defaults + 3 force-push/rm-rf rules). Today only arbitrary-code patterns hit the classifier; `classifyAllShell` would subject every shell command — closing the gap where a benign-looking but dangerous command bypasses classification. Trade-off: latency + more frequent classifier passes on routine commands.
- **Check**: `jq -e '.autoMode.classifyAllShell' ~/.claude/settings.json 2>/dev/null`
- **Action**: Evaluate `"autoMode": { "classifyAllShell": true }` for stronger coverage of the hard_deny rules. Weigh against per-command classifier latency on a high-throughput orchestration flow (`/apply:all`).
- **Introduced**: `v2.1.193`, 2026-06

### sandbox-credentials-v2.1.187

- **What**: `sandbox.credentials` setting blocks sandboxed commands from reading credential files and secret environment variables.
- **Why**: cc has no `sandbox` config today (`permissions.deny` covers `Read(.env*)` / `Write(*credentials*)` but not arbitrary sandboxed reads of secret env). A sandbox credential-block is defense-in-depth against a subagent or tool exfiltrating secrets via the environment. Pairs with the existing `permissions.deny` env rules.
- **Check**: `jq -e '.sandbox.credentials' ~/.claude/settings.json 2>/dev/null`
- **Action**: Evaluate adding a `sandbox.credentials` block. Confirm it does not break legitimate tooling that reads `~/.claude` tokens or `POSTGRES_URL`-style env the engineers need.
- **Introduced**: `v2.1.187`, 2026-06

### max-retries-watchdog-v2.1.186

- **What**: `CLAUDE_CODE_MAX_RETRIES` now caps at 15; for unattended sessions, use `CLAUDE_CODE_RETRY_WATCHDOG` instead (which keeps retrying past the cap with a watchdog).
- **Why**: Multi-hour `/apply:all` waves and `ralph-loop` runs are exactly the unattended sessions this targets. With neither var set, cc uses the default retry behavior; a transient overload mid-wave could exhaust retries and kill the orchestrator. `CLAUDE_CODE_RETRY_WATCHDOG` is the resilience lever, complementing the already-adopted `fallbackModel` chain.
- **Check**: `jq -e '.env.CLAUDE_CODE_RETRY_WATCHDOG // .env.CLAUDE_CODE_MAX_RETRIES' ~/.claude/settings.json 2>/dev/null`
- **Action**: For unattended autonomous runs, set `CLAUDE_CODE_RETRY_WATCHDOG` in settings.json env so long waves survive transient API overload. Pairs with `fallbackModel` (adopted 2026-06-20).
- **Introduced**: `v2.1.186`, 2026-06

## permissions (v2.1.184–195 delta)

### agent-type-permission-enforced-v2.1.186

- **What**: `Agent(type)` deny rules and `Agent(x,y)` allowed-types restrictions are now **enforced for named subagent spawns** (previously only loosely applied). `--print`/headless already honors agent frontmatter `tools:`/`disallowedTools:` (v2.1.119).
- **Why**: cc's Master Orchestrator discipline ("Spawn engineers, NEVER edit code directly") is prose-enforced. With `Agent(type)` now enforced, the orchestrator role could be hardened to deny spawning specific agent types, or — combined with the existing `ask Agent(model:opus)` rule — gain real harness teeth. Extends the v2.1.178 tool-param permission signal.
- **Check**: `jq -r '.permissions | (.deny[]?, .ask[]?, .allow[]?) | select(test("Agent\\("))' ~/.claude/settings.json 2>/dev/null | grep -q 'Agent(' && echo present || true`
- **Action**: Evaluate `Agent(type:...)` deny/allow rules to encode orchestrator-vs-engineer role boundaries in the harness. Confirm it doesn't block legitimate fan-out (audit/apply spawn many engineer types).
- **Introduced**: `v2.1.186`, 2026-06

## otel (v2.1.184–195 delta)

### otel-assistant-response-v2.1.193

- **What**: New `claude_code.assistant_response` OpenTelemetry log event carries the model's response text. Redacted unless `OTEL_LOG_ASSISTANT_RESPONSES=1`; when unset it **follows `OTEL_LOG_USER_PROMPTS`** — so a deployment already logging prompt content starts receiving response content on upgrade. Set `OTEL_LOG_ASSISTANT_RESPONSES=0` to keep prompts-only.
- **Why**: cc runs a full OTel pipeline to nexus-agent (localhost:4318). `OTEL_LOG_USER_PROMPTS` is **unset** here, so `assistant_response` stays redacted on upgrade — NOT the upgrade foot-gun it is for prompt-logging deployments. But the event is a new content signal nexus could opt into for response-text analytics if desired.
- **Check**: `jq -e '.env.OTEL_LOG_ASSISTANT_RESPONSES' ~/.claude/settings.json 2>/dev/null`
- **Action**: Inform-only — safe by default (no prompt logging here). Optionally set `OTEL_LOG_ASSISTANT_RESPONSES=1` only if nexus wants response-text capture, weighing storage + privacy.
- **Introduced**: `v2.1.193`, 2026-06

## settings (v2.1.159–183 delta)

### fallback-model-settings-key-v2.1.166

- **What**: `fallbackModel` settings.json key configures up to 3 fallback models tried in order on overload/unavailability; `--fallback-model` now also applies to interactive sessions (v2.1.166) and compaction honors the chain (v2.1.178). Non-additive across scopes (highest-precedence file wins the whole chain).
- **Why**: A shell alias only covers shell-launched sessions; the settings key extends fallback to IDE/desktop/background-daemon + compaction — resilience for multi-hour ultracode / apply:all runs on Opus 4.8.
- **Check**: `jq -e '.fallbackModel' ~/.claude/settings.json 2>/dev/null`
- **Action**: Add `"fallbackModel": ["claude-sonnet-4-6", "claude-haiku-4-5"]` to settings.json. **DONE 2026-06-20** — coexists with the shell alias (CLI flag overrides settings).
- **Introduced**: `v2.1.166`, 2026-06

### attribution-sessionurl-v2.1.183

- **What**: `attribution.sessionUrl` setting omits the claude.ai session link from commits/PRs in web/Remote Control sessions.
- **Why**: Only affects web/RC commits — low relevance for a CLI-primary flow.
- **Check**: `jq -e '.attribution.sessionUrl' ~/.claude/settings.json 2>/dev/null`
- **Action**: Set `"attribution": {"sessionUrl": false}` only if committing from web/RC. Skipped 2026-06-20 (no surface).
- **Introduced**: `v2.1.183`, 2026-06

### automode-builtin-destructive-defaults-v2.1.183

- **What**: Auto-mode built-in deny expanded — `git reset --hard` / `checkout -- .` / `clean -fd` / `stash drop` (when not discarding), `git commit --amend` on non-agent commits, `terraform`/`pulumi`/`cdk destroy`.
- **Why**: `autoMode.hard_deny` with `$defaults` pulls these in automatically — verify `$defaults` is present.
- **Check**: `jq -e '.autoMode.hard_deny | index("$defaults")' ~/.claude/settings.json 2>/dev/null`
- **Action**: None — `$defaults` already present (verified 2026-06-20). Informational.
- **Introduced**: `v2.1.183`, 2026-06

## skills (v2.1.159–183 delta)

### disable-bundled-skills-v2.1.169

- **What**: `disableBundledSkills` setting + `CLAUDE_CODE_DISABLE_BUNDLED_SKILLS` env hide bundled skills, workflows, and built-in slash commands from the model catalog.
- **Why**: Token-budget lever for the 90-skill custom suite — but all-or-nothing.
- **Check**: `jq -e '.disableBundledSkills' ~/.claude/settings.json 2>/dev/null`
- **Action**: DEFERRED — all-or-nothing toggle would drop `/loop` + `/claude-api` (both used, no custom equivalents). ~800 tok saving not worth it; revisit if upstream adds per-skill granularity.
- **Introduced**: `v2.1.169`, 2026-06

## permissions (v2.1.159–183 delta)

### tool-param-permission-syntax-v2.1.178

- **What**: `Tool(param:value)` permission-rule syntax matches a tool's input params with `*` wildcard — e.g. `Agent(model:opus)` blocks Opus subagents; glob in deny tool-name position (`"*"`) denies all (v2.1.166).
- **Why**: Harness-enforce subagent cost tiers (20/32 agents opus) + Master-Orchestrator role discipline (currently prose-only).
- **Check**: `jq -c '.permissions.ask' ~/.claude/settings.json` (expect `[]`)
- **Action**: REVERTED 2026-08-04 — rule removed from `permissions.ask`, exactly the one-line revert its own decision record flagged ("revert if it impedes waves"). Transcript evidence: ~5.9K historical opus subagent dispatches, so it prompted on every `/apply` and `/audit` engineer spawn — a direct collision with CORE.md § No Fabricated Pauses. Role discipline is already enforced by `gate_agent_spawn` (`scripts/hooks/gate.sh:929`); cost visibility now comes from OTEL + `scripts/lib/cost-rates.sh`. No `validate-cc` row or test asserted the rule.
- **Prior**: RESOLVED 2026-06-20 (CC 2.1.183, bd cc-c3grf). **`Agent(model:opus)` matches the RESOLVED model incl. frontmatter** (docs-confirmed) — so a `deny` would block all 21 opus agents and `ask` prompts on EVERY opus spawn. Leo chose `ask Agent(model:opus)` (cost visibility) over the safer Fable-5 ceiling, accepting the autonomous-wave prompt cost. Single-param rules can't scope opus-by-agent-type. Permissions hot-reload.
- **Introduced**: `v2.1.178`, 2026-06

## agents (v2.1.159–183 delta)

### subagent-nested-spawn-v2.1.172

- **What**: Sub-agents can spawn their own sub-agents (up to 5 levels deep); foreground subagents respect the same limit (v2.1.181).
- **Why**: Flat orchestration is a deliberate cc design; CHA/qa-orchestrator (agent since deleted 2026-07-12, cowork-audit-remediation C5 — orphan, cited a phantom command) already operated at depth-2 passively (now legal).
- **Check**: (no setting — capability change)
- **Action**: DEFERRED — no concrete win over root-orchestrated fan-out. Optional hygiene: add depth guards to engineer agents that carry `Agent` in tools.
- **Introduced**: `v2.1.172`, 2026-06

### teamcreate-teamdelete-removed-v2.1.178

- **What**: `TeamCreate`/`TeamDelete` tools removed — with `AGENT_TEAMS=1` every session has one implicit team; spawn teammates via the Agent tool `name` param (`team_name` ignored).
- **Why**: Any reference is now dead. (AGENT_TEAMS=1 is active here.)
- **Check**: `grep -rln 'TeamCreate\|TeamDelete\|team_name' ~/.claude/commands/ ~/.claude/skills/ ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null | grep -v cache`
- **Action**: None — cc surface verified clean 2026-06-20 (zero refs). Recorded in deprecations.md.
- **Introduced**: removed `v2.1.178`, 2026-06

## hooks (v2.1.159–183 delta)

### stop-hook-additionalcontext-v2.1.163

- **What**: `Stop`/`SubagentStop` hooks can return `hookSpecificOutput.additionalContext` to feed Claude corrective context and keep the turn going without a hook-error label.
- **Why**: Enforce the "not done until push" iron law softly at session close, mirroring the adopted PostToolUse `continueOnBlock`.
- **Check**: `grep -rln 'additionalContext' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/ 2>/dev/null`
- **Action**: **DONE 2026-06-20** — new `scripts/hooks/stop-push-gate.sh` wired as a 2nd Stop hook; nudges once/session on uncommitted/unpushed work (per-session marker + stop_hook_active guard).
- **Introduced**: `v2.1.163`, 2026-06

### hook-if-path-conditions-v2.1.176

- **What**: Hook `if:` conditions for Read/Edit/Write paths now match (`Edit(src/**)`, `Read(.env)`); `if: "Bash(...)"` matches inside `$()`/backticks (v2.1.163). **v2.1.214 refinement**: a single-segment `dir/**` `if:` condition now matches only `<cwd>/dir` (was any-depth) — write `**/dir/**` if any-depth matching is actually wanted. Permission `deny`/`ask` rules are unaffected and keep any-depth match; only hook `if:` semantics changed.
- **Why**: Could scope cc's context-blind matchers (rules/TOOLING.md anti-pattern) to relevant paths. The v2.1.214 scoping change matters if/when this is ever applied — a single-segment glob like `Edit(scripts/**)` would now mean "only scripts/ directly under the target dir" rather than "any scripts/ anywhere in the tree," which changes the brace-glob example below.
- **Check**: `jq -r '.hooks | .. | objects | select(.if) | .if' ~/.claude/settings.json 2>/dev/null | head`
- **Action**: HELD (verify-then-apply) — no `if:` precedent + hooks load at session start (unverifiable same-session) + `if:` FAILS CLOSED (unsupported brace-glob → validation silently OFF) for a marginal saving (validate-file already fast-exits non-target types). Verify CC brace-glob support AND the v2.1.214 single-segment-scoping semantics after a restart before applying `if: Edit(**/*.{ts,tsx,md,rs})` to the validate-file hook.
- **Introduced**: `v2.1.176`, 2026-06 (dir/** scoping refined `v2.1.214`, 2026-07)

## commands (v2.1.159–183 delta)

### safe-mode-flag-v2.1.169

- **What**: `--safe-mode` flag / `CLAUDE_CODE_SAFE_MODE` env start CC with all customizations (CLAUDE.md, plugins, skills, hooks, MCP) disabled.
- **Why**: Fast "is it my config?" bisect given cc's heavy customization (25+ hooks, 90 skills, 12 MCP).
- **Check**: `grep -rn 'safe-mode\|CLAUDE_CODE_SAFE_MODE' ~/.claude/skills/cc-reference/ ~/.claude/skills/cc-tooling/ 2>/dev/null`
- **Action**: DEFERRED — doc-only candidate for cc-reference/cc-tooling troubleshooting; not urgent.
- **Introduced**: `v2.1.169`, 2026-06

## settings (v2.1.157–159 delta)

### opus4-6-fast-mode-deprecation-v2.1.154

- **What**: `CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE` env var deprecated in v2.1.154 and **removed 2026-06-01** (today). Previously allowed fast mode on Opus 4.6. To use fast mode on Opus 4.6, switch via `/model claude-opus-4-6[1m]` then `/fast on`.
- **Why**: If this env var is present in `~/.claude/settings.json` env block or shell init files, it is now a dead no-op at best; an error source at worst. Urgent cleanup.
- **Check**: `grep -rn 'CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE' ~/.claude/settings.json ~/.zshrc ~/.bashrc 2>/dev/null`
- **Action**: Remove `CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE` from any settings or shell files where it appears. No replacement needed unless Opus 4.6 fast mode is still wanted (use `/model` + `/fast` instead).
- **Introduced**: `v2.1.154`, deprecated 2026-05-30, removed 2026-06-01

## agents (v2.1.157–159 delta)

### bg-exec-shell-session-v2.1.154

- **What**: In `claude agents`, type `! <command>` to run a shell command as a background session you can attach/detach. Also available as `claude --bg --exec '<command>'` from the CLI.
- **Why**: leo currently manages long-running processes (ralph-loop polling, dev servers, build watchers) via separate terminals. `! <command>` turns any shell command into a managed background session visible in `claude agents`, with attach/detach and logging. Complements pinned-bg-session persistence (v2.1.147).
- **Check**: `grep -rn 'ralph\|bg.*exec\|--bg.*exec' ~/.claude/commands/ ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null | head`
- **Action**: Evaluate whether `ralph-loop` or recurring monitor scripts should be launched via `! <command>` / `claude --bg --exec` so they appear in the agents dashboard and survive CC updates. Low friction to try.
- **Introduced**: `v2.1.154`, 2026-05

## skills (v2.1.157–159 delta)

### plugin-default-disabled-v2.1.154

- **What**: Plugins can now declare `defaultEnabled: false` in `plugin.json` or a marketplace entry. They start disabled and must be explicitly enabled via `/plugin` or `claude plugin enable`. Dependencies of enabled plugins auto-enable.
- **Why**: With 200+ skills in the catalog, token budget for the skill-description context is a real constraint. `defaultEnabled: false` lets heavy single-use skills (design-system-starter, algorithmic-art, aceternity) ship in the catalog without consuming description tokens until explicitly enabled. Pairs with `skillOverrides` (v2.1.129) for a layered token-reduction strategy.
- **Check**: `grep -rn '"defaultEnabled"' ~/.claude/plugins/ /home/user/central-claude/skills/ 2>/dev/null | head`
- **Action**: For heavy, rarely-used skills in the cc marketplace, add `"defaultEnabled": false` to their `plugin.json`. Candidates: aceternity, algorithmic-art, awesome-design-md, wayfinder. Test that explicit `/plugin enable` works.
- **Introduced**: `v2.1.154`, 2026-05

### plugin-discover-dir-relevance-v2.1.154

- **What**: `/plugin` Discover tab pins plugins whose relevance signals match the current directory with a "suggested for this directory" annotation.
- **Why**: cc project detection (`project.toml` stack signals) could double as plugin relevance metadata — visiting an Effect project would surface the `effect` skill; a T3 project surfaces `t3-code-patterns`. Informational for now but worth ensuring cc's skills carry appropriate relevance metadata.
- **Check**: (no automated check — UX feature; inspect a project dir and run `/plugin` to observe suggestions)
- **Action**: Review skill `plugin.json` files to ensure relevance/keywords metadata is populated for directory-signal matching. Candidates: t3-code-patterns, drizzle, effect, docker.
- **Introduced**: `v2.1.154`, 2026-05

## orchestration (v2.1.150–158 delta — FOCUS CLUSTER)

### dynamic-workflows-v2.1.154

- **What**: Dynamic workflows — the `Workflow` tool primitive. Ask Claude to author a JS workflow script and it orchestrates tens-to-hundreds of agents in the background via `pipeline()`/`parallel()`/`agent()`. `/workflows` views runs; resume via `resumeFromRunId`.
- **Why**: cc's orchestration commands (`/apply:all`, `/audit:waves`, `/audit:code`, `/workflow:evolve`) currently hand-roll fan-out with parallel `Agent` tool calls + bash state files (`scripts/lib/orchestrator-patterns`, `wave-state`, on-disk fan-out artifacts). The native primitive offers deterministic control flow, crash-resume via journal, token-budget gating, and worktree isolation per agent — superseding large parts of the hand-rolled machinery. This is the headline ultracode capability.
- **Check**: `grep -rln 'export const meta' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null` (presence of any Workflow script authored in our command surface)
- **Action**: Decide which orchestrators migrate to `Workflow`. Strong candidates: `/audit:code` (read-only fan-out, no user gate), `/audit:waves`, `/audit:journeys`. Poor fit: anything with a mid-run user gate (`/workflow:evolve` Phase 5, `/triage`) — Workflow can't pause for input. Pilot one read-only audit command; compare resume + cost vs the bash-state approach.
- **Introduced**: `v2.1.154`, 2026-05

### workflow-keyword-trigger-setting-v2.1.158

- **What**: `workflowKeywordTriggerEnabled` controls whether the dynamic-workflow trigger keyword auto-launches a workflow. **The keyword was renamed `workflow` → `ultracode` in v2.1.160**, then scoped to explicit phrases ("run a workflow", "workflow:") in v2.1.178.
- **Why**: The original concern (cc's `/workflow:*` command names spuriously triggering) is **obsolete** post-rename. Setting is `false` on this machine; keeping it `false` is harmless (dynamic workflows still launch explicitly via the Workflow tool / saved workflows / ultracode mode).
- **Check**: `grep -q 'workflowKeywordTriggerEnabled' ~/.claude/settings.json && echo present || echo absent` (NB: `jq '. // "ABSENT"'` lies here — `false // x` returns the fallback. Use grep or `jq -e 'has(...)'`.)
- **Action**: RESOLVED 2026-06-20 — present = `false` at settings.json:423 (NOT drift; the prior "ABSENT" was a jq `//`-operator false-flag). Leo chose keep `false`. NOTE: the `ultracode` keyword could collide with `/effort ultracode`.
- **Introduced**: `v2.1.158`, 2026-05 (keyword renamed `v2.1.160`)

### streaming-tool-exec-always-v2.1.154

- **What**: Streaming tool execution is now always enabled, including telemetry-disabled and Bedrock/Vertex/Foundry runs. Previously behind a feature flag.
- **Why**: Any feature-flag env var we set to opt into streaming tool execution is now dead config. Dead flags are silent drift.
- **Check**: `grep -rn 'STREAMING_TOOL\|STREAM_TOOL_EXEC\|TOOL_STREAMING' ~/.claude/settings.json ~/.zshrc 2>/dev/null || true`
- **Action**: If any streaming-tool-execution feature-flag env var is set, remove it (now a no-op). Otherwise no-op — informational.
- **Introduced**: `v2.1.154`, 2026-05

## skills (v2.1.150–158 delta)

### frontmatter-disallowed-tools-v2.1.152

- **What**: Skills and slash commands can set `disallowed-tools:` in frontmatter to remove tools from the model while the skill/command is active.
- **Why**: Leo's Master Orchestrator rule ("Spawn engineers. NEVER edit `.ts/.tsx/.js/.sql`. NEVER run `pnpm build/test`") is currently enforced by prose + discipline. `disallowed-tools` makes it a hard runtime constraint: orchestrator commands (`/apply:all`, `/audit:waves`) could disallow `Edit`/`Write`/`NotebookEdit` so the orchestrator physically cannot write code. Converts a documented norm into an enforced invariant.
- **Check**: `grep -rln 'disallowed-tools' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null`
- **Action**: Add `disallowed-tools: [Edit, Write, NotebookEdit]` to the frontmatter of orchestrator commands that must never touch code directly (`/apply:all`, `/audit:waves`, `/audit:code`, `/workflow:evolve`). Verify it doesn't block the MD-only edits the orchestrator legitimately makes.
- **Introduced**: `v2.1.152`, 2026-05

### reload-skills-v2.1.152

- **What**: `/reload-skills` command re-scans skill directories without a session restart; `SessionStart` hooks can return `reloadSkills: true` to make hook-installed skills available in the same session.
- **Why**: Flows that install skills mid-session (`find-skills` install path, `project:init`, `npx skills add`) currently require a restart before the new skill is usable. `reloadSkills` closes that gap.
- **Check**: `grep -rln 'reloadSkills\|/reload-skills' ~/.claude/commands/ ~/.claude/skills/ ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/ 2>/dev/null`
- **Action**: Document `/reload-skills` in `cc-tooling` skill. If any SessionStart hook installs skills, have it return `reloadSkills: true`.
- **Introduced**: `v2.1.152`, 2026-05

### claude-skills-plugin-autoload-v2.1.158

- **What**: Plugins placed in `.claude/skills` directories are auto-loaded with no marketplace required; `claude plugin init <name>` scaffolds a new plugin there.
- **Why**: cc ships 3 in-house skills (`find-skills`, `request-radar`, and others) plus a custom marketplace. Auto-load simplifies distributing single-skill plugins without marketplace ceremony. Affects `scripts/bin/cc-inventory` (must handle both shapes) and our plugin-authoring workflow.
- **Check**: `find ~/.claude/skills -maxdepth 2 -name plugin.json 2>/dev/null | head` (presence of plugin-shaped skills under the skills dir)
- **Action**: Inform-only for now; when authoring a new single-skill plugin, prefer `claude plugin init` into `.claude/skills`. Confirm `cc-inventory` enumerates auto-loaded plugin-skills.
- **Introduced**: `v2.1.158`, 2026-05

## hooks (v2.1.150–158 delta)

### messagedisplay-hook-v2.1.152

- **What**: New `MessageDisplay` hook event lets a hook transform or hide assistant message text as it is displayed (post-generation, pre-render).
- **Why**: Leo's no-emoji rule is enforced at commit time (pre-commit emoji guard) and by instruction, but not at chat-display time. `MessageDisplay` is a display-time enforcement channel: strip stray emoji, redact secrets in rendered output, or annotate. A complementary layer to the source-side guard.
- **Check**: `grep -rn 'MessageDisplay' ~/.claude/settings.json 2>/dev/null`
- **Action**: Evaluate a `MessageDisplay` hook that strips emoji from rendered assistant text as a last-line display guard (the rule is instruction-enforced today). Weigh against latency on every message render. Likely low priority unless emoji leakage is observed.
- **Introduced**: `v2.1.152`, 2026-05

### sessionstart-session-title-v2.1.152

- **What**: `SessionStart` hooks can set the session title via `hookSpecificOutput.sessionTitle`, on both startup and resume.
- **Why**: Leo runs many parallel sessions (`/apply:all` waves, worktree agents, ralph loops) surfaced in the `claude agents` dashboard. Untitled sessions are hard to disambiguate. A SessionStart hook could title each session with `<project> · <active-spec-or-wave>` for instant dashboard legibility.
- **Check**: `grep -rn 'sessionTitle' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/ ~/.claude/settings.json 2>/dev/null`
- **Action**: Add `sessionTitle` to the existing SessionStart hook output — derive from CWD project code + active beads/wave state (the SessionStart hook already computes project context per the startup output).
- **Introduced**: `v2.1.152`, 2026-05

## otel (v2.1.150–158 delta)

### otel-tool-parameters-v2.1.158

- **What**: `tool_decision` telemetry events now include `tool_parameters` (bash command strings, MCP/skill names) when `OTEL_LOG_TOOL_DETAILS=1`.
- **Why**: nexus-agent attributes cost/usage by tool type but not by which specific bash command or skill. `tool_parameters` enables "which bash commands dominate", "which skills are invoked most" without parsing transcripts. Pairs with the `skill_activated` and `agent_id` signals already tracked.
- **Check**: `jq -e '.env.OTEL_LOG_TOOL_DETAILS' ~/.claude/settings.json 2>/dev/null`
- **Action**: Set `OTEL_LOG_TOOL_DETAILS=1` in settings.json env, then update nexus-agent's OTel ingester to index `tool_parameters` on `tool_decision` events. Privacy tradeoff: bash command strings may contain sensitive args — confirm nexus storage is acceptable before enabling.
- **Introduced**: `v2.1.158`, 2026-05

### otel-app-entrypoint-v2.1.152

- **What**: `app.entrypoint` OpenTelemetry metric attribute distinguishes the session entrypoint (interactive, `-p`, SDK, background). Opt-in via `OTEL_METRICS_INCLUDE_ENTRYPOINT=true`.
- **Why**: cc cost is split across interactive sessions, `claude -p` script runs (scripts/bin/*), and `/apply:all` background orchestration. `app.entrypoint` lets nexus dashboards separate "ad-hoc interactive spend" from "automated -p spend" — a real budgeting facet.
- **Check**: `jq -e '.env.OTEL_METRICS_INCLUDE_ENTRYPOINT' ~/.claude/settings.json 2>/dev/null`
- **Action**: Set `OTEL_METRICS_INCLUDE_ENTRYPOINT=true` in settings.json env; add an `app.entrypoint` facet to nexus cost dashboards.
- **Introduced**: `v2.1.152`, 2026-05

## settings (v2.1.150–158 delta)

### settings-agent-field-v2.1.158

- **What**: The `agent` field in `settings.json` is now honored for dispatched sessions; `--agent <name>` overrides it per-invocation.
- **Why**: Lets a default agent persona apply to dispatched background sessions without passing `--agent` each time. Relevant if cc standardizes a default orchestrator/persona for `claude agents` dispatch or `--bg` runs.
- **Check**: `jq -e '.agent' ~/.claude/settings.json 2>/dev/null`
- **Action**: Evaluate whether a default dispatched-session agent makes sense. Likely defer — cc dispatches engineers explicitly via the `Agent` tool, not via settings default. Document the field's existence.
- **Introduced**: `v2.1.158`, 2026-05

### fallback-model-session-v2.1.152

- **What**: When the primary model is not found, Claude Code now switches to the configured `--fallback-model` for the rest of the session instead of failing every request.
- **Why**: Multi-hour `/apply:all` runs on Opus 4.8 are vulnerable to a transient model-not-found (gateway hiccup, model rename). A configured fallback keeps the orchestrator alive instead of failing every turn — direct resilience win for long autonomous runs.
- **Check**: `jq -e '.fallbackModel // .env.ANTHROPIC_FALLBACK_MODEL' ~/.claude/settings.json 2>/dev/null`
- **Action**: Set a `fallbackModel` (e.g. `claude-sonnet-4-6`) in settings.json so long background runs degrade gracefully rather than dying. Confirm the exact settings key via `/config`.
- **Introduced**: `v2.1.152`, 2026-05

## worktree (v2.1.150–158 delta)

### enterworktree-switch-v2.1.158

- **What**: `EnterWorktree` can now switch between Claude-managed worktrees mid-session (not just enter one).
- **Why**: `/apply:all` runs per-session worktree isolation. Mid-session switching could let one orchestrator hop between wave worktrees instead of spawning fresh sessions — but may conflict with our `wt`-CLI model where each session owns one worktree.
- **Check**: `grep -rn 'EnterWorktree' ~/.claude/commands/ ~/.claude/skills/ ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null | head`
- **Action**: Inform-only. Evaluate whether any orchestrator benefits from mid-session worktree switching vs the current one-session-one-worktree model. Likely skip — our isolation model is deliberate.
- **Introduced**: `v2.1.158`, 2026-05

### worktree-unlocked-on-finish-v2.1.158

- **What**: Claude-managed worktrees are now left unlocked when the agent finishes, so `git worktree remove`/`prune` can clean them up. Also fixes background-agent worktrees orphaned after the 30-day retention sweep.
- **Why**: Our `wt reap` + `wt_destroy` (worktree-helpers.sh) clean up stale worktrees. CC's native cleanup just got more cooperative — our reaper may now be redundant or could simplify (no need to force-unlock before remove).
- **Check**: `grep -nE 'worktree (unlock|remove|prune)|wt reap' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/lib/worktree-helpers.sh ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/wt 2>/dev/null | head`
- **Action**: Audit `wt reap`/`wt_destroy` against the new native unlock-on-finish behavior; simplify if our force-unlock step is now redundant.
- **Introduced**: `v2.1.158`, 2026-05

## mcp (v2.1.150–158 delta)

### mcp-stdio-session-env-v2.1.154

- **What**: Stdio MCP server subprocesses now receive `CLAUDE_CODE_SESSION_ID` and `CLAUDECODE=1` in their environment (matching the Bash-tool env from v2.1.132).
- **Why**: Extends session-correlation to MCP servers. nova-memory MCP (nv project scope) could tag stored memories with the originating `CLAUDE_CODE_SESSION_ID`, enabling cross-session memory provenance and per-session telemetry correlation for MCP-emitted events.
- **Check**: `grep -rn 'CLAUDE_CODE_SESSION_ID' ~/dev/nv 2>/dev/null | head` (does nova-memory read it?)
- **Action**: Update nova-memory MCP server to read `CLAUDE_CODE_SESSION_ID` from its env and attach as provenance on stored memories. Pairs with the v2.1.132 bash-session-id signal.
- **Introduced**: `v2.1.154`, 2026-05

## commands

### goal-command-v2.1.139

- **What**: `/goal <completion-condition>` command lets CC keep working across turns until a specified completion condition is met. Works in interactive, `-p`, and Remote Control modes. Shows live elapsed/turns/tokens as an overlay panel.
- **Why**: Native primitive for what Leo currently approximates with `ralph-loop` and ad-hoc multi-turn orchestration. A completion-condition gate (e.g. "all e2e tests pass" or "wave 3 status=completed") could replace ralph's polling loop for a class of long-running checks.
- **Check**: `grep -rn '/goal ' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null`
- **Action**: Evaluate whether `/apply:all` resume loops, `/p2p` deploy-wait gates, or `unified-gate-runner` retry attempts (invoked from `/m2m`) could be expressed as a `/goal` condition. Document one canonical usage example.
- **Introduced**: `v2.1.139`, 2026-05

### agent-view-v2.1.139

- **What**: `claude agents` opens a dashboard of every CC session (running, blocked on you, done). Replaces the old `agent --list` and aggregates state across worktrees and `/bg`-launched agents.
- **Why**: Leo runs many parallel `/apply:all` waves + worktree agents — a multi-session view is the natural mental model. Knowing which session is "blocked on you" is the main UX gain.
- **Check**: (no setting — UX feature; check usage with `which claude && claude agents --help 2>&1 | head`)
- **Action**: Manual — try `claude agents` once to see if it fits the cc workflow. No config change required.
- **Introduced**: `v2.1.139`, 2026-05

## hooks

### hook-args-exec-form-v2.1.139

- **What**: Hook config now accepts `args: string[]` field (exec form) that spawns the command directly without a shell. Eliminates shell quoting / path-placeholder escaping bugs.
- **Why**: `~/.claude/settings.json` has 25+ hooks shelling to `bash -c "..."`. Each `command:` string is a shell-quoting hazard. `args:` form avoids the entire class of bugs (especially with paths containing spaces or special chars in worktrees).
- **Check**: `jq -r '.hooks | .. | objects | select(.args) | .args' ~/.claude/settings.json 2>/dev/null | head`
- **Action**: Audit `~/.claude/settings.json` hooks; convert top offenders (those passing `$CLAUDE_HOOK_INPUT` or worktree paths to scripts) from `"command": "bash -c '...'"` to `"command": "...", "args": [...]`.
- **Introduced**: `v2.1.139`, 2026-05

### posttooluse-continueonblock-v2.1.139

- **What**: PostToolUse hooks can now set `continueOnBlock: true` to feed the hook's rejection reason back to Claude and continue the turn instead of halting it. Previously, a PostToolUse block always halted.
- **Why**: Memory `hook_architecture.md` notes PostToolUse validate-file is silently broken for Edit ops. Even if fixed, current design forces "block halts the turn" — meaning a validation failure becomes a session-killer. `continueOnBlock` lets validators emit corrective signal without breaking the flow.
- **Check**: `grep -rn 'continueOnBlock' ~/.claude/settings.json 2>/dev/null`
- **Action**: For any PostToolUse hook that does best-effort validation (lint, format, schema-check), add `"continueOnBlock": true` so a failure becomes a heads-up not a halt. Pairs with the `fix-posttooluse-validate` openspec.
- **Introduced**: `v2.1.139`, 2026-05

### hook-terminal-sequence-v2.1.141

- **What**: Hook JSON output now supports `terminalSequence` field — emit desktop notifications, window titles, and bell sequences without holding a controlling terminal.
- **Why**: Today's nx-send TTS pipeline routes through nexus-agent → external speaker. `terminalSequence` is a complementary native channel: terminal bell + title-change for events that don't need TTS but do need attention (e.g. "permission prompt waiting").
- **Check**: `grep -rn 'terminalSequence' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/ ~/.claude/settings.json 2>/dev/null`
- **Action**: Identify hook events worth surfacing in the terminal natively (e.g. Stop hook on long-running session, Notification event for permission requests). Emit a bell or title change via `terminalSequence`.
- **Introduced**: `v2.1.141`, 2026-05

## mcp

### mcp-stdio-claude-project-dir-v2.1.139

- **What**: MCP stdio servers now receive `CLAUDE_PROJECT_DIR` in their environment (matching what hooks already get). Plugin `.mcp.json` configs can reference `${CLAUDE_PROJECT_DIR}` in commands and args.
- **Why**: Project-scoped MCP servers previously had no way to resolve "the current project root" without ambient state. Plugin authors can now ship MCP commands that adapt to the invoking project automatically.
- **Check**: `grep -rn '\${CLAUDE_PROJECT_DIR}\|CLAUDE_PROJECT_DIR' ~/.claude/.mcp.json ~/dev/*/.mcp.json 2>/dev/null`
- **Action**: For project-scoped MCP servers (nova-memory in `nv/`), audit `.mcp.json` for hard-coded paths that could be `${CLAUDE_PROJECT_DIR}`-relative. Improves portability when symlinks shift.
- **Introduced**: `v2.1.139`, 2026-05

## otel

### agent-id-headers-otel-v2.1.139

- **What**: API requests from subagents now carry `x-claude-code-agent-id` and `x-claude-code-parent-agent-id` headers. `claude_code.llm_request` OTEL spans include matching `agent_id` and `parent_agent_id` attributes.
- **Why**: nexus-agent currently attributes cost by session, not by sub-agent. With agent-level tagging, Leo can answer "which agent type costs the most" and "are parallel waves attributing tokens correctly". Direct cost-attribution win.
- **Check**: `grep -rn 'agent_id\|x-claude-code-agent-id' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null`
- **Action**: Update nexus-agent OTEL ingester to index `agent_id` / `parent_agent_id`. Update `scripts/bin/cost-by-command` to optionally break out by `agent_id`. Audit any cost dashboards in nv/ to add agent-level facets.
- **Introduced**: `v2.1.139`, 2026-05

## settings

### automode-hard-deny-v2.1.136

- **What**: `settings.autoMode.hard_deny` lets you define classifier rules that **block unconditionally** regardless of user intent or allow exceptions — a hard safety net layered on top of `autoMode.soft_deny`.
- **Why**: Soft-deny rules can be overridden by user intent (the classifier weighs context). Hard-deny is absolute — useful for catastrophic-blast-radius commands (force-push to main, `rm -rf /`, dropping prod DBs). With `defaultMode: auto` in cc settings.json, this is a real gap.
- **Check**: `jq -e '.autoMode.hard_deny' ~/.claude/settings.json`
- **Action**: Add `"hard_deny": ["$defaults", "<custom-rule>"]` under `autoMode` in `~/.claude/settings.json` (uses the v2.1.118 `$defaults` extension token). Candidates: `git push --force origin main`, `bd dolt push --force`, any `rm -rf /` variant.
- **Introduced**: `v2.1.136`, 2026-05

### worktree-baseref-fresh-v2.1.133

- **What**: `worktree.baseRef` setting (`"fresh"` | `"head"`) controls whether `--worktree`, `EnterWorktree`, and agent-isolation worktrees branch from `origin/<default>` or local `HEAD`. **Default reverted to `fresh`** in v2.1.133 (was `head` in 2.1.128–132).
- **Why**: `fresh` means new worktrees DO NOT inherit your unpushed commits — surprising if you stage work on main before spawning isolated agents. `head` keeps local commits but can cause divergence noise. Leo's `/apply:all` orchestration spawns agent worktrees frequently; the default change matters.
- **Check**: `jq -e '.worktree.baseRef' ~/.claude/settings.json`
- **Action**: Explicitly set `"worktree": { "baseRef": "head" }` if you want unpushed commits carried into new worktrees (matches the 2.1.128 behavior), or `"fresh"` to be explicit about the current default. Either way, **make the choice explicit** so version drift doesn't silently change orchestration semantics.
- **Introduced**: `v2.1.133`, 2026-05

### disable-alternate-screen-v2.1.132

- **What**: `CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1` env var opts out of the fullscreen alternate-screen renderer; conversation stays in the terminal's native scrollback.
- **Why**: Quality-of-life for users who rely on terminal scrollback search (`grep` across past output), tmux pane copy-mode, or terminal split views. Not relevant if `/tui` fullscreen mode is preferred.
- **Check**: `jq -e '.env.CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN' ~/.claude/settings.json`
- **Action**: If Leo prefers persistent scrollback over fullscreen rendering, add `"env": { "CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN": "1" }` to settings.json. Otherwise skip — fullscreen has its own benefits (mouse, lower memory).
- **Introduced**: `v2.1.132`, 2026-05

## hooks

### hooks-effort-level-input-v2.1.133

- **What**: `PreToolUse`/`PostToolUse`/`SessionStart` hook JSON inputs now include `effort.level` field, and hook commands can read `$CLAUDE_EFFORT` env var. Bash tool subprocesses also inherit `$CLAUDE_EFFORT`.
- **Why**: Lets hooks branch on effort tier — e.g. skip heavy telemetry at `low` effort, enable additional gates at `max`. Pairs with the v2.1.120 skill-content `${CLAUDE_EFFORT}` interpolation to build effort-stratified workflows end-to-end.
- **Check**: `grep -rln 'CLAUDE_EFFORT\|effort.level' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/ 2>/dev/null`
- **Action**: Audit `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/telemetry.sh` to forward `effort.level` to nexus-agent. Audit `commands/` for orchestrators that benefit from effort-aware branching (`/apply:all`, `/workflow:evolve`).
- **Introduced**: `v2.1.133`, 2026-05

### bash-session-id-env-v2.1.132

- **What**: `CLAUDE_CODE_SESSION_ID` env var is set in the Bash tool subprocess environment — matches the `session_id` value passed to hooks.
- **Why**: Enables telemetry correlation between hook-emitted events and Bash-tool emitted events (e.g. nexus-agent events fired from `bd remember`, `bd close`, or script runs). Previously, you had to thread session_id through stdin or env-pass it manually.
- **Check**: `grep -rln 'CLAUDE_CODE_SESSION_ID' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null`
- **Action**: Update `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/lib/nx-send.sh` (and any wrapper that emits nexus events) to read `$CLAUDE_CODE_SESSION_ID` and attach as `session_id` on the outbound event payload. Verify `nexus_notify` already picks it up.
- **Introduced**: `v2.1.132`, 2026-05

## skills

### skill-overrides-setting-v2.1.129

- **What**: `skillOverrides` setting now works (previously silent no-op). Per-skill override modes: `"off"` (hide from model AND `/`-menu), `"user-invocable-only"` (hide from model auto-discovery, keep in `/`-menu), `"name-only"` (collapse description to name only — saves description tokens).
- **Why**: Leo has 200+ installed skills (`~/.agents/skills/` × `~/.claude/skills/`). Every skill description ships in the model's skill catalog, costing tokens on every turn. Skills used only via explicit `/skill-name` slash command don't need their description visible to the model — `"user-invocable-only"` or `"name-only"` is the right tier.
- **Check**: `jq -e '.skillOverrides' ~/.claude/settings.json`
- **Action**: Audit `/skills` menu for skills that are user-typed-only (never proactively suggested). Add `"skillOverrides": { "skill-name": "user-invocable-only" }` for those. High-value targets: heavy single-use skills (`/skill-judge`, `/skill-creator`, `awesome-design-md`, brand-specific design skills).
- **Introduced**: `v2.1.129`, 2026-05

## commands

### plugin-url-flag-v2.1.129

- **What**: `--plugin-url <url>` flag fetches a plugin `.zip` archive from a URL for the current session.
- **Why**: Ad-hoc plugin loading without committing to install — useful for testing forks or experimental plugins.
- **Check**: (no automated check — informational signal; no setting to flip)
- **Action**: Manual — remember the flag exists for one-off plugin testing. Skip if Leo doesn't experiment with third-party plugins.
- **Introduced**: `v2.1.129`, 2026-05

## mcp

### alwaysload-mcp-server-v2.1.121

- **What**: `alwaysLoad: true` on an MCP server config causes all its tools to skip tool-search deferral — always available without a ToolSearch round-trip.
- **Why**: Eliminates ToolSearch latency for high-traffic MCP servers. GitHub MCP tools (used every session in cc) currently require a ToolSearch fetch each time they're invoked.
- **Check**: `jq -e '.. | objects | select(.alwaysLoad == true)' ~/.claude/settings.json`
- **Action**: Add `"alwaysLoad": true` to the github MCP server entry in `~/.claude/settings.json`. Consider any other server invoked on every session (e.g. sequential-thinking).
- **Introduced**: `v2.1.121`, 2026-04-26

## hooks

### mcp-tool-hook-handler-v2.1.118

- **What**: New hook handler `type: "mcp_tool"` lets a hook directly invoke an MCP tool without a shell shim or HTTP callback.
- **Why**: Replaces the brittle "hook → bash script → MCP CLI" chain with a typed direct call. Faster, fewer moving parts, no JSON shell-quoting bugs.
- **Check**: `grep -q '"type": "mcp_tool"' ~/.claude/settings.json`
- **Action**: For any hook currently shelling to MCP servers via `command:` + `nx-send.sh`, evaluate replacing with `type: "mcp_tool"` for direct invocation. Most natural fit: telemetry hooks that push to nova-* MCP servers.
- **Introduced**: `v2.1.118`, 2026-04-22

### posttooluse-replace-all-tools-v2.1.121

- **What**: PostToolUse hooks can now replace tool output for **all** built-in tools via `hookSpecificOutput.updatedToolOutput` — previously only worked for MCP tool results.
- **Why**: Enables pre-model output transformation: sanitize, annotate, compress, or gate any tool result before it enters the model context. Previously required an MCP wrapper to intercept built-in tool output.
- **Check**: `grep -rn 'updatedToolOutput' ~/.claude/settings.json ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/ 2>/dev/null`
- **Action**: Evaluate whether any existing hooks (e.g. telemetry.sh) benefit from modifying tool output. Immediate candidate: strip noise from Read tool output (file-modified reminders) before model sees it.
- **Introduced**: `v2.1.121`, 2026-04-26

### duration-ms-on-posttooluse-v2.1.119

- **What**: `PostToolUse` and `PostToolUseFailure` hook inputs now include `duration_ms` field with tool execution time (excluding permission prompts and PreToolUse hooks).
- **Why**: Enables tool-latency telemetry without timestamp arithmetic in the hook script. Direct number from CC's instrumentation.
- **Check**: `grep -q 'duration_ms' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/telemetry.sh`
- **Action**: In `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/hooks/telemetry.sh`, read `duration_ms` from `$CLAUDE_HOOK_INPUT` and forward to nexus-agent for the existing telemetry stream.
- **Introduced**: `v2.1.119`, 2026-04-25

## skills

### skill-effort-var-v2.1.120

- **What**: Skill content can now reference `${CLAUDE_EFFORT}` to get the current effort level (`low`, `medium`, `high`, `xhigh`, `max`).
- **Why**: Lets skills branch their guidance based on effort — e.g. recommend a lightweight shortcut at `low` effort, full due-diligence at `high+`. Makes skill instruction density adaptive.
- **Check**: `grep -rln 'CLAUDE_EFFORT' ~/.claude/skills/ 2>/dev/null`
- **Action**: Audit high-token skills (workflow:evolve, review, brainstorming) to see if guidance can be stratified by effort. Low-effort sessions skip exhaustive checks; max-effort unlocks full analysis depth.
- **Introduced**: `v2.1.120`, 2026-04-25

## settings

### sandbox-network-denieddomains-v2.1.113

- **What**: `sandbox.network.deniedDomains` setting blocks specific domains even when a broader `allowedDomains` wildcard would otherwise permit them.
- **Why**: Defense-in-depth against accidental data exfiltration. Lets you keep `*` allowed for general dev work while blocking known-sensitive endpoints (telemetry leaks, internal APIs).
- **Check**: `jq -e '.sandbox.network.deniedDomains' ~/.claude/settings.json`
- **Action**: Add a `sandbox.network.deniedDomains` array to settings.json with sensitive endpoints (e.g. internal APIs, paid telemetry). Pair with existing `permissions.deny` for layered security.
- **Introduced**: `v2.1.113`, 2026-04

### automode-defaults-extension-v2.1.118

- **What**: `autoMode.allow`, `autoMode.soft_deny`, and `autoMode.environment` accept a `"$defaults"` token to extend the built-in lists instead of replacing them entirely.
- **Why**: Previously, customizing one auto-mode rule meant re-implementing the whole built-in safety list. `$defaults` lets you add custom rules on top of Anthropic's curated set.
- **Check**: `jq -e '.autoMode' ~/.claude/settings.json` — only relevant if `defaultMode: auto` is set (it is, line 381 of cc settings).
- **Action**: If using auto mode, define `autoMode.allow` as `["$defaults", "<custom-rule>"]` to extend rather than replace.
- **Introduced**: `v2.1.118`, 2026-04-22

### disable-updates-env-v2.1.118

- **What**: `DISABLE_UPDATES` environment variable blocks all update paths including manual `claude update` — stricter than `DISABLE_AUTOUPDATER`.
- **Why**: For shared machines, CI runners, or pinned-version workflows where ANY update (auto or manual) should be impossible.
- **Check**: `jq -e '.env.DISABLE_UPDATES' ~/.claude/settings.json`
- **Action**: For Leo's setup (multiple machines, Dolt-synced state), evaluate whether pinning is desired. Likely `defer` — Leo updates intentionally.
- **Introduced**: `v2.1.118`, 2026-04-22

## agents

### agent-frontmatter-mcpservers-v2.1.117

- **What**: Agent frontmatter now accepts `mcpServers:` block, loaded for main-thread agent sessions invoked via `--agent`.
- **Why**: Previously, agents inherited the orchestrator's MCP config. Now an agent can declare its own MCP server requirements explicitly — and `--agent` headless runs honor them.
- **Check**: `grep -rln 'mcpServers:' ~/.claude/agents/ 2>/dev/null`
- **Action**: For agents with specialized MCP needs (e.g., `cc-feature-analyst` reading nova-memory MCP for prior research), declare `mcpServers:` in frontmatter so headless runs work.
- **Introduced**: `v2.1.117`, 2026-04

### forked-subagents-env-v2.1.117

- **What**: `CLAUDE_CODE_FORK_SUBAGENT=1` enables forked subagents on external builds.
- **Why**: Forked subagents share the parent's tool state but run in isolated contexts — useful for parallel exploratory work without polluting the main session.
- **Check**: `jq -e '.env.CLAUDE_CODE_FORK_SUBAGENT' ~/.claude/settings.json`
- **Action**: Evaluate for cc workflow. Most cc agent fan-outs (e.g. audit:waves) currently spawn fresh subagents — forking might reduce startup overhead but also share state in unexpected ways.
- **Introduced**: `v2.1.117`, 2026-04

## commands

### print-honors-agent-frontmatter-v2.1.119

- **What**: `--print` (`-p`) headless mode now honors agent frontmatter `tools:` and `disallowedTools:` fields, matching interactive-mode behavior.
- **Why**: Closes a long-standing gap where `claude -p --agent <name>` could call tools the agent definition forbade. Makes scripted runs reliably reproduce interactive constraints.
- **Check**: `grep -rln 'disallowedTools:' ~/.claude/agents/ 2>/dev/null` (presence of either field is enough — the new behavior is automatic when set)
- **Action**: Audit agents that are invoked via `-p` (any in scripts/bin/* that shell out to `claude -p --agent ...`). Confirm their `tools:`/`disallowedTools:` lists are accurate, since they're now enforced.
- **Introduced**: `v2.1.119`, 2026-04-25

## otel

### otel-skill-activated-v2.1.126

- **What**: `claude_code.skill_activated` OTel event fires whenever a skill is invoked, carrying `invocation_trigger`: `"user-slash"`, `"claude-proactive"`, or `"nested-skill"`.
- **Why**: Enables tracking which skills are most used, which are proactively suggested vs user-typed, and which are nested inside other skills. Direct input to skill portfolio prioritization.
- **Check**: `grep -rn 'skill_activated' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null`
- **Action**: If nexus-agent's OTel pipeline ingests `claude_code.*` log events, `skill_activated` arrives automatically — no config change needed. Verify the nexus event filter passes it through; confirm `invocation_trigger` is indexed.
- **Introduced**: `v2.1.126`, 2026-05-01

## Known noise

Patterns that produce false-positive recommendations or context injections. **Not actionable** —
documented so analysts can recognize and skip rather than recommend fixes for upstream-controlled
plugins.

### vercel-plugin-path-substring-matcher — RESOLVED, plugin removed 2026-07-21

Was: PreToolUse hook fired on any tool input where the file path or argument contained substrings
like `workflow`, `shadcn`, `react`, `next`, `prisma` (e.g. reading `commands/workflow/evolve.md`
triggered Vercel Workflow DevKit guidance injection). Originally logged here as "not actionable"
because the plugin was upstream-controlled and disabling would drop legitimate Vercel guidance.
Superseded 2026-07-21: `vercel-plugin@vercel-vercel-plugin` was removed entirely (confirmed absent
from `installed_plugins.json`/`mcp.json`, bead `cc-f6l66` closed) — Vercel interaction now goes
through the raw `vercel` CLI instead of the plugin's MCP tools. See
`docs/vercel-plugin-hook-config.md` for the full decision record. No live analyst action needed;
kept as a tombstone since the pattern (path-substring context-blind matching) recurs with other
plugins and is still worth recognizing on sight.

## commands (v2.1.144–149 delta)

### usage-category-breakdown-v2.1.149

- **What**: `/usage` now shows a per-category cost breakdown: skills, subagents, plugins, and per-MCP-server. Previously showed only aggregate totals.
- **Why**: Surfaces which category drives token spend so operators can tune the right lever. With CONCURRENCY=20 + AGENT_TEAMS + 12 MCP servers, the ratio of subagent vs MCP vs skills cost is non-obvious.
- **Check**: `claude /usage` — observe whether per-category rows are present (no config change needed)
- **Action**: Run `/usage` after a heavy `/apply:all` wave. Cross-check skills category vs MCP category. Correlate with nexus-agent OTel spans for per-workflow cost baselines.
- **Introduced**: `v2.1.149`, 2026-05

### code-review-command-rename-v2.1.147

- **What**: `/simplify` renamed to `/code-review`. Now reports correctness bugs at configurable effort levels with optional `--comment` flag for GitHub PR inline comments.
- **Why**: Three stale `/simplify` references existed in `commands/review.md` and `skills/cc-tooling/SKILL.md`. Updated in this run.
- **Check**: `claude /code-review --help 2>&1 | head -5`
- **Action**: References updated. Note: `--comment` flag posts findings as GitHub PR inline comments — complements `/review` skill (pre-merge local) and `/ultrareview` (cloud multi-agent).
- **Introduced**: `v2.1.147`, 2026-05

## agents (v2.1.144–147 delta)

### bg-session-pinned-persistence-v2.1.147

- **What**: Pinned background sessions persist when idle and restart in-place during CC auto-updates. `/resume` now supports sessions started via `claude --bg` or agent view.
- **Why**: The primary cause of mid-wave interruptions on multi-hour `/apply:all` runs was CC auto-update killing the orchestrator. Pinning + in-place restart eliminates this.
- **Check**: `claude agents` — verify Ctrl+T pin action is available on a bg session row
- **Action**: After launching `/apply:all` orchestrator via `claude --bg`, press Ctrl+T to pin it. Apply the same habit to `ralph-loop` polling sessions. Added guidance to `commands/apply/all.md`.
- **Introduced**: `v2.1.147`, 2026-05

## settings (v2.1.147 delta)

### subagent-model-teammate-v2.1.147

- **What**: `CLAUDE_CODE_SUBAGENT_MODEL` env var now applies to teammate processes (spawned under `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`), not just standard subagents.
- **Why**: With AGENT_TEAMS=1 + CONCURRENCY=20 active and `CLAUDE_CODE_SUBAGENT_MODEL` unset, all teammate processes run on the default model at full cost. This is the lever for wave-cost reduction.
- **Check**: `jq -e '.env.CLAUDE_CODE_SUBAGENT_MODEL' ~/.claude/settings.json 2>/dev/null`
- **Action**: Add `"CLAUDE_CODE_SUBAGENT_MODEL": "claude-sonnet-4-5"` to the `env` block in `~/.claude/settings.json`. Sonnet is the safe floor (protects analyst/engineer quality); Haiku is 15x cheaper but risks weaker output on code-generation agents.
- **Introduced**: `v2.1.147`, 2026-05

## worktree (v2.1.142–143 delta)

### bg-isolation-none-v2.1.143

- **What**: New `worktree.bgIsolation: "none"` setting lets background sessions edit the working copy directly without `EnterWorktree`, for repos where worktrees are impractical.
- **Why**: We JUST shipped per-session worktree isolation for `/apply` and `/apply:all` (2026-05-16, `apply-concurrent-session-isolation` spec). This native setting is the opposite trade-off — disable worktrees entirely for background sessions. Need to evaluate: does it overlap with our impl? Should our `wt_create` skip when this is set? Is there a hybrid?
- **Check**: `jq -e '.worktree.bgIsolation' ~/.claude/settings.json 2>/dev/null`
- **Action**: Audit relationship with our worktree-helpers.sh. Decide whether to honor this setting in `wt_create` (skip + warn when `bgIsolation: "none"`) or document the divergence.
- **Introduced**: `v2.1.143`, 2026-05

### worktree-rm-rf-safety-v2.1.143

- **What**: Worktree cleanup no longer falls back to `rm -rf` when `git worktree remove` fails — preserves gitignored or in-progress files.
- **Why**: Our `wt_destroy` (just shipped) uses `git worktree remove` + `rm -rf` fallback for the directory. CC's native cleanup just got safer; our impl may now be MORE aggressive than CC's own. Worth aligning.
- **Check**: `grep -nE 'rm -rf|git worktree (remove|prune)' ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/lib/worktree-helpers.sh`
- **Action**: Inspect `wt_destroy` — remove unconditional `rm -rf` fallback OR add the same "preserve gitignored" guard.
- **Introduced**: `v2.1.143`, 2026-05

## hooks (v2.1.143 delta)

### stop-hook-block-cap-v2.1.143

- **What**: Stop hooks that block repeatedly now end the turn with a warning after 8 consecutive blocks (override via `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`).
- **Why**: We have multiple Stop hooks (telemetry, session-closer, dolt push). If any of them cascade-blocks on a transient error, this 8-block cap is the safety net. May be worth raising for legitimate long-running session closure flows, or lowering for quick-iteration.
- **Check**: `env | grep CLAUDE_CODE_STOP_HOOK_BLOCK_CAP; grep -rn 'Stop' ~/.claude/settings.json | head`
- **Action**: Audit Stop hook return codes; set `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP` in `~/.zshrc` or `settings.json` env if default 8 is wrong for our session-close flow.
- **Introduced**: `v2.1.143`, 2026-05

## agents (v2.1.142 delta)

### agents-dispatch-flags-v2.1.142

- **What**: `claude agents` now accepts `--add-dir`, `--settings`, `--mcp-config`, `--plugin-dir`, `--permission-mode`, `--model`, `--effort`, `--dangerously-skip-permissions` to configure dispatched background sessions.
- **Why**: Our `/apply:all` orchestration spawns engineer agents via the `Agent` tool, NOT `claude agents` dispatch. But for cases where we run `/bg` or fork sessions (e.g., long-running ralph loops), the flag-set is now richer. Worth knowing.
- **Check**: `grep -rn 'claude agents' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null | head`
- **Action**: Document the flag set in `cc-reference` skill if any cc command/skill ever invokes `claude agents`. Otherwise: skip (we don't use it).
- **Introduced**: `v2.1.142`, 2026-05

## skills (v2.1.142 delta)

### plugin-root-skill-md-v2.1.142

- **What**: Plugins with a root-level `SKILL.md` and no `skills/` subdirectory are now surfaced as a skill.
- **Why**: Simplifies plugin authoring for single-skill plugins — no need for the `skills/<name>/SKILL.md` ceremony. Plugins we depend on (figma, vercel, beads) may eventually flatten.
- **Check**: `find ~/.claude/plugins -name SKILL.md -maxdepth 3 2>/dev/null | head -20`
- **Action**: Inform-only. Watch for plugin updates that flatten structure; ensure cc consumption code (`scripts/bin/cc-inventory`) handles both shapes.
- **Introduced**: `v2.1.142`, 2026-05
