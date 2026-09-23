#[tokio::test]
async fn handle_resume_session_allows_live_attach_when_existing_agent_is_busy() -> Result<()> {
    let _guard = crate::storage::lock_test_env();
    let (_runtime, prev_runtime) = setup_runtime_dir()?;

    let target_session_id = "session_existing_live_busy";
    let temp_session_id = "session_temp_connecting_busy";

    let persisted_message = crate::session::StoredMessage {
        id: "msg-live-busy".to_string(),
        role: crate::message::Role::User,
        content: vec![crate::message::ContentBlock::Text {
            text: "persisted busy attach history".to_string(),
            cache_control: None,
        }],
        display_role: None,
        timestamp: None,
        tool_duration_ms: None,
        token_usage: None,
    };

    let provider: Arc<dyn Provider> = Arc::new(MockProvider);
    let existing_registry = Registry::new(provider.clone()).await;
    let existing_agent = Arc::new(Mutex::new(build_test_agent_with_id(
        provider.clone(),
        existing_registry,
        target_session_id,
        vec![persisted_message],
    )));

    let new_registry = Registry::new(provider.clone()).await;
    let new_agent = Arc::new(Mutex::new(build_test_agent_with_id(
        provider.clone(),
        new_registry.clone(),
        temp_session_id,
        Vec::new(),
    )));

    let sessions = Arc::new(RwLock::new(HashMap::from([
        (target_session_id.to_string(), Arc::clone(&existing_agent)),
        (temp_session_id.to_string(), Arc::clone(&new_agent)),
    ])));
    let shutdown_signals = Arc::new(RwLock::new(HashMap::<String, InterruptSignal>::new()));
    let soft_interrupt_queues: SessionInterruptQueues = Arc::new(RwLock::new(HashMap::new()));
    let now = Instant::now();
    let client_connections = Arc::new(RwLock::new(HashMap::from([
        (
            "conn_existing".to_string(),
            ClientConnectionInfo {
                client_id: "conn_existing".to_string(),
                session_id: target_session_id.to_string(),
                client_instance_id: None,
                debug_client_id: Some("debug_existing".to_string()),
                connected_at: now,
                last_seen: now,
                is_processing: true,
                current_tool_name: Some("bash".to_string()),
                terminal_env: Vec::new(),
                disconnect_tx: mpsc::unbounded_channel().0,
            },
        ),
        (
            "conn_new".to_string(),
            ClientConnectionInfo {
                client_id: "conn_new".to_string(),
                session_id: temp_session_id.to_string(),
                client_instance_id: None,
                debug_client_id: Some("debug_new".to_string()),
                connected_at: now,
                last_seen: now,
                is_processing: false,
                current_tool_name: None,
                terminal_env: Vec::new(),
                disconnect_tx: mpsc::unbounded_channel().0,
            },
        ),
    ])));
    let rename_source_member = test_swarm_member("rename_source", "ready");
    let swarm_members = Arc::new(RwLock::new(HashMap::from([(
        "rename_source".to_string(),
        rename_source_member,
    )])));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(
        "swarm-test".to_string(),
        HashSet::from(["rename_source".to_string()]),
    )])));
    let file_touch = FileTouchService::new();
    let channel_subscriptions = Arc::new(RwLock::new(HashMap::<
        String,
        HashMap<String, HashSet<String>>,
    >::new()));
    let channel_subscriptions_by_session = Arc::new(RwLock::new(HashMap::<
        String,
        HashMap<String, HashSet<String>>,
    >::new()));
    let swarm_plans = Arc::new(RwLock::new(HashMap::<String, VersionedPlan>::new()));
    let swarm_coordinators = Arc::new(RwLock::new(HashMap::<String, String>::new()));
    let client_count = Arc::new(RwLock::new(2usize));
    let (writer, peer_stream) = test_writer()?;
    let (client_event_tx, mut client_event_rx) = mpsc::unbounded_channel::<ServerEvent>();
    let event_history = Arc::new(RwLock::new(VecDeque::<SwarmEvent>::new()));
    let event_counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let (swarm_event_tx, _swarm_event_rx) = broadcast::channel::<SwarmEvent>(8);
    let mcp_pool = Arc::new(crate::mcp::SharedMcpPool::from_default_config());

    let mut client_selfdev = false;
    let mut client_session_id = temp_session_id.to_string();
    let _busy_guard = existing_agent.lock().await;

    handle_resume_session(
        77,
        target_session_id.to_string(),
        None,
        None,
        false,
        false,
        &mut client_selfdev,
        &mut client_session_id,
        "conn_new",
        &new_agent,
        &provider,
        &new_registry,
        &sessions,
        &shutdown_signals,
        &soft_interrupt_queues,
        &client_connections,
        &Arc::new(RwLock::new(ClientDebugState::default())),
        &swarm_members,
        &swarms_by_id,
        &file_touch,
        &channel_subscriptions,
        &channel_subscriptions_by_session,
        &swarm_plans,
        &swarm_coordinators,
        &client_count,
        &writer,
        "test-server",
        "🌿",
        &client_event_tx,
        &mcp_pool,
        &event_history,
        &event_counter,
        &swarm_event_tx,
    )
    .await?;

    // The desktop follows a target-aware resume with the normal subscribe
    // bookkeeping while the original client can still be processing. That
    // bookkeeping must never wait for the live agent lock, otherwise the
    // desktop's immediately following state request remains unread and times
    // out after ten seconds.
    // Reproduce the lock-order cycle deterministically. The rename queues for
    // the member map first. Coordinator election then snapshots the swarm
    // index and pauses at the test hook while holding no index guard. Finally,
    // subscribe queues for the member map. Once released, rename can update the
    // index, coordinator election can inspect members, and subscribe can finish.
    let member_map_guard = swarm_members.write().await;
    let (rename_queued_tx, rename_queued_rx) = oneshot::channel();
    let rename_task = tokio::spawn({
        let swarm_members = Arc::clone(&swarm_members);
        let swarms_by_id = Arc::clone(&swarms_by_id);
        async move {
            let _ = rename_queued_tx.send(());
            rename_swarm_member_session(
                "rename_source",
                "rename_target",
                &swarm_members,
                &swarms_by_id,
            )
            .await;
        }
    });
    rename_queued_rx.await?;

    let (snapshot_complete_tx, snapshot_complete_rx) = oneshot::channel();
    let election_task = tokio::spawn({
        let swarm_members = Arc::clone(&swarm_members);
        let swarms_by_id = Arc::clone(&swarms_by_id);
        async move {
            crate::server::swarm::elect_swarm_coordinator_candidate_after_snapshot_for_test(
                "swarm-test",
                &swarm_members,
                &swarms_by_id,
                snapshot_complete_tx,
            )
            .await
        }
    });
    snapshot_complete_rx.await?;

    let subscribe_task = tokio::spawn({
        let existing_agent = Arc::clone(&existing_agent);
        let new_registry = new_registry.clone();
        let swarm_members = Arc::clone(&swarm_members);
        let swarms_by_id = Arc::clone(&swarms_by_id);
        let channel_subscriptions = Arc::clone(&channel_subscriptions);
        let channel_subscriptions_by_session = Arc::clone(&channel_subscriptions_by_session);
        let swarm_plans = Arc::clone(&swarm_plans);
        let swarm_coordinators = Arc::clone(&swarm_coordinators);
        let client_event_tx = client_event_tx.clone();
        let mcp_pool = Arc::clone(&mcp_pool);
        let event_history = Arc::clone(&event_history);
        let event_counter = Arc::clone(&event_counter);
        let swarm_event_tx = swarm_event_tx.clone();
        async move {
            let mut subscribe_client_selfdev = false;
            handle_subscribe(
                77,
                Some("/tmp/jcode-busy-desktop-attach".to_string()),
                Some(true),
                false,
                &mut subscribe_client_selfdev,
                target_session_id,
                "conn_new",
                &None,
                &existing_agent,
                &new_registry,
                true,
                &swarm_members,
                &swarms_by_id,
                &channel_subscriptions,
                &channel_subscriptions_by_session,
                &swarm_plans,
                &swarm_coordinators,
                &client_event_tx,
                &mcp_pool,
                &event_history,
                &event_counter,
                &swarm_event_tx,
            )
            .await;
        }
    });

    drop(member_map_guard);
    let (rename_result, election_result, subscribe_result) =
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            tokio::join!(rename_task, election_task, subscribe_task)
        })
        .await
        .expect("subscribe lock-order regression scenario must settle");
    rename_result.expect("session rename task");
    election_result.expect("coordinator election task");
    subscribe_result.expect("subscribe bookkeeping task");

    // Resume and subscribe both answer request id 77, so each emits its own
    // Done. Collect both batches, otherwise the assertions below only ever see
    // resume's events and subscribe's are invisible.
    let mut events = collect_events_until_done(&mut client_event_rx, 77).await;
    events.extend(collect_events_until_done(&mut client_event_rx, 77).await);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, ServerEvent::Done { id } if *id == 77)),
        "expected Done event for busy live attach, got {events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, ServerEvent::Error { .. })),
        "busy live attach should not emit error events: {events:?}"
    );
    // A remote (gateway) client has no other way to learn its session id, and
    // without it a dropped connection cannot reattach: the next Subscribe
    // carries no `target_session_id`, so the server hands it a fresh session
    // and the in-flight turn becomes unreachable.
    assert!(
        events.iter().any(|event| matches!(
            event,
            ServerEvent::SessionId { session_id } if session_id == target_session_id
        )),
        "subscribe must report the bound session id so clients can reattach: {events:?}"
    );

    let mut peer_reader = tokio::io::BufReader::new(peer_stream);
    let mut line = String::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(1),
        tokio::io::AsyncBufReadExt::read_line(&mut peer_reader, &mut line),
    )
    .await
    .expect("history should be written promptly")?;
    let event: ServerEvent = serde_json::from_str(line.trim())?;
    match event {
        ServerEvent::History {
            session_id,
            messages,
            ..
        } => {
            assert_eq!(session_id, target_session_id);
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].content, "persisted busy attach history");
        }
        other => panic!("expected history event, got {other:?}"),
    }

    restore_runtime_dir(prev_runtime);
    Ok(())
}
