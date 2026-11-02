## Why

Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy). Supporting: P10.

Forensics (60d): session store is 6.2 GB / ~14k files; 5525 of 8427 recent sessions are `is_debug`, and 2581 have <=3 messages. Only `.bak` files are pruned today (`prune_old_session_backups` in `src/cli/startup.rs`). Debug/tiny sessions bloat disk, slow `session_search` index warmup (`crates/jcode-app-core/src/tool/session_search.rs`), and pollute recall results.

## What Changes

- Add a `jcode sessions gc` command that classifies sessions (`crates/jcode-base/src/session.rs` fields `is_debug`, `saved`, `messages`, mtime) into retention candidates and reports them. **Dry-run is the default**; `--apply` deletes.
- Add `[sessions.retention]` config: `enabled` (default `false`), `debug_max_age_days` (7), `tiny_max_messages` (3), `tiny_max_age_days` (14), `auto_gc` (default `false`).
- Deletion requires BOTH `enabled = true` and an explicit `--apply` (or `auto_gc = true` for background startup GC).
- Sessions with `saved = true` are never candidates, regardless of flags. Live sessions held by a running server are never deleted.
- `session_search` excludes debug sessions from the recent index by default (opt in via existing `debug: true` filter); tiny sessions stay indexed.

## Capabilities

### New Capabilities
- `session-store-retention`: classification, dry-run report, opt-in deletion, saved-session protection, and search-index exclusion of debug sessions.

## Impact

- `crates/jcode-base/src/session.rs` (classification + delete), `src/cli/` (new subcommand), config schema, `session_search.rs` / `session_search_index`, `recent_session_index.rs`.
- No data migration. Deletion is destructive; gated by config + flag.

## Open decisions

1. **Delete vs. trash dir.** Recommended: move to `~/.jcode/sessions/.trash/`, purge after 7 days. Alternative: hard delete (frees space immediately, no undo).
2. **Debug exclusion from search by default.** Recommended: exclude (most recent sessions are debug noise). Alternative: keep indexed but down-rank (preserves self-dev recall).
3. **Tiny threshold.** Recommended: <=3 stored messages and older than 14 days. Alternative: count user messages only (avoids deleting short but tool-heavy sessions).
