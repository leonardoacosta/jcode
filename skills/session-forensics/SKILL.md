---
name: session-forensics
description: >
  Mine session transcripts and telemetry for usage evidence — command/skill/agent adoption,
  hook fire counts, MCP invocations, cost attribution, failure patterns. Triggers: is this used,
  usage evidence, adoption, dispatch count, transcript mining, telemetry, how often, dead weight,
  did it ever fire, zero invocations.
allowed-tools: Read, Bash, Grep, Glob
---

# Session Forensics

Every "should we keep/retire/fix X" decision in cc is supposed to be evidence-backed, and the
evidence already exists on disk — the failure mode is guessing, or querying the WRONG store
(`rtk discover` reads pre-rewrite transcripts and reports fake 0% adoption; memory claims go
stale in ~30 days). This skill maps each question to its authoritative store and gives the
extraction recipes. Recipes live in [references/query-cookbook.md](references/query-cookbook.md).

## Store Routing Table (question -> source of truth)

| Question | Authoritative store | NOT this |
| --- | --- | --- |
| Was command /X ever invoked? How often? | `~/.claude/telemetry/command-invocations.jsonl` | Grepping CLAUDE.md for mentions |
| Which agents get dispatched, at what volume? | `~/.claude/projects/*/*/subagents/agent-*.meta.json` (`agentType` + mtime), or `scripts/bin/agent-dispatch-census --json` | The agents/ directory listing (defined != used) |
| Did hook Y ever fire? | Transcript `hookEvent`/`hookName` entries + `~/.claude/telemetry/agents-active.json` deregistration | settings.json presence — config presence != runtime liveness (SubagentStop was wired 3 months, fired ZERO times) |
| Which MCP servers/tools are used? | `~/.claude.json` `toolUsage` map (`mcp__<server>__<tool>` -> `usageCount`, `lastUsedAt` epoch-ms) | `rtk history.db` (rtk's own rewrite ledger, not an MCP log) |
| RTK rewrite **adoption** (how often a rewrite fired) | `~/.local/share/rtk/history.db` (sqlite, `commands` table, count only) + `rtk gain` | `rtk discover` (pre-rewrite artifact — fake 0%) |
| RTK cost **benefit** | ccusage billing over a declared window — `docs/rtk-upstream-trial.md` § How to measure | `saved_tokens` / `rtk gain`'s Saved column / `rtk-adoption` — rtk's own estimate, structurally unable to report a loss (bead `cc-w83ov.217`: 98.5% of a 30d total came from 148 outlier rows) |
| What sequences/patterns repeat across sessions? | Session JSONLs under `~/.claude/projects/<proj>/*.jsonl`; `/workflow:retrospect` + `sequence-mine` for command windows | Memory files (hypotheses, not counts) |
| What did a specific agent/workflow actually return? | `subagents/agent-*.jsonl` transcripts; workflow `journal.jsonl` | The orchestrator's summary of it |
| Cost by model/session | Transcript model IDs + `scripts/lib/cost-rates.sh` (`_compute_cost`) | Hand-copied rate tables (single source of truth rule) |
| Repeated failures | `$HOME/.claude/scripts/state/failures/*.jsonl` | Anecdote |

## Evidence Standards (binding)

1. **Counts, windows, and the query itself** — every claim ships as
   `N events in <window> per <store>` plus the command that produced it. An adoption claim
   without a window is not evidence.
2. **Absence needs a denominator.** "0 fires" only matters against matched opportunities
   (the hook-liveness pattern: matched dispatches > 0 AND fires = 0 in the same window). Zero
   over zero is "no data yet," not "dead."
3. **Bound windows to artifact age.** Measure from `max(window cutoff, artifact mtime)` — a
   feature that landed last week legitimately has one week of data, not 90 days of failure.
4. **Homelab is the primary machine; Mac-side transcripts are a partial sample, not a census.**
   2026-07-21 homelab audit: 2,680 sessions / 4.94GB on homelab vs. 2 live project dirs on the
   Mac's `projects/` store. Treat homelab transcripts as the near-complete evidence base; when a
   session isn't running on homelab, use the staged server-side extractor pattern (§ query-cookbook
   "remote-machine stores") rather than drawing conclusions from the Mac's own thin local store.
   (Supersedes the prior "this machine is near-complete evidence" framing, corrected 2026-07-04,
   which assumed Mac-primary usage.)
5. **Re-verify recalled claims >30 days old** against the live stores before reusing them.
6. **Do not write findings to files.** Forensics returns in response text (or the artifact the
   requesting command owns: bead note, plan, ratchet row) — no `analysis-*.md` dumps.

## Standard Procedure

1. Route the question via the table above; note which stores are needed.
2. Pick the window: 90d for keep/retire calls (matches `agent-dispatch-census` +
   `mcp-zero-invocations` convention), 30d for cost, "since artifact mtime" for new things.
3. Run the cookbook recipe; prefer existing harnesses (`agent-dispatch-census`,
   `improvement-ledger list`) over bespoke one-liners when one exists.
4. Cross-check one independent signal before a strong conclusion (the SubagentStop finding was
   confirmed two ways: transcript hookEvent count AND 20,890 never-deregistered telemetry
   entries).
5. Report: claim + count + window + store + query, then the recommendation.

## NEVER

| Never | Why |
| --- | --- |
| Use `rtk discover` for adoption numbers | Reads pre-rewrite transcripts; documented fake-0% artifact |
| Treat settings.json/agents-dir presence as usage | Defined-but-never-dispatched is the exact class `zero_dispatch` exists to find |
| Read a whole session `.jsonl` into context | Multi-MB files; extract with python/jq per the cookbook, pull only matching lines |
| Follow `subagents/*.output` symlinks via Read | They point at full transcripts — context blowout; parse the `.jsonl` selectively |
| Claim "unused, safe to delete" from one store | Cross-check a second signal; usage may route through an alias, hook, or another surface |
| Extrapolate rates without checking `cost-rates.sh` handles the model ID | Dated suffixes and unlisted IDs silently zeroed costs for months (cost-coverage row history) |
