# Worktrees — RETIRED 2026-07-17 (tombstone)

> **This reference is retired.** It described the per-session git-worktree `/apply` model —
> `.worktrees/<session-id>/` provisioning, `EnterWorktree` entry, merge-back, the `wt` CLI as
> the primary `/apply` mechanism, worktree memory cleanup, and the beads skip-worktree git-index
> flags. That model was **retired 2026-07-17** by the `retire-session-worktrees` proposal in
> favor of **shared-tree execution + a single-flight per-repo apply lock**.

## Where the current mechanism lives

`/apply` and `/apply:all` now run directly in the main checkout on the current branch — no
worktree, no session branch, no merge-back. Concurrent runs are serialized by a single-flight
per-repo apply lock (`apply_lock_acquire` / `apply_lock_release`, `scripts/lib/merge-slot-helpers.sh`).

- **Current apply-lock mechanism:** `commands/apply.md` § Phase 0a: Apply Lock (and
  `commands/apply/all.md` § Phase 0a: Single-Flight Apply Lock).
- **Shared-tree model + anti-patterns:** `CLAUDE.md` § How Work Ships, `rules/CORE.md`
  § Concurrent-Session Note, and the `cc-reference` skill body (§ Session Worktrees, Workflow
  Anti-Patterns).
- **bd sync-equivalent commands** (`bd import` / `bd dolt status` / `bd export`), formerly filed
  here, now live in the `cc-reference` skill body § bd Sync-Equivalent Commands.

## Drain period (historical `wt` reference)

The `wt` CLI, `scripts/lib/worktree-helpers.sh`, the nightly `wt-reap.timer`, and
`prune-project-memory` remain operational **only to drain** `.worktrees/*` dirs minted before the
cutover — not for normal `/apply`. If you need the historical worktree-lifecycle reference
(`wt` subcommands, memory-cleanup policy, skip-worktree git-index flags) during the drain window,
see `openspec/changes/retire-session-worktrees/design.md` for the drain-vs-delete tooling status.

## Drain-Window Blocks (demoted from rules/CORE.md, 2026-07-25)

> Transitional by their own text — worktrees were retired 2026-07-17 by `retire-session-worktrees`.
> Live only while a repo still has `.worktrees/*` on disk. Demoted by `prune-core-stale-and-rescope-narrow`.

  **Drain-window caveat (transitional)**: `.worktrees/*` directories minted before this change
  still exist on disk and need draining — verified-merged ones force-reaped, unmerged ones listed
  for manual review. This is a SEPARATE, one-time concern, no longer part of normal `/apply`
  operation. `scripts/lib/worktree-helpers.sh`, `scripts/bin/wt` (`wt list`/`current`/`path`/
  `status`/`reap`/`check-symlinks`), `wt-reap-fleet`, and the nightly `wt-reap.timer` stay
  operational until the fleet drain completes.

  **Drain-window-only footgun (nv-nhm2j)**: while any repo still has active worktree-based `/apply`
  sessions or leftover `.worktrees/*`, running `pnpm install` from the main checkout can race a
  worktree's own install, leaving EITHER side's `node_modules/@scope/*` symlinks pointing into the
  other tree (bidirectional; an install-timing race, not a `pnpm-workspace.yaml` glob bug). Symptom:
  a rebuild/typecheck fails with `Cannot find module <workspace-pkg>` and an import trace resolves
  through `.worktrees/<id>/`. This can no longer occur once no repo has worktree-based sessions and
  the drain is complete — it is a live risk only during the drain period for any repo with
  `.worktrees/*` still present. `wt check-symlinks` (`--json` for scripting) scans `node_modules`
  for any symlink resolving into `.worktrees/`; fix is a fresh `pnpm install --frozen-lockfile`
  (`CI=true` to skip the interactive prompt) from the affected checkout's own root.
