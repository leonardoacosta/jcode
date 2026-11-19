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
        &Default::default(),
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
#[tokio::test]
async fn unknown_member_never_satisfies_terminal_wait() {
    let statuses = awaited_member_statuses(
        "requester",
        "swarm",
        &["missing".to_string()],
        &["stopped".to_string(), "completed".to_string()],
        &HashMap::new(),
        &Arc::new(RwLock::new(HashMap::new())),
        &Arc::new(RwLock::new(HashMap::new())),
    )
    .await;
    assert_eq!(statuses[0].status, "unknown");
    assert!(
        !statuses[0].done,
        "missing membership cannot prove execution settlement"
    );
    assert!(!mode_satisfied(&statuses, None));
}

#[tokio::test]
async fn wait_bound_to_old_generation_never_completes_from_resumed_work() {
    use crate::server::session_control;
    let (_env, _runtime_dir) = RuntimeEnvGuard::new();
    let store = session_control::store();
    let id = "bound-worker";
    let swarm_id = "swarm-bound";
    store.register(id, Some("owner")).unwrap();
    let first = store.admit(id).unwrap();
    session_control::scope(Some(first), async {
        store.project_report(id, "ready", true, || ()).unwrap();
    })
    .await;
    let mut ready = member(id, swarm_id, "ready");
    ready.latest_completion_report = Some("generation 1 report".to_string());
    let members = Arc::new(RwLock::new(HashMap::from([(id.to_string(), ready)])));
    let swarms = Arc::new(RwLock::new(HashMap::new()));
    let ids = [id.to_string()];
    let targets = ["ready".to_string()];
    let status = |bound: HashMap<String, u64>| {
        let (members, swarms) = (members.clone(), swarms.clone());
        let (ids, targets) = (ids.clone(), targets.clone());
        async move {
            awaited_member_statuses("req", swarm_id, &ids, &targets, &bound, &members, &swarms)
                .await
                .remove(0)
        }
    };
    // Unbound (legacy) and bound-to-current behave as before.
    assert!(status(HashMap::new()).await.done);
    let current = status(HashMap::from([(id.to_string(), 1)])).await;
    assert!(current.done);
    assert_eq!(current.generation, Some(1));
    // A wait started for the next generation ignores the generation 1 report.
    let next = status(HashMap::from([(id.to_string(), 2)])).await;
    assert!(!next.done);
    assert!(next.completion_report.is_none());
    // The member resumes at generation 2 and reports: a wait bound to
    // generation 1 must neither complete from, nor surface, generation 2's work.
    let second = store.admit(id).unwrap();
    session_control::scope(Some(second), async {
        store.project_report(id, "ready", true, || ()).unwrap();
    })
    .await;
    members.write().await.get_mut(id).unwrap().latest_completion_report =
        Some("generation 2 report".to_string());
    let stale = status(HashMap::from([(id.to_string(), 1)])).await;
    assert!(!stale.done, "wait bound to generation 1 completed from generation 2");
    assert!(stale.completion_report.is_none());
    let fresh = status(HashMap::from([(id.to_string(), 2)])).await;
    assert!(fresh.done);
    assert_eq!(fresh.completion_report.as_deref(), Some("generation 2 report"));
}

#[test]
fn pending_wait_generation_bindings_survive_restart_and_legacy_states_stay_unbound() {
    use crate::server::await_members_state::{
        PersistedAwaitMembersState, ensure_pending_state, load_state, save_state,
    };
    let (_env, _runtime_dir) = RuntimeEnvGuard::new();
    let now = crate::todo::await_dependency::now_unix_ms();
    let mut state = ensure_pending_state(
        "bound-restart",
        "owner",
        "swarm",
        &["worker".to_string()],
        &["ready".to_string()],
        None,
        now + 60_000,
        true,
        false,
        false,
    );
    assert!(state.bound_generations.is_empty());
    state.bound_generations.insert("worker".to_string(), 3);
    save_state(&state);
    // "Restart": nothing in memory, only the durable record.
    let reloaded = load_state("bound-restart").expect("persisted wait");
    assert_eq!(reloaded.bound_generations.get("worker"), Some(&3));
    // A state written before this field existed deserializes unbound.
    let mut legacy = serde_json::to_value(&reloaded).unwrap();
    legacy.as_object_mut().unwrap().remove("bound_generations");
    let legacy: PersistedAwaitMembersState = serde_json::from_value(legacy).unwrap();
    assert!(legacy.bound_generations.is_empty());
}
