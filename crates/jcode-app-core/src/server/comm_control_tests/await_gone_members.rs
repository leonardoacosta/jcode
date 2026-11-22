// A watched member that was present and then left the swarm can never finish.
// The live watcher must end early (never as success), while members that were
// never observed (registry empty before hydration) keep the wait pending.
async fn start_background_await(
    requester: &str,
    swarm_id: &str,
    peers: &[&str],
    mode: Option<&str>,
    present: &[(&str, &str)],
) -> (
    Arc<RwLock<HashMap<String, SwarmMember>>>,
    Arc<RwLock<HashMap<String, HashSet<String>>>>,
    broadcast::Sender<SwarmEvent>,
    tokio::sync::broadcast::Receiver<crate::bus::BusEvent>,
) {
    let mut members = HashMap::from([(requester.to_string(), member(requester, swarm_id, "ready"))]);
    let mut ids = HashSet::from([requester.to_string()]);
    for (id, status) in present {
        members.insert((*id).to_string(), member(id, swarm_id, status));
        ids.insert((*id).to_string());
    }
    let swarm_members = Arc::new(RwLock::new(members));
    let swarms_by_id = Arc::new(RwLock::new(HashMap::from([(swarm_id.to_string(), ids)])));
    let (swarm_event_tx, _rx) = broadcast::channel(32);
    let bus = crate::bus::Bus::global().subscribe();
    let (client_tx, mut client_rx) = mpsc::unbounded_channel();
    handle_comm_await_members(
        1,
        requester.to_string(),
        vec!["completed".to_string()],
        peers.iter().map(|id| (*id).to_string()).collect(),
        mode.map(str::to_string),
        Some(60),
        true,
        true,
        true,
        CommAwaitMembersContext {
            client_event_tx: &client_tx,
            swarm_members: &swarm_members,
            swarms_by_id: &swarms_by_id,
            swarm_event_tx: &swarm_event_tx,
            await_members_runtime: &AwaitMembersRuntime::default(),
        },
    )
    .await;
    match client_rx.recv().await {
        Some(ServerEvent::CommAwaitMembersResponse {
            background_started: true,
            ..
        }) => {}
        other => panic!("expected background start, got {other:?}"),
    }
    (swarm_members, swarms_by_id, swarm_event_tx, bus)
}

async fn next_await_event(
    bus: &mut tokio::sync::broadcast::Receiver<crate::bus::BusEvent>,
    requester: &str,
    wait: Duration,
) -> Option<crate::bus::SwarmAwaitCompleted> {
    tokio::time::timeout(wait, async {
        loop {
            if let Ok(crate::bus::BusEvent::SwarmAwaitCompleted(event)) = bus.recv().await
                && event.session_id == requester
            {
                return event;
            }
        }
    })
    .await
    .ok()
}

async fn remove_peer(
    peer: &str,
    swarm_id: &str,
    swarm_members: &Arc<RwLock<HashMap<String, SwarmMember>>>,
    swarms_by_id: &Arc<RwLock<HashMap<String, HashSet<String>>>>,
    swarm_event_tx: &broadcast::Sender<SwarmEvent>,
) {
    swarm_members.write().await.remove(peer);
    swarms_by_id
        .write()
        .await
        .get_mut(swarm_id)
        .expect("swarm exists")
        .remove(peer);
    let _ = swarm_event_tx.send(swarm_event(
        peer,
        swarm_id,
        SwarmEventType::MemberChange {
            action: "left".to_string(),
        },
    ));
}

#[tokio::test]
async fn await_members_ends_early_when_pending_member_left_swarm() {
    let (_env, _runtime) = RuntimeEnvGuard::new();
    let (swarm_id, requester) = ("swarm-gone-1", "req-gone-1");
    let (done_peer, gone_peer) = ("peer-done", "peer-gone");
    let (members, swarms, events, mut bus) = start_background_await(
        requester,
        swarm_id,
        &[done_peer, gone_peer],
        None,
        &[(done_peer, "running"), (gone_peer, "running")],
    )
    .await;

    members.write().await.get_mut(done_peer).unwrap().status = "completed".to_string();
    remove_peer(gone_peer, swarm_id, &members, &swarms, &events).await;

    let event = next_await_event(&mut bus, requester, Duration::from_secs(5))
        .await
        .expect("wait should end well before its 60s deadline");
    assert!(!event.completed, "a missing member is never success");
    assert!(!event.wake, "nothing actionable must not wake the agent");
    assert!(event.notify, "notify stays as configured");
    assert!(event.summary.contains("Ended early"), "{}", event.summary);
    assert!(event.summary.contains(gone_peer), "{}", event.summary);
}

#[tokio::test]
async fn await_members_never_observed_member_stays_pending() {
    let (_env, _runtime) = RuntimeEnvGuard::new();
    let (swarm_id, requester) = ("swarm-gone-2", "req-gone-2");
    let (members, _swarms, events, mut bus) = start_background_await(
        requester,
        swarm_id,
        &["peer-ghost"],
        None,
        &[],
    )
    .await;

    let _ = events.send(swarm_event(
        "peer-ghost",
        swarm_id,
        SwarmEventType::MemberChange {
            action: "left".to_string(),
        },
    ));
    drop(members);
    assert!(
        next_await_event(&mut bus, requester, Duration::from_millis(500))
            .await
            .is_none(),
        "unknown-but-never-seen members must keep waiting (reload hydration)"
    );
}

#[tokio::test]
async fn await_members_any_mode_keeps_waiting_when_other_member_still_running() {
    let (_env, _runtime) = RuntimeEnvGuard::new();
    let (swarm_id, requester) = ("swarm-gone-3", "req-gone-3");
    let (gone_peer, live_peer) = ("peer-gone", "peer-live");
    let (members, swarms, events, mut bus) = start_background_await(
        requester,
        swarm_id,
        &[gone_peer, live_peer],
        Some("any"),
        &[(gone_peer, "running"), (live_peer, "running")],
    )
    .await;

    remove_peer(gone_peer, swarm_id, &members, &swarms, &events).await;
    assert!(
        next_await_event(&mut bus, requester, Duration::from_millis(500))
            .await
            .is_none(),
        "a running member can still satisfy the wait"
    );

    members.write().await.get_mut(live_peer).unwrap().status = "completed".to_string();
    let _ = events.send(swarm_event(
        live_peer,
        swarm_id,
        SwarmEventType::StatusChange {
            old_status: "running".to_string(),
            new_status: "completed".to_string(),
        },
    ));
    let event = next_await_event(&mut bus, requester, Duration::from_secs(5))
        .await
        .expect("live member completion resolves the wait");
    assert!(event.completed);
}
