// Roadmap P3 (frame telemetry): the draw-stats debug payload must expose the
// recorded frame timings as percentile stats split by frame kind, so an
// operator can see render cost without attaching a profiler. This pins the
// JSON contract (key names, kind split, percentile ordering) end to end from
// `FrameTimingRecorder` through `handle_debug_command("draw-stats")`.

#[test]
fn draw_stats_reports_frame_timing_percentiles() {
    use crate::tui::frame_clock::FrameKind;
    use std::time::Duration;

    let mut app = create_test_app();

    // A fresh session has recorded no frames yet: the key must still be
    // present with a zero count so dashboards can rely on the shape.
    let fresh: serde_json::Value =
        serde_json::from_str(&app.handle_debug_command("draw-stats"))
            .expect("draw-stats returns valid JSON");
    let fresh_timing = fresh
        .get("frame_timing")
        .expect("frame_timing present on a fresh session");
    assert_eq!(fresh_timing["count"], 0);
    assert_eq!(fresh_timing["full_count"], 0);
    assert_eq!(fresh_timing["patch_count"], 0);

    // Record a known mix: three full frames and one animation patch.
    for ms in [10u64, 20, 40] {
        app.frame_timings
            .record(FrameKind::Full, Duration::from_millis(ms));
    }
    app.frame_timings
        .record(FrameKind::AnimationPatch, Duration::from_millis(2));

    let payload: serde_json::Value =
        serde_json::from_str(&app.handle_debug_command("draw-stats"))
            .expect("draw-stats returns valid JSON");
    let timing = payload
        .get("frame_timing")
        .expect("frame_timing present after draws");

    assert_eq!(timing["count"], 4, "every recorded frame is counted");
    assert_eq!(timing["full_count"], 3, "full frames counted separately");
    assert_eq!(
        timing["patch_count"], 1,
        "animation patches counted separately"
    );

    let p50 = timing["p50_ms"].as_f64().expect("p50_ms is a number");
    let p95 = timing["p95_ms"].as_f64().expect("p95_ms is a number");
    let max = timing["max_ms"].as_f64().expect("max_ms is a number");
    let mean = timing["mean_ms"].as_f64().expect("mean_ms is a number");

    assert!(
        (max - 40.0).abs() < 0.001,
        "max tracks the slowest frame, got {max}"
    );
    assert!(
        p50 <= p95 && p95 <= max,
        "percentiles are ordered: p50={p50} p95={p95} max={max}"
    );
    // Mean of {10, 20, 40, 2} = 18ms.
    assert!(
        (mean - 18.0).abs() < 0.001,
        "mean covers all recorded frames, got {mean}"
    );
}
