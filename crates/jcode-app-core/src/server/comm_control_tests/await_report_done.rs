#[tokio::test]
async fn awaited_statuses_require_report_for_ready_and_keep_reported_stopped_done() {
    let swarm_id = "swarm-report";
    let mut no_report = member("a", swarm_id, "ready");
    no_report.latest_completion_report = None;
    let mut ready_reported = member("b", swarm_id, "ready");
    ready_reported.latest_completion_report = Some("done b".to_string());
    let mut stopped_reported = member("c", swarm_id, "stopped");
    stopped_reported.latest_completion_report = Some("done c".to_string());
    let swarm_members = Arc::new(RwLock::new(HashMap::from([
        ("a".to_string(), no_report),
        ("b".to_string(), ready_reported),
        ("c".to_string(), stopped_reported),
    ])));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::new()));
    // Default targets include ready/completed but not stopped.
    let statuses = awaited_member_statuses(
        "req",
        swarm_id,
        &["a".to_string(), "b".to_string(), "c".to_string()],
        &["ready".to_string(), "completed".to_string()],
        &swarm_members,
        &swarms_by_id,
    )
    .await;
    assert!(!statuses[0].done, "ready without report is not done");
    assert!(statuses[1].done);
    assert!(statuses[2].done, "reported then stopped stays done");
    assert!(mode_satisfied(&statuses[1..], None));
    let _ = timeout_summary;
}
