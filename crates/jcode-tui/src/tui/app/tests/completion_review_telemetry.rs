#[test]
fn completion_review_summary_maps_digest_resolution_cycles_and_repeats() {
    let summary = crate::todo::GateObservationSummary {
        kind: crate::todo::GateObservationKind::FeedbackLoopCoverage,
        group: Some("release".to_string()),
        continuation_cycles: 3,
        resolved: true,
        repeated_prompt: true,
    };

    let telemetry = super::input::completion_review_telemetry_for_gate_summary(summary);

    assert_eq!(
        telemetry.trigger,
        crate::telemetry::CompletionReviewTrigger::GateDigest
    );
    assert!(telemetry.goal_grouped);
    assert_eq!(
        telemetry.unresolved_concern,
        crate::telemetry::CompletionReviewConcern::FeedbackLoopCoverage
    );
    assert_eq!(telemetry.continuation_cycles, 3);
    assert_eq!(
        telemetry.resolution,
        crate::telemetry::CompletionReviewResolution::Resolved
    );
    assert!(telemetry.repeated_prompt);
}
