# Proposal

## Why

AGENTS.md requires UI chrome to never wrap, to count Unicode display cells, and to truncate with an ellipsis. The swarm info widget and swarm page violate this: `truncate_smart` (`crates/jcode-tui/src/tui/info_widget_text.rs:1-2`) measures `chars().count()`, so CJK/emoji names overflow; member rows (`info_widget_swarm_background.rs:34,102-118`) budget `width - 6` for text but prepend a role prefix and status icon outside that budget; and the swarm page hint row (`info_widget_swarm_gallery.rs:196-206`) is hard-clipped by `clamp_line_to_width` (`:426`) with no ellipsis. Count wording also differs between surfaces (`3 agents · 1 active` vs `3s`).

## What Changes

- Make `truncate_smart` measure and cut by display cells (`unicode-width`), keeping its word-boundary and `...` behavior.
- Compute the member-row text budget as `inner.width - width(role_prefix) - width(icon + " ")`, so the full row fits.
- Make `clamp_line_to_width` end with `…` when it drops content, so the hint row and tree rows show truncation.
- Keep compact count wording in the side widget and document it; defer display-order/tree caching (see Resolved decisions).

## Capabilities

### New Capabilities

- `swarm-widget-layout`: Display-cell-correct truncation for swarm widget and swarm page rows.

## Impact

`crates/jcode-tui/src/tui/{info_widget_text.rs,info_widget_swarm_background.rs,info_widget_swarm_gallery.rs}`. `truncate_smart` has 22 call sites; all benefit from cell-correct widths. No protocol or config change.

## Evidence

Checked in `/home/nyaptor/.jcode` on 2026-11-01:

- Confirmed: `truncate_smart` uses `s.chars().count()` and `truncate_chars` (info_widget_text.rs:1-11).
- Confirmed: `max_name_len = inner.width.saturating_sub(6)` (background.rs:34); `swarm_member_line` (:102-118) prepends `swarm_role_prefix` (2 cells, e.g. `★ `) plus `"{icon} "` (2 cells) outside that budget, and truncation is char-based.
- Confirmed: hint row (gallery.rs:196-206) is about 60 cells; every line passes through `clamp_line_to_width` (:426-450), which cuts by cell width but adds no ellipsis.
- Confirmed: gallery header says `{n} agent(s) · {active} active` (:186-191); background title says `{n}s` / `{c}c` (background.rs:127,136).
- Partially confirmed (perf): `render_swarm_page_lines` rebuilds `members_to_gallery`, `display_order` and `swarm_tree_rows` per call (:166-171); `app/tui_state.rs:2421` also calls `members_display_order`. No measurement shows a cost, so caching is deferred (see Resolved decisions).

## Resolved decisions

1. Count wording: keep compact `3s`/`2c` in the narrow side widget and full words (`3 agents · 1 active`) on the swarm page. Documented on `render_swarm_stats_line`.
2. Order/tree caching: deferred until profiling shows a cost (swarms are small). No memoization in this change.
