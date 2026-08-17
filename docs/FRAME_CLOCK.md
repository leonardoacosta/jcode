# Frame Clock

The TUI's animation-time authority (roadmap P3). One page explains how the
three scheduling layers fit together, what the clock guarantees, and where the
telemetry shows up.

## The three layers

| Layer | Role | Home |
|-------|------|------|
| **Time authority** | what time it is for animation | `crates/jcode-tui/src/tui/frame_clock.rs` (`FrameClock`) |
| **Cadence authority** | how often to redraw | `crates/jcode-tui/src/tui/redraw_schedule.rs` |
| **Tier authority** | fps caps and decorative on/off per performance tier | `crates/jcode-app-core/src/perf.rs` (`TuiPerfPolicy`) |

The cadence and tier layers existed before P3 and are unchanged. P3 added the
time layer beneath them: previously every animated surface derived its phase
from `app_started.elapsed()` — raw, unpausable, unbounded wall clock.

## FrameClock

`App` owns one `FrameClock` whose epoch is `app_started`. Every animated
surface (spinner frames, info widgets, swarm gallery, memory render, workspace
animation ticks, idle donut shaders) reads animation time through
`TuiState::animation_elapsed()`, which returns `frame_clock.elapsed_secs()`.
The viewport prompt-entry animation reads `TuiState::now_millis()`, which
returns `frame_clock.now()` in milliseconds. Zero call-site churn: the
existing funnels are the adoption path.

Guarantees:

- **Local-only**: the clock reads monotonic `Instant`s only. Provider latency,
  stream stalls, and network state can never advance, stall, or warp
  animation time.
- **Byte-identical when unpaused**: same epoch as `app_started`, so the
  pre-change value is reproduced exactly (the unmodified render/snapshot
  suites prove this).
- **Monotone**: `now()` never decreases, including across pause/resume cycles.

### Pause semantics

`pause()`/`resume()` are idempotent. While paused, `now()` is frozen exactly
(not drifting); resuming excludes the paused span — no payback jump.

Wired trigger: **terminal focus loss**. `set_client_focused(false)` pauses the
clock and `set_client_focused(true)` resumes it. This matches the existing
behavior of suppressing decorative redraws while unfocused: animation phase
now also freezes, so a refocused window continues the animation where it left
off instead of jumping. All three focus handlers (local, remote, reconnect)
route through `set_client_focused`.

### Bounded frame math

- `frame_index(fps)` — `(now * fps).floor()` with saturating arithmetic. Two
  surfaces at the same fps always agree on the frame index for the same clock
  reading.
- `bounded_elapsed(max)` — elapsed clamped to a hard duration, for effects
  that must end.

## Frame timing telemetry

`App` also owns a `FrameTimingRecorder`: a bounded ring (512 entries, ~8.5s at
60fps) of per-frame timings. The render loop records every completed frame:

- `FrameKind::Full` — full terminal frames (`draw_full`), timed around the
  whole draw including flush.
- `FrameKind::AnimationPatch` — animation-only partial repaints (idle donut
  fast path), so cheap ticks are separable from real frames.

Stats (count, full/patch split, mean, p50, p95, max — nearest-rank
percentiles) are exposed in the `draw-stats` debug payload as `frame_timing`,
next to the `redraw_schedule` block:

```json
"frame_timing": { "count": 512, "full_count": 40, "patch_count": 472,
  "mean_ms": 0.8, "p50_ms": 0.4, "p95_ms": 6.1, "max_ms": 12.4 }
```

## Budget and starvation contract

- **Frame budget**: under synthetic streaming + tool load, full-frame p95 must
  stay under a generous absolute ceiling (250ms p95 / 1000ms max in a debug
  build) that catches algorithmic render-cost regressions. Pinned by
  `frame_budget_streaming_and_tool_load_stays_within_ceiling` in
  `tui/app/tests/frame_budget.rs`, which drives scripted streaming bursts and
  tool-card updates through real `ui::draw` frames on a TestBackend.
- **No starvation**: redraw and spinner intervals use
  `MissedTickBehavior::Skip`, so a slow frame drops missed ticks instead of
  queueing catch-up bursts; pending input and agent events go first. Pinned by
  `redraw_timer_skips_missed_ticks_instead_of_bursting` and the spinner
  variant in `run_shell.rs` (both fail under `Burst`).

## Follow-up: motion policy

Reduced-motion and non-color accessibility fallbacks build on this clock as a
separate change: a `ui.motion` policy (full / reduced / off) that substitutes
static frames for animated surfaces. The clock's pause/bounds API is the
integration point; defaults stay full (byte-identical).

## Validation

```
cargo test -p jcode-tui --lib frame_clock        # clock + recorder units
cargo test -p jcode-tui --lib freezes_animation  # focus pause integration
cargo test -p jcode-tui --lib frame_budget       # budget gate
cargo test -p jcode-tui --lib skips_missed_ticks # starvation gates
cargo test -p jcode-tui --lib -- --test-threads=2 # full-suite parity
openspec validate add-frame-clock --strict --no-interactive
```
