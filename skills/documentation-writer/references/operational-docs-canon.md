# Operational Docs Canon

> The conventions a repo's `docs/` tree should follow to stay trustworthy and machine-checkable.

## Directory Roles

A repo's `docs/` tree has four roles, each with a distinct lifecycle. Do not blend them — a
runbook that drifts into investigation narrative, or a reference page that turns into a dated
journal entry, breaks the reader's ability to trust the directory by its name alone.

| Directory | Role | Lifecycle | Archive peer |
| --- | --- | --- | --- |
| `reference/` | How-it-IS state per subsystem — current facts, not history. Some pages are machine-written pipeline artifacts (JSON/generated tables), not hand-authored. | Long-lived; updated in place as the system changes | N/A — reference pages are corrected, not archived |
| `notes/` | Dated investigation journal — "here's what I found and when." One note per investigation, never edited after the fact except a dated correction banner (below). | Short-lived; superseded notes move to `archive/` | `notes/archive/` |
| `guides/` | Runbooks — step-by-step operational procedures a human or agent executes. | Long-lived; updated when the procedure changes | N/A — guides are corrected, not archived |
| `diagrams/` | Visual explainers (mermaid, HTML, rendered images) referenced from the other three. | Long-lived, paired with its source | N/A |

Each of `notes/` (and optionally `guides/`) SHOULD carry an `archive/` peer subdirectory for
superseded content — moving a stale note out of the active tree is what keeps a dangling-ref
sweep meaningful (a note the sweep still scans should still be live).

## Frontmatter Contract

Every doc page opens with YAML frontmatter carrying at minimum:

```yaml
---
title: <human title>
type: reference|guide|note|diagram|moc|machine|adr
domain: <subsystem or capability area>
tags: [<searchable keywords>]
status: current|draft|superseded
updated: <YYYY-MM-DD>
---
```

`status` and `updated` are the machine-checkable freshness signal — a dangling-ref sweep or a
staleness classifier reads these two fields, not the file's git mtime (git history can be
misleading after a rename or a bulk reformat commit touches unrelated lines).

**Accepted `status` extension for roadmap-type docs**: `approved-pending-execution` — a
fourth value, additive to `current|draft|superseded`, for a `type: guide` roadmap doc that has
been reviewed and approved but not yet executed. Collapsing it to `draft` (implies not yet
reviewed) or `current` (implies the described state already holds) would lose real information
a 3-value enum can't express. Scope this narrowly — a roadmap doc genuinely in that state, not
a general escape hatch for inventing new status values per-doc. Exemplar:
`docs/observability/consolidation-roadmap.md`.

See § Doc-Type
Vocabulary below for what each `type` value means.

## Navigation Layer: MOCs and `index.md`

> Research base for this section and the four below it:
> `recon://legacy/docs-canon-v2-recon-2026-07-22-913141c0b6f3/legacy-docs-canon-v2-recon-2026-07-22-913141c0b6f38ffc` (§ 4.4, § 5-8).

Every directory covered by a project's `[docs.tree]` manifest
(`commands/apply/references/project-toml-schema.md` § `[docs.tree]`) SHOULD carry an
`index.md` — a MOC (map of content): a curated, annotated list of that directory's live pages,
each with a one-line "read this when X" routing note. A MOC is not a directory listing —
`ls docs/reference/` already gives that for free; a MOC's value is the routing annotation a
listing can't carry.

- MOCs nest: a per-directory `index.md` MOC rolls up into a repo-root MOC (`docs/README.md`, or
  a generated atlas page) the same way — same pattern, wider scope.
- MOCs use standard relative markdown links only. Wikilinks are rejected: GitHub doesn't render
  them, and they break for any agent reader that isn't the specific tool that invented the
  convention.
- `start-here/` role-routing (ws's `for-pm`/`for-dev`/`for-data` split — each page routes a
  reader by ROLE rather than by directory) is documented here as the cleanest known exemplar of
  this pattern taken further, but it is deliberately **not** a canon directory role. One
  instance does not justify baking a repo-specific idiosyncrasy into every repo's scaffold — the
  two-instance bar (`rules/TOOLING.md` § Ambient Surfacing) applies to canon roles the same way
  it applies to reusable script patterns. A repo MAY declare a `start-here`-shaped custom
  directory in its own `[docs.tree]` manifest today; the canon generalizes it only after a
  second, independent instance.

## Doc-Type Vocabulary

The `type:` frontmatter field (Frontmatter Contract, above) uses this fixed vocabulary —
extended from the original four to cover the navigation and decision-record surfaces below:

| `type` | Meaning | Typical directory |
| --- | --- | --- |
| `reference` | How-it-IS state, hand-authored | `reference/` |
| `guide` | Step-by-step operational runbook | `guides/` |
| `note` | Dated investigation journal entry | `notes/` |
| `diagram` | Visual explainer (mermaid fence, HTML, image) | `diagrams/` |
| `moc` | Curated, annotated index — see Navigation Layer above | any `index.md`; repo-root MOC |
| `machine` | Pipeline-generated artifact — see Machine-Artifact Convention below | `reference/`, or a manifest-declared generator dir |
| `adr` | Architecture decision record | `docs/adr/` |

One mode per doc (Diátaxis applied as per-page tagging, not directory religion — recon § 4.1):
a page that mixes current-state reference facts with a dated investigation narrative is two
documents sharing one frontmatter block. Split it.

## Dated Self-Correction Banners

When a documented claim is later found to be wrong, do NOT silently edit it away. Add a dated
banner directly above the corrected claim:

```markdown
> **CORRECTION (2026-06-20):** the section below originally claimed X; verified against
> <source> and updated to Y. Left the banner so a reader who cached the old claim isn't
> silently contradicted.
```

This is what makes a `notes/` journal trustworthy over months — a reader can tell a claim was
revisited and when, rather than wondering whether the doc is simply stale.

## Machine-Artifact Convention

Some `reference/` pages are not hand-authored — a script or pipeline writes them directly (e.g.
a generated inventory table, a nightly-refreshed status page). These paths are **load-bearing**:
downstream automation reads them, so:

- Never hand-edit a machine-written page's generated section — edit the script that produces it.
- Mark the generated section clearly (`<!-- GENERATED by scripts/bin/<name> — do not hand-edit -->`)
  so a human editor doesn't accidentally overwrite it on the next manual pass.
- A machine-artifact page's `status`/`updated` frontmatter reflects the LAST GENERATION run, not
  the last hand-edit.

## Generated-Region Markers

Extends the Machine-Artifact Convention above from whole-page generated docs to generated
REGIONS inside a hand-written page (recon § 7):

- **Whole-file generated** (unchanged from above): `<!-- GENERATED by scripts/bin/<name> — do
  not hand-edit -->` at the top.
- **Generated region inside a hand-written page** (terraform-docs pattern — a page mixing prose
  and a generated table): bracket the region with begin/end markers naming the generator:

  ```markdown
  <!-- GENERATED-BEGIN: scripts/bin/<name> -->
  ...generated content...
  <!-- GENERATED-END: scripts/bin/<name> -->
  ```

- **Drift is a build failure, not a review opinion.** Every generator that owns a whole-file or
  regioned surface ships a `--check` GATE mode (non-zero exit on drift, by design — documented
  as such per the `scripts/bin/` quality bar's arg-parse/GATE distinction). A ratchet row
  (`rules/TOOLING.md` § Config Ratchet Lane) consumes the `--check` mode; `hook-inventory
  --check` is the first live instance.
- Regenerate on a trigger (CI, hook, ratchet run) — never "on request only." A one-shot
  generated page is pre-rotted the moment the source it derives from changes.
- Prefer derivable/executable content over asserted prose where possible — a `docs-sweep` probe
  block that runs and matches beats a paragraph that claims the same fact with no check behind
  it (see § Probe blocks below).

## HTML vs Markdown Promotion

Markdown is the default surface — agents and GitHub read it natively, and a `<details>` block
inside markdown covers most of the "progressive disclosure" middle ground before a page needs
full HTML. Promote a page to a self-contained HTML artifact only when at least one of these
three triggers holds (each independently sufficient — recon § 5):

| # | Trigger | In-fleet example |
| --- | --- | --- |
| 1 | A table needs sort/filter/search, or block content inside a cell (markdown tables can't) | ws `diagrams/apim-inventory` |
| 2 | The reader must synthesize across many source files at once (the "seventeen tabs" problem) | ws `cutover-cockpit-spec.html` |
| 3 | Data density needs progressive disclosure — collapse/expand, drill-in | `docs/diagrams/cc-map.html` |

Hard constraints on any promoted HTML page:

- **Generated-artifact only** — a promoted HTML page is derived from markdown/JSON source by a
  committed script; it is never a hand-maintained parallel document. Edit the source, rerun the
  generator, refresh the page.
- Single-file, self-contained (no external asset fetches).
- The source of truth stays markdown/data in the tree; the HTML page is a rendering, not a
  second copy.

A full static-site generator is justified only for a genuinely published surface (ws's Starlight
SWA) — never for in-repo reading.

## Mermaid Tiering

**Supersedes** the prior unconditional "`.mmd` source + always-render" rule (`rules/TOOLING.md`
§ Mermaid Rendering, `cc-reference` skill § Quality Bar's Doc row) — those now apply only to the
non-rendering-surface tier below, not to every diagram (recon § 6).

Tiering, in order of preference:

1. **Inline fence first.** Wherever the reading surface renders mermaid natively (GitHub,
   GitLab, this canon's own consumers), a fenced ` ```mermaid ` block IS the source — subject to
   normal PR review like any other text, no companion file to keep in sync.
2. **Committed `.mmd` + rendered SVG only for non-rendering surfaces** — a published site
   without mermaid support, a README badge, or any consumer that can't execute the fence at read
   time. This is the ONLY tier that still needs the `mmdc` render step (`rules/TOOLING.md` §
   Mermaid Rendering).
3. **External-tool SVG last** — hand-drawn or exported from a non-mermaid tool, only when
   neither of the above fits.

**Six stable types** — the type mapping is settled, use it instead of re-deriving per diagram:
`flowchart` (process/decision), `sequenceDiagram` (integrations/auth handshakes),
`stateDiagram-v2` (lifecycles), `erDiagram` (schemas), `gantt`/`timeline` (phases). C4 and
`architecture-beta` are experimental and unrendered on GitHub — fake C4 with subgraph-nested
flowcharts instead (the documented workaround; see `c4-architecture` skill).

Bar for existence: a diagram only when the information is too complex to follow from text
alone. Every diagram carries `accTitle`/`accDescr`.

## Vendor-Junk Ban

`docs/` (and any subdirectory) MUST NOT contain vendored third-party build output — e.g. a
git-tracked `node_modules/` under a docs-site generator. The ws audit found a 12MB
`docs/site/node_modules` tree (18 vendor READMEs) inflating doc counts and polluting a
dangling-ref sweep with paths nobody authored. If a docs static-site generator needs
`node_modules/`, it belongs in `.gitignore` like any other build artifact — the generator
reinstalls it, it should never be committed.

## No Provenance Narration

A reference/guide page describes current state — not the story of how it came to exist. A
changelog-style dateline ("Extracted DATE (change-id) from REPO — here's why we did this now")
is process history, not "how-it-IS" content: it doesn't help a reader use the page, it just
accretes as dead weight nobody re-verifies. That history already has a durable home —
`git log`/`git blame` (exact commit, date, author, message), the originating beads feature, and,
transiently, harness auto-memory if a future session needs the "why now" context. Do not
duplicate it into the doc body. This canon file used to open with exactly that anti-pattern
(an "Extracted 2026-07-04 (advisor-plans/023) from the `ws` repo's `docs/` tree..." blockquote)
— fixed by cutting it; `git log --follow` on this file recovers the same information.

**Rule**: a page MAY explain *why a convention is shaped this way* when that rationale helps a
reader apply it correctly — that's durable content, not provenance. It MUST NOT open with (or
bury inline) a "created/extracted/introduced on DATE by CHANGE-ID because..." narrative. Test:
would `git log --follow <path>` already answer this question? If yes, it doesn't belong in the
doc — cut it.

## Age Is Not Decay — Broken Refs Are

Per `docs/workflow-metrics.md` § Decay: **age alone is NOT decay.** An old doc whose claims still
verify against the current repo state is healthy — do not treat a page's age as a signal to
rewrite or delete it. Decay is a doc or memory carrying a dangling path/command reference, OR a
claim contradicted by current repo state. A staleness sweep's job is to find THOSE two failure
modes, not to flag every page older than N days.

## Verifiability

For the fleet-hygiene dangling-ref check (and any future staleness classifier) to have a
contract to check against, a doc's claims should be phrased so they're machine-verifiable where
possible: cite an exact path, an exact command, or a count, rather than vague prose ("the script
handles this" vs. "`scripts/bin/foo --json` handles this"). A claim that names a concrete
artifact is a claim a sweep can grep for and confirm still exists.

### Probe blocks — the classifier's contract

The "future staleness classifier" named above is `scripts/bin/docs-sweep` (tier 1, deterministic,
zero tokens). It verifies a doc mechanically: frontmatter present, cited paths exist, and — the
strongest signal — **probe blocks execute with output matching the stated claim**. A doc whose
countable claims all carry passing probe blocks is verified for free and never reaches the haiku
tier.

A probe block is a fenced ` ```bash ` code block whose FIRST line is a `# probe:` comment naming
the expected substring, placed immediately beside the claim it backs:

````markdown
The suite runs 57 checks across 8 categories.

```bash
# probe: 57 checks
scripts/bin/validate-cc --list | grep -c '^check:'
```
````

The sweep runs the block (cwd = repo root, 10s timeout) and flags the doc if stdout does not
contain the `# probe:` string. Rules:

- Only blocks carrying the `# probe:` marker are executed — the sweep never runs arbitrary bash.
- Keep probes cheap and side-effect-free (a `grep -c`, a `jq`, a `--json | ...` pipe). No network,
  no writes, no `pnpm build`.
- Back **countable** claims (a count, an exact path, an exact command's output) — not prose.
- Exemplar: `docs/research/dotenv-vs-dotenvx-eval.md` embeds probe blocks beside its verdict
  claims so the sweep re-confirms them each run.

`register = "stakeholder"` docs (leo-writing-voice prose) rarely carry countable claims and
usually verify on frontmatter + cited-path checks alone — that is fine; probes are for the
reference/notes tier where drift bites.

## Receipts vs State, and the Journal Layer

Every durable-knowledge surface in cc splits into two categories with opposite maintenance
rules (recon § 8) — blending them is the root failure mode this canon exists to prevent:

| Category | Behavior | Examples |
| --- | --- | --- |
| **Receipts** | Append-only, dated, never edited after the fact — trustworthy BECAUSE unedited | `notes/` entries, `docs/adr/` records, closed beads, dated correction banners |
| **State** | Living, corrected in place, aggressively pruned | `reference/` pages, `guides/`, active beads, MEMORY.md's index |

This is the same split the Directory Roles table already encodes per-directory
(`reference/`/`guides/` = state, `notes/` = receipts) — named explicitly here so it generalizes
past `docs/` to beads, memory, and CLAUDE.md/`rules/*.md` (each has a receipts-shaped and a
state-shaped part; conflating them is why wikis rot).

**Journal layer** (net-new convention, definition only — no automation ships with this change):
a gitignored, dated, append-only file for a long autonomous run's verbatim attempts and dead
ends — `journal-*.md`, matched by the `.gitignore` entry of the same name, never committed. It
exists to survive a mid-run compaction: a session that loses its transcript can still read back
its own journal.

Promotion paths out of the journal (a journal entry is disposable by default — promotion is
what makes a fragment durable):

| From | To | Trigger |
| --- | --- | --- |
| journal entry | beads issue | discovered work item (>2 min of scope) |
| journal entry | memory topic file | a repeat signal (second occurrence of the same problem class — Promotion-on-Repeat, `rules/BEADS.md`) |
| journal entry | `docs/adr/` record | a decision worth a durable, citable record |
| journal entry | `notes/` entry | an investigation worth keeping past the session, reviewed before landing |

A journal entry that is never promoted is working scratch, correctly discarded — same posture
as the existing scratchpad rule (`rules/CORE.md` § Artifacts).
