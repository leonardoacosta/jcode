# Design

## Context

The TUI render loop already has two centralized policy layers:

- `tui/redraw_schedule.rs` (748 lines) — cadence authority: chooses redraw intervals (idle 250ms, deep idle 5s, remote startup 1s, swarm spinner, etc.) and the animation-only partial-repaint policy with `idle_animation_repaint.rs`.
- `crates/jcode-app-core/src/perf.rs::TuiPerfPolicy` — tier authority: fps caps and `enable_decorative_animations` per detected performance tier (native / WSL / SSH / minimal).

What is missing is the layer *beneath* them: a time authority. `TuiState::animation_elapsed()` (trait method in `tui/mod.rs`, implemented in `tui/app/tui_state.rs`) returns `app_started.elapsed().as_secs_f32()`. Every animated surface derives phase from it:

- `tui_state.rs:1454` — spinner frame `(animation_elapsed * fps) as usize`
- `info_widget.rs`, `info_widget_swarm_gallery.rs`, `info_widget_memory_render.rs` — spinner/gallery frames
- idle donut / shader sampling via `jcode-tui-anim` (`sample_donut`, `sample_black_hole`, `sample_gyroscope`, `sample_orbit_rings`)

Because the value is raw wall clock, it is unpausable (suspend jumps), unbounded (per-site ad-hoc frame math), and unrecorded (no production render timing; only `/debug bench` harness runs).

## Goals / Non-Goals

Goals: one animation-time authority (pausable, bounded, local-only); zero call-site churn via the existing `animation_elapsed` funnel; production frame-timing telemetry with percentile stats; a tested frame budget under synthetic load; byte-identical default behavior.

Non-goals: motion accessibility policy (follow-up change), replacing redraw/perf policy layers, new visual effects, wall-clock displays.

## Architecture

### 1. FrameClock — the time authority

New module `crates/jcode-tui/src/tui/frame_clock.rs`:

```rust
pub struct FrameClock {
    epoch: Instant,                 // == app_started
    paused_since: Option<Instant>,  // Some while paused
    paused_total: Duration,         // accumulated completed pauses
}

impl FrameClock {
    pub fn new(epoch: Instant) -> Self;
    /// Animation time: epoch -> now, minus all paused spans.
    pub fn now(&self) -> Duration;
    pub fn elapsed_secs(&self) -> f32;          // now().as_secs_f32()
    pub fn pause(&mut self);                    // idempotent
    pub fn resume(&mut self);                   // idempotent
    pub fn is_paused(&self) -> bool;
    /// Bounded frame counter: (now * fps).floor(), saturating.
    pub fn frame_index(&self, fps: u32) -> u64;
    /// Elapsed clamped to `max` for effects with a hard duration.
    pub fn bounded_elapsed(&self, max: Duration) -> Duration;
}
```

**Why pause-excluding instead of offset-shifting:** pause/resume must be exact and idempotent; accumulating completed pauses and subtracting keeps `now()` monotone and makes double-pause/double-resume harmless.

**Why local-only:** the clock reads only `Instant`. Provider latency, stream gaps, and network state are never inputs — a stalled provider cannot stall or warp animation time (roadmap: "independent of provider latency").

### 2. Adoption without call-site churn

`App` gains `frame_clock: FrameClock` initialized with `epoch = app_started`. `TuiState::animation_elapsed()` for App becomes `self.frame_clock.elapsed_secs()`. Every consumer (spinners, info widgets, swarm gallery, memory render, idle donut) rides the clock unchanged. **Byte-identity:** when never paused, `now() == app_started.elapsed()` exactly (same epoch, no pauses), so the unmodified render/snapshot suites are the proof.

Pause wiring: the existing terminal suspend/resume path in `run_shell.rs` calls `pause()` on suspend and `resume()` on return. This is the one behavior change and it is invisible in tests (no test suspends mid-render; a dedicated test drives pause/resume directly).

### 3. FrameTimingRecorder — telemetry

Same module:

```rust
pub enum FrameKind { Full, AnimationPatch }

pub struct FrameTiming { pub at: Instant, pub kind: FrameKind, pub duration: Duration }

pub struct FrameTimingRecorder { ring: VecDeque<FrameTiming>, capacity: usize } // default 512

pub struct FrameTimingStats {
    pub count: usize,
    pub full_count: usize,
    pub patch_count: usize,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub max: Duration,
}
```

The render loop records each completed frame (both the full-frame path and the animation-only partial repaint). Stats compute over the ring (sorted durations; nearest-rank percentiles). Exposed in the existing debug state JSON (`debug_cmds.rs`, next to `decorative_animations`) as `frame_timing: { count, p50_ms, p95_ms, max_ms, mean_ms }`.

**Why a ring and not a counter aggregate:** p95 needs order statistics; 512 frames covers ~8.5s at 60fps, enough for live diagnosis without unbounded memory. Deterministic, allocation-free after warmup.

### 4. Frame budget contract

Budget: under synthetic streaming + tool-load, full-frame p95 must stay under the active tier's frame interval (from `TuiPerfPolicy.animation_fps`), and animation patches must stay a small fraction of it. Enforced by a benchmark-style test reusing the P2 harness (`debug_bench.rs` pattern): drive N frames with streaming deltas + tool card updates through the render path, record into a `FrameTimingRecorder`, assert p95/max bounds. Budget values are asserted relative to the tier interval, not absolute milliseconds, so the test is machine-independent within generous absolute caps (absolute ceiling guards pathological regressions on slow CI).

### 5. No starvation

Input and agent events keep priority: the loop already uses `MissedTickBehavior::Skip` (documented in `run_shell.rs`), so a slow frame drops ticks rather than queueing catch-up bursts. The clock does not change scheduling; a test asserts that after a deliberately slow frame, pending input is processed before the next animation tick is served.

## Testing strategy

- **Unit (frame_clock.rs)**: monotonic advance; pause excludes elapsed; resume continues without jump; idempotent pause/resume; `now()` never decreases; `frame_index` math; `bounded_elapsed` clamp; recorder ring capacity/drop-oldest; stats percentiles on known inputs.
- **Byte-identity**: existing render/snapshot/ui suites pass unmodified (clock epoch = app_started, never paused in tests by default).
- **Pause integration**: App test — render spinner, pause clock, render again, spinner frame identical; resume, frame advances.
- **Budget**: benchmark-style test asserts p95/ceiling under synthetic streaming + tool load; starvation test as above.
- **Parity**: full `jcode-tui --lib --test-threads=2` baseline comparison (recorded baseline: 17 known failures).

## Rollback

Additive: delete the module and restore `app_started.elapsed()` in `animation_elapsed`; remove the debug JSON field. No persisted state, no schema, no config.
