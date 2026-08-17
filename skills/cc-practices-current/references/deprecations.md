# CC Deprecations

_Last refreshed: 2026-07-18_

## Active deprecations

- **Task tool `mode` parameter** → **ignored; subagents inherit the parent session's permission mode by default** (deprecated in `v2.1.212`, 2026-07)
  - **Migration**: none needed unless a caller explicitly passed `mode` expecting it to override the spawned subagent's permission mode — that override no longer applies.
  - **Detection**: `grep -rn '"mode"' ~/.claude/commands/ ~/.claude/skills/ ~/.claude/agents/ 2>/dev/null` then manually confirm any hit is an actual Task/Agent tool call param, not an unrelated JSON schema field (cc's own hits were all unrelated schemas — audit findings `mode: static/live`, wave-state `mode: consolidated`, fallow config `mode: mild` — verified 2026-07-18, zero real Task-tool `mode` usage in cc).
  - **Removed in**: not yet — parameter accepted but ignored.

- **`Write(path)` / `NotebookEdit(path)` / `Glob(path)` permission rules** → **`Edit(path)` / `Read(path)`** (startup warning added in `v2.1.210`, 2026-07)
  - **Migration**: rewrite permission rules using the `Edit(...)` prefix for write-class tools and `Read(...)` for read-class tools.
  - **Detection**: `grep -nE '"(Write|NotebookEdit|Glob)\(' ~/.claude/settings.json ~/.claude/settings.local.json 2>/dev/null`
  - **Removed in**: not yet — warning only.

- **`autoMode` in repo `.claude/settings.local.json`** → **`~/.claude/settings.json`** (ignored from repo-resident settings since `v2.1.207`, 2026-07)
  - **Migration**: move any `autoMode` key from a repo's `.claude/settings.local.json` to user settings.
  - **Detection**: `grep -l '"autoMode"' ~/dev/*/.claude/settings.local.json 2>/dev/null`
  - **Removed in**: v2.1.207 — repo-resident value is a silent no-op.

- **`pluginConfigs` in project `.claude/settings.json`** → **user / `--settings` / managed settings** (ignored since `v2.1.207`, 2026-07)
  - **Migration**: move plugin option values to user-level settings.
  - **Detection**: `grep -l '"pluginConfigs"' ~/dev/*/.claude/settings.json 2>/dev/null`
  - **Removed in**: v2.1.207 — project-level value is a silent no-op.

- **`${user_config.*}` in shell-form plugin hook/monitor/headersHelper commands** → **exec form (`args` array) or `$CLAUDE_PLUGIN_OPTION_<KEY>`** (rejected since `v2.1.207`, 2026-07, shell-injection fix)
  - **Migration**: hooks use exec form or the env var; monitors/headersHelper read the value inside the script.
  - **Detection**: `grep -rn 'user_config\.' ~/.claude/plugins/ 2>/dev/null`
  - **Removed in**: v2.1.207 — shell-form interpolation hard-rejected.

- **`/review <pr>` as multi-agent review** → **`/code-review <level> <pr#>`** (reverted in `v2.1.202`, 2026-07)
  - **Migration**: `/review <pr>` is a fast single-pass again; use `/code-review <level> <pr#>` for the multi-agent effort-tiered review.
  - **Detection**: `grep -rn '/review ' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null` — audit docs describing `/review` as multi-agent.
  - **Removed in**: n/a — semantics reverted, both commands live.

- **User MCP servers named "Claude Browser" or "Claude Preview"** → **rename** (reserved in `v2.1.205`, 2026-07)
  - **Migration**: rename any user-configured MCP server using either name.
  - **Detection**: `grep -nE '"Claude (Browser|Preview)"' ~/.claude.json ~/.claude/mcp.json 2>/dev/null`
  - **Removed in**: v2.1.205 — registration under these names refused.

- **Dynamic-workflow trigger keyword `workflow`** → **`ultracode`** (renamed in `v2.1.160`, 2026-06; refined in `v2.1.178`)
  - **Migration**: the literal word "workflow" no longer auto-triggers a dynamic workflow; the trigger keyword is now `ultracode` (violet shimmer), and it fires only on explicit phrases ("run a workflow", "workflow:") as of v2.1.178. The `workflowKeywordTriggerEnabled` setting (v2.1.158) predates this rename — re-verify whether it still controls the keyword and whether cc still needs it set.
  - **Detection**: `grep -n 'workflowKeywordTriggerEnabled' ~/.claude/settings.json` (cc: PRESENT = `false` at line 423, verified 2026-06-20 — NOT drift; an earlier `jq '. // "ABSENT"'` check false-flagged it because `false // "ABSENT"` returns the fallback string). Installed CC is 2.1.177, so the v2.1.178 phrase-only refinement is not yet live.
  - **Removed in**: n/a — keyword renamed, not removed. Open question: keep `false` (harmless — dynamic workflows still launch explicitly) vs flip `true` now the original "/workflow:* command-name" motivation is obsolete. NOTE: the `ultracode` trigger keyword could collide with `/effort ultracode`.

- **`CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE`** → switch with `/model claude-opus-4-6[1m]` then `/fast on` (deprecated `v2.1.154`; **hard-removed `v2.1.160`**, 2026-06)
  - **Migration**: remove the env var; use the `/model` + `/fast on` flow for fast mode on Opus 4.6.
  - **Detection**: `grep -rn 'CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE' ~/.claude/settings.json ~/.claude/.env* ~/.zshrc 2>/dev/null`
  - **Removed in**: v2.1.160 — confirmed no-op (changelog: "Removed `CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE`; the environment variable is now a no-op").

- **`/simplify` as a bug-hunting review** → **`/code-review --fix`** (behavior change in `v2.1.154`, 2026-05)
  - **Migration**: `/simplify` is now cleanup-only (reuse/simplification/efficiency/altitude). For correctness bug-hunting with fixes, use `/code-review --fix`.
  - **Detection**: `grep -rn '/simplify' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null`
  - **Removed in**: not a removal — semantics narrowed; audit any doc that describes `/simplify` as bug-finding.

- **keybinding `modelPicker:setAsDefault`** → **`modelPicker:thisSessionOnly`** (renamed in `v2.1.153`, 2026-05)
  - **Migration**: rename the action in `~/.claude/keybindings.json` (the `d` action became `s`).
  - **Detection**: `grep -n 'modelPicker:setAsDefault' ~/.claude/keybindings.json 2>/dev/null`
  - **Removed in**: the old binding no longer functions.

- **`/cost` and `/stats`** → **`/usage`** (deprecated in `v2.1.118`, 2026-04-22)
  - **Migration**: type `/usage` instead. Both old commands still work as typing shortcuts.
  - **Detection**: `grep -rn '/cost\|/stats' ~/.claude/commands/ ~/.claude/skills/ 2>/dev/null`
  - **Removed in**: not yet (kept as aliases)

- **`ENABLE_PROMPT_CACHING_1H_BEDROCK`** → **`ENABLE_PROMPT_CACHING_1H`** (deprecated in `v2.1.108`)
  - **Migration**: rename env var; new name applies to API key, Bedrock, Vertex, and Foundry uniformly.
  - **Detection**: `grep -n 'ENABLE_PROMPT_CACHING_1H_BEDROCK' ~/.claude/settings.json ~/.claude/.env* 2>/dev/null`
  - **Removed in**: not yet (still honored)

- **`/output-style`** → **`/config`** (deprecated earlier, March 2026)
  - **Migration**: configure output style via `/config`.
  - **Detection**: `grep -rn '/output-style' ~/.claude/ 2>/dev/null`
  - **Removed in**: not yet

## Removed

- **`TeamCreate` / `TeamDelete` tools** (removed in `v2.1.178`, 2026-06) — with `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`, every session has one implicit team; spawn teammates via the Agent tool's `name` parameter. The `team_name` Agent-tool param is accepted but ignored. **Detection**: `grep -rln 'TeamCreate\|TeamDelete\|team_name' ~/.claude/commands/ ~/.claude/skills/ ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/ 2>/dev/null` — cc surface verified clean 2026-06-20 (zero references), no cleanup needed.
- **`CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE`** (removed in `v2.1.160`, 2026-06) — see Active deprecations above; now a no-op.

_No other hard removals observed in v2.1.105 → v2.1.183._

## Soft deprecations / "no longer recommended"

- **Edge Functions for new code** — Vercel guidance (Fluid Compute is now default; not a CC deprecation but commonly cited)
- **Bundled JavaScript distribution** (replaced by native binaries in `v2.1.113`) — npm fallback still works on Windows
