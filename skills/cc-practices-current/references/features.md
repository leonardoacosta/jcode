# CC Features — Current

_Last refreshed: 2026-07-21 — sources: docs sha256:4f0de0cf8e, gh v2.1.217, npm 2.1.217 (stable 2.1.206)_

## Added since v2.1.214 (current delta)

### v2.1.217
- **Cap on concurrently-running subagents** (default 20, override `CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS`) — so one message can't fan out unbounded background agents. Distinct from the existing per-session 200-spawn cap (`CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION`, v2.1.212) — this one throttles concurrency, not lifetime count.
- **Changed subagents to no longer spawn nested subagents by default** — override with `CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH` to allow deeper nesting.
- **Fixed `--max-budget-usd` not stopping background subagents** — once the cap is reached, new spawns are now denied and running background agents are halted (was previously a silent no-op for background agents specifically).
- Added emoji shortcode autocomplete in the prompt input (`:heart:` → ❤️) — disable via the `emojiCompletionEnabled` setting
- Added warnings when transcript writes are failing (e.g. disk full) or session saving is off due to an inherited env var, instead of losing transcripts silently
- Fixed a memory leak where truncated MCP tool outputs kept the full untruncated result in memory for the rest of the session
- Fixed background session isolation not canonicalizing symlinked working directories, which could let sessions escape their workspace folder
- Fixed managed `OTEL_EXPORTER_OTLP_ENDPOINT` not governing all signals — lower-scope signal-specific overrides no longer redirect telemetry away from the managed endpoint
- Fixed auto-compact never triggering for Claude Opus 4.8 on Bedrock and `/compact` failing once over the limit
- Changed the login-expiry warning to appear 3 days before expiry instead of 5

### v2.1.216
- **Added `sandbox.filesystem.disabled` setting** — skip filesystem isolation while keeping network egress control
- Fixed a slowdown in long sessions where message normalization cost grew quadratically with turn count, causing multi-second stalls and slow resumes
- Fixed worktree-isolated subagents redirecting git into the shared checkout via `git -C`, `--git-dir`, or `GIT_DIR`/`GIT_WORK_TREE` — closes a worktree-isolation escape
- Fixed `AskUserQuestion` telling Claude to continue even when the answer asked it to wait or explain first — free-text answers now get neutral wording
- Fixed resumed background agent sessions reverting to the default agent: the agent's prompt and tool restrictions are now restored
- Fixed workflow saves and scheduled-task writes following a symlink at `.claude`, which could redirect writes outside the project
- Fixed telemetry misreporting permission denials: failed permission-prompt requests no longer count as user rejections; user interrupts now report as user aborts instead of rejections
- Fixed skills and commands changed during a session not appearing in the slash menu until restart
- `/rewind` no longer restores or deletes files through symlinks or hard links at tracked paths, and reports how many paths it skipped
- Updated the bundled dataviz skill: reordered the default chart palette and fixed guidance that suggested direct labels for four-series charts

### v2.1.215
- **Claude no longer runs the `/verify` and `/code-review` skills on its own** — invoke them explicitly with `/verify` or `/code-review` when wanted

## Added v2.1.212–214
- **Added the `EndConversation` tool** — Claude can end sessions with sustained user abuse or jailbreak attempts (mirrors claude.ai since 2025)
- **Added a periodic progress heartbeat** for long-running tool calls that previously went silent
- **Added an ISO `modified` timestamp to memory file frontmatter**
- **Added `message.uuid`, `client_request_id`, `tool_source` OTel log attributes** for message-level correlation and tool provenance
- **`CLAUDE_CODE_OTEL_CONTENT_MAX_LENGTH`** — configure the 60 KB OTel content-attribute truncation limit
- **Added reasoning effort to the `subagentStatusLine` payload** — custom agent rows can render model + effort
- **docker commands with daemon-redirect flags** (`--url`/`--connection`/`--identity`, Podman remote mode) now require a permission prompt
- **Fixed single-segment `dir/**` allow rules** (e.g. `Edit(src/**)`) auto-approving nested `dir/` anywhere in the tree instead of only `<cwd>/dir` -- security-relevant permission bug
- **Changed single-segment `dir/**` hook `if:` conditions** to match only `<cwd>/dir`; write `**/dir/**` for any-depth (permission `deny`/`ask` rules keep any-depth match)
- **Changed SessionStart hooks to report source `"fork"`** for a forked session (was `"resume"`) -- distinct from the v2.1.212 `/fork` background-copy feature
- Fixed Bash permission checks: commands over 10,000 chars now always prompt (previously auto-ran); zsh `[[ ]]` variable subscripts/modifiers now prompt instead of being treated as inert text; unsafe `help`/`man` command forms no longer auto-approved
- Fixed Bash tool killing the CLI's own session when a `pkill -f` pattern accidentally matched the CLI's own process (Linux)
- Fixed hooks with exit code 2 not blocking as documented when the hook's stdout JSON fails schema validation
- Fixed memory frontmatter values being silently truncated at an inline `#`
- Fixed a displaced background daemon deleting its successor's control socket on shutdown; parked/idle background sessions no longer keep the daemon+worker alive indefinitely; completed background sessions can now be removed via `claude rm`/agent view even from a non-git folder
- Fixed scheduled tasks refusing their own configured prompt as untrusted input
- Fixed `/ultrareview` refusing to run in repos with no merge base -- now offers to review all tracked files
- Fixed session cost/token telemetry double-counting on streams emitting multiple cumulative `message_delta` frames

### v2.1.212
- **`/fork` copies the conversation into a new background session** (its own row in `claude agents`) while you keep working in the original; the in-session subagent previously named this is now `/subtask`
- **Per-session subagent-spawn cap** -- default 200, override `CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION`; `/clear` resets the budget
- **Session-wide WebSearch cap** -- default 200, override `CLAUDE_CODE_MAX_WEB_SEARCHES_PER_SESSION`
- **MCP tool calls running >2min auto-background** -- configure/disable via `CLAUDE_CODE_MCP_AUTO_BACKGROUND_MS`
- **`claude auto-mode reset`** restores default auto-mode config (confirmation prompt, `--yes` to skip)
- **Deprecated the Task tool's `mode` parameter** (now ignored) -- subagents inherit the parent session's permission mode by default
- Fixed plan mode auto-running file-modifying Bash commands (`touch`, `rm`) without a permission prompt or SDK `canUseTool` callback
- Fixed worktree creation following a repository-committed symlink at `.claude/worktrees`, which could create files outside the repository
- Reduced token usage in inter-agent messaging: `SendMessage` bodies no longer duplicated into replayed history/tool results

## Added v2.1.202-211

### v2.1.211
- **`--forward-subagent-text` flag + `CLAUDE_CODE_FORWARD_SUBAGENT_TEXT`** — include subagent text/thinking in stream-json output (headless observability)
- **"Always allow" permission rules save at repo root** — approvals granted in a git worktree now persist across sessions and worktrees
- **Integer env vars accept `1e6` / `64_000` spellings** (timeouts, token budgets, retry counts)
- Auto mode: a PreToolUse hook `ask` now floors the decision at a prompt (no longer overridden)
- Background agent result reporting hardened — reports still-running status instead of fabricating results
- Memory index over-limit warning measures only loaded content (excludes frontmatter/HTML comments)
- Fixed prompt-caching regression on Bedrock/Vertex/Mantle/Foundry billing trailing system block as fresh input

### v2.1.210
- **Startup warning for `Write(path)`/`NotebookEdit(path)`/`Glob(path)` permission rules** — use `Edit(path)` / `Read(path)` instead
- **`$1`/`$2` positional placeholders in skills/commands now preserved verbatim** (previously silently stripped)
- Fixed `isolation: 'worktree'` subagents mutating the main checkout; `ultracode` keyword no longer fires on non-human input
- Hook callback timeout no longer misreported as user rejection (unattended sessions kept waiting)
- Memory writes leaving MEMORY.md index over read limit now error explicitly instead of silent truncation
- Auto mode permission classifier defaults to Sonnet 5, pinned per session
- Bundled dataviz skill: perceptual OKLab color-difference validation

### v2.1.208
- **Screen reader mode** — `claude --ax-screen-reader`, `CLAUDE_AX_SCREEN_READER=1`, or `"axScreenReader": true`
- **`vimInsertModeRemaps` setting** — map insert-mode sequences like `jj` to Escape
- **`CLAUDE_CODE_PROCESS_WRAPPER`** — corporate launcher wrapper for every CC self-spawn
- Perf: permission deny/ask rule matchers compiled once + cached (fixes multi-second per-turn slowdowns with many rules)
- Perf: transcript size reduced up to 79x in edit-heavy sessions; MCP stderr / LSP doc / edit-cache memory leaks fixed
- Completed background agents stay in `/tasks` until cleanup

### v2.1.207
- **Auto mode GA on Bedrock/Vertex/Foundry** (no `CLAUDE_CODE_ENABLE_AUTO_MODE` opt-in; disable via `disableAutoMode`)
- **`autoMode` no longer read from repo `.claude/settings.local.json`** — use `~/.claude/settings.json`
- Plugin security: `${user_config.*}` rejected in shell-form commands; `pluginConfigs` no longer read from project settings
- Bedrock/Vertex/Claude-Platform-on-AWS default to Claude Opus 4.8

### v2.1.206
- **`/doctor` check proposing CLAUDE.md trims** — flags checked-in content Claude could derive from the codebase
- **`EnterWorktree` asks confirmation** before entering a worktree outside `.claude/worktrees/`
- `/commit-push-pr` auto-allows `git push` to the configured push remote
- `/cd` directory path suggestions; background agents upgrade in background post-update
- Improved `/code-review` findings quality on claude-opus-4-8 at all effort levels

### v2.1.205
- **`/doctor` is now a full setup checkup** that can diagnose and fix issues; `/checkup` alias
- Auto mode rule blocking session-transcript tampering; asks before `rm -rf` on unresolvable variables
- Background task notifications explicitly state no human input occurred (anti-fabrication)
- "Claude Browser" + "Claude Preview" MCP server names reserved (Claude Desktop pane rename)
- Agent view rows: colored state word + classifier headline; PR linking for edit/merge/comment/push

### v2.1.203–204
- Login-expiry warning before background sessions are interrupted; grey ⏸ footer badge in manual mode
- MCP `roots/list` now includes additional working directories + `roots/list_changed` notifications
- SessionStart hook events stream in headless sessions (fixes idle-reaping mid-hook)
- Subagents less likely to re-delegate their entire task to another subagent

### v2.1.202
- **"Dynamic workflow size" setting in `/config`** — advisory small/medium/large agent-count guideline
- **`workflow.run_id` + `workflow.name` OTel attributes** on workflow-spawned agent telemetry
- **`/review <pr>` back to fast single-pass**; `/code-review <level> <pr#>` is the multi-agent form
- Re-invoking an already-loaded skill no longer appends a duplicate copy to context

## Added earlier (v2.1.159–v2.1.201)

### v2.1.201
- **Sonnet 5 sessions no longer use mid-conversation system role** for harness reminders — reduces context overhead and improves prompt-cache hit rate

### v2.1.200
- **`AskUserQuestion` dialogs no longer auto-continue** — wait indefinitely by default; opt into idle timeout via `/config`
- **Permission mode "default" renamed "Manual"** — `--permission-mode manual` and `"defaultMode": "manual"` accepted alongside legacy `"default"`
- Fixed rendering flicker under tmux 3.4+ via synchronized terminal output
- Improved screen-reader output (decorative glyphs hidden, transcript symbols as short labels)
- Fixed background sessions stopping mid-turn after sleep/wake; daemon handover now uses build timestamp for recency

### v2.1.199
- **Stacked slash-skill invocations** — `/skill-a /skill-b do X` loads all leading skills (up to 5); human-typed CLI only
- `CLAUDE_CODE_RETRY_WATCHDOG` default retry count raised to 300; `CLAUDE_CODE_MAX_RETRIES` cap of 15 lifted; transient non-usage 429s auto-retry with backoff for subscribers

### v2.1.198
- **Subagents run in background by default** (GA rollout) — orchestrator keeps working while they run; notified on completion
- **`/dataviz` skill** — chart and dashboard design guidance with runnable color-palette validator
- **Background agents auto-commit, push, and open a draft PR** on finish in a worktree
- **Built-in Explore agent inherits session model** (capped opus) instead of haiku; subagents + compaction inherit extended-thinking config

### v2.1.197
- **Claude Sonnet 5** — new CC default; native 1M context; promotional $2/$10/Mtok through 2026-08-31 (then $3/$15)

### v2.1.196
- **Org default models** — admins set in org console; surfaces as "Org default" in `/model`
- **Readable default session names** generated at start for easier `claude agents` identification
- Security: `claude mcp list`/`get` no longer spawns repo-self-approved MCP servers from untrusted workspaces
- Fixed waking a bg job permanently deleting its conversation (critical bg-session reliability fix)

### v2.1.195
- **`CLAUDE_CODE_DISABLE_MOUSE_CLICKS`** — disable mouse click/drag/hover in fullscreen mode, keep wheel scroll
- **Hook matchers exact-match hyphenated identifiers** (`code-reviewer`, `mcp__brave-search`) instead of substring; use `mcp__brave-search__.*` to match all tools of a hyphenated MCP server
- External plugins enabled only by project `.claude/settings.json` now require explicit install consent on every loader path

### v2.1.193
- **`autoMode.classifyAllShell` setting** — route ALL Bash/PowerShell through the auto-mode classifier, not just arbitrary-code patterns; denial reasons now in transcript/toast/`/permissions`
- **`claude_code.assistant_response` OTEL log event** — model response text; redacted unless `OTEL_LOG_ASSISTANT_RESPONSES=1` (else follows `OTEL_LOG_USER_PROMPTS`)
- Live file-path autocomplete in bash mode (`!`); startup notice when MCP servers need auth; idle bg-shell memory-pressure reaping (`CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP=1`)

### v2.1.191
- **`/rewind` resumes from before `/clear`**; comma-separated hook matchers (`"Bash,PowerShell"`) that silently never fired are FIXED — use pipe form
- MCP capability discovery + OAuth discovery/token retry transient errors; sandbox network host-allow remembered for the session
- CPU usage during streaming reduced ~37%; stopping a bg agent from the tasks panel is now permanent

### v2.1.187
- **`sandbox.credentials` setting** — block sandboxed commands from reading credential files and secret env vars
- **`--json-schema` / Workflow `agent({schema})` fix** — model can no longer re-call `StructuredOutput` indefinitely; follow-up turns reliably return structured output
- Remote MCP tool calls abort after 5 min hang (`CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT` override); leaked agent worktree registrations auto-cleaned; org model restrictions in picker

### v2.1.186
- **`claude mcp login <name>` / `claude mcp logout <name>`** — authenticate MCP servers from the CLI without `/mcp`; `--no-browser` stdin redirect for SSH
- **`!` bash commands auto-trigger a Claude response** — set `"respondToBashCommands": false` for previous context-only behavior
- **`Agent(type)` deny + `Agent(x,y)` allowed-types now enforced for named subagent spawns**; bg subagents surface permission prompts in main session instead of auto-denying
- Skill frontmatter (`display-name`, `default-enabled`, `fallback`, `metadata.*`) accepts kebab/snake/camelCase; malformed SKILL.md YAML loads body with empty metadata
- `CLAUDE_CODE_MAX_RETRIES` caps at 15 (use `CLAUDE_CODE_RETRY_WATCHDOG` for unattended); `/review <pr>` uses `/code-review medium` engine; MEMORY.md compaction reminder near size limit; status filtering (`f`) in `/workflows` detail; Workflow `agent({schema})` aborts after 5 schema-validation failures

### v2.1.183
- **`attribution.sessionUrl` setting** — omit the claude.ai session link from commits/PRs in web/Remote Control sessions
- **Auto mode safety expanded** — built-in deny now covers `git reset --hard` / `checkout -- .` / `clean -fd` / `stash drop` (when not discarding), `git commit --amend` on non-agent commits, and `terraform`/`pulumi`/`cdk destroy`
- **`/config --help`** lists shorthand keys; `/config` Enter/Space change the setting, Esc saves+closes; model-deprecation warning now covers agent-frontmatter models

### v2.1.181
- **`/config key=value`** sets any setting from the prompt (interactive, `-p`, Remote Control)
- **`CLAUDE_CLIENT_PRESENCE_FILE`** env var — suppress mobile push notifications while you're at the machine
- Bundled Bun runtime upgraded to 1.4; foreground subagents respect the 5-level depth limit

### v2.1.178
- **`TeamCreate`/`TeamDelete` tools REMOVED** — with `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1`, every session has one implicit team; spawn teammates via the Agent tool's `name` param (`team_name` accepted but ignored)
- **`Tool(param:value)` permission syntax** — match a tool's input params with `*` wildcard, e.g. `Agent(model:opus)` blocks Opus subagents
- Nested `.claude/skills` load when working on files there (clash → `<dir>:<name>`); closest `.claude/` agent/workflow/output-style wins on collision
- Auto mode classifies subagent spawns before launch; MCP server-level specs in subagent `disallowedTools` no longer silently ignored; compaction honors `--fallback-model`
- Workflow keyword triggers only on explicit phrases ("run a workflow", "workflow:"), purple shimmer highlight

### v2.1.176
- Session titles generated in the conversation's language (`language` setting pins it)
- **`footerLinksRegexes` setting** — regex-matched link badges in the footer row
- **Hook `if` conditions for Read/Edit/Write paths now match** (`Edit(src/**)`, `Read(~/.ssh/**)`, `Read(.env)`)

### v2.1.174
- **`wheelScrollAccelerationEnabled` setting**; `/model` picker shows the family Default resolves to
- **Workflow `agent()` subagents now carry per-agent attribution headers** (agent_id)

### v2.1.172
- **Sub-agents can spawn their own sub-agents** (up to 5 levels deep)
- `model` attribute added to the `claude_code.lines_of_code.count` OTEL metric
- Workflow validation no longer rejects scripts merely mentioning `Date.now()`/`Math.random()` in comments/strings
- Search bar in `/plugin`; idle-CPU reductions (`/goal` chip no longer re-renders at 5Hz)

### v2.1.170
- **Claude Fable 5** (Mythos-class) — `claude-fable-5`, 1M context by default

### v2.1.169
- **`--safe-mode` flag / `CLAUDE_CODE_SAFE_MODE`** — start with all customizations (CLAUDE.md, plugins, skills, hooks, MCP) disabled for troubleshooting
- **`disableBundledSkills` setting / `CLAUDE_CODE_DISABLE_BUNDLED_SKILLS`** — hide bundled skills, workflows, built-in slash commands from the model
- **`/cd` command** — move a session to a new working dir without breaking the prompt cache
- Self-hosted runner `post-session` lifecycle hook; plugin `.in_use` PID locks swept once/day

### v2.1.166
- **`fallbackModel` setting** — configure up to 3 fallback models tried in order on overload/unavailability; `--fallback-model` now also applies to interactive sessions
- Glob patterns in deny-rule tool-name position (`"*"` denies all tools); cross-session `SendMessage` no longer carries user authority

### v2.1.163
- **Stop/SubagentStop hooks can return `hookSpecificOutput.additionalContext`** — feed Claude feedback and keep the turn going without a hook-error label
- **`/plugin list`** with `--enabled`/`--disabled`; `requiredMinimum`/`requiredMaximumVersion` managed settings
- stdio MCP servers receive `CLAUDE_CODE_SESSION_ID` on `--resume`; skill `\$` escape for a literal `$` before a digit

### v2.1.161
- **`OTEL_RESOURCE_ATTRIBUTES` values now labels on metric datapoints** (slice usage by team/repo)
- Fixed OTEL log events dropped when emitted before telemetry init; `claude mcp` no longer prints secrets (redacted)

### v2.1.160
- **Dynamic-workflow trigger keyword renamed `workflow` → `ultracode`** (violet highlight); the word "workflow" no longer triggers a run
- **`CLAUDE_CODE_OPUS_4_6_FAST_MODE_OVERRIDE` REMOVED** (now a no-op)
- Prompts before writing shell-startup / build-tool config (`.npmrc`, `.bazelrc`, `.pre-commit-config.yaml`) in acceptEdits; Edit no longer needs a separate Read after a single-file `grep`

### v2.1.158
- **Opus 4.8 auto mode on Bedrock/Vertex/Foundry** — opt in via `CLAUDE_CODE_ENABLE_AUTO_MODE=1` for Opus 4.7 and 4.8
- **Plugins in `.claude/skills` auto-load** — no marketplace required; `claude plugin init <name>` scaffolds a new plugin there
- **`agent` field in `settings.json` honored for dispatched sessions** — `--agent <name>` overrides it
- **`EnterWorktree` switches between Claude-managed worktrees mid-session**
- **`tool_decision` telemetry includes `tool_parameters`** (bash commands, MCP/skill names) when `OTEL_LOG_TOOL_DETAILS=1`
- **Claude-managed worktrees left unlocked when the agent finishes** — `git worktree remove`/`prune` can now clean them up
- **`/config` "Workflow keyword trigger" setting** — stop the word "workflow" in a prompt from triggering a dynamic workflow; backspace right after the trigger keyword also dismisses (same as alt+w)
- `/plugin` autocomplete for subcommands, installed plugin names, marketplace plugins
- `claude agents`: slash-command autocomplete matches substrings

### v2.1.154 (Opus 4.8 launch)
- **Opus 4.8** — defaults to high effort; `/effort xhigh` for hardest tasks. Fast mode on 4.8 at 2x standard rate / 2.5x speed
- **Dynamic workflows** — ask Claude to create a workflow and it orchestrates work across tens-to-hundreds of agents in the background; `/workflows` views runs. (The `Workflow` tool primitive.)
- **Lean system prompt is now default** for all models except Haiku, Sonnet, and Opus 4.7-and-earlier
- **MCQ reservation** — Claude reserves the multiple-choice prompt for decisions it genuinely cannot make itself (asks less when it has enough context)
- **`/simplify` is now cleanup-only** (reuse, simplification, efficiency, altitude) and applies fixes — no longer the full `/code-review --fix` bug-hunt
- **Streaming tool execution always enabled** — including telemetry-disabled and Bedrock/Vertex/Foundry (was behind a feature flag)
- **Stdio MCP subprocesses receive `CLAUDE_CODE_SESSION_ID` + `CLAUDECODE=1`** in their env
- `claude agents`: `! <command>` runs a shell command as an attachable background session; also `claude --bg --exec '<command>'`
- Plugins can declare `defaultEnabled: false` in `plugin.json`/marketplace entry
- Effort slider labels renamed Speed/Intelligence → Faster/Smarter

### v2.1.153
- `skipLfs` option on `github`/`git` plugin marketplace sources (skip Git LFS during clone/update)
- Status line commands receive `COLUMNS` and `LINES` env vars
- `claude agents` dispatch autocomplete suggests native slash commands and bundled skills
- `/model` now saves selection as default for new sessions; press `s` for session-only. **Keybinding `modelPicker:setAsDefault` renamed to `modelPicker:thisSessionOnly`**

### v2.1.152
- **Skills/slash commands can set `disallowed-tools` in frontmatter** — remove tools from the model while the skill/command is active
- **`/reload-skills` command** — re-scan skill directories without restarting the session
- **`SessionStart` hooks can return `reloadSkills: true`** — skills installed by the hook become available in the same session
- **`SessionStart` hooks can set the session title** via `hookSpecificOutput.sessionTitle` (startup and resume)
- **`MessageDisplay` hook event** — transform or hide assistant message text as it is displayed
- **`pluginSuggestionMarketplaces` managed setting** — admins allowlist org marketplaces for context-aware plugin tips
- **`--fallback-model` now applies for the rest of the session** when the primary model is not found (was failing every request)
- **`app.entrypoint` OTEL metric attribute** — opt-in via `OTEL_METRICS_INCLUDE_ENTRYPOINT=true`
- `/code-review --fix` applies review findings to the working tree; `/simplify` invoked it (later split in 2.1.154)
- Auto mode no longer requires opt-in consent
- Workflow tool inline progress simplified; post-response timer reports "Waiting for N background agents/workflows to finish"
- `claude plugin marketplace remove` accepts `--scope user|project|local`

### v2.1.141
- **`terminalSequence` field on hook JSON output** — hooks can emit desktop notifications, window titles, and bells without a controlling terminal
- **`CLAUDE_CODE_PLUGIN_PREFER_HTTPS`** — clone GitHub plugin sources over HTTPS instead of SSH (for environments without a GitHub SSH key)
- **`ANTHROPIC_WORKSPACE_ID`** — scopes the minted workload-identity-federation token to a specific workspace
- **`claude agents --cwd <path>`** — scope the session list to a directory
- `/feedback` can now include recent sessions (last 24h or 7d) for cross-session issues
- Rewind menu: added "Summarize up to here" to compress earlier context while keeping recent turns intact
- Background agents launched via `/bg` or `←←` now preserve current permission mode (was reverting to default)

### v2.1.140
- Agent tool `subagent_type` matching is now case- and separator-insensitive (e.g. `"Code Reviewer"` → `code-reviewer`)
- Updated agent color palette

### v2.1.139
- **Agent view (Research Preview)** — `claude agents` opens a dashboard of every CC session (running, blocked, done). See https://code.claude.com/docs/en/agent-view
- **`/goal` command** — set a completion condition and CC keeps working across turns until it's met. Works in interactive, `-p`, and Remote Control. Shows live elapsed/turns/tokens
- **`/scroll-speed` command** — tune mouse wheel scroll speed with live preview
- **`claude plugin details <name>`** — show a plugin's component inventory and projected per-session token cost
- **Hook `args: string[]` field (exec form)** — spawn the command directly without a shell, so path placeholders never need quoting
- **Hook `continueOnBlock: true` for PostToolUse** — feed the hook's rejection reason back to Claude and continue the turn instead of stopping
- **MCP stdio servers receive `CLAUDE_PROJECT_DIR`** in their env, matching hooks; plugin configs can reference `${CLAUDE_PROJECT_DIR}` in commands
- **`x-claude-code-agent-id` / `x-claude-code-parent-agent-id` headers** on API requests from subagents; `claude_code.llm_request` OTEL spans include `agent_id` / `parent_agent_id` attributes
- Transcript view navigation: `?` for shortcuts, `{`/`}` to jump between user prompts, `v` toggles shortcut panel
- Compaction prompt now asks the model to preserve sensitive user instructions
- Remote MCP server reconnect retry is now enabled for all users
- Remote Control / `/schedule` / claude.ai connectors / notifications now disabled when `ANTHROPIC_API_KEY` / `apiKeyHelper` / `ANTHROPIC_AUTH_TOKEN` is set (unset key to use these features)

### v2.1.136
- **`settings.autoMode.hard_deny`** — classifier rules that block unconditionally regardless of user intent or allow exceptions (hard safety net for auto mode)
- **`CLAUDE_CODE_ENABLE_FEEDBACK_SURVEY_FOR_OTEL`** — re-enable session quality survey for enterprises capturing responses through OTel
- Fixed `--resume` / `--continue` not finding sessions when project path contains underscores
- Fixed MCP OAuth refresh tokens lost when multiple servers refresh concurrently
- Fixed plan mode not blocking writes when matching `Edit(...)` allow rule exists
- Fixed `AskUserQuestion` discarding multi-select answers when supplied as an array

### v2.1.133
- **`worktree.baseRef` setting** (`fresh` | `head`) — choose whether `--worktree`/`EnterWorktree`/agent-isolation worktrees branch from `origin/<default>` or local `HEAD`. **Default `fresh` is a reversal from 2.1.128's `head`** — set `head` to keep unpushed commits in new worktrees
- **`sandbox.bwrapPath` / `sandbox.socatPath`** (Linux/WSL managed settings) — custom bubblewrap and socat binary paths
- **`parentSettingsBehavior`** admin-tier key (`'first-wins' | 'merge'`) — opt SDK `managedSettings` into policy merge
- **Hooks receive `effort.level` JSON input + `$CLAUDE_EFFORT` env var**; Bash tool commands can also read `$CLAUDE_EFFORT`
- Fixed `Edit`/`Write` allow rules scoped to drive root (`C:\`) or POSIX `/` always prompting
- Fixed `HTTP(S)_PROXY` / `NO_PROXY` / mTLS not respected for MCP OAuth flow
- Fixed subagents not discovering project/user/plugin skills via the Skill tool

### v2.1.132
- **`CLAUDE_CODE_SESSION_ID` env var** in Bash tool subprocess — matches `session_id` passed to hooks (telemetry correlation)
- **`CLAUDE_CODE_DISABLE_ALTERNATE_SCREEN=1`** — opt out of fullscreen alt-screen renderer; keep conversation in terminal's native scrollback
- Fixed external SIGINT not running graceful shutdown (terminal mode restore)
- Fixed `--permission-mode` ignored when resuming plan-mode with `-p --continue/--resume`
- Fixed Bedrock/Vertex 400 errors when `ENABLE_PROMPT_CACHING_1H` is set
- Fixed statusline `context_window` counts reflecting cumulative session totals instead of current usage
- Fixed unbounded memory growth (10GB+ RSS) when stdio MCP server writes non-protocol data to stdout

### v2.1.129
- **`--plugin-url <url>` flag** — fetch a plugin `.zip` from a URL for the current session
- **`skillOverrides` setting now works** — `off` hides from model and `/`, `user-invocable-only` hides from model only, `name-only` collapses description
- **`CLAUDE_CODE_FORCE_SYNC_OUTPUT=1`** — force synchronized output on terminals auto-detection misses
- **`CLAUDE_CODE_PACKAGE_MANAGER_AUTO_UPDATE`** — Homebrew/WinGet auto-update in background with restart prompt
- **Plugin manifests:** `themes` and `monitors` should now be under `"experimental": {}` (top-level still works but `claude plugin validate` warns)
- **Gateway `/v1/models` discovery for `/model` picker is now opt-in** via `CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1` (was automatic in 2.1.126–128)
- **`claude_code.pull_request.count` OTel metric** now counts PRs/MRs created via MCP tools
- Fixed 1-hour prompt cache TTL silently downgraded to 5 minutes
- Fixed `Bash(mkdir *)`, `Bash(touch *)` allow rules not honored for in-project paths
- Fixed OAuth refresh race after wake-from-sleep logging out all sessions

### v2.1.128
- `--plugin-dir` now accepts `.zip` archives
- `--channels` works with console (API key) auth
- `EnterWorktree` creates branch from local HEAD (later reverted in v2.1.133 default)
- Subprocesses no longer inherit `OTEL_*` env vars (Bash-launched OTEL apps no longer pick up CLI's OTLP endpoint)
- MCP: `workspace` reserved as server name; reconnects summarize re-announced tools
- Fixed parallel shell tool calls: failing read-only command no longer cancels siblings
- Fixed sub-agent progress summaries missing prompt cache (~3× cache_creation reduction)

### v2.1.126
- **`claude project purge [path]`** — deletes all CC state for a project (transcripts, tasks, file history, config entry); supports `--dry-run`, `-y`, `-i`, `--all`
- **`--dangerously-skip-permissions` extended** — now bypasses prompts for writes to `.claude/`, `.git/`, `.vscode/`, shell config files; catastrophic-removal commands still prompt
- **`claude_code.skill_activated` OTel event** — fires for user-typed slash commands and Claude-proactive skill invocations; carries `invocation_trigger` (`"user-slash"`, `"claude-proactive"`, `"nested-skill"`)
- **Security:** Fixed `allowManagedDomainsOnly` / `allowManagedReadPathsOnly` being ignored when higher-priority managed-settings source lacked a `sandbox` block
- **Fixed deferred tools unavailable to skills with `context: fork`** on their first turn (v2.1.126)
- `/model` picker now lists models from your gateway's `/v1/models` endpoint when `ANTHROPIC_BASE_URL` points at a compatible gateway
- Read tool: removed per-file malware-assessment reminder that caused spurious refusals on legacy models

### v2.1.122
- **`ANTHROPIC_BEDROCK_SERVICE_TIER` env var** — select Bedrock service tier (`default`, `flex`, `priority`)
- **OTel: numeric attributes** on `api_request`/`api_error` log events now emitted as numbers (not strings)
- **OTel: `claude_code.at_mention` log event** — fires on `@`-mention resolution

### v2.1.121
- **`alwaysLoad` option on MCP server config** — when `true`, all tools from that server skip tool-search deferral and are always available
- **`claude plugin prune`** — removes orphaned auto-installed plugin dependencies; `plugin uninstall --prune` cascades
- **PostToolUse `hookSpecificOutput.updatedToolOutput` for ALL tools** — previously MCP-tool-only; now any built-in tool output can be replaced by a hook
- **`CLAUDE_CODE_FORK_SUBAGENT=1` in non-interactive sessions** — now works in SDK and `claude -p` mode

### v2.1.120
- **`${CLAUDE_EFFORT}` in skill content** — skills can reference the current effort level dynamically
- **`AI_AGENT` env var for subprocesses** — set so `gh` and other tools can attribute traffic to Claude Code
- **`claude ultrareview [target]` subcommand** — run `/ultrareview` non-interactively from CI; `--json` for raw output

### v2.1.119
- **`duration_ms` on hook inputs** (2026-04-25) — `PostToolUse` / `PostToolUseFailure` hooks now receive tool execution time.
- **Subagent/MCP parallel reconnect** (2026-04-25) — SDK MCP server reconfiguration connects in parallel.
- **`--print` honors agent frontmatter** — `tools:` and `disallowedTools:` enforced in headless mode.
- **`--agent <name>` honors `permissionMode`** for built-in agents.
- **PowerShell auto-approve** in permission mode (matches Bash behavior).
- **`prUrlTemplate` setting** — point footer PR badge at custom code-review URL.
- **`/config` settings persist to `settings.json`** with project/local/policy override precedence.
- **OTel `tool_result` adds `tool_use_id` + `tool_input_size_bytes`**.
- **Status line stdin includes `effort.level` + `thinking.enabled`**.

### v2.1.118
- **`type: "mcp_tool"` hooks** (2026-04-22) — hooks can directly invoke MCP tools, no shell shim.
- **Custom themes** in `~/.claude/themes/` JSON files; plugins ship themes via `themes/` directory.
- **`DISABLE_UPDATES` env var** — stricter than `DISABLE_AUTOUPDATER`, blocks all update paths.
- **`wslInheritsWindowsSettings` policy key** — WSL inherits Windows-side managed settings.
- **`autoMode.allow/soft_deny/environment` `"$defaults"` token** — extend built-ins instead of replacing.
- **`claude plugin tag`** — create release git tags for plugins with version validation.

### v2.1.117
- **`CLAUDE_CODE_FORK_SUBAGENT=1`** — forked subagents on external builds.
- **Agent frontmatter `mcpServers`** loaded for main-thread `--agent` sessions.
- **Native `bfs`/`ugrep` on macOS/Linux** — Glob and Grep tools now embedded in Bash, faster searches.
- **`cleanupPeriodDays` covers `tasks/`, `shell-snapshots/`, `backups/`** — full retention sweep.
- **OTel `user_prompt` events include `command_name` + `command_source`**.

### v2.1.116
- **67% faster `/resume`** on 40MB+ sessions.
- **Faster MCP startup** with multiple stdio servers (concurrent connect by default).
- **Sandbox auto-allow safety check** for `rm`/`rmdir` against `/`, `$HOME`, critical paths.

### v2.1.113 (native build cutover)
- **Native Claude Code binary** via per-platform optional dependency (replaces bundled JS).
- **`sandbox.network.deniedDomains` setting** — block specific domains even when broader allowedDomains permits.
- **Bash deny rules** match commands wrapped in `env`/`sudo`/`watch`/`ionice`/`setsid`.
- **`Bash(find:*)` allow rules** no longer auto-approve `-exec`/`-delete`.

### v2.1.111-110
- **`/tui` command + `tui` setting** — toggle flicker-free fullscreen rendering mid-session.
- **Push notification tool** — Claude can send mobile pushes when Remote Control + "Push when Claude decides" enabled.
- **`/loop` Esc cancels pending wakeups**, wakeups display as "Claude resuming /loop wakeup".
- **`/less-permission-prompts` skill** — scans transcripts for read-only Bash/MCP calls, proposes allowlist.
- **`/ultrareview` command** — multi-agent cloud code review with parallel analysis.
- **`xhigh` effort level for Opus 4.7** — between `high` and `max`.
- **`autoScrollEnabled` config** — disable auto-scroll in fullscreen.
- **`--enable-auto-mode` no longer required** for auto mode.

## Stable capabilities

### Hooks (12 events)
- `SessionStart`, `SessionEnd`, `UserPromptSubmit`, `Notification`, `PermissionRequest`, `PermissionDenied`
- `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PreCompact`, `PostCompact`
- `SubagentStart`, `SubagentStop`, `Stop`, `StopFailure`, `WorktreeCreate`, `WorktreeRemove`, `TaskCompleted`, `TeammateIdle`, `ConfigChange`, `InstructionsLoaded`
- Handler types: `command`, `http`, `prompt`, `mcp_tool` (new in v2.1.118), `agent`
- Async execution via `async: true`

### Skills
- File-based skills in `~/.claude/skills/<name>/SKILL.md` with YAML frontmatter (`name`, `description`)
- Plugin skills via `plugins/.../skills/`
- `/skills` menu (sortable by token count via `t`)
- Universal SKILL.md format compatible with Cursor, Codex CLI, Antigravity, Gemini CLI

### Commands / CLI
- Slash commands at `~/.claude/commands/<group>/<name>.md`
- Plugin commands as `plugin:command-name`
- `--from-pr` accepts GitLab MR, Bitbucket PR, GitHub Enterprise URLs (v2.1.119)
- `--print` / `-p` headless mode honors agent frontmatter
- `--bare` flag skips hooks/LSP/plugin sync (requires API key)

### MCP
- Three scope levels: `--scope user` (global), `--scope local` (per-project), `--scope project` (`.mcp.json` shared)
- Tool Search (lazy loading) for context efficiency
- `MCP_CONNECTION_NONBLOCKING=1` for `-p` mode
- HTTP/SSE/WebSocket transports with `${ENV_VAR}` header substitution

### Settings / permissions
- `permissions.allow` / `permissions.deny` rules
- `permissions.deny` overrides PreToolUse hook decisions (fixed in v2.1.x)
- `auto` mode + `bypassPermissions` mode
- `effortLevel`: `low`, `medium`, `high`, `xhigh`, `max`
- `autoCompact` setting + `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`

### IDE integrations
- VSCode, Cursor, Windsurf — Manage Plugins panel, Remote Control
- Neovim, JetBrains
- Voice dictation in VSCode (macOS)
