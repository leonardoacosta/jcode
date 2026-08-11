# Refresh -> Rewrite -> Answer Workflow

The skill separates two concerns:

1. **Deterministic fetch + diff** — `scripts/refresh.sh` pulls the three upstream sources,
   compares signatures against `state/last-checked.json`, and reports whether anything changed.
   It never interprets content; it only decides "is the cache stale?".
2. **Structured extraction** — When the script reports a change, you (Claude) read the raw cache
   and rewrite the three reference files with a summary of what the sources are saying. This is
   the LLM step — the one where you add value over a pure RSS reader.

This split matters because the diff has to be cheap (runs every load) while the extraction is
only worth doing when something actually moved.

## Step 1: Refresh the cache

Always run the refresh script first. It's idempotent and fast (under 2 seconds when upstream is
reachable).

```bash
bash "$(dirname "$0")/scripts/refresh.sh"
# or from anywhere:
bash ~/.claude/skills/cc-practices-current/scripts/refresh.sh
```

The script exits with one of:

| Exit | Meaning | What to do next |
|------|---------|-----------------|
| `0` | All sources unchanged since last check | Skip to Step 3, read `references/` as-is |
| `2` | At least one source moved | Proceed to Step 2 — regenerate the reference files |
| `1` | One or more sources failed to fetch and nothing new | Proceed to Step 3 with a caveat noting which source is stale |

The script writes fresh copies under `state/cache/` and updates `state/last-checked.json`
regardless of outcome.

## Step 2: Rewrite references (only on exit 2)

Read the primary cache files:

- `state/cache/changelog.md` — GitHub CHANGELOG.md (canonical markdown, always current)
- `state/cache/github-releases.json` — array of releases with `tag_name`, `published_at`, `body`
- `state/cache/npm.json` — package metadata; check `dist-tags.latest` and `dist-tags.stable`

Then check which of the five reference-doc sources actually changed (compare each
`*_hash` field in `state/last-checked.json` against the prior run's value, or just look for a
`state/cache/<name>.prev.md` file — `refresh.sh` only writes one when that specific doc's hash
moved). For each one that changed:

- `state/cache/env-vars.md` (+ `.prev.md` if present) — environment variables reference
- `state/cache/tools-reference.md` (+ `.prev.md`) — tools reference
- `state/cache/hooks.md` (+ `.prev.md`) — hooks reference
- `state/cache/plugins-reference.md` (+ `.prev.md`) — plugins reference
- `state/cache/channels-reference.md` (+ `.prev.md`) — channels reference

Run `diff state/cache/<name>.prev.md state/cache/<name>.md` to see exactly what changed — these
are full reference pages, not changelogs, so a bare hash flip tells you nothing about content on
its own. Only promote a diffed change to a signal per the § adoption-signals.md rules below
(a real recommendation, not an existence fact); most reference-doc diffs are wording/formatting
churn, not adoption-worthy. If a doc's hash didn't change, skip it entirely — don't re-read an
unchanged 50-300KB reference page every run.

Also read the prior `references/*.md` — you are **updating**, not clobbering. Preserve entries
that are still accurate; add new ones at the top; move newly-deprecated items from features.md
to deprecations.md.

Then rewrite the three reference files per the schemas below.

### references/features.md

Canonical list of current CC capabilities. Use this exact top-to-bottom structure:

```markdown
# CC Features — Current

_Last refreshed: <ISO date> — sources: docs <hash>, gh <tag>, npm <version>_

## Added since <prior-version>
- **Feature name** (`v2.1.x`, YYYY-MM-DD) — one-line description with the key invocation, flag, or config snippet.
- ...

## Stable capabilities
### Hooks
- ...
### Skills
- ...
### Commands / CLI
- ...
### MCP
- ...
### Settings / permissions
- ...
### IDE integrations
- ...
```

The "Added since" section is the delta that `/workflow:evolve` consumes to identify
opportunities. Keep it narrow to the actual delta since the previous `last_change_detected`
version — if you dump the whole changelog here it stops being useful.

Keep every line terse: feature name, version, one sentence, maybe a code fragment. No
paragraphs.

### references/deprecations.md

Anything the changelog marks removed, renamed, soft-deprecated, or "no longer recommended":

```markdown
# CC Deprecations

_Last refreshed: <ISO date>_

## Active deprecations
- **Old pattern** → **New pattern** (deprecated in `v2.1.x`, YYYY-MM-DD)
  - **Migration**: _one sentence_
  - **Detection**: _grep regex or glob pattern that finds the old pattern in a user's setup_
  - **Removed in**: _version if known, else "not yet"_

## Removed
- **Old pattern** (removed in `v2.1.x`) — brief note on what replaced it, or "no replacement".
```

The **Detection** field is the load-bearing contribution: it lets auditors flag specific files
without re-parsing the changelog. Prefer specific patterns (paths + content) over keyword
matches, because false positives make the audit noisy.

### references/adoption-signals.md

The actionable layer — the output `/workflow:evolve` scores against. Each signal describes a
thing the user *should check in their setup*:

```markdown
# CC Adoption Signals

_Last refreshed: <ISO date>_

## <Area: hooks | skills | mcp | commands | settings | agents | memory>

### <Signal name>
- **What**: one-line description of the current recommendation
- **Why**: why adopting matters (safety, performance, DX, correctness)
- **Check**: a shell one-liner whose exit code tells you whether the user already has it (exit 0 = present, non-zero = missing). Must conform to the verb-list constraint below.
- **Action**: what to do if missing, as a concrete instruction or command
- **Introduced**: `v2.1.x`, YYYY-MM-DD
```

#### Verb-list constraint for `Check` fields

The `Check` field is executed by `/workflow:evolve` with `bash -c "$check"`. To make that safe,
auditable, and portable, every `Check` value must conform to these rules:

- **Allowed tools only** — `grep`, `rg`, `jq`, `test`, `[`, `ls`, `find`, `cat`, `head`, `tail`,
  `wc`. Nothing else. No `bash`/`sh`/`eval`, no process substitution that invokes other binaries,
  no `|` piping into non-allowlisted commands.
- **No network** — no `curl`, `wget`, `nc`, `ping`. If the check requires network, it's not a
  local-setup check and doesn't belong here.
- **No writes** — the check is read-only. No `>`, `>>`, `tee`, `touch`, `mv`, `rm`, `mkdir`.
- **Absolute paths** — use `~/.claude/...` paths (tilde expansion is fine). Do not `cd` into a
  directory, do not rely on `$PWD`.
- **Fast** — must complete in under 2 seconds on a normal machine. If a `find` could take longer,
  scope it with `-maxdepth`.
- **Exit semantics** — exit 0 = signal is present (user is current), non-zero = signal is missing
  (gap). If a check naturally inverts (e.g. `! grep -q ...`), use `|| true` / `&& false`
  explicitly so the exit code is unambiguous.

If a signal cannot be expressed within this verb list, **omit the `Check` field and describe the
check in the `Action` field as a manual instruction instead**. A vague automated check is worse
than an honest manual one — it would produce confidently wrong audit scores.

Signals should be **gradeable**: if the `Check` command passes, the signal is resolved; if it
fails, it's an open gap. This is what lets the audit produce a numeric score instead of a vibes
read.

Only promote an item to a signal if it's a real recommendation, not just an existence fact.
"Hooks exist" is a feature; "PreToolUse hooks should use a tool-matcher to avoid regex
false-positives" is a signal.

## Step 2.5: Decision Log Maintenance

After Step 2 rewrites the reference files, ensure `state/decisions.json` is consistent:

1. **New signals** (in `adoption-signals.md` but not in `decisions.json`):
   - Skip — `/workflow:evolve` is responsible for populating these via `cc-feature-analyst`.
   - This skill only ensures the file exists with a valid empty shape if it's missing.

2. **Removed signals** (in `decisions.json` but no longer in `adoption-signals.md`):
   - Do **not** delete. Keep the entry; it represents historical decision context. The signal
     may have been removed because it was superseded — the history shows what we decided when
     it was current.

3. **First-time skill use**:
   - Create `state/decisions.json` with `{"version": 1, "decisions": {}}` if missing.

## Step 3: Answer the user's question

Use the three reference files as your source of truth. When citing specific features or
deprecations, include the version and date from the file so the user can verify.

If the script reported exit 1 (stale), mention which source is stale in your answer — don't
silently serve outdated data.
