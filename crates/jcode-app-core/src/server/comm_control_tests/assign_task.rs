#[tokio::test]
async fn assign_task_without_task_id_picks_highest_priority_runnable_task() {
    let (_env, _runtime) = RuntimeEnvGuard::new();
    let swarm_id = "swarm-assign";
    let requester = "coord";
    let worker = "worker";
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    let soft_interrupt_queues = Arc::new(RwLock::new(HashMap::new()));
    let client_connections = Arc::new(RwLock::new(HashMap::new()));
    let swarm_members = Arc::new(RwLock::new(HashMap::from([
        (requester.to_string(), {
            let mut member = member(requester, swarm_id, "ready");
            member.role = "coordinator".to_string();
            member
        }),
        (worker.to_string(), member(worker, swarm_id, "ready")),
    ])));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        HashSet::from([requester.to_string(), worker.to_string()]),
    )])));
    let swarm_plans = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        VersionedPlan {
            items: vec![
                plan_item("done", "completed", "high", &[]),
                plan_item("blocked", "queued", "high", &["high-ready"]),
                plan_item("low-ready", "queued", "low", &["done"]),
                plan_item("high-ready", "queued", "high", &["done"]),
            ],
            version: 1,
            participants: HashSet::from([requester.to_string(), worker.to_string()]),
            task_progress: HashMap::new(),
        },
    )])));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        requester.to_string(),
    )])));
    let event_history = Arc::new(RwLock::new(VecDeque::new()));
    let event_counter = Arc::new(AtomicU64::new(1));
    let (swarm_event_tx, _swarm_event_rx) = broadcast::channel(32);
    let mutation_runtime = SwarmMutationRuntime::default();

    handle_comm_assign_task(
        77,
        requester.to_string(),
        Some(worker.to_string()),
        None,
        Some("Pick the next task".to_string()),
        &client_tx,
        &sessions,
        &soft_interrupt_queues,
        &client_connections,
        &swarm_members,
        &swarms_by_id,
        &swarm_plans,
        &swarm_coordinators,
        &event_history,
        &event_counter,
        &swarm_event_tx,
        &mutation_runtime,
    )
    .await;

    let response = client_rx.recv().await.expect("response");
    match response {
        ServerEvent::CommAssignTaskResponse {
            id,
            task_id,
            target_session,
        } => {
            assert_eq!(id, 77);
            assert_eq!(task_id, "high-ready");
            assert_eq!(target_session, worker);
        }
        other => panic!("expected CommAssignTaskResponse, got {other:?}"),
    }

    let plans = swarm_plans.read().await;
    let plan = plans.get(swarm_id).expect("plan exists");
    let selected = plan
        .items
        .iter()
        .find(|item| item.id == "high-ready")
        .expect("selected task exists");
    assert_eq!(selected.assigned_to.as_deref(), Some(worker));
    assert_eq!(selected.status, "queued");

    let blocked = plan
        .items
        .iter()
        .find(|item| item.id == "blocked")
        .expect("blocked task exists");
    assert!(
        blocked.assigned_to.is_none(),
        "blocked task should not be auto-assigned"
    );

    let members = swarm_members.read().await;
    let worker_member = members.get(worker).expect("worker member exists");
    assert_eq!(
        worker_member.status, "queued",
        "assigned worker should stop looking completed/ready before async execution starts"
    );
}

#[tokio::test]
async fn assign_task_marks_completed_worker_queued_before_returning() {
    let (_env, _runtime) = RuntimeEnvGuard::new();
    let swarm_id = "swarm-assign-completed-worker";
    let requester = "coord";
    let worker = "worker-completed";
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    let soft_interrupt_queues = Arc::new(RwLock::new(HashMap::new()));
    let client_connections = Arc::new(RwLock::new(HashMap::new()));
    let swarm_members = Arc::new(RwLock::new(HashMap::from([
        (requester.to_string(), {
            let mut member = member(requester, swarm_id, "ready");
            member.role = "coordinator".to_string();
            member
        }),
        (worker.to_string(), member(worker, swarm_id, "completed")),
    ])));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        HashSet::from([requester.to_string(), worker.to_string()]),
    )])));
    let swarm_plans = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        VersionedPlan {
            items: vec![plan_item("next", "queued", "high", &[])],
            version: 1,
            participants: HashSet::from([requester.to_string(), worker.to_string()]),
            task_progress: HashMap::new(),
        },
    )])));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        requester.to_string(),
    )])));
    let event_history = Arc::new(RwLock::new(VecDeque::new()));
    let event_counter = Arc::new(AtomicU64::new(1));
    let (swarm_event_tx, _swarm_event_rx) = broadcast::channel(32);
    let mutation_runtime = SwarmMutationRuntime::default();

    handle_comm_assign_task(
        78,
        requester.to_string(),
        Some(worker.to_string()),
        Some("next".to_string()),
        None,
        &client_tx,
        &sessions,
        &soft_interrupt_queues,
        &client_connections,
        &swarm_members,
        &swarms_by_id,
        &swarm_plans,
        &swarm_coordinators,
        &event_history,
        &event_counter,
        &swarm_event_tx,
        &mutation_runtime,
    )
    .await;

    match client_rx.recv().await.expect("response") {
        ServerEvent::CommAssignTaskResponse {
            id,
            task_id,
            target_session,
        } => {
            assert_eq!(id, 78);
            assert_eq!(task_id, "next");
            assert_eq!(target_session, worker);
        }
        other => panic!("expected CommAssignTaskResponse, got {other:?}"),
    }

    let members = swarm_members.read().await;
    let worker_member = members.get(worker).expect("worker member exists");
    assert_eq!(worker_member.status, "queued");
    assert!(
        worker_member
            .detail
            .as_deref()
            .is_some_and(|detail| detail.contains("task next")),
        "queued member should include the assigned task in its detail"
    );
}

#[tokio::test]
async fn assign_task_refuses_stopped_worker_and_leaves_plan_unmutated() {
    let (_env, _runtime) = RuntimeEnvGuard::new();
    let swarm_id = "swarm-assign-stopped";
    let requester = "coord";
    let worker = "worker-stopped";
    let control = crate::server::session_control::store();
    control.register(worker, Some(requester)).unwrap();
    control.request_stop(requester, worker, false, "coord:stop").unwrap();

    let (client_tx, mut client_rx) = mpsc::unbounded_channel();
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    let soft_interrupt_queues = Arc::new(RwLock::new(HashMap::new()));
    let client_connections = Arc::new(RwLock::new(HashMap::new()));
    let swarm_members = Arc::new(RwLock::new(HashMap::from([
        (requester.to_string(), {
            let mut member = member(requester, swarm_id, "ready");
            member.role = "coordinator".to_string();
            member
        }),
        (worker.to_string(), member(worker, swarm_id, "ready")),
    ])));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        HashSet::from([requester.to_string(), worker.to_string()]),
    )])));
    let swarm_plans = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        VersionedPlan {
            items: vec![plan_item("t1", "queued", "high", &[])],
            version: 1,
            participants: HashSet::from([requester.to_string(), worker.to_string()]),
            task_progress: HashMap::new(),
        },
    )])));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        requester.to_string(),
    )])));
    let event_history = Arc::new(RwLock::new(VecDeque::new()));
    let event_counter = Arc::new(AtomicU64::new(1));
    let (swarm_event_tx, _swarm_event_rx) = broadcast::channel(32);
    let mutation_runtime = SwarmMutationRuntime::default();

    for (request_id, target) in [(1u64, Some(worker.to_string())), (2, None)] {
        handle_comm_assign_task(
            request_id,
            requester.to_string(),
            target,
            Some("t1".to_string()),
            None,
            &client_tx,
            &sessions,
            &soft_interrupt_queues,
            &client_connections,
            &swarm_members,
            &swarms_by_id,
            &swarm_plans,
            &swarm_coordinators,
            &event_history,
            &event_counter,
            &swarm_event_tx,
            &mutation_runtime,
        )
        .await;
        match client_rx.recv().await.expect("response") {
            ServerEvent::Error { message, .. } => assert!(
                message.contains("cannot receive work") || message.contains("No ready or completed swarm agents"),
                "{message}"
            ),
            other => panic!("stopped worker received an assignment: {other:?}"),
        }
    }

    let plans = swarm_plans.read().await;
    let item = &plans[swarm_id].items[0];
    assert!(item.assigned_to.is_none(), "plan was mutated for revoked worker");
    assert_eq!(item.status, "queued");
    assert_eq!(swarm_members.read().await[worker].status, "ready");
}

#[tokio::test]
async fn explicit_assign_refuses_task_of_unsettled_predecessor_and_leaves_plan_unmutated() {
    let (_env, _runtime) = RuntimeEnvGuard::new();
    let swarm_id = "swarm-assign-unsettled";
    let requester = "coord-unsettled";
    let predecessor = "worker-unsettled-old";
    let successor = "worker-unsettled-new";
    let control = crate::server::session_control::store();
    control.register(predecessor, Some(requester)).unwrap();
    control.register(successor, Some(requester)).unwrap();
    let token = control.admit(predecessor).unwrap();
    let _work = token.reserve().unwrap();
    control
        .request_stop(requester, predecessor, false, "coord:stop")
        .unwrap();
    assert!(control.predecessor_unsettled(predecessor));

    let (client_tx, mut client_rx) = mpsc::unbounded_channel();
    let sessions = Arc::new(crate::server::session_manager::SessionManager::from_agents(
        HashMap::new(),
    ));
    let soft_interrupt_queues = Arc::new(RwLock::new(HashMap::new()));
    let client_connections = Arc::new(RwLock::new(HashMap::new()));
    let swarm_members = Arc::new(RwLock::new(HashMap::from([
        (requester.to_string(), {
            let mut member = member(requester, swarm_id, "ready");
            member.role = "coordinator".to_string();
            member
        }),
        (predecessor.to_string(), member(predecessor, swarm_id, "ready")),
        (successor.to_string(), member(successor, swarm_id, "ready")),
    ])));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        HashSet::from([
            requester.to_string(),
            predecessor.to_string(),
            successor.to_string(),
        ]),
    )])));
    let mut item = plan_item("t1", "running", "high", &[]);
    item.assigned_to = Some(predecessor.to_string());
    let swarm_plans = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        VersionedPlan {
            items: vec![item],
            version: 1,
            participants: HashSet::from([requester.to_string(), predecessor.to_string()]),
            task_progress: HashMap::new(),
        },
    )])));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::from([(
        swarm_id.to_string(),
        requester.to_string(),
    )])));
    let event_history = Arc::new(RwLock::new(VecDeque::new()));
    let event_counter = Arc::new(AtomicU64::new(1));
    let (swarm_event_tx, _swarm_event_rx) = broadcast::channel(32);
    let mutation_runtime = SwarmMutationRuntime::default();

    handle_comm_assign_task(
        1,
        requester.to_string(),
        Some(successor.to_string()),
        Some("t1".to_string()),
        None,
        &client_tx,
        &sessions,
        &soft_interrupt_queues,
        &client_connections,
        &swarm_members,
        &swarms_by_id,
        &swarm_plans,
        &swarm_coordinators,
        &event_history,
        &event_counter,
        &swarm_event_tx,
        &mutation_runtime,
    )
    .await;
    match client_rx.recv().await.expect("response") {
        ServerEvent::Error { message, .. } => {
            assert!(message.contains("stop has not settled"), "{message}")
        }
        other => panic!("unsettled predecessor's task was reassigned: {other:?}"),
    }
    let plans = swarm_plans.read().await;
    let item = &plans[swarm_id].items[0];
    assert_eq!(item.assigned_to.as_deref(), Some(predecessor));
    assert_eq!(item.status, "running");
    assert_eq!(plans[swarm_id].version, 1);
}
