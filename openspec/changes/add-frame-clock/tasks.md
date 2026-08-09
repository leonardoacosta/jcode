# Tasks

## 1. FrameClock

- [ ] 1.1 Add `crates/jcode-tui/src/tui/frame_clock.rs` with `FrameClock` (epoch, pause/resume with accumulated pause exclusion, `now`/`elapsed_secs`/`frame_index`/`bounded_elapsed`) and unit tests for monotonicity, pause exclusion, idempotence, and bounded math.
  - touches: `crates/jcode-tui/src/tui/frame_clock.rs`, `crates/jcode-tui/src/tui/mod.rs`
  - depends on: none
  - Done when the clock excludes paused spans exactly, double-pause/double-resume are harmless, `now()` is monotone, and `frame_index(fps)` matches `(now * fps).floor()` saturating.

- [ ] 1.2 Wire `App` to own a `FrameClock` (epoch = `app_started`) and make `TuiState::animation_elapsed()` for App read `elapsed_secs()`.
  - touches: `crates/jcode-tui/src/tui/app.rs`, `crates/jcode-tui/src/tui/app/tui_lifecycle.rs`, `crates/jcode-tui/src/tui/app/tui_state.rs`
  - depends on: 1.1
  - Done when every animated surface rides the clock with zero call-site churn and the pre-existing render/snapshot suites pass unmodified (byte-identity, never paused by default).

- [ ] 1.3 Pause wiring: terminal suspend pauses the clock, resume resumes it, via the existing suspend/resume path in `run_shell.rs`; add an App test that pausing freezes the spinner frame across renders and resuming advances it.
  - touches: `crates/jcode-tui/src/tui/app/run_shell.rs`, `crates/jcode-tui/src/tui/app/tests/`
  - depends on: 1.2
  - Done when suspend no longer jumps animation phase on return and the pause/resume render test passes.

## 2. Frame timing telemetry

- [ ] 2.1 Add `FrameTimingRecorder` (bounded ring, `FrameKind::Full`/`AnimationPatch`, percentile stats) with unit tests for ring capacity, drop-oldest, and p50/p95/max/mean on known inputs.
  - touches: `crates/jcode-tui/src/tui/frame_clock.rs`
  - depends on: 1.1
  - Done when stats match hand-computed percentiles and the ring never grows past capacity.

- [ ] 2.2 Record every completed frame (full and animation-only patch) in the render loop and expose stats in the debug state JSON as `frame_timing` next to `decorative_animations`.
  - touches: `crates/jcode-tui/src/tui/app.rs`, `crates/jcode-tui/src/tui/app/run_shell.rs`, `crates/jcode-tui/src/tui/app/debug_cmds.rs`
  - depends on: 2.1
  - Done when a rendered session shows non-zero `frame_timing.count` with sane percentiles in the debug JSON.

## 3. Budget and starvation gates

- [ ] 3.1 Frame budget test: synthetic streaming + tool-load render benchmark asserts full-frame p95 under the active tier's frame interval (relative bound) plus a generous absolute ceiling.
  - touches: `crates/jcode-tui/src/tui/app/debug_bench.rs` or new bench test module
  - depends on: 2.2
  - Done when the benchmark records into a `FrameTimingRecorder` and asserts the budget deterministically.

- [ ] 3.2 No-starvation test: after a deliberately slow frame, pending input is processed before the next animation tick is served (MissedTickBehavior::Skip contract documented and pinned).
  - touches: `crates/jcode-tui/src/tui/app/run_shell.rs` tests
  - depends on: 3.1
  - Done when the test fails if slow frames queue catch-up bursts ahead of input.

## 4. Docs and validation

- [ ] 4.1 Document the clock: `docs/FRAME_CLOCK.md` (time authority vs cadence/tier authorities, pause semantics, telemetry fields, budget contract, reduced-motion follow-up pointer) and update `ROADMAP_HANDOFF.md` P3 status.
  - touches: `docs/FRAME_CLOCK.md`, `ROADMAP_HANDOFF.md`
  - depends on: 3.2
  - Done when the doc records the architecture split, the pause/budget semantics, and the validation commands.

- [ ] 4.2 Run strict OpenSpec validation, focused frame-clock tests, and the full `jcode-tui --lib --test-threads=2` parity run against the recorded baseline (17 known failures).
  - touches: `openspec/changes/add-frame-clock/*`
  - depends on: 4.1
  - Done when `openspec validate add-frame-clock --strict --no-interactive` passes, focused suites pass, and the full suite shows zero new failures against baseline.
