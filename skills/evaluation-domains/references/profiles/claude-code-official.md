
# Profile: claude-code-official

Domain: Claude Code / Claude Agent SDK / Anthropic API currency signals — hooks, skills, MCP,
CLI flags, model IDs, pricing, and relevant upstream releases. Caller: any release-currency
review that needs to decide whether a newly shipped capability should be adopted.

## 1. Primary-source definition

Anthropic's own changelog and GitHub releases outrank everything else. Order: (1) Anthropic
official changelog / docs site, (2) `anthropics/claude-code` GitHub releases + `CHANGELOG.md`,
(3) npm registry metadata for `@anthropic-ai/claude-code` (version, publish date — corroborates
a release actually shipped), (4) beads/OpenSpec upstream release notes for their own signals.
Community write-ups (blog posts, Twitter/X threads, Reddit) are never primary — they may surface
a signal worth checking, but the signal is only confirmed against sources 1-4.

## 2. Credibility axes + weights

CRAAP axes from `references/axes-and-procedures.md` § 2, weighted for an official-vendor domain:

| Axis | Weight | Rationale |
|---|---|---|
| Authority | High | The whole point of this domain — vendor-official beats every derivative |
| Currency | High | A feature can ship, then be deprecated within weeks; date matters as much as existence |
| Accuracy | Medium | Corroborate via a second official source (release notes + changelog agreement) when a signal is ambiguous |
| Relevance | Medium | Filter to the practices areas the consuming repository explicitly tracks |
| Purpose | Low | Vendor-official sources have no ranking/engagement incentive to discount |

## 3. Staleness horizon

Default >30-day re-verify rule (`references/axes-and-procedures.md` § 4). Repositories may use a
shorter automated refresh cadence; the 30-day horizon is the outer bound for a signal nobody
has re-checked, not a recommended polling interval.

## 4. Verification procedure

**Signal-bundle liveness** (`references/axes-and-procedures.md` § 3). A feature/deprecation
signal is not called "confirmed" from one source alone — corroborate changelog prose against
the GitHub release tag and, where version-gated, the npm-published version date. This mirrors
CHAOSS's caveat against single context-free indicators, applied here to release-claim liveness
rather than repo health.

## 5. Verdict vocabulary

Use a repository-local decision record per signal. A decision is one of: **adopt** (implement now),
**defer** (relevant, not yet prioritized), **reject** (not applicable to this setup), each
carrying the researching agent's rationale.

## 6. Memory home

Keep a cheap-to-scan adoption-signals index with one heading per signal and link each entry to
an append-only repository-local decision record such as
`{version, decisions: {<slug>: {..., history: [...]}}}`. A signal absent from the map, or present
with empty `history[]`, is undecided. This shape follows R1 (append-only record plus generated
view) and R3 (progressive disclosure); see `references/record-keeping.md`.
