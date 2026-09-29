# Global Graft integration for Jcode

Status: applied 2026-09-26. All changes are global (`~/.jcode`); no repository
was modified.

## What was done

| Change | File | Effect |
| --- | --- | --- |
| Added graft usage guidance | `~/.jcode/prompt-overlay.md` | Every Jcode session loads graft tool guidance: `map`, `ask --source`, `grep`, `skeleton`, `callers`, plus the per-turn savings tally rule. Points at the canonical skill `~/.agents/skills/graft/SKILL.md`. |
| Removed the managed graft section | `~/.jcode/AGENTS.md` | Deleted the `<!-- jcode:graft-startup -->` block that duplicated the same guidance. |
| Dropped the startup helper from `session_start` | `~/.jcode/config.toml` | `session_code_intelligence.py` no longer runs on session start. The Herdr `report` hook on `turn_start`, `turn_end`, `session_start`, and `session_end` is untouched. |

`~/.jcode/scripts/session_code_intelligence.py` and its test remain in place as
dormant opt-in utilities. Re-enable by adding the python3 command back to
`hooks.session_start`.

## Why these three

- Graft query commands refresh the graph themselves, so the startup
  `graft build --no-gitignore --no-ignore` was redundant work.
- Graft's installed Claude modules (`dist/claude/hooks.js`) dispatch on Claude
  event names and payload fields (transcript path, tool output). Jcode observers
  expose only tool name, status, duration, and output byte count, so no direct
  adapter was justified.
- Jcode has no edit-specific or stop hook; guidance carries that behavior.

## Deliberate omissions

- No `post_tool` savings parser: the hook cannot see tool output.
- No statusline equivalent: the savings rule lives in the prompt overlay.
- No `session_end` change.

## Verification performed

- `python3 -c "tomllib.load(...)"` on `config.toml`: parses, `session_start`
  now holds only the Herdr reporter.
- `grep` for `session_code_intelligence` in `config.toml` and `AGENTS.md`: no
  matches, `grep` for `jcode:graft-startup`: no matches.
- `AGENTS.md` tail ends at the pre-existing Jcode instructions; overlay begins
  with the new `# Graft code navigation` section.

## Remaining risk

Prompt-overlay changes apply to new sessions; a session already running keeps
its captured prompt.
