## Decisions

- Detection runs `git remote get-url origin` once per working dir; userinfo stripped before matching or logging.
- First matching rule wins; config order is precedence.
- Identity cache key: repo root + mtimes of repo `.git/config` and `~/.gitconfig`.
- Badge renders `ws:<name>`; mismatch appends `!id`. Width measured in display cells, truncated with `…`.
- Hint block capped near 400 chars.

## Risks

- Misconfigured rules mislabel repos; hints are phrased as configured, not authoritative.


## Workspace hooks and swarm

- Workspace `hooks` entries append to global hooks for the same event; matching `command` replaces. Requires `add-lifecycle-hooks`.
- Model precedence: explicit spawn `model` > workspace `swarm_model`/role slot > `agents.swarm_model` > inherit. Requires `add-role-model-slots`.
- Coordinator passes context as provenance; workers verify the actual target repository/workspace before applying hooks, tracker/identity hints or model overrides. Verified same-repository worktrees can inherit identity; differing/unknown targets cannot silently inherit another workspace. Config fingerprints include effective sources, not only root mtimes.

## Accepted limits (review 2026-11-19)

- Rules that can never match (`origin` containing `://`, `@` or a `.git` suffix, because matching runs on the sanitized `host/path`) are not warned about (m2).
- `with_workspace_hooks` merges the six lifecycle events known today; a future `LifecycleEvent` needs a matching merge line, and replacement is keyed by `command` alone, not by (command, matcher) (m5).
- Provenance is recorded after the worker session exists, so a headless worker's very first prompt may omit the mismatch note; authority is unaffected because provenance is advisory (m7).
- Local (non-server) TUI sessions (`turn_memory.rs`, no working dir passed) get no workspace prompt block; hints are server-session only (m9).
- At narrow widths the trailing `!id`/`?id` marker truncates first, and the badge disappears below about 10 cells; the row never wraps (m10). CJK workspace names are truncated by display cells like any header item.
- The `communicate` model-source display re-resolves with `params.working_dir` or the tool's working dir, while the server uses the coordinator session cwd; the two agree today and must be kept in sync (m12).
- Identity cache freshness: the stamp covers repo, `config.worktree`, system, global and XDG configs plus `include`/`includeIf` targets (3 levels), with a 60s TTL backstop for what a stamp cannot see (`GIT_CONFIG_COUNT`, glob includes). `git` calls time out at 2s and are killed.
