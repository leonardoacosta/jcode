#[tokio::test]
async fn council_three_seat_run_is_read_only_bounded_and_stops_all_seats() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(project.path())
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(project.path().join("evidence.txt"), "unchanged").unwrap();
    let git_status = || {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "-z", "--untracked-files=all"])
            .current_dir(project.path())
            .output()
            .unwrap();
        assert!(output.status.success());
        output.stdout
    };
    let before = git_status();
    let socket = runtime.path().join("council.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime.path());
    let _home = EnvGuard::set("JCODE_HOME", runtime.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket);
    let _swarm_model = EnvGuard::remove("JCODE_SWARM_MODEL");
    crate::config::invalidate_config_cache();

    struct CouncilProvider(
        Arc<std::sync::atomic::AtomicUsize>,
        Arc<std::sync::atomic::AtomicBool>,
    );
    #[async_trait]
    impl Provider for CouncilProvider {
        async fn complete(
            &self,
            messages: &[Message],
            tools: &[ToolDefinition],
            system: &str,
            resume: Option<&str>,
        ) -> Result<EventStream> {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            assert!(
                tools
                    .iter()
                    .all(|tool| crate::tool::council::SEAT_ALLOWED_TOOLS
                        .contains(&tool.name.as_str())),
                "seat advertised a mutating tool"
            );
            // Keep only the cancellation probe mid-opening.
            DelayedTestProvider {
                delay: Duration::from_millis(if self.1.load(std::sync::atomic::Ordering::SeqCst) {
                    30_000
                } else {
                    1
                }),
            }
            .complete(messages, tools, system, resume)
            .await
        }
        fn name(&self) -> &str {
            "test"
        }
        fn fork(&self) -> Arc<dyn Provider> {
            Arc::new(Self(self.0.clone(), self.1.clone()))
        }
    }
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let delay_opening = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let server = Arc::new(Server::new(Arc::new(CouncilProvider(
        calls.clone(),
        delay_opening.clone(),
    ))));
    let mut server_task = tokio::spawn(async move { server.run().await });
    wait_for_server_socket(&socket, &mut server_task)
        .await
        .unwrap();
    let mut owner = RawClient::connect(&socket).await.unwrap();
    owner.subscribe(project.path()).await.unwrap();
    let session_id = owner.session_id().await.unwrap();
    let ctx = ToolContext {
        session_id: session_id.clone(),
        message_id: "council-run".into(),
        tool_call_id: "council-run".into(),
        working_dir: Some(project.path().to_path_buf()),
        stdin_request_tx: None,
        graceful_shutdown_signal: None,
        execution_mode: ToolExecutionMode::Direct,
    };
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        crate::tool::council::CouncilTool::new().execute(
            json!({
                "action": "run", "topic": "Review the evidence without changing it", "rounds": 1,
                "max_calls": 7,
                "seats": [
                    {"name": "safety", "brief": "Review safety"},
                    {"name": "cost", "brief": "Review cost"},
                    {"name": "simplicity", "brief": "Review simplicity"}
                ]
            }),
            ctx.clone(),
        ),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 7);
    let metadata = result.metadata.unwrap();
    assert_eq!(metadata["council"]["calls"], 7);
    for seat in metadata["council"]["seats"].as_array().unwrap() {
        let id = seat["session_id"].as_str().unwrap();
        assert!(matches!(
            crate::session::Session::load(id).unwrap().status,
            crate::session::SessionStatus::Closed
        ));
    }
    let members = owner.comm_list(&session_id).await.unwrap();
    assert!(
        members.iter().all(|member| member.session_id == session_id),
        "council seats still live: {members:?}"
    );
    for obstruction in ["directory", "unwritable"] {
        let output = crate::tool::council::CouncilTool::new()
            .execute(json!({
                "action": "open", "topic": "Persistence failure cleanup", "rounds": 0,
                "seats": [{"name": "one", "brief": "First view"}, {"name": "two", "brief": "Second view"}]
            }), ctx.clone()).await.unwrap();
        let id = output.output.split_whitespace().nth(1).unwrap();
        let transcript = crate::session::session_path(&ctx.session_id)
            .unwrap()
            .parent()
            .unwrap()
            .join(format!("{}.council-{id}.json", ctx.session_id));
        let permissions = std::fs::metadata(&transcript).unwrap().permissions();
        if obstruction == "directory" {
            std::fs::remove_file(&transcript).unwrap();
            std::fs::create_dir(&transcript).unwrap();
        } else {
            let mut readonly = permissions.clone();
            readonly.set_readonly(true);
            std::fs::set_permissions(&transcript, readonly).unwrap();
        }
        let seats: Vec<_> = owner
            .comm_list(&session_id)
            .await
            .unwrap()
            .into_iter()
            .filter(|member| member.session_id != session_id)
            .map(|member| member.session_id)
            .collect();
        assert_eq!(seats.len(), 2);
        let result = crate::tool::council::CouncilTool::new()
            .execute(json!({"action": "round", "council_id": id}), ctx.clone())
            .await;
        if obstruction == "unwritable" {
            std::fs::set_permissions(&transcript, permissions).unwrap();
        }
        assert!(
            result.is_err(),
            "{obstruction} transcript persistence failure was suppressed"
        );
        assert!(format!("{:#}", result.unwrap_err()).contains("persist council transcript"));
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if owner
                    .comm_list(&session_id)
                    .await
                    .unwrap()
                    .iter()
                    .all(|member| member.session_id == session_id)
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("persistence failure left live seats");
        for seat in seats {
            assert!(matches!(
                crate::session::Session::load(&seat).unwrap().status,
                crate::session::SessionStatus::Closed
            ));
        }
    }
    let calls_before_opening = calls.load(std::sync::atomic::Ordering::SeqCst);
    delay_opening.store(true, std::sync::atomic::Ordering::SeqCst);
    let opening = tokio::spawn(async move {
        crate::tool::council::CouncilTool::new().execute(json!({
            "action": "open", "topic": "Abort during opening", "rounds": 0,
            "seats": [{"name": "one", "brief": "First view"}, {"name": "two", "brief": "Second view"}]
        }), ctx).await
    });
    tokio::time::timeout(Duration::from_secs(5), async {
        while calls.load(std::sync::atomic::Ordering::SeqCst) == calls_before_opening {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    opening.abort();
    assert!(opening.await.unwrap_err().is_cancelled());
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let members = owner.comm_list(&session_id).await.unwrap();
            if members.iter().all(|member| member.session_id == session_id) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("cancelled opening left live seats");
    assert_eq!(
        std::fs::read_to_string(project.path().join("evidence.txt")).unwrap(),
        "unchanged"
    );
    assert_eq!(git_status(), before, "council modified the worktree");
    server_task.abort();
}

#[tokio::test]
async fn communicate_list_and_await_members_work_end_to_end() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    // Keep the developer's real config (agents.swarm_model, providers) out of spawns.
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");
    // Independently created root sessions own separate swarms (83dbc36dc);
    // opt both clients into one shared swarm explicitly.
    let _swarm = EnvGuard::set("JCODE_SWARM_ID", "communicate-e2e-shared-swarm");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(300),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };

    let socket_path = runtime_dir.path().join("jcode.sock");
    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut watcher = RawClient::connect(&socket_path)
        .await
        .expect("watcher should connect");
    let mut peer = RawClient::connect(&socket_path)
        .await
        .expect("peer should connect");
    watcher
        .subscribe(&repo_dir)
        .await
        .expect("watcher subscribe");
    peer.subscribe(&repo_dir).await.expect("peer subscribe");

    let watcher_session = watcher.session_id().await.expect("watcher session id");
    let peer_session = peer.session_id().await.expect("peer session id");

    let tool = CommunicateTool::new();
    let ctx = test_ctx(&watcher_session, &repo_dir);

    let list_output = tool
        .execute(json!({"action": "list"}), ctx.clone())
        .await
        .expect("communicate list should succeed");
    assert!(
        list_output.output.contains("Status: ready"),
        "expected communicate list to render member status, got: {}",
        list_output.output
    );

    let peer_message_id = peer
        .send_message("Reply with a short acknowledgement.")
        .await
        .expect("peer message request should send");

    let running_members =
        wait_for_member_status(&mut watcher, &watcher_session, &peer_session, "running")
            .await
            .expect("peer should enter running state");
    let running_peer = running_members
        .iter()
        .find(|member| member.session_id == peer_session)
        .expect("peer should be listed while running");
    assert_eq!(running_peer.status.as_deref(), Some("running"));

    // Default await_members starts a durable asynchronous wait.
    let await_output = tokio::time::timeout(
        Duration::from_secs(5),
        tool.execute(
            json!({
                "action": "await_members",
                "session_ids": [peer_session.clone()],
                "timeout_minutes": 1
            }),
            ctx.clone(),
        ),
    )
    .await
    .expect("background wait should return promptly")
    .expect("await_members should start");
    assert!(
        await_output.output.contains("runs in the background"),
        "expected background hand-off output, got: {}",
        await_output.output
    );

    peer.wait_for_done(peer_message_id)
        .await
        .expect("peer message should finish");

    let event = watcher
        .read_until(Duration::from_secs(5), |event| {
            matches!(
                event,
                ServerEvent::Notification {
                    notification_type: NotificationType::Message { scope: Some(scope), .. },
                    ..
                } if scope == "swarm_await"
            )
        })
        .await
        .expect("upgraded asynchronous wait should notify on completion");
    let ServerEvent::Notification { message, .. } = event else {
        panic!("expected swarm_await notification, got: {event:?}");
    };
    assert!(
        message.contains("(ready)"),
        "expected await_members to treat ready as done, got: {}",
        message
    );

    let ready_members =
        wait_for_member_status(&mut watcher, &watcher_session, &peer_session, "ready")
            .await
            .expect("peer should return to ready state");
    let ready_peer = ready_members
        .iter()
        .find(|member| member.session_id == peer_session)
        .expect("peer should still be listed when ready");
    assert_eq!(ready_peer.status.as_deref(), Some("ready"));

    server_task.abort();
}

#[tokio::test]
async fn communicate_await_members_background_returns_immediately_and_notifies() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    // Keep the developer's real config (agents.swarm_model, providers) out of spawns.
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");
    // Independently created root sessions own separate swarms (83dbc36dc);
    // opt both clients into one shared swarm explicitly.
    let _swarm = EnvGuard::set("JCODE_SWARM_ID", "communicate-e2e-shared-swarm");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(300),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };

    let socket_path = runtime_dir.path().join("jcode.sock");
    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut watcher = RawClient::connect(&socket_path)
        .await
        .expect("watcher should connect");
    let mut peer = RawClient::connect(&socket_path)
        .await
        .expect("peer should connect");
    watcher
        .subscribe(&repo_dir)
        .await
        .expect("watcher subscribe");
    peer.subscribe(&repo_dir).await.expect("peer subscribe");

    let watcher_session = watcher.session_id().await.expect("watcher session id");
    let peer_session = peer.session_id().await.expect("peer session id");

    let tool = CommunicateTool::new();
    let ctx = test_ctx(&watcher_session, &repo_dir);

    // Put the peer into a running state so the await actually has to wait.
    let peer_message_id = peer
        .send_message("Reply with a short acknowledgement.")
        .await
        .expect("peer message request should send");
    wait_for_member_status(&mut watcher, &watcher_session, &peer_session, "running")
        .await
        .expect("peer should enter running state");

    // Background await (the default) must return promptly with a hand-off
    // message instead of blocking until the peer finishes.
    let await_output = tokio::time::timeout(
        Duration::from_secs(5),
        tool.execute(
            json!({
                "action": "await_members",
                "session_ids": [peer_session.clone()],
                "timeout_minutes": 1
            }),
            ctx.clone(),
        ),
    )
    .await
    .expect("background await should return promptly")
    .expect("await_members should succeed");
    assert!(
        await_output.output.contains("background"),
        "expected background hand-off message, got: {}",
        await_output.output
    );

    peer.wait_for_done(peer_message_id)
        .await
        .expect("peer message should finish");

    // The backgrounded watcher should deliver a swarm-await notification to the
    // requesting (watcher) session once the peer reaches ready.
    let event = watcher
        .read_until(Duration::from_secs(5), |event| {
            matches!(
                event,
                ServerEvent::Notification {
                    notification_type: NotificationType::Message { scope: Some(scope), .. },
                    ..
                } if scope == "swarm_await"
            )
        })
        .await
        .expect("background await should deliver a swarm_await notification");
    let ServerEvent::Notification { message, .. } = event else {
        panic!("expected swarm_await notification, got: {event:?}");
    };
    assert!(
        message.contains("Swarm await finished"),
        "expected swarm await completion body, got: {}",
        message
    );

    server_task.abort();
}

#[tokio::test]
async fn communicate_run_plan_with_empty_plan_returns_inline_even_in_background_mode() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    // Keep the developer's real config (agents.swarm_model, providers) out of spawns.
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(50),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };

    let socket_path = runtime_dir.path().join("jcode.sock");
    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut client = RawClient::connect(&socket_path)
        .await
        .expect("client should connect");
    client.subscribe(&repo_dir).await.expect("subscribe");
    let session = client.session_id().await.expect("session id");

    let tool = CommunicateTool::new();
    let ctx = test_ctx(&session, &repo_dir);

    // Background is the default; with no plan the validation happens inline and
    // no background task should be started.
    let output = tokio::time::timeout(
        Duration::from_secs(5),
        tool.execute(json!({"action": "run_plan"}), ctx.clone()),
    )
    .await
    .expect("run_plan should return promptly")
    .expect("run_plan should succeed");
    assert!(
        output.output.contains("No swarm plan items to run."),
        "expected inline empty-plan response, got: {}",
        output.output
    );
    assert!(
        output.metadata.is_none(),
        "empty plan must not start a background driver"
    );

    server_task.abort();
}

#[tokio::test]
async fn communicate_status_returns_busy_snapshot_for_running_member() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    // Keep the developer's real config (agents.swarm_model, providers) out of spawns.
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");
    // Independently created root sessions own separate swarms (83dbc36dc);
    // opt both clients into one shared swarm explicitly.
    let _swarm = EnvGuard::set("JCODE_SWARM_ID", "communicate-e2e-shared-swarm");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(300),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };

    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut watcher = RawClient::connect(&socket_path)
        .await
        .expect("watcher should connect");
    let mut peer = RawClient::connect(&socket_path)
        .await
        .expect("peer should connect");
    watcher
        .subscribe(&repo_dir)
        .await
        .expect("watcher subscribe");
    peer.subscribe(&repo_dir).await.expect("peer subscribe");

    let watcher_session = watcher.session_id().await.expect("watcher session id");
    let peer_session = peer.session_id().await.expect("peer session id");
    let tool = CommunicateTool::new();
    let ctx = test_ctx(&watcher_session, &repo_dir);

    let peer_message_id = peer
        .send_message("Reply with a short acknowledgement.")
        .await
        .expect("peer message request should send");

    wait_for_member_status(&mut watcher, &watcher_session, &peer_session, "running")
        .await
        .expect("peer should enter running state");

    let snapshot = watcher
        .comm_status(&watcher_session, &peer_session)
        .await
        .expect("comm_status should succeed while peer is busy");
    assert_eq!(snapshot.session_id, peer_session);
    assert_eq!(snapshot.status.as_deref(), Some("running"));
    assert!(
        snapshot
            .activity
            .as_ref()
            .is_some_and(|activity| activity.is_processing)
    );

    let output = tool
        .execute(
            json!({
                "action": "status",
                "target_session": peer_session.clone()
            }),
            ctx,
        )
        .await
        .expect("status action should succeed");
    assert!(output.output.contains("Lifecycle: running"));
    assert!(output.output.contains("Activity: busy"));

    peer.wait_for_done(peer_message_id)
        .await
        .expect("peer message should finish");

    server_task.abort();
}

#[tokio::test]
async fn communicate_spawn_reports_completion_back_to_spawner() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    // Keep the developer's real config (agents.swarm_model, providers) out of spawns.
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(100),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };

    let socket_path = runtime_dir.path().join("jcode.sock");
    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut watcher = RawClient::connect(&socket_path)
        .await
        .expect("watcher should connect");
    watcher
        .subscribe(&repo_dir)
        .await
        .expect("watcher subscribe");

    let watcher_session = watcher.session_id().await.expect("watcher session id");
    let tool = CommunicateTool::new();
    let ctx = test_ctx(&watcher_session, &repo_dir);

    let spawn_output = tool
        .execute(
            json!({
                "action": "spawn",
                "label": "report-back worker",
                "prompt": "Reply with exactly AUTH_TEST_OK and nothing else."
            }),
            ctx,
        )
        .await
        .expect("spawn with prompt should succeed");
    let spawned_session = spawn_output
        .output
        .strip_prefix("Spawned new agent: ")
        .expect("spawn output should include session id")
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();

    watcher
        .read_until(Duration::from_secs(15), |event| {
            matches!(
                event,
                ServerEvent::Notification {
                    from_session,
                    notification_type: crate::protocol::NotificationType::Message {
                        scope: Some(scope),
                        channel: None,
                        tldr: None,
                    },
                    message,
                    ..
                } if from_session == &spawned_session
                    && scope == "swarm"
                    && message.contains("finished their work and is ready for more")
            )
        })
        .await
        .expect("spawner should receive completion report-back notification");

    server_task.abort();
}

#[tokio::test]
async fn communicate_spawn_with_prompt_and_summary_work_end_to_end() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    // Keep the developer's real config (agents.swarm_model, providers) out of spawns.
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(100),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };

    let socket_path = runtime_dir.path().join("jcode.sock");
    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut watcher = RawClient::connect(&socket_path)
        .await
        .expect("watcher should connect");
    watcher
        .subscribe(&repo_dir)
        .await
        .expect("watcher subscribe");

    let watcher_session = watcher.session_id().await.expect("watcher session id");
    let tool = CommunicateTool::new();
    let ctx = test_ctx(&watcher_session, &repo_dir);

    let spawn_output = tool
        .execute(
            json!({
                "action": "spawn",
                "label": "summary worker",
                "prompt": "Reply with a short acknowledgement."
            }),
            ctx.clone(),
        )
        .await
        .expect("spawn with prompt should succeed");
    let spawned_session = spawn_output
        .output
        .strip_prefix("Spawned new agent: ")
        .expect("spawn output should include session id")
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    assert!(
        !spawned_session.is_empty(),
        "spawned session id should not be empty"
    );

    wait_for_member_presence(&mut watcher, &watcher_session, &spawned_session)
        .await
        .expect("spawned member should appear in swarm list");

    let summary_output = {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            match tool
                .execute(
                    json!({
                        "action": "summary",
                        "target_session": spawned_session
                    }),
                    ctx.clone(),
                )
                .await
            {
                Ok(output) => break output,
                Err(err)
                    if (err.to_string().contains("Unknown session")
                        || err.to_string().contains(" is busy;"))
                        && tokio::time::Instant::now() < deadline =>
                {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
                Err(err) => panic!("summary for spawned agent should succeed: {err}"),
            }
        }
    };
    assert!(
        summary_output.output.contains("Tool call summary for")
            || summary_output.output.contains("No tool calls found for"),
        "unexpected summary output: {}",
        summary_output.output
    );

    server_task.abort();
}

/// `message` routes by the fields supplied (DM when `to_session` is set,
/// broadcast otherwise), while `broadcast` is a group send scoped to the
/// sender's spawned subtree (whole swarm when the sender is the coordinator).
/// Regression test for the bug where `message` and `broadcast` were identical
/// because the tool discarded `to_session`/`channel` for both.
#[tokio::test]
async fn communicate_message_routes_as_dm_while_broadcast_targets_swarm() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    // Keep the developer's real config (agents.swarm_model, providers) out of spawns.
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");
    // Independently created root sessions own separate swarms (83dbc36dc);
    // opt both clients into one shared swarm explicitly.
    let _swarm = EnvGuard::set("JCODE_SWARM_ID", "communicate-e2e-shared-swarm");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(100),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };

    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut sender = RawClient::connect(&socket_path)
        .await
        .expect("sender should connect");
    let mut peer = RawClient::connect(&socket_path)
        .await
        .expect("peer should connect");
    sender.subscribe(&repo_dir).await.expect("sender subscribe");
    peer.subscribe(&repo_dir).await.expect("peer subscribe");

    let sender_session = sender.session_id().await.expect("sender session id");
    let peer_session = peer.session_id().await.expect("peer session id");

    // Ensure both sessions are part of the same swarm before messaging.
    wait_for_member_presence(&mut sender, &sender_session, &peer_session)
        .await
        .expect("peer should join the swarm");

    let tool = CommunicateTool::new();
    let ctx = test_ctx(&sender_session, &repo_dir);

    // `message` with a `to_session` should arrive at the peer scoped as a DM.
    let dm_output = tool
        .execute(
            json!({
                "action": "message",
                "message": "ping-dm",
                "to_session": peer_session.clone()
            }),
            ctx.clone(),
        )
        .await
        .expect("message with to_session should succeed");
    assert!(
        dm_output.output.contains("Direct message sent to"),
        "message with to_session should report a DM, got: {}",
        dm_output.output
    );
    let dm_scope = peer
        .next_message_notification(Duration::from_secs(5))
        .await
        .expect("peer should receive the targeted message");
    assert_eq!(
        dm_scope.as_deref(),
        Some("dm"),
        "message with to_session should be delivered with dm scope"
    );

    // A long DM without a sender summary must still deliver.
    let long_dm_output = tool
        .execute(
            json!({
                "action": "dm",
                "message": format!("long-dm {}", "x".repeat(244)),
                "to_session": peer_session.clone()
            }),
            ctx.clone(),
        )
        .await
        .expect("long DM without tldr should succeed");
    assert!(long_dm_output.output.contains("Direct message sent to"));
    assert_eq!(
        peer.next_message_notification(Duration::from_secs(5))
            .await
            .expect("peer should receive long DM")
            .as_deref(),
        Some("dm")
    );

    // Broadcasts are scoped to the sender's spawned subtree; the coordinator
    // keeps whole-swarm reach as an escape hatch. The peer was not spawned by
    // the sender, so promote the sender to coordinator (self-promotion is
    // allowed while the swarm has no coordinator) so the broadcast reaches it.
    // A non-coordinator `message` with no target is a mistake (forgot
    // to_session): it errors instead of broadcasting. The peer is a plain
    // agent here; the sender was elected coordinator on join.
    let peer_ctx = test_ctx(&peer_session, &repo_dir);
    let untargeted = tool
        .execute(
            json!({"action": "message", "message": "oops"}),
            peer_ctx.clone(),
        )
        .await;
    assert!(
        untargeted
            .as_ref()
            .err()
            .is_some_and(|e| e.to_string().contains("needs a target")),
        "worker message without a target should be rejected: {:?}",
        untargeted.map(|o| o.output)
    );
    // The same worker can still reach the coordinator directly.
    tool.execute(
        json!({"action": "message", "message": "hi boss", "to_session": sender_session.clone()}),
        peer_ctx,
    )
    .await
    .expect("worker message with a target still works");
    sender
        .next_message_notification(Duration::from_secs(5))
        .await
        .expect("coordinator should receive the worker's message");

    let assign_output = tool
        .execute(
            json!({
                "action": "assign_role",
                "target_session": sender_session.clone(),
                "role": "coordinator"
            }),
            ctx.clone(),
        )
        .await
        .expect("self-promotion to coordinator should succeed");
    assert!(
        assign_output.output.contains("Assigned role 'coordinator'"),
        "unexpected assign_role output: {}",
        assign_output.output
    );

    // The coordinator may message everyone by omitting the target.
    let to_all = tool
        .execute(
            json!({"action": "message", "message": "hello-all"}),
            ctx.clone(),
        )
        .await
        .expect("coordinator message without target should broadcast");
    assert!(
        to_all.output.contains("Broadcast sent"),
        "{}",
        to_all.output
    );
    assert_eq!(
        peer.next_message_notification(Duration::from_secs(5))
            .await
            .expect("peer should receive the coordinator's message")
            .as_deref(),
        Some("broadcast")
    );

    // `broadcast` should reach the peer scoped as a broadcast even though no
    // explicit target is supplied.
    let broadcast_output = tool
        .execute(
            json!({
                "action": "broadcast",
                "message": "ping-all"
            }),
            ctx.clone(),
        )
        .await
        .expect("broadcast should succeed");
    assert!(
        broadcast_output
            .output
            .contains("Broadcast sent to your spawned subtree"),
        "broadcast should report a subtree-scoped group send, got: {}",
        broadcast_output.output
    );
    let broadcast_scope = peer
        .next_message_notification(Duration::from_secs(5))
        .await
        .expect("peer should receive the broadcast");
    assert_eq!(
        broadcast_scope.as_deref(),
        Some("broadcast"),
        "broadcast should be delivered with broadcast scope"
    );

    server_task.abort();
}

#[tokio::test]
async fn communicate_retry_respawns_orphaned_worker_from_spawn_spec() {
    let _env_lock = crate::storage::lock_test_env();
    let runtime_dir = tempfile::TempDir::new().expect("runtime tempdir");
    let repo_dir = std::env::current_dir().expect("repo cwd");
    let socket_path = runtime_dir.path().join("jcode.sock");
    let _runtime = EnvGuard::set("JCODE_RUNTIME_DIR", runtime_dir.path());
    let _home = EnvGuard::set("JCODE_HOME", runtime_dir.path());
    let _socket = EnvGuard::set("JCODE_SOCKET", &socket_path);
    let _debug = EnvGuard::set("JCODE_DEBUG_CONTROL", "1");

    let provider: Arc<dyn Provider> = Arc::new(DelayedTestProvider {
        delay: Duration::from_millis(50),
    });
    let server = Arc::new(Server::new(provider));
    let mut server_task = {
        let server = Arc::clone(&server);
        tokio::spawn(async move { server.run().await })
    };
    wait_for_server_socket(&socket_path, &mut server_task)
        .await
        .expect("server socket should be ready");

    let mut watcher = RawClient::connect(&socket_path).await.expect("connect");
    watcher.subscribe(&repo_dir).await.expect("subscribe");
    let watcher_session = watcher.session_id().await.expect("watcher session id");
    let ctx = test_ctx(&watcher_session, &repo_dir);

    let orphan = "session_orphan_after_reload";
    let spec = crate::server::swarm_persistence::WorkerSpawnSpec {
        lane: "orphan lane".to_string(),
        prompt: Some("Reply with a short acknowledgement.".to_string()),
        model: None,
        working_dir: Some(repo_dir.display().to_string()),
        last_status: Some("orphaned".to_string()),
    };
    crate::server::swarm_persistence::record_worker_spawn_spec(orphan, &spec);

    let tool = CommunicateTool::new();
    let output = tool
        .execute(
            json!({"action": "retry", "target_session": orphan}),
            ctx.clone(),
        )
        .await
        .expect("retry of orphan should respawn");
    assert!(
        output.output.starts_with("Respawned orphaned worker"),
        "{}",
        output.output
    );
    let new_session = output
        .output
        .rsplit(' ')
        .next()
        .expect("new session id")
        .to_string();
    assert_ne!(new_session, orphan);

    let respawned = crate::server::swarm_persistence::load_worker_spawn_spec(&new_session)
        .expect("respawned worker records its spec");
    assert_eq!(respawned.lane, "orphan lane");
    assert_eq!(respawned.prompt, spec.prompt);
    assert_eq!(respawned.last_status, None);
    let consumed = crate::server::swarm_persistence::load_worker_spawn_spec(orphan)
        .expect("orphan spec retained");
    assert_eq!(consumed.last_status, None, "orphan marker consumed");

    // Second retry must not double-spawn through the orphan path.
    let again = tool
        .execute(json!({"action": "retry", "target_session": orphan}), ctx)
        .await;
    if let Ok(out) = &again {
        assert!(
            !out.output.contains("Respawned"),
            "double respawn: {}",
            out.output
        );
    }

    server_task.abort();
}
