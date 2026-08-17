# frame-clock Specification

## Purpose
TBD - created by archiving change add-frame-clock. Update Purpose after archive.
## Requirements
### Requirement: Central animation-time authority

Jcode SHALL provide a single `FrameClock` as the animation-time authority for all animated TUI surfaces (spinners, streamed reveal, progress transitions, notifications, shader effects). The clock SHALL derive only from local monotonic `Instant`s, so provider latency, stream stalls, and network state cannot advance or stall animation time. `TuiState::animation_elapsed()` SHALL read from this clock so existing consumers adopt it without call-site changes.

#### Scenario: Byte-identical default behavior

- **WHEN** the clock is never paused
- **THEN** `animation_elapsed()` SHALL equal the pre-change `app_started.elapsed()` value exactly
- **AND** the pre-existing render and snapshot suites SHALL pass unmodified.

#### Scenario: Provider independence

- **WHEN** a provider stream stalls or tool calls block for an arbitrary duration
- **THEN** animation time SHALL advance only by local elapsed wall time
- **AND** no provider or network input SHALL be read by the clock.

### Requirement: Pausable animation time

The frame clock SHALL support idempotent `pause()`/`resume()` such that `now()` excludes all paused spans. Terminal suspend SHALL pause the clock and resume SHALL resume it.

#### Scenario: Suspend does not jump animation phase

- **WHEN** the terminal is suspended and later resumed
- **THEN** animation time on return SHALL equal the value at suspend plus only unpaused elapsed time
- **AND** an animated surface rendered across a pause SHALL show the same frame before and after the pause.

#### Scenario: Idempotent pause and resume

- **WHEN** `pause()` or `resume()` is called redundantly
- **THEN** the clock state SHALL be unchanged and `now()` SHALL remain monotone.

### Requirement: Bounded frame math

The clock SHALL provide `frame_index(fps)` and `bounded_elapsed(max)` so surfaces share one bounded frame-math path instead of per-site arithmetic.

#### Scenario: Frame index contract

- **WHEN** a surface requests `frame_index(fps)`
- **THEN** the result SHALL equal `(now() * fps).floor()` computed with saturating arithmetic
- **AND** two surfaces using the same fps SHALL observe identical frame indices for the same clock reading.

#### Scenario: Bounded effects

- **WHEN** an effect requests `bounded_elapsed(max)` after `max` has passed
- **THEN** the result SHALL equal `max` exactly.

### Requirement: Frame timing telemetry

The render loop SHALL record every completed frame (full frames and animation-only patches) with kind, duration, and timestamp into a bounded ring recorder, and SHALL expose count, mean, p50, p95, and max in the debug state surface.

#### Scenario: Recorded frames

- **WHEN** frames complete during a session
- **THEN** the recorder SHALL contain their timings up to its bounded capacity, dropping oldest first
- **AND** the debug state JSON SHALL expose `frame_timing` with count and percentile stats.

#### Scenario: Bounded memory

- **WHEN** more frames are recorded than the ring capacity
- **THEN** the recorder SHALL keep exactly the most recent capacity entries.

### Requirement: Frame budget under load

Under synthetic streaming and tool-load, full-frame render p95 SHALL stay within the active performance tier's frame interval, subject to a generous absolute ceiling, and input or agent events SHALL NOT be starved by slow frames.

#### Scenario: Budget holds under load

- **WHEN** a benchmark drives frames with streaming deltas and tool-card updates
- **THEN** recorded full-frame p95 SHALL be below the tier's frame interval or the documented absolute ceiling.

#### Scenario: No input starvation after slow frames

- **WHEN** a frame exceeds its tick budget
- **THEN** missed ticks SHALL be skipped rather than queued as catch-up bursts
- **AND** pending input SHALL be processed before further animation ticks are served.

