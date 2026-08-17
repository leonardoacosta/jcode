# Design

## Context

The jcode TUI draws its chat column as a fixed vertical chunk stack in `crates/jcode-tui/src/tui/ui.rs` (`draw_inner`): messages, queued, swarm strip, one transient status line, notification, inline UI, spacing, input, overscroll, idle animation. Overlays (pickers, help, changelog, control room) short-circuit the frame before this stack is drawn. Floating info widgets render inside the transcript margins from `app.info_widget_data()`.

All data the footer needs already exists and is maintained for other chrome:

- `InfoWidgetData` (working_dir, session_name, model, provider_name, reasoning_effort, context_info, observed_context_tokens, context_limit, upstream_provider, service_tier) assembled per frame in read-only fashion.
- TTL-cached git facts (`gather_git_info()` in `tui/app/helpers.rs`, branch + dirty file set, with explicit invalidation and a backdated-TTL test hook). No per-frame subprocess.
- `CostState.total_cost` accrues per completed provider call and is seeded from history on resume (`tui/app/misc_ui.rs`).
- Execution mode is known to the app (`AppRuntimeMode` local/remote, with remote client sessions subscribing over the socket).
- Colors resolve through `DisplayConfig.theme` + `DisplayConfig.colors` (`docs/TUI_COLOR_CONFIGURATION.md`), and terminal capability detection exists (`color_support.rs`, `theme_detect.rs`).

The Zentui reference (pi-zentui) styles three surfaces independently; this change ports only its Starship-style footer surface, adapted to jcode's data model and layout ownership rules.

## Goals / Non-Goals

**Goals:**

- One persistent bottom row that answers: where am I (directory, git, execution mode), what am I running (model, provider, effort), and what is it costing me (context, tokens, cost).
- Deterministic rendering: identical state produces byte-identical rows; resize behavior is defined by a fixed segment-drop priority.
- Degraded-terminal correctness: ASCII icon mode, no-color mode, and narrow widths all produce documented, stable layouts.
- Zero agent-state coupling: the footer reads cached snapshots only, never mutates, never spawns work, never probes the filesystem or git on the render path.
- Config-gated rollback: `display.footer.style = "off"` restores the exact pre-change layout.

**Non-Goals:**

- Custom format-template strings (Zentui's `$variable` template engine).
- Toolchain/runtime detection segments (Starship language modules).
- Third-party extension status placement.
- A `/footer` or `/zentui`-style interactive configuration menu.
- Clickable segments or mouse interactions on the footer row.
- Changes to the transient activity status line, info widgets, or idle animation.
- New telemetry events or Herdr hooks (P5 owns that contract).

## Architecture

### 1. Footer snapshot model

Add a `FooterSnapshot` struct (in `crates/jcode-tui/src/tui/app/footer.rs`) assembled once per frame from existing state:

- `working_dir`: display path per `path_display` config (basename, last N components, or full; `$HOME` collapses to `~`).
- `git`: optional branch (truncated to fit), dirty flag, ahead/behind counts when the cached facts provide them.
- `mode`: `Local` | `Remote` | `Hybrid` marker derived from the app's runtime mode.
- `model`: label per config (`id` default, `name` fallback), provider label, optional upstream provider.
- `effort`: reasoning effort short label, omitted when off/none.
- `context`: percentage plus used/limit tokens when known; staleness flag from `context_info_stale`.
- `tokens`: streaming input/output totals when available.
- `cost`: session total when greater than zero or when pricing is known; omitted otherwise.
- `session_name`: explicit session name only; unnamed sessions contribute nothing (no placeholder).

Assembly is a pure function over `info_widget_data()`, `gather_git_info()`, cost state, runtime mode, and config. No locks are held during rendering beyond the existing cache reads already performed for other chrome.

**Why:** Keeping the snapshot a pure mapping preserves the P2 rendering-vs-agent-state separation and makes every degradation path unit-testable without a terminal.

### 2. Segment layout and width degradation

The row is split into a left zone (directory, mode, git, session name) and a right zone (model, effort, context, tokens, cost) with a single flexible gap, mirroring the Starship one-fill layout. When the composed row exceeds the available width, segments drop in this fixed order until it fits:

1. session name
2. cost
3. tokens
4. effort
5. upstream provider / service tier extras
6. directory depth (full → depth N → basename)
7. git ahead/behind counts (branch and dirty marker remain)

If the row still does not fit, the branch and context labels truncate with the existing `truncate_smart` behavior; directory and model are the last to truncate. The row never wraps and never scrolls.

**Why:** A fixed, documented priority order makes resize behavior deterministic and snapshot-testable; truncation rules match existing TUI conventions instead of inventing new ones.

### 3. Layout integration

`draw_inner` gains one bottom chunk: `Constraint::Length(footer_height)` appended after the idle-animation chunk in both packed and scrolling constraint sets, where `footer_height` is 1 when the footer style is `segments` and 0 when `off`. Because chunk heights feed the packed/scrolling decision (`content_height + fixed_height <= available_height`), the footer participates in existing layout math with no special casing. Overlay short-circuits keep current behavior: full-screen overlays draw over the whole frame including the footer row; the footer re-renders when they close.

The footer registers no copy-selection rows and no mouse hit targets. Frame metrics treat it as part of the chrome pass (`chrome_start` block) so the existing per-frame timing capture covers it.

**Why:** Reusing the chunk stack keeps resize, packed-layout, and terminal-clear-collapse behavior consistent with every other chrome row, and keeps the stall gate measurable through the existing chrome timing.

### 4. Configuration schema

Additive `display.footer` section in `DisplayConfig`:

- `style`: `segments` (default) | `off`.
- `segments`: per-segment booleans (`cwd`, `mode`, `git`, `session_name`, `model`, `effort`, `context`, `tokens`, `cost`). Defaults: all on except `session_name`.
- `icon_mode`: `auto` (default; Nerd Font glyphs when the terminal is detected capable, ASCII otherwise) | `ascii` | `nerd`.
- `path_display`: `basename` (default) | `depth:N` | `full`.
- `context_thresholds`: `warning` (default 70) and `error` (default 90) percentages.

Unknown keys remain ignored per existing config leniency. Segment colors resolve through the existing color map with new documented keys (e.g. `footerCwd`, `footerGit`, `footerContextWarning`), each falling back to theme tokens when unset.

**Why:** Additive config with defaults preserves every existing config file, and routing colors through the established map keeps the "every color is configurable" guarantee intact.

### 5. Degradation and capability handling

- **Icons:** `auto` resolves once per process using existing terminal detection; ASCII mode swaps glyphs for text markers (e.g. `git:` prefix, `!` dirty marker, `+N/-N` ahead/behind).
- **Color:** no-color terminals render the row unstyled; context thresholds degrade to a textual marker when color is unavailable.
- **Missing data:** non-repo directories omit the git segment entirely; unknown cost omits the segment rather than rendering `$0.00`; stale context renders the last known value with a stale marker; unnamed sessions omit the name segment without leaving separator artifacts.

**Why:** Each degraded state is a first-class snapshot fixture, which is what P2's gate requires (deterministic snapshots, degraded-terminal behavior).

## Session isolation and security

- The footer displays only the connected session's own snapshot data. Remote clients render from their subscribed session's state; no cross-session reads are introduced.
- No credentials, API keys, or raw prompts are ever segments. Token counts and cost are numeric aggregates only.
- Paths are display-processed (`$HOME` collapse, depth truncation) before rendering; no filesystem access occurs on the render path.

## Validation strategy

- Unit tests for snapshot assembly: every segment present/absent, truncation, thresholds, mode markers, `$HOME` collapse.
- Deterministic frame snapshots at widths 60/80/100/120/160 in packed and scrolling layouts, repeated-render identity checks, ASCII and no-color variants, remote mode, and post-resume state.
- Copy-selection regression test: footer row contributes no selectable rows.
- Frame benchmark: footer enabled during streamed output stays within the recorded chrome-pass budget in `docs/STATUS_FOOTER.md`.
- Rollback test: `style = "off"` produces a byte-identical layout to the pre-change baseline snapshot.

## Risks and mitigations

- **Git cache staleness shows wrong branch.** The TTL cache is already the accepted freshness model for the git info widget; the footer inherits it and `invalidate_git_info_cache()` keeps working. Mitigation: none needed; freshness parity with existing chrome.
- **One extra row reduces transcript height at small terminals.** At very small heights the existing layout already sheds optional rows (donut, overscroll, notification). The footer is chrome like the status line; if floors become a problem in practice, a `min_height` hiding rule can be added without schema change. The 60x15 snapshot fixture pins current behavior.
- **Segment flicker during streaming as numbers change.** Values update on the normal redraw tick like all other chrome; no animation is introduced, keeping reduced-motion behavior trivially satisfied.
