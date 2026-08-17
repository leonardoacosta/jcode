---
name: cc-practices-current
description: Cached catalog of current CC features and deprecations from upstream sources. Explicit-only, loaded by /workflow:evolve.
allowed-tools: Read, Glob, Grep, Bash
---

# Claude Code Currency Reference

Maintains an up-to-date catalog of Claude Code features, deprecations, and adoption signals by tracking ten upstream sources: CC changelog, CC GitHub releases, CC npm metadata, beads releases, OpenSpec releases, plus five CC reference docs sha256-hash-diffed independently of the changelog (env vars, tools, hooks, plugins, channels — added 2026-07-21 because a silent reference-doc update carries no changelog bullet and was previously invisible to evolve entirely). This is the authoritative cache behind `cc-practices-analyst` and `/workflow:evolve` — the slow path (web search + changelog parsing) only fires when upstream state actually changes.

## Effort Gates

Behavior is adaptive on `${CLAUDE_EFFORT}` (v2.1.120+) — `low`/`medium` trusts the cache and
skips the refresh script, `high`/`xhigh`/`max` runs a full upstream refresh. Read
`references/effort-gates.md` when deciding whether this run needs a fresh fetch or can trust the
on-disk cache.

## When to use this skill

- Someone asks "what's new in Claude Code" or "what CC features should I know about"
- Running `/workflow:evolve` or any CC currency audit
- Deciding whether a pattern in the user's setup is current, legacy, or deprecated
- Answering "should I adopt X" for any CC feature, hook, MCP server, or command
- Migrating away from a deprecated CC pattern

**Do not** use this for:
- Implementation questions about a specific project's application code
- One-off quick lookups where you already know the answer from baked-in context and the user isn't asking about currency
- Topics outside the CC / Claude Agent SDK / Anthropic API surface

## How it works

The skill separates two concerns: a **deterministic fetch + diff** (`scripts/refresh.sh` pulls
the five upstream sources and reports whether anything changed — it never interprets content,
only decides "is the cache stale?") and a **structured extraction** (when the script reports a
change, you read the raw cache and rewrite the three reference files — the LLM step, the one
where you add value over a pure RSS reader). This split matters because the diff has to be
cheap (runs every load) while the extraction is only worth doing when something actually moved.

## Workflow

Three steps: refresh the cache (`scripts/refresh.sh`, exit 0/1/2 branches what happens next),
rewrite the three `references/*.md` files only on exit 2 (each has its own required structure
and a verb-list constraint on `Check` fields), then answer the user's question from those files.
Read `references/workflow.md` when actually running this skill end-to-end — the exit-code
branch table, the exact per-file structure for `features.md`/`deprecations.md`/
`adoption-signals.md`, the `Check`-field safety constraints, and Step 2.5's decision-log
maintenance rules all live there.

## State & Decision-Log Schemas

`state/last-checked.json` tracks fetch signatures; `state/decisions.json` is the append-only
decision log `/workflow:evolve` writes verdicts into (never delete a superseded entry — it's
historical context). Read `references/schemas.md` when writing or reading either file directly,
or when reconciling `decisions.json` against a changed `adoption-signals.md`.

## Integration points

- **`cc-practices-analyst` agent** — primary consumer. Loads this skill before answering any CC practices question. The agent needs `Bash` (to run refresh.sh) and `Read` (to read references/) in its frontmatter tools list.
- **`/workflow:evolve`** — spawns `cc-practices-analyst`, which loads this skill. Step 1 (Discovery) of evolve.md reads `adoption-signals.md` instead of re-running WebSearch queries from scratch. Only the items that the skill flags as new since last refresh trigger fresh web lookups.
- **Direct invocation** — any session asking "what's new in CC" should trigger this skill via description match on the frontmatter.

## NEVER (Anti-Patterns)

- **NEVER serve stale data silently.** If `refresh.sh` exits 1 (a source failed to fetch), still
  answer from the existing references — but name which source is stale in your answer
  ("my reference material is from `<last_checked>`, and I couldn't reach upstream on this run").
  Presenting old data as current, with no caveat, is worse than an honest "this might be behind."
- **NEVER treat one failed source as a total failure.** `refresh.sh` still writes fresh copies of
  the other four sources' caches even when one fails (exit 1 with `CHANGED=0`, or exit 2 if another
  source changed). Treat only the failed source as "last-known-good" from its prior cache — don't
  discard the ones that succeeded.
- **NEVER clobber `references/*.md` when rewriting.** Step 2 is an update, not a from-scratch
  regeneration: preserve entries still accurate, add new ones at the top, move newly-deprecated
  items to `deprecations.md`. A full clobber destroys institutional history the next Step 2 pass
  can't reconstruct from the raw cache alone.
- **NEVER delete a signal from `state/decisions.json` because it dropped out of
  `adoption-signals.md`.** Keep the entry — it's historical decision context showing what was
  decided while the signal was current; it may simply have been superseded, not invalidated.
- **NEVER write a `Check` field that falls outside the verb-list allowlist** (`bash`/`sh`/`eval`,
  network calls, writes, relative paths, anything over ~2s). A vague or unsafe automated check
  produces confidently wrong audit scores — omit `Check` and push the check into `Action` as a
  manual instruction instead.
- **NEVER run the full refresh path at `low`/`medium` effort.** Trust the cache and skip
  `scripts/refresh.sh` — the refresh path is only load-bearing during `/workflow:evolve`-class
  full-currency audits, not routine "is X deprecated?" lookups.
- **NEVER populate a first-run `features.md` by dumping the entire changelog.** On an empty-cache
  first run (exit 2 because all five "prior" signatures are `none`), "Added since" should read
  "Initial cache — everything is new" and list only the most recent 10–15 entries — a full dump
  stops the section from being the delta `/workflow:evolve` actually needs.
