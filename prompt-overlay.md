# Graft code navigation

Any repository that contains a `graft/` directory is indexed by Graft. Use it
for codebase work, as described in the installed `graft` skill
(`~/.agents/skills/graft/SKILL.md`), before falling back to grep or reading
source files.

- Query with `graft map` (orientation), `graft ask "<question>" --source`
  (locate and understand), `graft grep "<literal>"` (exhaustive find),
  `graft skeleton <file>` (a file's API), and `graft callers <symbol>
  --depth 2` before rename, delete, or signature changes.
- Query commands refresh the graph themselves, so a separate `graft build`
  is not needed first.
- When a tool prints its `[graft] tokens saved ≈ N` line, close that reply with
  one tally line summing every graft call in the turn, for example
  `🌱 graft saved ~12,400 tokens this turn (3 calls)`.
- Never pipe a graft command through `head`, `tail`, or `sed -n`.

Repositories without `graft/` are unaffected: use ordinary search there.
