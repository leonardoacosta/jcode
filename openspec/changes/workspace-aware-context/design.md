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
- Coordinator passes resolved workspace id to workers so they do not re-resolve from a different cwd.
