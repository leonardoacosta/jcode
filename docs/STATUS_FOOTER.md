# Status Footer

A persistent single-row status bar in the TUI chrome stack (roadmap P2,
Zentui port). It answers three questions at a glance: **where am I**
(directory, git, execution mode), **what am I running** (model, provider,
effort), and **what is it costing me** (context, tokens, cost).

The footer is the last row of the chat-column layout: at the physical
bottom of the screen whenever the transcript fills the viewport, and
directly below the input row in the packed top-anchored layout of a fresh
session, exactly like the status line above it. It is on by default.
Setting `display.footer.style = "off"` removes the reserved row entirely
and restores the exact pre-footer layout.

## Layout

```text
~/dev/jcode (local) main !2 ↑3        fable · anthropic · high · 40%/200k · 12k/45k tok · $1.23
└──────────── left zone ───────────┘  └──────────────── right zone ─────────────────────┘
```

The left zone is context (where am I), the right zone is the session
metadata stack (what am I running / costing). The row never wraps to a
second line; segments degrade instead (see below).

## Segments

Left zone, separated by single spaces:

| Segment | Format | Source | Color role |
| --- | --- | --- | --- |
| Directory | `~/dev/jcode` (home-collapsed) | session working dir | `info` |
| Mode | `(local)` / `(remote)` | runtime mode | `dim` |
| Branch | `main` | git cache (TTL-refreshed) | `accent`, bold |
| Dirty | `!2 +1 ?3` (modified/staged/untracked) | git cache | `warning` |
| Ahead/behind | `↑3 ↓1` | git cache | `dim` |
| Session name | `fox` (off by default) | explicit session name | `dim` |

Right zone, separated by ` · ` (or ` | ` in ASCII mode):

| Segment | Format | Source | Color role |
| --- | --- | --- | --- |
| Model | shortened (`fable`) | active model | `accent` |
| Provider | `anthropic` (+ `/upstream` when present) | provider registry | `dim` |
| Effort | `high` | reasoning effort | `dim` |
| Context | `40%/200k`; `~40%` when stale | observed tokens, else estimate | `dim`, `warning` ≥ 70%, `error` ≥ 90% |
| Tokens | `12k/45k tok` (in/out) | session usage | `dim` |
| Cost | `$1.23` | session cost | `success` |

Segments with no data are omitted entirely (no placeholder dashes, no
double separators): not a git repo, zero cost, zero tokens, no explicit
session name, or no context estimate all remove their segment. Cost below
$0.005 is treated as zero.

All colors resolve through the existing `[display.colors]` map (see
`TUI_COLOR_CONFIGURATION.md`): set the role key to recolor every footer
segment using it; unset roles fall back to the built-in theme tokens. The
footer adds no new color keys.

## Degradation

When the composed row exceeds the terminal width, segments are shed in a
fixed order before any label is truncated:

1. Session name
2. Cost
3. Tokens
4. Effort
5. Provider / upstream extras
6. Directory depth (full → configured depth → basename)
7. Git ahead/behind counts
8. Branch truncation (`long-branch…`, ≥ 10 chars)
9. Context limit suffix (`40%/200k` → `40%`)
10. Directory truncation (≥ 8 chars)
11. Model truncation (≥ 8 chars)

If the row still overflows, the right zone and then the left zone are
ellipsis-truncated as a last resort. The one-row guarantee holds at every
width.

## Configuration

All keys live under `[display.footer]` in `~/.jcode/config.toml` and parse
with these defaults when omitted:

```toml
[display.footer]
style = "segments"      # "segments" | "off"
icon_mode = "auto"      # "auto" | "ascii"
path_display = "basename" # "basename" | "depth" | "full"
path_depth = 2          # components kept when path_display = "depth"
context_warning = 70    # percent, warning color at/above
context_error = 90      # percent, error color at/above

[display.footer.segments]
cwd = true
mode = true
git = true
session_name = false
model = true
provider = true
effort = true
context = true
tokens = true
cost = true
```

- `style = "off"` reserves zero rows; the layout is byte-identical to the
  pre-footer layout. This is the rollback path.
- `icon_mode = "ascii"` replaces `↑/↓` with `^/v` and `·` with `|` for
  terminals with broken glyph widths. `auto` uses the Unicode glyphs.
- `path_display = "depth"` shows the last `path_depth` components with an
  ellipsis marker (`…/dev/jcode`); `"full"` shows the home-collapsed path.

## Render-state guarantees

- The footer composes from the per-frame info snapshot (`InfoWidgetData`)
  plus config. It never runs a subprocess, touches the filesystem, or
  queries git on the render path; git data comes from the existing
  TTL-refreshed cache.
- Identical inputs produce byte-identical rows (no animation state, no
  randomness).
- The footer row is drawn outside the messages area, so its decoration is
  excluded from copy selection by construction.
- The render pass is recorded in the frame debug capture as `draw_footer`
  inside the chrome timing block.

## Validation

Focused suites:

```bash
cargo test -p jcode-tui --lib footer          # compose + frame tests
cargo test -p jcode-tui --lib smoothness      # streaming no-stall gate
cargo test -p jcode-tui --lib                 # full TUI suite
openspec validate add-status-footer --strict --no-interactive
```

Gate evidence (roadmap P2, footer portion):

- Deterministic renders at widths 60/80/100/120/160 in both packed and
  scrolling layouts: `ui_tests/footer.rs`
  (`footer_holds_one_row_at_gate_widths`,
  `footer_present_in_packed_and_scrolling_layouts`).
- Repeated-render byte identity, on and off:
  `footer_on_is_deterministic_and_differs_from_off`,
  `footer_off_releases_the_row_and_stays_deterministic`.
- ASCII icon mode: `footer_ascii_mode_uses_ascii_glyphs` plus
  `footer::tests::ascii_mode_uses_ascii_glyphs`.
- Missing git/cost/context data omission and drop order:
  `footer::tests` unit suite.
- Copy-selection exclusion: `footer_decoration_stays_out_of_transcript_region`.
- Streaming no-stall: the footer renders from the same per-frame snapshot
  inside the existing chrome pass; the
  `smoothness_benchmark_simulated_streaming_turn_stays_within_budget` gate
  runs with the footer enabled and stays within its anchor-stability
  budget. `footer_renders_consistently_while_streaming` pins row stability
  across streaming frames.
- Rollback: `style = "off"` reserves zero height
  (`footer_height == 0` in `draw_inner`), so every layout input
  (`fixed_height`, packed/scrolling choice, chunk rects) is identical to
  the pre-footer code path; the off-mode frame tests above are the
  executable proof.
