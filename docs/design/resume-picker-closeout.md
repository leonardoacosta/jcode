# Resume Picker: Implementation Closeout

Date: 2026-09-22
Source: `plans/2026-09-21-resume-picker-exploration.md`
Commits: `02d618ada` → `dc235d289` (6 commits)

## Changes delivered

```diff
crates/jcode-tui/src/tui/session_picker/
  render.rs    (+418/-384)   compact rows · truncation · excerpt fallback · badges
  filter.rs    (+355)        sort model · cmp_sessions · relevance sort on search
  navigation.rs (+24)        dynamic page-step (visible-area-derived)
  loading.rs   (+1/-1)       first_user_prompt extracted for display_role messages
crates/jcode-tui-session-picker/src/lib.rs (+51)
                             SessionSortMode enum
```

### Compact 3-line rows

```text
○ 🦆 Factory durability research              5m
     Resume workflows without launching duplicate attempts
     jcode · 📌 saved · ▸ here · ready
```

Line 1: selection marker · icon · title · activity age
Line 2: first-user-prompt excerpt (or "N user · ~K tok" fallback)
Line 3: project dirname · source badge · saved · here · current · status

Removed from rows: message counts, token estimates, created time, "prompt:" prefix, blank spacer, crash reason detail.

### Sort model

| Key | Action |
|-----|--------|
| `o` | Cycle sort: recency → title → saved-first → creation |
| `O` | Reverse direction |
| `/` | Search triggers relevance sort (title=3pts, dir=2pts, body=1pt) |

Title bar shows `↓ title (o sort · O reverse)` when non-default.

### Adaptive pane split

| Terminal width | List | Preview |
|---------------|------|---------|
| ≥120 cols | 55% | 45% |
| 100-119 | 50% | 50% |
| 80-99 | 45% | 55% |
| <80 | 60% | 40% |

### Search

- Match count shown in search bar: `🔍 query  15/261`
- Relevance sort on search (tiered scoring)
- Highlighting preserved in titles and excerpts

### Navigation

- PgUp/PgDn covers a full screen of sessions (was always 3 items)
- Step = `ceil(list_visible_rows / 3)`, clamped to `[3, list_visible_rows]`

### Correctness

- All truncation uses `unicode_width::UnicodeWidthStr` (CJK, emoji, combining chars)
- `first_user_prompt` extracted for messages with `display_role` set (1-line fix in loading.rs)
- Canary/debug markers restored in compact rows

## Files changed

```
crates/jcode-tui-session-picker/src/lib.rs         |  51 +++
crates/jcode-tui/src/tui/session_picker/filter.rs  | 355 ++++++++++-------
crates/jcode-tui/src/tui/session_picker/loading.rs |   2 +-
crates/jcode-tui/src/tui/session_picker/navigation.rs | 24 +-
crates/jcode-tui/src/tui/session_picker/render.rs  | 418 +++++++++------------
5 files, +466/-384
```

## Open items

| Item | Why deferred |
|------|-------------|
| Snapshot tests at 80/100/120/160 cols | Needs test fixtures with real terminal dimensions |
| Narrow-terminal preview toggle (<80 cols) | Needs new interaction mode (keybinding + state) |
| Search corpus coverage indicator | Needs scan-limit plumbing from loading.rs |
| Frecency scoring (frequency + recency) | Needs visit-counting infrastructure |
| Persisted sort preference | Needs config schema change |
| Composable filters (checkboxes vs cycle) | Needs filter UI redesign |
| Deep-search action for full transcripts | Needs async search pipeline |