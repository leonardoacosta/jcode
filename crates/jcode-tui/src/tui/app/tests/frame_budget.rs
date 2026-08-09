// Frame budget benchmark (roadmap P3): drive a synthetic streaming + tool-load
// turn through full `ui::draw` frames, record each frame's wall-time cost into
// a `FrameTimingRecorder`, and assert the budget contract: full-frame p95 stays
// within a generous absolute ceiling that guards pathological render-cost
// regressions, and the recorder plumbing itself reports sane stats.
//
// The ceiling is intentionally generous: this runs in a debug build on shared
// CI, so it cannot pin production latency. What it catches is algorithmic
// regressions (per-frame work going quadratic in transcript size), which blow
// past any ceiling quickly.

/// Time one full frame and record it.
fn observe_budget_frame(
    app: &App,
    terminal: &mut ratatui::Terminal<ratatui::backend::TestBackend>,
    recorder: &mut crate::tui::frame_clock::FrameTimingRecorder,
) {
    let start = std::time::Instant::now();
    terminal
        .draw(|f| crate::tui::ui::draw(f, app))
        .expect("draw");
    recorder.record(
        crate::tui::frame_clock::FrameKind::Full,
        start.elapsed(),
    );
}

#[test]
fn frame_budget_streaming_and_tool_load_stays_within_ceiling() {
    let _render_lock = scroll_render_test_lock();
    with_reasoning_current_home(|| {
        let mut app = create_test_app();
        app.session.short_name = Some("budget".to_string());
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _guard = rt.enter();
        let mut remote = crate::tui::backend::RemoteConnection::dummy();

        let backend = ratatui::backend::TestBackend::new(100, 60);
        let mut terminal = ratatui::Terminal::new(backend).expect("terminal");
        let mut recorder = crate::tui::frame_clock::FrameTimingRecorder::new(256);

        app.is_processing = true;
        app.status = ProcessingStatus::Streaming;
        observe_budget_frame(&app, &mut terminal, &mut recorder);

        // Streaming bursts with paced drains, like the smoothness benchmark.
        for i in 0..6 {
            app.handle_server_event(
                crate::protocol::ServerEvent::TextDelta {
                    text: format!("Streaming chunk {i}: plenty of visible text to wrap. "),
                },
                &mut remote,
            );
            for _ in 0..4 {
                let ops = app.stream_buffer.flush_smooth_frame();
                app.apply_stream_ops(ops);
                observe_budget_frame(&app, &mut terminal, &mut recorder);
            }
        }

        // Tool load: tool cards start/exec/finish while streaming continues.
        for tool in 0..4 {
            let id = format!("tool-{tool}");
            app.handle_server_event(
                crate::protocol::ServerEvent::ToolStart {
                    id: id.clone(),
                    name: "bash".to_string(),
                },
                &mut remote,
            );
            observe_budget_frame(&app, &mut terminal, &mut recorder);
            app.handle_server_event(
                crate::protocol::ServerEvent::ToolExec {
                    id: id.clone(),
                    name: "bash".to_string(),
                },
                &mut remote,
            );
            app.handle_server_event(
                crate::protocol::ServerEvent::TextDelta {
                    text: format!("Output from {id}: line one\nline two\nline three\n"),
                },
                &mut remote,
            );
            let ops = app.stream_buffer.flush();
            app.apply_stream_ops(ops);
            observe_budget_frame(&app, &mut terminal, &mut recorder);
        }

        let stats = recorder.stats();
        assert!(
            stats.count >= 30,
            "benchmark should record a meaningful sample, got {}",
            stats.count
        );
        assert_eq!(
            stats.full_count, stats.count,
            "every recorded frame here is a full frame"
        );
        assert_eq!(stats.patch_count, 0);
        assert!(
            stats.mean <= stats.p95 && stats.p95 <= stats.max,
            "percentile ordering must hold: {stats:?}"
        );
        // Generous absolute ceilings (debug build, shared CI). Production
        // full frames are single-digit ms; these catch algorithmic
        // regressions, not jitter.
        assert!(
            stats.p95 <= std::time::Duration::from_millis(250),
            "full-frame p95 blew the 250ms ceiling: {stats:?}"
        );
        assert!(
            stats.max <= std::time::Duration::from_millis(1000),
            "slowest full frame blew the 1000ms ceiling: {stats:?}"
        );

        // Relative contract: the tier's animation interval is the human-scale
        // budget. Report how far under it we are so a drift toward it is
        // visible in test output before it ever fails.
        let policy = crate::perf::tui_policy();
        let tier_interval = std::time::Duration::from_secs_f64(
            1.0 / f64::from(policy.animation_fps.max(1)),
        );
        eprintln!(
            "frame budget: p95={:?} max={:?} tier_interval={:?} ({} fps)",
            stats.p95, stats.max, tier_interval, policy.animation_fps
        );
    });
}
