# Beads Ops Reference

Demoted from `rules/BEADS.md` by `slim-beads-rules-to-reference` (2026-07-25). These sections are
consulted when you are actually touching `.beads/*` git state, the feature-bead approval lifecycle,
or `/feature`'s order codes — not on every turn. `rules/BEADS.md` keeps a one-line pointer to each.

Load via `Skill({skill:"cc-reference"})` when a task names any of the sections below. The full
incident narratives live in `docs/notes/beads-jsonl-and-locking-history.md`; this file carries the
operative contracts.

## Sanctioned Bead Title Prefixes

> Demoted from `rules/BEADS.md § Bead Hygiene Standard` rule 2. The rule ("only sanctioned
> prefixes; everything else banned") stays resident — this is the lookup table behind it.
> Machine source of truth: `scripts/config/bead-hygiene.json` `sanctioned_prefixes`. Adding a
> prefix requires editing BOTH that array AND this table (`spec-sync-mint-dedup`).

| Prefix | Source |
| --- | --- |
| `[CAPABILITY]`, `[PROPOSAL]` | spec-sync structural |
| `[audit:P0..P3\|GCF]` | audit pipeline |
| `[ratchet]`, `[docs-sweep]` | nightly lanes |
| `[PROPAGATE]`, `[TARGET]` | cross-repo propagation |
| `[user]` | escalation — **HITL** (human-in-the-loop): resolvable only by Leo's own action/decision; agents prepare, surface, and block on it, never resolve/self-answer/checkbox it. Untagged tasks/beads default to AFK. Full doctrine: `docs/adr/0001-user-tag-is-hitl.md` |
| `[BLOCKER]`, `[MERGED INTO <id>]` | escalation / drift end-state |
| `[MMDDx]` | `/feature` order-code allocation (mirrors proposal.md `order:` frontmatter) |

## Session-Closer Sync Steps

> Demoted from `rules/BEADS.md § Session Close Protocol`. The ad-hoc commit sequence and the
> "never enable export.auto" warning stay resident; these are the Stop hook's internals.

`scripts/bin/session-closer` runs three steps:

1. **`bd dolt commit`, unconditional.** `dolt.auto-commit: batch` holds writes in the working set
   and `bd dolt push` moves only existing commits. Kept OUTSIDE the push rate-limit: gating it
   strands writes and leaves the HEAD stale enough to false-skip step 3.
2. **`bd dolt push`** — rate-limited to once/10min (network cost, not correctness).
3. **JSONL export** — gated on the Dolt HEAD commit, not wall-clock (unchanged DB: ~1.8s, no
   rewrite). Temp cleaned on every exit path, plus a `-mmin +60` sweep for SIGKILL residue.

Pre-2026-07-08, step 3 called the nonexistent `bd sync --flush-only` — see § JSONL Git-Merge
Conflicts below and `docs/notes/beads-jsonl-and-locking-history.md`.

## Funnel Convergence — provenance and the fourth-ledger ban

> Demoted from `rules/BEADS.md § Funnel Convergence`. The three-source rule and the
> advisory-not-executable distinction stay resident; this is the rationale behind them.

**advisor-plans/027 (2026-07-04), Leo direction:** ideas enter the funnel from three distinct
sources — product intent (`/plan:*`), a human idea (`/explore` → `/feature`), or machine-detected
debt (`improve:*` lenses). All three MUST converge on the openspec + beads workflow before
anything executes. `docs/plan/<name>/` artifacts (`scope-lock.md`, `prd.md`, `roadmap.md`) and
`plans/` (the `improve:*` advisory output) are **advisory context that every funnel stage
consults** — never a second execution ledger running in parallel to `openspec/changes/`.

Concretely: a plan-pipeline roadmap doesn't get "executed" directly — it generates
`openspec/changes/<slug>/` proposals via `/feature`, and THOSE are what `/apply`/`/apply:all`
run. An `improve:*` lens doesn't patch code directly either — its confirmed findings route
through `/audit:waves`'s singleton machinery (finding → `/feature --quick` singleton →
conflict-aware wave plan), landing as `openspec/changes/<slug>/` proposals directly, same as any
other funnel stage. `plans/` is only a bare-repo fallback (a repo with no `openspec/` directory)
and a historical ledger — never a second execution path for a repo that has `openspec/`. If a
funnel stage is found executing without landing in this workflow, that's the defect to fix
(`openspec-funnel-health` Req-3 covers `/bootstrap:roadmap`'s convergence) — not a reason to add
a fourth ledger.

## Capability Epic Drift — the duplicate-epic walkthrough

> Demoted from `rules/BEADS.md § Capability Epics MUST NOT Be Closed`. The Allowed/Banned lists
> and the recurrence guard stay resident; this is the failure sequence they prevent.

1. Premature close of `cc-XXX [CAPABILITY] foo`
2. Next `beads:spec-sync` invocation searches open epics by title, finds nothing
3. A new epic `cc-YYY [CAPABILITY] foo` is created
4. Children scatter across both → `bd epic status` lies, `bd ready` priority bubble-up breaks

`scripts/bin/spec-sync` revives automatically (`find_or_create_epic` searches `--status all` and
reopens closed/deferred matches). The drift check emits
`{"closed_capability_epics":N,"dup_titled_capabilities":M,"offenders":[...]}` — both counts MUST
be `0`; `[MERGED INTO …]` rows are excluded as the resolved end-state.

## Approval Signal: Feature-Bead Lifecycle

> **As of 2026-07-16** (`retire-status-frontmatter-lifecycle`): the `status: draft` →
> `status: approved` proposal-frontmatter convention documented below through 2026-07-16 is
> **retired**. It created an unbounded, invisible parking state (`draft` never forced a
> decision) and the funnel piled up unreviewed. The approval signal now lives entirely on
> beads — this section describes the replacement. `order:`/`after:` proposal frontmatter are
> **unaffected** by this retirement; see § "Order Codes: Same-Day Proposal Sequencing" below.

### The model

- **Presence in `openspec/changes/` (non-archived) = actionable.** There is no author-facing
  "not yet reviewed" state distinct from "exists" — a proposal is a candidate the moment
  `/feature` creates it.
- **Cleared-to-work = the proposal's FEATURE BEAD is `in_progress`.** The feature bead already
  exists for every synced proposal (`<!-- beads:feature:cc-xxxx -->` marker in its own
  `tasks.md`, written by `scripts/bin/spec-sync`). `in_progress` is standard bd status
  vocabulary — no new field, label, or status value was invented.
- **`/triage`'s approve action is the canonical transition.** On approve, it resolves the
  proposal's feature bead from its own tasks.md marker and runs
  `bd update <feature-id> --status in_progress` — no proposal.md write. "Keep as draft" is a
  pure no-op (no write at all). A proposal with no feature-bead marker (sync never ran) renders
  as `"unsynced — run spec-sync"` and cannot be approved through `/triage` until synced.
- **Consumers read the bead, not frontmatter:**
  - `scripts/bin/triage-list-drafts` (interactive readers: `/triage`, `openspec-status --queue`)
    — a candidate is any non-archived proposal whose feature bead is NOT `in_progress`/`closed`.
    Degrades **permissively** when bd is unavailable or the marker is missing (includes the
    proposal anyway, flagged `bead_unknown: true` — never a silent drop).
  - `scripts/bin/wave-extend-scan` (`/apply:all`'s autonomous wave-extension gate) — admits a
    candidate iff its feature bead is `in_progress` AND the spec dir is live. Degrades
    **fail-closed** when bd is unavailable (zero candidates + a WARN) — the opposite posture
    from the interactive readers, because an unattended run admitting everything would recreate
    the exact unreviewed-pickup this gate exists to prevent.

### Funnel-pressure warning (replaces the old parking-lot signal)

Retiring `draft` as an implicit park state removes the old "I'll come back to this" escape
hatch, so `/feature` (Step 2.3) surfaces pressure explicitly instead: when more than **5**
non-archived proposals exist, it renders a loud table (slug, order, age, staleness, epic) plus
overlap detection between the new feature's planned `- touches:` and every open proposal's.
**Staleness threshold: 14 days** (`scripts/bin/openspec-status`'s `stale`/`age_days` fields,
the single computation both `/feature` and `session-primer`'s OpenSpec funnel-health line
consume). This is **friction, never a cap** — no proposal is ever auto-parked, auto-deleted, or
blocked from being authored; the operator explicitly rejected a WIP cap or TTL auto-park in
favor of visibility.

### Backwards compatibility

Proposals from before this retirement may still carry `status:`/`approved-by`/`approved-at`
frontmatter keys if they were never swept — every reader above treats their absence as the
normal (and now sole) candidate state, so a stray leftover key is inert, not a required field to
strip retroactively on sight. `openspec validate <slug> --strict --no-interactive` ignores YAML
frontmatter entirely regardless.

### Cross-references

- `openspec/changes/archive/2026-07-16-retire-status-frontmatter-lifecycle/` — the archived change that
  landed this retirement; its `design.md` documents the full decision record and rejected
  alternatives (a triage-queue state file, WIP caps, TTL auto-park).
- `openspec/changes/archive/2026-04-27-add-autonomous-wave-extension/` — the original consumer motivating
  the frontmatter convention (now superseded by the bead gate above).
- `commands/triage.md` — the `/triage` command implementing the approve transition.

## Order Codes: Same-Day Proposal Sequencing

Every proposal `/feature` creates is auto-stamped with an `order:` frontmatter field —
`MMDD` plus a lowercase letter, e.g. `order: 0715a`. This solves a narrow problem: `ls
openspec/changes/` gives no clue which of two proposals authored the same day should be worked
first, and kebab-case slugs are too long to use as a spoken/typed handle.

**Additive metadata only — never identity.** `order:` does NOT rename the proposal's directory,
does NOT change what `/apply <spec-name>` accepts, and is NOT consumed by `- depends on:` or
`- touches:` parsing. Those all keep working exactly as documented above, unmodified. `order:`
is a display label surfaced by `/triage` and `openspec-status --queue`/`--json` — it never
feeds `/triage`'s actual ranking (still topological → priority → `after:` hint → age).

**Allocation is automatic.** `/feature` calls `scripts/bin/triage-list-drafts
--next-order-code --json` at spec-creation time (Step 3.1, before the `openspec validate`
gate), which scans non-archived `openspec/changes/*/proposal.md` frontmatter for today's
already-issued codes and returns the next free one — `a`, then `b`, then `c`, overflowing to
`aa` past 26 same-day proposals (spreadsheet-column style). Authors never hand-write this
field.

**Archived proposals don't reserve their code.** The allocator scans the same non-archived set
`triage-list-drafts` already excludes `archive/` from — a code issued earlier today and since
archived is free for reuse. This keeps the allocator a cheap scan, not a permanent ledger.

**Pre-convention proposals are unaffected.** A proposal authored before this convention existed
simply has no `order:` field; `/triage` and `openspec-status` render it with an empty/null order
column and rank it exactly as before.

Spec: `openspec/changes/archive/2026-07-15-add-plan-promotion-and-order-codes/specs/openspec-frontmatter-conventions/spec.md`.


## JSONL Git-Merge Conflicts: Regenerate From Dolt, Don't Text-Merge

`bd hooks install` wires `.gitattributes` (`*.jsonl merge=beads`) to a git merge driver of `bd
merge %A %O %A %B` — but `bd merge` does not exist as a subcommand (confirmed against both bd
1.0.3 and 1.1.0; `bd merge-slot` exists but is an unrelated exclusive-access primitive, not a
JSONL merge tool). Any real conflict in `.beads/issues.jsonl` makes git invoke a nonexistent
command and fail outright. Even a working `bd merge` would solve the wrong problem —
`issues.jsonl` is a generated **export** of the real Dolt database, not hand-edited source; the
correct reconciliation of concurrent issue changes lives at the Dolt layer (`bd dolt push`/`bd
dolt pull`).

> Full incident narrative (discovery, verification, the related `bd sync` finding) in
> `docs/notes/beads-jsonl-and-locking-history.md` — per `documentation-writer`'s
> operational-docs-canon § No Provenance Narration, git log/beads already retain "who found this
> and when."

**Fix**: the merge driver (reference implementation: `tc`'s `scripts/hooks/beads-jsonl-merge-driver.sh`)
does NOT attempt a content merge — it runs `bd dolt pull` then `bd export -o "$2"` to regenerate a
correct export, making the git-level "merge" a passthrough to the authoritative source. Exit 1 on
`bd dolt pull` failure (leave as a real conflict) rather than guessing.

**Re-apply after every `bd hooks install`**: the driver *command* lives in `.git/config`'s `[merge
"beads"]` block, which is NOT versioned — any reinstall resets it to the broken `bd merge ...`
form:
```bash
git config merge.beads.driver 'scripts/hooks/<driver-script> %O %A %B'
```

**A driver command with no `.gitattributes` target is dead config** — cc had exactly that until
2026-07-25 (driver set, `.gitattributes` 1 byte, `check-attr` reporting `unspecified`). Both
halves are required. Scoped to the audit log only; `issues.jsonl` is gitignored and cannot
conflict. Verify with `git check-attr merge -- .beads/interactions.jsonl` (expect `merge: beads`):
```
.beads/interactions.jsonl merge=beads
```

**`bd sync` also does not exist** (bd 1.0.3 and 1.1.0 both: `Error: unknown command "sync" for
"bd"`) — `scripts/bin/session-closer`'s Stop hook called `bd sync --flush-only` with `2>/dev/null
|| true`, silently no-op'ing its flush step fleet-wide until fixed. Real sync-equivalents: `bd
import` (after `git pull`), `bd dolt status` (connectivity check), `bd export` (forced flush —
what `session-closer` now calls). Detail: `cc-reference` skill § bd Sync-Equivalents.

## JSONL Non-Deterministic Export Order: Sort Before Commit

`bd export` (1.0.3/1.0.4) wrote `.beads/issues.jsonl` rows in non-deterministic order between
runs, so every flush rewrote the entire multi-MB file even with zero real content change
(`sort | md5sum` byte-identical across runs — only line order differed). Neither bd version
advertises a `--sort`/determinism flag.

> Full incident narrative in `docs/notes/beads-jsonl-and-locking-history.md`.

**Current state**: `.beads/issues.jsonl` is gitignored/untracked from git as of
`untrack-runtime-state` (2026-07-12) — Dolt is the real sync mechanism, so this is now moot for
git commits specifically. `scripts/hooks/beads-jsonl-sort.sh` (`LC_ALL=C sort`, re-stages only on
real diff) stays installed defensively in `.beads/hooks/pre-commit`, appended AFTER the `# --- END
BEADS INTEGRATION ---` marker (never substring-insert into bd's managed block).

**Footgun**: this repo's `core.hooksPath` is `.beads/hooks` (`bd hooks install --beads`), NOT
`.git/hooks/` — editing `.git/hooks/pre-commit` directly is a silent no-op here. Check `git config
--get core.hooksPath` before editing any git hook in a beads-managed repo; re-apply the sort
script's post-marker block after any `bd hooks install --beads --force`.

## Which `.beads/*` Files Are Git-Tracked (Verified Against Upstream Docs)

Not every `.beads/*` file follows the `issues.jsonl` untrack decision above — confirmed against
`bd`'s own upstream docs (`gastownhall/beads` `docs/reference/git-integration.md` +
`docs/reference/configuration.md`, re-verified 2026-07-25 against bd 1.1.0; these were the
top-level `GIT_INTEGRATION` / `CONFIG` pages when first cited on 2026-07-14 and have since moved
under `docs/reference/` — conclusions below unchanged):

| File | Tracked in git? | Why |
| --- | --- | --- |
| `.beads/issues.jsonl` | **No** (gitignored) | Upstream treats JSONL exports as "optional... for interchange and migration," not the source of truth — `dolt push`/`pull` is. |
| `.beads/interactions.jsonl` | **Yes** | Upstream `docs/cli-reference/audit.md` (re-verified 2026-07-25): entries are append-only and the file "is intended to be versioned in git" for auditing and dataset generation. Intentional upstream design — do not untrack by analogy with `issues.jsonl`. It carries a `merge=beads` attribute in `.gitattributes` (the only JSONL that does) since it is the one tracked, multi-machine, append-only file that can actually conflict. |
| `.beads/last-touched` | **No** (untracked 2026-07-14) | Never mentioned in any upstream doc; bd's stock `.gitignore` lists it under "Runtime files." This repo had force-tracked it with zero live readers of its *value* — restored to bd's default. |
| `.beads/config.yaml`, `.beads/metadata.json` | Yes | Upstream: "both... tracked by Git" (project config + backend metadata, not runtime state). |
| `.beads/dolt/`, `.beads/embeddeddolt/`, `.beads/backup/`, `*.db*`, `daemon.*`, lock files | No | Upstream: explicitly gitignored — the Dolt database itself, never git-portable. |

**Rule of thumb**: before adding/removing a `.beads/*` path from `TRACKED_IGNORED_EXEMPT`, check
bd's own `.beads/.gitignore` template and upstream docs first — presence there is the tiebreaker,
not "it's already tracked." A force-tracked file with zero readers of its *value* is a strong
signal it should never have been force-tracked.

## No Standalone `interactions.jsonl` Flush Commits

`.beads/interactions.jsonl` is git-tracked (see table above) and goes dirty on ordinary `bd`
writes throughout a session — it should not drift away from the real work it records. **Rule**
(decided 2026-07-16, amended 2026-07-30 by the plugin-pack follow-up): when it shows dirty at
commit time, add it to the SAME `git add <files>` as whatever real work is being committed —
never a standalone `chore(beads): flush/sync interactions log` commit. If the related local
commit already landed without the file, amend that still-unpushed commit before push so the audit
diff ships with the code/spec/docs change it belongs to. If a session genuinely has nothing else
to commit, a beads-only commit is legitimate but MUST be titled for the actual content (e.g.
`chore(beads): close <feature> — <n> tasks`), never the generic flush message.

**File-growth verdict** (2026-07-20, decided by Leo): accept-and-document, no
rotation/compaction/untracking — `bd` offers no audit-log compaction primitive, and unlike
`issues.jsonl`, upstream explicitly wants `interactions.jsonl` versioned in git for auditability.

> Full decision rationale (rejected alternatives) in `docs/notes/beads-jsonl-and-locking-history.md`.


## Single-Flight Apply Lock (/apply workflow)

`/apply` and `/apply:all` run **directly in the main checkout on the current branch** — no
per-session worktree, no session branch, no Phase 4 merge-back (retired 2026-07-17 by
`retire-session-worktrees`). Concurrent runs are serialized by a **single-flight per-repo apply
lock**: Phase 0a calls `apply_lock_acquire <repo_root> <spec_csv> [session_id]`
(`scripts/lib/merge-slot-helpers.sh`); a second `/apply` in the same repo prints the holder
(session / specs / age) and STOPs.

**Liveness is a TTL lease, not a PID check** — `holder.json` carries `expires_at` (now +
`_APPLY_LOCK_TTL_SECONDS`, default 1200s); a holder is live iff `now < expires_at`. (An earlier PID-based
revision gave effectively zero protection: each Bash tool call is a fresh subprocess that exits
within seconds, so `kill -0 $holder_pid` read as dead almost immediately — see
`docs/notes/beads-jsonl-and-locking-history.md` for the incident.) `apply_lock_renew <repo_root>
[session_id]` extends the lease at each phase/wave-boundary checkpoint; `apply_lock_release` is
the sole normal-path release, called explicitly at run completion (deliberately no `trap ... EXIT`
— a trap inside one Bash-tool-call subprocess only protects that call). `apply_lock_check_spec_active`
(the `/feature`/`/triage` write-fence) carries the same expiry check, so a crashed `/apply`
session's lease doesn't block legitimate writes indefinitely. Full contract:
`commands/apply.md` § Phase 0a: Apply Lock.


## Hierarchy — detection, landing-pad, dormancy, backcompat

> Demoted from `rules/BEADS.md § Hierarchy`. The 3-level table and the
> capability-epics-never-close rule stay resident; these subsections are consulted when
> spec-sync resolves a capability or when triaging a dormant epic.

#### Dormancy: defer, don't hoard open

A capability epic with zero open children SHOULD be marked `deferred`, not left open —
measured 2026-07-13: 81% of cc's open epics (46/57) and 95% of tc's (52/55) were zero-child
shells polluting `bd epic status`, while oo carried 59 duplicate-titled OPEN epics (the
never-close rule alone never prevented that drift class). `deferred` epics no longer clutter
listings, and `scripts/bin/spec-sync` auto-revives deferred/closed epics when a new proposal
lands under them (advisor-plans/036). Close remains decommission-only. Retroactive defer
sweep: advisor-plans/039.

### Capability detection (first match wins)

`beads:spec-sync` resolves a proposal to its capability in this order:

1. `openspec/changes/<slug>/specs/<capability>/spec.md` — the delta directory IS the capability (primary case).
2. Proposal front-matter `capability:` field (if present).
3. First `## ADDED|MODIFIED Requirements` directive under a capability name in the proposal.
4. Fall back to `unsorted`.

### Landing-pad epic

Each project has one `<prefix>-unsorted-epic` as a home for features with no matched capability.
It is auto-created by `beads:spec-sync` on first use. Features under `unsorted` can be reparented
later:

```bash
bd dep remove <feature> <unsorted-epic>
bd dep add <feature> <real-epic> --type parent-child
```

### Backward compatibility

Pre-change 2-level epics (proposal-as-epic) continue to function without forced migration. New
work MUST use the 3-level model; old work MAY remain flat until an opt-in Phase B backfill runs.
`/apply` archive logic handles both shapes: closes the feature if present, else closes the epic
directly.
