//! Frame clock: the single animation-time authority for the TUI (roadmap P3).
//!
//! Two policy layers already exist above this module:
//! `tui/redraw_schedule.rs` decides *how often* to redraw (cadence authority)
//! and `crates/jcode-app-core/src/perf.rs::TuiPerfPolicy` decides fps caps per
//! performance tier (tier authority). This module provides what they sit on:
//! *what time it is* for animation purposes.
//!
//! Properties:
//! - **Local-only**: derives exclusively from monotonic `Instant`s. Provider
//!   latency, stream stalls, and network state are never inputs, so a stalled
//!   provider cannot stall or warp animation time.
//! - **Pausable**: `pause()`/`resume()` freeze animation time; suspended
//!   terminals no longer jump animation phase on return.
//! - **Bounded**: `frame_index(fps)` and `bounded_elapsed(max)` give surfaces
//!   one shared frame-math path instead of per-site arithmetic.
//!
//! It also hosts the frame-timing telemetry ring (`FrameTimingRecorder`)
//! recorded by the render loop and surfaced in the debug state JSON.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// The animation-time authority. One per `App`, epoch = `app_started`.
///
/// Pause is implemented by accumulating completed pauses and subtracting, so
/// `now()` is monotone and double-pause/double-resume are harmless.
#[derive(Debug, Clone)]
pub struct FrameClock {
    epoch: Instant,
    paused_since: Option<Instant>,
    paused_total: Duration,
}

impl FrameClock {
    pub fn new(epoch: Instant) -> Self {
        Self {
            epoch,
            paused_since: None,
            paused_total: Duration::ZERO,
        }
    }

    /// Animation time: epoch -> now, minus every completed paused span. While
    /// paused, the end instant freezes at the pause start, which is what
    /// excludes the in-progress pause; subtracting it too would double-count
    /// and run the clock backwards.
    pub fn now(&self) -> Duration {
        let end = self.paused_since.unwrap_or_else(Instant::now);
        end.saturating_duration_since(self.epoch)
            .saturating_sub(self.paused_total)
    }

    pub fn elapsed_secs(&self) -> f32 {
        self.now().as_secs_f32()
    }

    /// Freeze animation time. Idempotent.
    pub fn pause(&mut self) {
        if self.paused_since.is_none() {
            self.paused_since = Some(Instant::now());
        }
    }

    /// Unfreeze animation time. Idempotent.
    pub fn resume(&mut self) {
        if let Some(since) = self.paused_since.take() {
            self.paused_total += since.elapsed();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused_since.is_some()
    }

    /// Bounded frame counter shared by every surface at a given fps:
    /// `(now * fps).floor()` with saturating arithmetic.
    pub fn frame_index(&self, fps: u32) -> u64 {
        self.now()
            .as_nanos()
            .saturating_mul(u128::from(fps))
            .saturating_div(1_000_000_000)
            .min(u128::from(u64::MAX)) as u64
    }

    /// Elapsed time clamped to `max`, for effects with a hard duration.
    pub fn bounded_elapsed(&self, max: Duration) -> Duration {
        self.now().min(max)
    }
}

/// What kind of frame a timing sample describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    /// A full terminal frame.
    Full,
    /// An animation-only partial repaint (idle donut fast path).
    AnimationPatch,
}

/// One completed frame's timing sample.
#[derive(Debug, Clone, Copy)]
pub struct FrameTiming {
    pub at: Instant,
    pub kind: FrameKind,
    pub duration: Duration,
}

/// Aggregate stats over a set of timing samples.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTimingStats {
    pub count: usize,
    pub full_count: usize,
    pub patch_count: usize,
    pub mean: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub max: Duration,
}

/// Bounded ring of recent frame timings. p95 needs order statistics, so a
/// ring (not counter aggregates) is kept; 512 frames covers ~8.5s at 60fps,
/// enough for live diagnosis with bounded memory.
#[derive(Debug)]
pub struct FrameTimingRecorder {
    ring: VecDeque<FrameTiming>,
    capacity: usize,
}

impl Default for FrameTimingRecorder {
    fn default() -> Self {
        Self::new(512)
    }
}

impl FrameTimingRecorder {
    pub fn new(capacity: usize) -> Self {
        Self {
            ring: VecDeque::with_capacity(capacity.min(1024)),
            capacity,
        }
    }

    pub fn record(&mut self, kind: FrameKind, duration: Duration) {
        self.record_at(Instant::now(), kind, duration);
    }

    pub fn record_at(&mut self, at: Instant, kind: FrameKind, duration: Duration) {
        if self.capacity == 0 {
            return;
        }
        if self.ring.len() == self.capacity {
            self.ring.pop_front();
        }
        self.ring.push_back(FrameTiming { at, kind, duration });
    }

    pub fn len(&self) -> usize {
        self.ring.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    /// Nearest-rank percentiles over recorded durations.
    pub fn stats(&self) -> FrameTimingStats {
        let count = self.ring.len();
        if count == 0 {
            return FrameTimingStats {
                count: 0,
                full_count: 0,
                patch_count: 0,
                mean: Duration::ZERO,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                max: Duration::ZERO,
            };
        }
        let mut durations: Vec<Duration> = self.ring.iter().map(|t| t.duration).collect();
        durations.sort_unstable();
        let full_count = self
            .ring
            .iter()
            .filter(|t| t.kind == FrameKind::Full)
            .count();
        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
        let mean =
            Duration::from_nanos((total_nanos / count as u128).min(u128::from(u64::MAX)) as u64);
        let nearest_rank = |p: usize| -> Duration {
            // Nearest-rank: ceil(p/100 * n), 1-based.
            let n = durations.len();
            let rank = (p * n).div_ceil(100).clamp(1, n);
            durations[rank - 1]
        };
        FrameTimingStats {
            count,
            full_count,
            patch_count: count - full_count,
            mean,
            p50: nearest_rank(50),
            p95: nearest_rank(95),
            max: *durations.last().unwrap_or(&Duration::ZERO),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_advances_monotonically() {
        let clock = FrameClock::new(Instant::now());
        let a = clock.now();
        let b = clock.now();
        assert!(b >= a, "clock must be monotone: {a:?} then {b:?}");
    }

    #[test]
    fn pause_excludes_elapsed_time() {
        let mut clock = FrameClock::new(Instant::now());
        std::thread::sleep(Duration::from_millis(5));
        let before = clock.now();
        clock.pause();
        std::thread::sleep(Duration::from_millis(20));
        let during = clock.now();
        std::thread::sleep(Duration::from_millis(5));
        let during_later = clock.now();
        clock.resume();
        let after = clock.now();

        assert_eq!(
            during, during_later,
            "a paused clock must be frozen exactly, not drift"
        );
        assert!(
            during <= before + Duration::from_millis(2),
            "paused clock must not advance: before={before:?} during={during:?}"
        );
        assert!(
            after <= before + Duration::from_millis(10),
            "resume must not pay back paused time: before={before:?} after={after:?}"
        );
    }

    #[test]
    fn pause_and_resume_are_idempotent() {
        let mut clock = FrameClock::new(Instant::now());
        clock.pause();
        let t1 = clock.now();
        clock.pause();
        clock.pause();
        let t2 = clock.now();
        std::thread::sleep(Duration::from_millis(10));
        let t3 = clock.now();
        clock.resume();
        clock.resume();
        let t4 = clock.now();

        assert_eq!(t1, t2, "redundant pause must not change state");
        assert!(
            t3 <= t1 + Duration::from_millis(2),
            "redundant pauses must stay frozen"
        );
        assert!(
            t4 <= t1 + Duration::from_millis(5),
            "redundant resume must not pay back time"
        );
        assert!(!clock.is_paused());
    }

    #[test]
    fn now_never_goes_backwards_across_pause_cycles() {
        let mut clock = FrameClock::new(Instant::now());
        let mut last = clock.now();
        for _ in 0..5 {
            clock.pause();
            std::thread::sleep(Duration::from_millis(1));
            clock.resume();
            let current = clock.now();
            assert!(current >= last, "clock went backwards across pause cycle");
            last = current;
        }
    }

    #[test]
    fn frame_index_matches_floor_math() {
        let clock = FrameClock::new(Instant::now() - Duration::from_millis(1500));
        // 1.5s at 60fps = 90 frames (allow +-1 for scheduling jitter).
        let idx = clock.frame_index(60);
        assert!(
            (89..=91).contains(&idx),
            "expected ~90 frames at 60fps after 1.5s, got {idx}"
        );
        assert_eq!(clock.frame_index(0), 0, "zero fps yields frame 0");
    }

    #[test]
    fn two_surfaces_share_identical_frame_indices() {
        let clock = FrameClock::new(Instant::now());
        let spinner = clock.frame_index(10);
        let gallery = clock.frame_index(10);
        assert_eq!(spinner, gallery);
    }

    #[test]
    fn bounded_elapsed_clamps_to_max() {
        let clock = FrameClock::new(Instant::now() - Duration::from_secs(10));
        let max = Duration::from_millis(250);
        assert_eq!(clock.bounded_elapsed(max), max);
        let fresh = FrameClock::new(Instant::now());
        assert!(fresh.bounded_elapsed(max) < max);
    }

    #[test]
    fn recorder_drops_oldest_at_capacity() {
        let mut recorder = FrameTimingRecorder::new(3);
        for i in 0..5 {
            recorder.record_at(
                Instant::now(),
                FrameKind::Full,
                Duration::from_millis(i as u64 + 1),
            );
        }
        assert_eq!(recorder.len(), 3, "ring must stay at capacity");
        let stats = recorder.stats();
        assert_eq!(stats.count, 3);
        // Kept samples are the last three: 3ms, 4ms, 5ms.
        assert_eq!(stats.max, Duration::from_millis(5));
        assert_eq!(stats.p50, Duration::from_millis(4));
    }

    #[test]
    fn recorder_stats_match_hand_computed_percentiles() {
        let mut recorder = FrameTimingRecorder::new(100);
        // 20 samples: 1ms..=20ms, alternating kinds.
        for i in 1..=20u64 {
            let kind = if i % 2 == 0 {
                FrameKind::AnimationPatch
            } else {
                FrameKind::Full
            };
            recorder.record_at(Instant::now(), kind, Duration::from_millis(i));
        }
        let stats = recorder.stats();
        assert_eq!(stats.count, 20);
        assert_eq!(stats.full_count, 10);
        assert_eq!(stats.patch_count, 10);
        assert_eq!(stats.mean, Duration::from_nanos(10_500_000)); // avg 10.5ms
        assert_eq!(stats.p50, Duration::from_millis(10)); // ceil(0.5*20)=10
        assert_eq!(stats.p95, Duration::from_millis(19)); // ceil(0.95*20)=19
        assert_eq!(stats.max, Duration::from_millis(20));
    }

    #[test]
    fn empty_recorder_reports_zero_stats() {
        let recorder = FrameTimingRecorder::new(8);
        let stats = recorder.stats();
        assert!(recorder.is_empty());
        assert_eq!(stats.count, 0);
        assert_eq!(stats.p95, Duration::ZERO);
        let zero_capacity = FrameTimingRecorder::new(0);
        assert!(zero_capacity.is_empty());
    }
}
