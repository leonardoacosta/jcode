# Quality Bar Per Deliverable

> Moved from global `CLAUDE.md` § 4 by the CLAUDE.md-split pattern (`rules/TOOLING.md` §
> CLAUDE.md-Split Pattern) — per-deliverable-type checklists, only relevant when actively
> finalizing that specific artifact type. `CLAUDE.md` keeps a one-line pointer.

A deliverable is done when every box in its row checks. If you cannot check a box, the work
is not done — say so.

**Code change (any repo)**
- [ ] Design step recorded before first edit (multi-file)
- [ ] Reuse search cited (what was searched, why nothing fit)
- [ ] Gates pass with pasted output (typecheck/lint/test per project.toml)
- [ ] Diff contains only the requested change — no drive-by renames/formatting
- [ ] Runtime evidence pasted for the user-facing assertion

**`scripts/bin/` script**
- [ ] `--json` mode emits a single JSON object; runtime failure = exit 0 + `error` key
- [ ] Arg-parse misuse may exit 1; GATE scripts (non-zero by design) documented as such
- [ ] <200ms warm, no network, no info-level stderr
- [ ] Mutations via `.tmp` + `os.replace`; fan-out scripts use the 0/1/2 exit contract
- [ ] `git add -f` executed; verified by failure-mode execution, not lexical grep

**`scripts/hooks/` hook**
- [ ] Specific matcher (never `""` on PostToolUse/Notification/PostToolUseFailure)
- [ ] `# requires-settings:` header for any settings key it depends on
- [ ] `# liveness:` header (or `HOOK_LIVENESS_INLINE` entry) if critical
- [ ] Per-turn payloads dedup via marker file (see `skill-list-dedup.sh`)
- [ ] Proven fired once for real (`cc-runtime-evidence` skill), not just wired

**Skill (`skills/<name>/`)**
- [ ] Description class chosen: auto-triggered (keywords, `Triggers:` list) or explicit-only (<=200 chars) — budget is a blocking ratchet
- [ ] Body leads with a decision/routing table; NEVER section states WHY per row
- [ ] Long material split to `references/` and linked from SKILL.md (no orphan refs)
- [ ] Would clear skill-judge B floor (99/120); wired into a routing surface (PATTERNS.md row, agent frontmatter, or command `Skill()` call)

**Command (`commands/**.md`)**
- [ ] `model:` pin in frontmatter (opus = orchestrates agents; haiku = trivial fetch; else sonnet)
- [ ] Render-time decisions consume ` ```! ` script JSON, never re-implement detection in prose
- [ ] Heavy bash/tables extracted to `references/` with `disable-model-invocation: true`

**Agent (`agents/**.md`)**
- [ ] Frontmatter: `name`, `description`, `model` pin, `skills:` including `verification-before-completion`, `allowed-tools`
- [ ] Suffix from the sanctioned vocabulary (`-analyst/-architect/-engineer/-reviewer/-specialist`)
- [ ] Write-capable => `## Worktree Contract` section + covered by the completion-verification matcher
- [ ] Write-capable => prompt body states the read-before-edit sentence ("Read any file in full before your first Edit to it; the tool rejects blind edits — a rejected edit is a wasted turn.")

**Doc**
- [ ] Lives under `docs/<role>/` (never root); screenshots never in-repo (homelab `~/screenshots/`, see `rules/CORE.md` § Screenshots)
- [ ] Diagrams: tiering per `operational-docs-canon.md` § Mermaid Tiering — inline fence by default; `.mmd` source committed beside rendered output only for non-rendering surfaces
- [ ] Claims verifiable — probe blocks or exact paths, not vibes

**OpenSpec spec / bead / commit**
- [ ] Spec: `openspec validate --strict` passes; literal batch headers; `[user]` tasks carry `searched:`; `## Testing` present; beads-synced
- [ ] Bead: level matches artifact (epic=capability, feature=proposal, task=checkbox); priority set on features
- [ ] Commit: `type(scope): subject`, message via `-F` file, targeted adds, pushed (or explicitly held with a reason)
