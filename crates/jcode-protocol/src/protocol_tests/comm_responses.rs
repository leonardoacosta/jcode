#[test]
fn test_swarm_plan_event_roundtrip_with_summary() -> Result<()> {
    let event = ServerEvent::SwarmPlan {
        swarm_id: "swarm_123".to_string(),
        version: 7,
        items: vec![PlanItem {
            content: "Investigate planner state".to_string(),
            status: "queued".to_string(),
            priority: "high".to_string(),
            id: "task-1".to_string(),
            subsystem: None,
            file_scope: Vec::new(),
            blocked_by: vec![],
            assigned_to: None,
        }],
        participants: vec!["session_fox".to_string()],
        reason: Some("task_completed".to_string()),
        summary: Some(PlanGraphStatus {
            swarm_id: Some("swarm_123".to_string()),
            version: 7,
            item_count: 1,
            ready_ids: vec!["task-1".to_string()],
            blocked_ids: Vec::new(),
            active_ids: Vec::new(),
            completed_ids: Vec::new(),
            failed_ids: Vec::new(),
            failed_reasons: Default::default(),
            cycle_ids: Vec::new(),
            unresolved_dependency_ids: Vec::new(),
            next_ready_ids: vec!["task-1".to_string()],
            newly_ready_ids: Vec::new(),
        }),
    };
    let json = encode_event(&event);
    assert!(json.contains("\"type\":\"swarm_plan\""));
    assert!(json.contains("\"summary\""));
    let decoded = parse_event_json(json.trim())?;
    let ServerEvent::SwarmPlan {
        swarm_id,
        version,
        items,
        participants,
        reason,
        summary,
    } = decoded
    else {
        return Err(anyhow!("expected SwarmPlan event"));
    };
    assert_eq!(swarm_id, "swarm_123");
    assert_eq!(version, 7);
    assert_eq!(participants, vec!["session_fox"]);
    assert_eq!(reason.as_deref(), Some("task_completed"));
    assert_eq!(items.len(), 1);
    let summary = summary.ok_or_else(|| anyhow!("expected plan summary"))?;
    assert_eq!(summary.ready_ids, vec!["task-1"]);
    assert_eq!(summary.next_ready_ids, vec!["task-1"]);
    Ok(())
}

#[test]
fn test_comm_task_control_response_roundtrip() -> Result<()> {
    let event = ServerEvent::CommTaskControlResponse {
        id: 61,
        action: "start".to_string(),
        task_id: "task-1".to_string(),
        target_session: Some("sess_worker".to_string()),
        status: "running".to_string(),
        summary: PlanGraphStatus {
            swarm_id: Some("swarm_123".to_string()),
            version: 3,
            item_count: 2,
            ready_ids: vec!["task-2".to_string()],
            blocked_ids: Vec::new(),
            active_ids: vec!["task-1".to_string()],
            completed_ids: vec!["setup".to_string()],
            failed_ids: Vec::new(),
            failed_reasons: Default::default(),
            cycle_ids: Vec::new(),
            unresolved_dependency_ids: Vec::new(),
            next_ready_ids: vec!["task-2".to_string()],
            newly_ready_ids: vec!["task-2".to_string()],
        },
    };
    let json = encode_event(&event);
    assert!(json.contains("\"type\":\"comm_task_control_response\""));
    let decoded = parse_event_json(json.trim())?;
    let ServerEvent::CommTaskControlResponse {
        id,
        action,
        task_id,
        target_session,
        status,
        summary,
    } = decoded
    else {
        return Err(anyhow!("expected CommTaskControlResponse"));
    };
    assert_eq!(id, 61);
    assert_eq!(action, "start");
    assert_eq!(task_id, "task-1");
    assert_eq!(target_session.as_deref(), Some("sess_worker"));
    assert_eq!(status, "running");
    assert_eq!(summary.next_ready_ids, vec!["task-2"]);
    assert_eq!(summary.newly_ready_ids, vec!["task-2"]);
    Ok(())
}

#[test]
fn test_comm_status_roundtrip() -> Result<()> {
    let req = Request::CommStatus {
        id: 56,
        session_id: "sess_watcher".to_string(),
        target_session: "sess_peer".to_string(),
    };
    let json = serde_json::to_string(&req)?;
    assert!(json.contains("\"type\":\"comm_status\""));
    let decoded = parse_request_json(&json)?;
    assert_eq!(decoded.id(), 56);
    let Request::CommStatus {
        session_id,
        target_session,
        ..
    } = decoded
    else {
        return Err(anyhow!("expected CommStatus"));
    };
    assert_eq!(session_id, "sess_watcher");
    assert_eq!(target_session, "sess_peer");
    Ok(())
}

#[test]
fn test_comm_plan_status_roundtrip() -> Result<()> {
    let req = Request::CommPlanStatus {
        id: 59,
        session_id: "sess_coord".to_string(),
    };
    let json = serde_json::to_string(&req)?;
    assert!(json.contains("\"type\":\"comm_plan_status\""));
    let decoded = parse_request_json(&json)?;
    assert_eq!(decoded.id(), 59);
    let Request::CommPlanStatus { session_id, .. } = decoded else {
        return Err(anyhow!("expected CommPlanStatus"));
    };
    assert_eq!(session_id, "sess_coord");
    Ok(())
}

#[test]
fn test_comm_members_roundtrip_includes_status() -> Result<()> {
    let event = ServerEvent::CommMembers {
        id: 9,
        members: vec![AgentInfo {
            session_id: "sess-peer".to_string(),
            friendly_name: Some("bear".to_string()),
            files_touched: vec!["src/main.rs".to_string()],
            status: Some("running".to_string()),
            detail: Some("working on tests".to_string()),
            role: Some("agent".to_string()),
            is_headless: Some(true),
            report_back_to_session_id: Some("sess-coord".to_string()),
            latest_completion_report: Some("Done.".to_string()),
            live_attachments: Some(0),
            status_age_secs: Some(12),
            ..Default::default()
        }],
    };

    let json = encode_event(&event);
    assert!(json.contains("\"type\":\"comm_members\""));
    assert!(json.contains("\"status\":\"running\""));

    let decoded = parse_event_json(json.trim())?;
    let ServerEvent::CommMembers { id, members } = decoded else {
        return Err(anyhow!("expected CommMembers"));
    };
    assert_eq!(id, 9);
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].friendly_name.as_deref(), Some("bear"));
    assert_eq!(members[0].status.as_deref(), Some("running"));
    assert_eq!(members[0].detail.as_deref(), Some("working on tests"));
    assert_eq!(members[0].is_headless, Some(true));
    assert_eq!(
        members[0].report_back_to_session_id.as_deref(),
        Some("sess-coord")
    );
    assert_eq!(
        members[0].latest_completion_report.as_deref(),
        Some("Done.")
    );
    assert_eq!(members[0].live_attachments, Some(0));
    assert_eq!(members[0].status_age_secs, Some(12));
    Ok(())
}

#[test]
fn test_session_close_requested_roundtrip() -> Result<()> {
    let event = ServerEvent::SessionCloseRequested {
        reason: "Stopped by coordinator coord".to_string(),
    };
    let json = encode_event(&event);
    assert!(json.contains("\"type\":\"session_close_requested\""));
    let decoded = parse_event_json(json.trim())?;
    let ServerEvent::SessionCloseRequested { reason } = decoded else {
        return Err(anyhow!("expected SessionCloseRequested"));
    };
    assert_eq!(reason, "Stopped by coordinator coord");
    Ok(())
}

#[test]
fn test_comm_status_response_roundtrip() -> Result<()> {
    let event = ServerEvent::CommStatusResponse {
        id: 57,
        snapshot: AgentStatusSnapshot {
            session_id: "sess-peer".to_string(),
            friendly_name: Some("bear".to_string()),
            swarm_id: Some("swarm-test".to_string()),
            status: Some("running".to_string()),
            detail: Some("working on tests".to_string()),
            role: Some("agent".to_string()),
            is_headless: Some(true),
            live_attachments: Some(0),
            status_age_secs: Some(5),
            last_activity_age_secs: Some(2),
            stalled_secs: None,
            joined_age_secs: Some(30),
            files_touched: vec!["src/main.rs".to_string()],
            file_claims: Vec::new(),
            activity: Some(SessionActivitySnapshot {
                is_processing: true,
                current_tool_name: Some("bash".to_string()),
            }),
            provider_name: None,
            provider_model: None,
            control: None,
        },
    };

    let json = encode_event(&event);
    assert!(json.contains("\"type\":\"comm_status_response\""));
    let decoded = parse_event_json(json.trim())?;
    let ServerEvent::CommStatusResponse { id, snapshot } = decoded else {
        return Err(anyhow!("expected CommStatusResponse"));
    };
    assert_eq!(id, 57);
    assert_eq!(snapshot.session_id, "sess-peer");
    assert_eq!(snapshot.friendly_name.as_deref(), Some("bear"));
    assert_eq!(
        snapshot
            .activity
            .and_then(|activity| activity.current_tool_name),
        Some("bash".to_string())
    );
    Ok(())
}

#[test]
fn status_snapshot_without_control_decodes_and_never_implies_runnable() {
    let legacy = r#"{"type":"comm_status_response","id":9,"snapshot":{"session_id":"old"}}"#;
    let event: ServerEvent = serde_json::from_str(legacy).expect("legacy status decodes");
    let ServerEvent::CommStatusResponse { snapshot, .. } = event else {
        panic!("expected status response");
    };
    assert!(snapshot.control.is_none());
    assert!(!encode_event(&ServerEvent::CommStatusResponse { id: 9, snapshot }).contains("control"));
}

#[test]
fn status_snapshot_control_roundtrips_and_formats_pending_stop() {
    let snapshot = AgentStatusSnapshot {
        session_id: "worker".to_string(),
        status: Some("running".to_string()),
        control: Some(ControlStatusSnapshot {
            state: "draining".to_string(),
            generation: 3,
            admitted_generation: None,
            revoked_through: 3,
            revision: 5,
            stop_request: Some("coord:7".to_string()),
            unresolved_categories: vec!["shell_children".to_string()],
        }),
        ..Default::default()
    };
    let json = serde_json::to_string(&snapshot).unwrap();
    let back: AgentStatusSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(back.control, snapshot.control);
    let text = format_comm_status_snapshot(&snapshot);
    assert!(text.contains("Control: draining (generation 3, revoked through 3, revision 5)"));
    assert!(text.contains("stop request coord:7"));
    assert!(text.contains("unresolved: shell_children"));
}

#[test]
fn stop_requested_and_draining_are_nonterminal_additive_events() {
    let requested = ServerEvent::StopRequested {
        request_id: 9,
        session_id: "worker".into(),
        generation: 3,
        revision: 5,
        terminal: false,
    };
    let json = serde_json::to_value(&requested).unwrap();
    assert_eq!(json["type"], "stop_requested");
    // Correlated by request_id so id-matching legacy readers never see a response.
    assert!(json.get("id").is_none());
    let draining: ServerEvent = serde_json::from_str(
        r#"{"type":"draining","request_id":9,"session_id":"worker","generation":3}"#,
    )
    .unwrap();
    assert!(matches!(
        draining,
        ServerEvent::Draining { request_id: 9, revision: 0, terminal: false, .. }
    ));
}
