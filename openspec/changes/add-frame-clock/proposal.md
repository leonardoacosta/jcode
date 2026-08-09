# Add Frame Clock (Motion and Frame Scheduler, P3)

## Why

The roadmap handoff (`ROADMAP_HANDOFF.md`, block P3 — Motion and frame scheduler) requires a **centralized frame clock/scheduler** for spinners, streamed token reveal, progress transitions, notifications, and shader-like effects; animation that is **pausable, bounded, and independent of provider latency**; and **recorded frame/render timings** for telemetry.

Today animation time is ad-hoc and cannot satisfy those properties:

- `TuiState::animation_elapsed()` is `app_started.elapsed()` — raw wall clock since process start. Every animated surface (spinner frames in `tui_state.rs`, info widgets, swarm gallery, memory render, idle donut shaders in `jcode-tui-anim`) derives its phase from that one unpausable, unbounded value.
- **Not pausable**: when the terminal is suspended or a blocking surface owns the screen, animation time keeps advancing, so surfaces jump on return.
- **Not bounded**: there is no frame-index contract; each call site does its own `(elapsed * fps) as usize` math with its own divisor, so cadence drift between surfaces is invisible and untestable.
- **No telemetry**: render durations are only observable through manual `/debug bench` runs (P2 smoothness benchmarks). Nothing records per-frame timings in production, so "stable frame budget under streaming/tool load" cannot be measured outside a benchmark harness.
- Cadence *policy* is already centralized (`tui/redraw_schedule.rs` decides how often to redraw; `perf.rs::TuiPerfPolicy` decides fps caps per performance tier), but there is no time *authority* beneath it — the scheduler has no clock.

## What Changes

- Introduce `FrameClock` (`crates/jcode-tui/src/tui/frame_clock.rs`): the single animation-time authority. `now()` returns animation time excluding paused spans; `pause()`/`resume()` freeze it; `frame_index(fps)` and `bounded_elapsed(max)` give surfaces one bounded math path instead of per-site arithmetic. The clock derives only from local `Instant`s — provider latency, stream stalls, and network state can never advance or stall it.
- App owns one `FrameClock`; `TuiState::animation_elapsed()` for App reads from it (same epoch as `app_started`), so **every existing consumer rides the clock with zero call-site churn** and default behavior is byte-identical when never paused.
- Pause wiring: the clock pauses on terminal suspend and resumes on return (the existing suspend/resume path in `run_shell`), with the API documented for future blocking surfaces.
- Introduce `FrameTimingRecorder`: a bounded ring buffer of per-frame timings (kind: full frame vs animation-only patch, duration, timestamp) recorded in the render loop, with p50/p95/max/mean stats exposed through the existing debug state JSON next to `decorative_animations`.
- Frame budget contract: document and test the budget (full frame p95 under the tier's frame interval under synthetic streaming + tool load), reusing the P2 benchmark harness pattern.

## Non-goals

- Reduced-motion / non-color accessibility fallbacks (a separate change builds the motion policy on top of this clock; P3's fallback gate lands there).
- Replacing `redraw_schedule.rs` interval policy or `perf.rs` tier detection — they stay the cadence authorities; this change adds the time authority beneath them.
- New animation effects, shader work, or visual redesign.
- Wall-clock/session-time displays (timestamps, uptime) — those are not animation time and stay on `Instant`/`DateTime` directly.

## Impact

- Affected specs: `frame-clock` (new).
- Affected code: `crates/jcode-tui/src/tui/frame_clock.rs` (new), `tui/app.rs` + `tui/app/tui_lifecycle.rs` (field), `tui/app/tui_state.rs` (`animation_elapsed` reads clock), `tui/app/run_shell.rs` (pause wiring + timing recording), `tui/app/debug_cmds.rs` (telemetry surface).
- Compatibility: fully additive. Default (never paused) produces byte-identical renders, proven by the unmodified pre-existing render/snapshot suites.
- Rollback: delete the delegation and restore `app_started.elapsed()`; no persisted state or schema.
