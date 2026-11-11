use crate::test_support::*;

// ============================================================================
// Binary Integration Tests
// These tests run the actual jcode binary and require real credentials.
// Run with: cargo test --test e2e binary_integration -- --ignored
// ============================================================================

// ----------------------------------------------------------------------------
// Reload/handoff robustness coverage map (for future contributors)
//
// Unit-level (no credentials, run by default):
//   - server::reload_state::tests + server::socket_tests: marker/handoff state
//     machine (Ready/Waiting/Failed/Idle verdicts, dead-pid crash detection,
//     stale/foreign/completed marker cleanup, Failed-marker preservation,
//     corrupt-marker tolerance, bounded handoff-event wait).
//   - server::reload::reload_tests: graceful shutdown signaling, timeout, and
//     partial-checkpoint behavior; recovery-intent persistence for peers.
//   - server::reload_recovery::tests: recovery-store path-traversal safety,
//     persist/peek roundtrip, non-consuming directive peek, delivery
//     idempotency + continuation mismatch.
//   - server::util::reload_target_tests: no-downgrade exec-target guard.
//
// E2E (real spawned process, run with --ignored; need a release binary):
//   - binary_integration_reload_handoff: server identity changes, marker clears.
//   - binary_integration_selfdev_reload_reconnects_quickly: repeated reloads.
//   - binary_integration_selfdev_client_reload_resumes_session.
//   - binary_integration_selfdev_full_reload_resumes_session_quickly.
//
// Known E2E gaps worth adding when a release binary is available:
//   - Concurrent/rapid `client.reload()` calls collapsing into one handoff
//     without stranding the client or leaving a stuck marker.
//   - A pre-existing *foreign* stale reload marker (different pid) in the
//     runtime dir at boot being cleared rather than blocking startup.
//   - Crash-during-boot of the replacement server (e.g. point the reload
//     candidate at a binary that exits non-zero) resolving the waiting client
//     to a Failed verdict instead of an indefinite hang.
// ----------------------------------------------------------------------------

/// Test that the jcode binary can run independent with Claude provider
#[tokio::test]
#[ignore] // Requires Claude credentials
async fn binary_integration_independent_claude() -> Result<()> {
    use std::process::Command;
    let _env = setup_test_env()?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--bin",
            "jcode",
            "--",
            "run",
            "Say 'test-ok' and nothing else",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success() || stdout.contains("test") || stderr.contains("Claude"),
        "Binary should run successfully. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    Ok(())
}

/// Test that the jcode binary can run with OpenAI provider
#[tokio::test]
#[ignore] // Requires OpenAI/Codex credentials
async fn binary_integration_openai_provider() -> Result<()> {
    use std::process::Command;
    let _env = setup_test_env()?;

    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--bin",
            "jcode",
            "--",
            "--provider",
            "openai",
            "run",
            "Say 'openai-ok' and nothing else",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check either success or identifiable OpenAI response
    let has_response = stdout.to_lowercase().contains("openai")
        || stdout.to_lowercase().contains("ok")
        || stderr.contains("OpenAI");

    assert!(
        output.status.success() || has_response,
        "OpenAI provider should work. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    Ok(())
}

/// Test that jcode version command works
#[tokio::test]
async fn binary_version_command() -> Result<()> {
    use std::process::Command;
    let _env = setup_test_env()?;

    let output = Command::new(env!("CARGO_BIN_EXE_jcode"))
        .arg("--version")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Version command should succeed");
    assert!(
        stdout.contains("jcode") || stdout.contains("20"),
        "Version should contain 'jcode' or date. Got: {}",
        stdout
    );

    Ok(())
}

/// Test full server reload handoff against a real spawned server process.
///
/// Requires a built release binary at target/release/jcode because the reload
/// flow execs into the repo's reload candidate.
#[tokio::test]
#[ignore]
async fn binary_integration_reload_handoff() -> Result<()> {
    let _env = setup_test_env()?;

    let release_binary =
        jcode::build::release_binary_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    if !release_binary.exists() {
        anyhow::bail!(
            "release binary missing at {} (run `cargo build --release` first)",
            release_binary.display()
        );
    }

    let temp_root = tempfile::Builder::new()
        .prefix("jcode-reload-e2e-")
        .tempdir()?;
    let runtime_dir = temp_root.path().join("runtime");
    let home_dir = temp_root.path().join("home");
    let install_dir = temp_root.path().join("install");
    let stderr_path = temp_root.path().join("server-stderr.log");
    std::fs::create_dir_all(&runtime_dir)?;
    std::fs::create_dir_all(&home_dir)?;
    std::fs::create_dir_all(&install_dir)?;

    let socket_path = runtime_dir.join("jcode.sock");
    let debug_socket_path = runtime_dir.join("jcode-debug.sock");

    let stderr_file = std::fs::File::create(&stderr_path)?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_jcode"))
        .arg("--no-update")
        .arg("--socket")
        .arg(&socket_path)
        .arg("serve")
        // This test must exercise the real exec-based reload handoff, not the
        // in-process test shortcut used by other e2e cases.
        .env_remove("JCODE_TEST_SESSION")
        .env("JCODE_HOME", &home_dir)
        .env("JCODE_RUNTIME_DIR", &runtime_dir)
        .env("JCODE_INSTALL_DIR", &install_dir)
        .env("JCODE_DEBUG_CONTROL", "1")
        .env("JCODE_TEMP_SERVER", "1")
        .env("JCODE_SERVER_OWNER_PID", std::process::id().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(stderr_file))
        .spawn()?;

    let test_result = async {
        wait_for_server_ready(&socket_path, &debug_socket_path).await?;
        let server_info_before =
            debug_run_command(debug_socket_path.clone(), "server:info", None).await?;
        let server_info_before_json: serde_json::Value = serde_json::from_str(&server_info_before)?;
        let server_id_before = server_info_before_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing server id before reload"))?
            .to_string();

        let mut client = wait_for_server_client(&socket_path).await?;
        client.reload().await?;

        let disconnect_deadline = Instant::now() + Duration::from_secs(10);
        let mut saw_disconnect = false;
        while Instant::now() < disconnect_deadline {
            match tokio::time::timeout(Duration::from_secs(1), client.read_event()).await {
                Ok(Ok(_)) => continue,
                Ok(Err(_)) | Err(_) => {
                    saw_disconnect = true;
                    break;
                }
            }
        }
        assert!(
            saw_disconnect,
            "old client connection never disconnected during reload"
        );

        let marker_deadline = Instant::now() + Duration::from_secs(20);
        while jcode::server::reload_marker_active(Duration::from_secs(30)) {
            if Instant::now() >= marker_deadline {
                anyhow::bail!("reload marker remained active too long after restart");
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        wait_for_server_ready(&socket_path, &debug_socket_path).await?;
        let _client = wait_for_server_client(&socket_path).await?;

        let server_info_after =
            debug_run_command(debug_socket_path.clone(), "server:info", None).await?;
        let server_info_after_json: serde_json::Value = serde_json::from_str(&server_info_after)?;
        let server_id_after = server_info_after_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing server id after reload"))?;

        assert_ne!(
            server_id_after, server_id_before,
            "server identity should change after exec-based reload"
        );
        assert!(
            server_info_after_json
                .get("uptime_secs")
                .and_then(|v| v.as_u64())
                .is_some(),
            "replacement server should answer debug state queries after reload"
        );

        Ok::<_, anyhow::Error>(())
    }
    .await;

    kill_child(&mut child);
    if let Err(ref error) = test_result {
        if let Ok(stderr) = std::fs::read_to_string(&stderr_path) {
            eprintln!("spawned server stderr:\n{}", stderr);
        }
        eprintln!("reload e2e test error: {error:#}");
    }
    test_result
}

/// Test repeated self-dev reload handoff against a real TUI client running in a PTY.
///
/// Requires a built release binary at target/release/jcode because the
/// self-dev server reload path execs into the repo's reload candidate.
#[cfg(unix)]
#[tokio::test]
#[ignore]
async fn binary_integration_selfdev_reload_reconnects_quickly() -> Result<()> {
    let _env = setup_test_env()?;

    let release_binary =
        jcode::build::release_binary_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    if !release_binary.exists() {
        anyhow::bail!(
            "release binary missing at {} (run `cargo build --release` first)",
            release_binary.display()
        );
    }

    let temp_root = tempfile::Builder::new()
        .prefix("jcode-selfdev-reload-e2e-")
        .tempdir()?;
    let runtime_dir = temp_root.path().join("runtime");
    let home_dir = temp_root.path().join("home");
    let install_dir = temp_root.path().join("install");
    std::fs::create_dir_all(&runtime_dir)?;
    std::fs::create_dir_all(&home_dir)?;
    std::fs::create_dir_all(&install_dir)?;

    let _home_guard = EnvVarGuard::set("JCODE_HOME", &home_dir);
    let _runtime_guard = EnvVarGuard::set("JCODE_RUNTIME_DIR", &runtime_dir);
    let _install_guard = EnvVarGuard::set("JCODE_INSTALL_DIR", &install_dir);

    let socket_path = runtime_dir.join("jcode.sock");
    let debug_socket_path = runtime_dir.join("jcode-debug.sock");
    let mut command = Command::new(&release_binary);
    command
        .arg("--no-update")
        .arg("--provider")
        .arg("antigravity")
        .arg("self-dev")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env_remove("JCODE_TEST_SESSION")
        .env("JCODE_HOME", &home_dir)
        .env("JCODE_RUNTIME_DIR", &runtime_dir)
        .env("JCODE_INSTALL_DIR", &install_dir);

    let mut child = spawn_pty_child(command)?;

    let test_result = async {
        wait_for_server_ready(&socket_path, &debug_socket_path).await?;
        let session_id = wait_for_default_connected_client_session(&debug_socket_path).await?;

        let state_before =
            debug_run_command(debug_socket_path.clone(), "client:state", None).await?;
        let _: serde_json::Value = serde_json::from_str(&state_before)?;

        let server_info_before =
            debug_run_command(debug_socket_path.clone(), "server:info", None).await?;
        let server_info_before_json: serde_json::Value = serde_json::from_str(&server_info_before)?;
        let mut server_id_before = server_info_before_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing initial server id"))?
            .to_string();

        for cycle in 1..=3 {
            child.send_command("/server-reload")?;

            let server_id_after = wait_for_selfdev_reload_cycle(
                &debug_socket_path,
                &session_id,
                &server_id_before,
                Duration::from_secs(20),
            )
            .await?;
            assert_ne!(
                server_id_after, server_id_before,
                "self-dev reload cycle {} should replace the server process",
                cycle
            );
            server_id_before = server_id_after;
        }

        Ok::<_, anyhow::Error>(())
    }
    .await;

    let _ = tokio::time::timeout(
        Duration::from_secs(2),
        debug_run_command(debug_socket_path.clone(), "client:quit", None),
    )
    .await;
    kill_child(&mut child.child);

    if let Err(ref error) = test_result {
        eprintln!("self-dev reload e2e test error: {error:#}");
        eprintln!("self-dev client PTY output:\n{}", child.output_text());
        if let Some(log_excerpt) = latest_log_excerpt(&home_dir) {
            eprintln!("self-dev reload logs (tail):\n{}", log_excerpt);
        }
    }

    test_result
}

/// Test self-dev client binary reload against a real TUI client running in a PTY.
///
/// Starts from the test binary, then forces `/client-reload` to re-exec into
/// the built release candidate while keeping the shared server online.
#[cfg(unix)]
#[tokio::test]
#[ignore]
async fn binary_integration_selfdev_client_reload_resumes_session() -> Result<()> {
    let _env = setup_test_env()?;

    let release_binary =
        jcode::build::release_binary_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    if !release_binary.exists() {
        anyhow::bail!(
            "release binary missing at {} (run `cargo build --release` first)",
            release_binary.display()
        );
    }

    let temp_root = tempfile::Builder::new()
        .prefix("jcode-selfdev-client-reload-e2e-")
        .tempdir()?;
    let runtime_dir = temp_root.path().join("runtime");
    let home_dir = temp_root.path().join("home");
    let install_dir = temp_root.path().join("install");
    std::fs::create_dir_all(&runtime_dir)?;
    std::fs::create_dir_all(&home_dir)?;
    std::fs::create_dir_all(&install_dir)?;

    let _home_guard = EnvVarGuard::set("JCODE_HOME", &home_dir);
    let _runtime_guard = EnvVarGuard::set("JCODE_RUNTIME_DIR", &runtime_dir);
    let _install_guard = EnvVarGuard::set("JCODE_INSTALL_DIR", &install_dir);

    let socket_path = runtime_dir.join("jcode.sock");
    let debug_socket_path = runtime_dir.join("jcode-debug.sock");
    let starter_binary = temp_root.path().join("jcode-selfdev-client-starter");
    std::fs::copy(env!("CARGO_BIN_EXE_jcode"), &starter_binary)?;
    let starter_mtime = std::fs::metadata(&release_binary)?
        .modified()?
        .checked_sub(Duration::from_secs(60))
        .unwrap_or(std::time::UNIX_EPOCH + Duration::from_secs(1));
    set_file_mtime(&starter_binary, starter_mtime)?;

    let mut command = Command::new(&starter_binary);
    command
        .arg("--no-update")
        .arg("--provider")
        .arg("antigravity")
        .arg("self-dev")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env_remove("JCODE_TEST_SESSION")
        .env("JCODE_HOME", &home_dir)
        .env("JCODE_RUNTIME_DIR", &runtime_dir)
        .env("JCODE_INSTALL_DIR", &install_dir);

    let mut child = spawn_pty_child(command)?;

    let test_result = async {
        wait_for_server_ready(&socket_path, &debug_socket_path).await?;

        let session_id = wait_for_default_connected_client_session(&debug_socket_path).await?;

        let state_before =
            debug_run_command(debug_socket_path.clone(), "client:state", Some(&session_id)).await?;
        let state_before_json: serde_json::Value = serde_json::from_str(&state_before)?;
        let version_before = state_before_json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing client version before reload"))?
            .to_string();

        let clients_before =
            debug_run_command(debug_socket_path.clone(), "clients:map", None).await?;
        let clients_before_json: serde_json::Value = serde_json::from_str(&clients_before)?;
        let client_id_before = clients_before_json
            .get("clients")
            .and_then(|v| v.as_array())
            .and_then(|clients| {
                clients.iter().find_map(|client| {
                    let session = client.get("session_id").and_then(|v| v.as_str())?;
                    if session != session_id {
                        return None;
                    }
                    client
                        .get("client_id")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                })
            })
            .ok_or_else(|| anyhow::anyhow!("missing client id before reload"))?;

        let server_info_before =
            debug_run_command(debug_socket_path.clone(), "server:info", None).await?;
        let server_info_before_json: serde_json::Value = serde_json::from_str(&server_info_before)?;
        let server_id_before = server_info_before_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing server id before client reload"))?
            .to_string();

        child.send_command("/client-reload")?;

        let client_id_after = wait_for_selfdev_client_reload_cycle(
            &debug_socket_path,
            &session_id,
            &client_id_before,
            &server_id_before,
            Duration::from_secs(20),
        )
        .await?;

        let state_after =
            debug_run_command(debug_socket_path.clone(), "client:state", Some(&session_id)).await?;
        let state_after_json: serde_json::Value = serde_json::from_str(&state_after)?;
        let version_after = state_after_json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing client version after reload"))?;

        assert_ne!(
            client_id_after, client_id_before,
            "client reload should reconnect with a different client id"
        );
        assert_ne!(
            version_after, version_before,
            "client reload should switch binaries"
        );

        let server_info_after =
            debug_run_command(debug_socket_path.clone(), "server:info", None).await?;
        let server_info_after_json: serde_json::Value = serde_json::from_str(&server_info_after)?;
        let server_id_after = server_info_after_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing server id after client reload"))?;
        assert_eq!(
            server_id_after, server_id_before,
            "client reload should not replace the server process"
        );

        Ok::<_, anyhow::Error>(())
    }
    .await;

    let _ = tokio::time::timeout(
        Duration::from_secs(2),
        debug_run_command(debug_socket_path.clone(), "client:quit", None),
    )
    .await;
    kill_child(&mut child.child);

    if let Err(ref error) = test_result {
        eprintln!("self-dev client reload e2e test error: {error:#}");
        eprintln!("self-dev client PTY output:\n{}", child.output_text());
        if let Some(log_excerpt) = latest_log_excerpt(&home_dir) {
            eprintln!("self-dev client reload logs (tail):\n{}", log_excerpt);
        }
    }

    test_result
}

/// Test full self-dev `/reload` against a real TUI client running in a PTY.
///
/// Starts from an older starter binary so the client reloads into the built
/// release candidate while the shared server also restarts.
#[cfg(unix)]
#[tokio::test]
#[ignore]
async fn binary_integration_selfdev_full_reload_resumes_session_quickly() -> Result<()> {
    let _env = setup_test_env()?;

    let release_binary =
        jcode::build::release_binary_path(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
    if !release_binary.exists() {
        anyhow::bail!(
            "release binary missing at {} (run `cargo build --release` first)",
            release_binary.display()
        );
    }

    let temp_root = tempfile::Builder::new()
        .prefix("jcode-selfdev-full-reload-e2e-")
        .tempdir()?;
    let runtime_dir = temp_root.path().join("runtime");
    let home_dir = temp_root.path().join("home");
    let install_dir = temp_root.path().join("install");
    std::fs::create_dir_all(&runtime_dir)?;
    std::fs::create_dir_all(&home_dir)?;
    std::fs::create_dir_all(&install_dir)?;

    let _home_guard = EnvVarGuard::set("JCODE_HOME", &home_dir);
    let _runtime_guard = EnvVarGuard::set("JCODE_RUNTIME_DIR", &runtime_dir);
    let _install_guard = EnvVarGuard::set("JCODE_INSTALL_DIR", &install_dir);

    let socket_path = runtime_dir.join("jcode.sock");
    let debug_socket_path = runtime_dir.join("jcode-debug.sock");
    let starter_binary = temp_root.path().join("jcode-selfdev-full-reload-starter");
    std::fs::copy(env!("CARGO_BIN_EXE_jcode"), &starter_binary)?;
    let starter_mtime = std::fs::metadata(&release_binary)?
        .modified()?
        .checked_sub(Duration::from_secs(60))
        .unwrap_or(std::time::UNIX_EPOCH + Duration::from_secs(1));
    set_file_mtime(&starter_binary, starter_mtime)?;

    let mut command = Command::new(&starter_binary);
    command
        .arg("--no-update")
        .arg("--provider")
        .arg("antigravity")
        .arg("self-dev")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env_remove("JCODE_TEST_SESSION")
        .env("JCODE_HOME", &home_dir)
        .env("JCODE_RUNTIME_DIR", &runtime_dir)
        .env("JCODE_INSTALL_DIR", &install_dir);

    let mut child = spawn_pty_child(command)?;

    let test_result = async {
        wait_for_server_ready(&socket_path, &debug_socket_path).await?;

        let session_id = wait_for_default_connected_client_session(&debug_socket_path).await?;

        let state_before =
            debug_run_command(debug_socket_path.clone(), "client:state", Some(&session_id)).await?;
        let state_before_json: serde_json::Value = serde_json::from_str(&state_before)?;
        let version_before = state_before_json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing client version before full reload"))?
            .to_string();

        let clients_before =
            debug_run_command(debug_socket_path.clone(), "clients:map", None).await?;
        let clients_before_json: serde_json::Value = serde_json::from_str(&clients_before)?;
        let client_id_before = clients_before_json
            .get("clients")
            .and_then(|v| v.as_array())
            .and_then(|clients| {
                clients.iter().find_map(|client| {
                    let session = client.get("session_id").and_then(|v| v.as_str())?;
                    if session != session_id {
                        return None;
                    }
                    client
                        .get("client_id")
                        .and_then(|v| v.as_str())
                        .map(str::to_string)
                })
            })
            .ok_or_else(|| anyhow::anyhow!("missing client id before full reload"))?;

        let server_info_before =
            debug_run_command(debug_socket_path.clone(), "server:info", None).await?;
        let server_info_before_json: serde_json::Value = serde_json::from_str(&server_info_before)?;
        let server_id_before = server_info_before_json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing server id before full reload"))?
            .to_string();

        child.send_command("/reload")?;

        let server_id_after = wait_for_selfdev_reload_cycle(
            &debug_socket_path,
            &session_id,
            &server_id_before,
            Duration::from_secs(20),
        )
        .await?;

        let client_id_after = wait_for_selfdev_client_reload_cycle(
            &debug_socket_path,
            &session_id,
            &client_id_before,
            &server_id_after,
            Duration::from_secs(20),
        )
        .await?;

        let state_after =
            debug_run_command(debug_socket_path.clone(), "client:state", Some(&session_id)).await?;
        let state_after_json: serde_json::Value = serde_json::from_str(&state_after)?;
        let version_after = state_after_json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing client version after full reload"))?;

        assert_ne!(
            server_id_after, server_id_before,
            "full reload should replace the server process"
        );
        assert_ne!(
            client_id_after, client_id_before,
            "full reload should reconnect with a different client id"
        );
        assert_ne!(
            version_after, version_before,
            "full reload should switch binaries"
        );

        Ok::<_, anyhow::Error>(())
    }
    .await;

    let _ = tokio::time::timeout(
        Duration::from_secs(2),
        debug_run_command(debug_socket_path.clone(), "client:quit", None),
    )
    .await;
    kill_child(&mut child.child);

    if let Err(ref error) = test_result {
        eprintln!("self-dev full reload e2e test error: {error:#}");
        eprintln!("self-dev client PTY output:\n{}", child.output_text());
        if let Some(log_excerpt) = latest_log_excerpt(&home_dir) {
            eprintln!("self-dev full reload logs (tail):\n{}", log_excerpt);
        }
    }

    test_result
}

// Synthetic OAuth endpoints only. Linux namespace has loopback and no external route.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "actual binary, isolated Linux network namespace and synthetic TLS CA"]
fn binary_oauth_failure_taxonomy_keeps_route_and_identity() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let namespace = std::fs::read_link("/proc/self/ns/net").unwrap();
    let output = std::process::Command::new("/usr/bin/timeout")
        .args(["--kill-after=5", "120", "/usr/bin/unshare", "--user", "--map-root-user", "--net", "/usr/bin/python3", "-c"])
        .arg(r###"import hashlib,http.server,json,os,select,socket,ssl,subprocess,sys,tempfile,threading,time
from pathlib import Path
root,binary,parent_net=Path(sys.argv[1]),Path(sys.argv[2]).resolve(),sys.argv[3]
assert os.readlink('/proc/self/ns/net')!=parent_net,'refusing shared network namespace'
subprocess.run(['/usr/sbin/ip','link','set','lo','up'],check=True)
assert {x['ifname'] for x in json.loads(subprocess.check_output(['/usr/sbin/ip','-json','link','show']))}=={'lo'}
assert not subprocess.check_output(['/usr/sbin/ip','route','show','default'])
source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
sha=lambda data:hashlib.sha256(data).hexdigest()
with tempfile.TemporaryDirectory(prefix='jc-auth-',dir=root/'scratch') as tmp:
 state=Path(tmp);cert,key,ca,cakey,csr,extensions=[state/name for name in ['cert.pem','key.pem','ca.pem','ca-key.pem','leaf.csr','leaf.ext']]
 subprocess.run(['/usr/bin/openssl','req','-x509','-newkey','rsa:2048','-nodes','-keyout',str(cakey),'-out',str(ca),'-days','1','-subj','/CN=isolated-oauth-fixture-ca','-addext','basicConstraints=critical,CA:TRUE','-addext','keyUsage=critical,keyCertSign,cRLSign'],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 subprocess.run(['/usr/bin/openssl','req','-new','-newkey','rsa:2048','-nodes','-keyout',str(key),'-out',str(csr),'-subj','/CN=api.anthropic.com'],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 extensions.write_text('subjectAltName=DNS:api.anthropic.com,DNS:platform.claude.com\nbasicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n')
 subprocess.run(['/usr/bin/openssl','x509','-req','-in',str(csr),'-CA',str(ca),'-CAkey',str(cakey),'-CAcreateserial','-out',str(cert),'-days','1','-extfile',str(extensions)],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 tls=ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER);tls.load_cert_chain(cert,key)
 for case in (sys.argv[4:] or ['refresh_once','revoked','quota','refresh_transport']):
  home=state/case;home.mkdir(mode=0o700);runtime=home/'runtime';runtime.mkdir(mode=0o700)
  native=runtime/'native.sock';assert not native.exists()
  (home/'config.toml').write_text('[features]\nmemory=false\nauto_poke=false\ncheck_updates=false\n[autojudge]\nmax_continuations_per_turn=0\n[agents]\nmemory_sidecar_enabled=false\n[provider]\nrequest_compression="off"\n')
  initial={'anthropic_accounts':[{'label':'claude-otter','access':'synthetic-access-old','refresh':'synthetic-refresh-old','expires':int(time.time()*1000)+3600000,'scopes':['user:inference']},{'label':'claude-fox','access':'synthetic-alternate-never-used','refresh':'synthetic-alternate-refresh','expires':int(time.time()*1000)+3600000,'scopes':['user:inference']}],'active_anthropic_account':'claude-otter'}
  auth=home/'auth.json';auth.write_text(json.dumps(initial));auth.chmod(0o600)
  counts={'messages':0,'refresh':0};faults=[];frames=[]
  class Fixture(http.server.BaseHTTPRequestHandler):
   protocol_version='HTTP/1.1'
   def log_message(self,*args):pass
   def do_CONNECT(self):
    if self.path not in ['api.anthropic.com:443','platform.claude.com:443']:
     faults.append('unexpected host');self.send_error(403);return
    self.send_response(200);self.end_headers();self.wfile.flush()
    try:
     secured=tls.wrap_socket(self.connection,server_side=True)
     Fixture(secured,self.client_address,self.server)
    except (ssl.SSLError,ConnectionError,OSError):pass
    finally:self.close_connection=True
   def reply(self,status,body):
    data=json.dumps(body).encode();self.send_response(status);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(data)));self.send_header('Connection','close');self.end_headers();self.wfile.write(data);self.close_connection=True
   def do_GET(self):
    if self.path.startswith('/v1/models'):self.reply(200,{'data':[{'id':'claude-sonnet-4-6','type':'model','display_name':'Synthetic fixture'}],'has_more':False})
    elif 'usage' in self.path:self.reply(200,{})
    else:self.reply(404,{'error':'synthetic unsupported fixture path'})
   def do_POST(self):
    body=self.rfile.read(int(self.headers.get('Content-Length','0')))
    if self.path=='/v1/oauth/token':
     counts['refresh']+=1
     if json.loads(body).get('refresh_token')!='synthetic-refresh-old':faults.append('wrong refresh identity')
     if case=='revoked':self.reply(400,{'error':'invalid_grant'});return
     if case=='refresh_transport':self.reply(503,{'error':'service unavailable'});return
     self.reply(200,{'access_token':'synthetic-access-new','refresh_token':'synthetic-refresh-new','expires_in':3600,'scope':'user:inference'});return
    if self.path.startswith('/v1/messages'):
     counts['messages']+=1;n=counts['messages'];expected='synthetic-access-new' if n==2 else 'synthetic-access-old'
     if self.headers.get('Authorization')!='Bearer '+expected:faults.append('wrong message identity')
     if json.loads(body).get('model')!='claude-sonnet-4-6':faults.append('model substituted')
     if case=='quota':self.reply(429,{'error':{'type':'insufficient_quota','message':'billing exhausted'}})
     else:self.reply(401,{'error':{'type':'authentication_error','message':'token expired'}})
     return
    faults.append('unexpected POST');self.reply(400,{'error':'unexpected fixture path'})
  fixture=http.server.ThreadingHTTPServer(('127.0.0.1',0),Fixture);fixture.daemon_threads=True
  thread=threading.Thread(target=fixture.serve_forever,daemon=True);thread.start()
  env={'PATH':'/usr/bin:/bin','LANG':'C.UTF-8','HOME':str(home),'JCODE_HOME':str(home),'JCODE_RUNTIME_DIR':str(runtime),'JCODE_INSTALL_DIR':str(home/'install'),'XDG_RUNTIME_DIR':str(runtime),'XDG_CONFIG_HOME':str(home/'config'),'XDG_DATA_HOME':str(home/'data'),'XDG_CACHE_HOME':str(home/'cache'),'JCODE_NO_TELEMETRY':'1','JCODE_MEMORY_ENABLED':'0','JCODE_MEMORY_SIDECAR_ENABLED':'0','JCODE_REQUEST_COMPRESSION':'off','SSL_CERT_FILE':str(ca),'HTTPS_PROXY':f'http://127.0.0.1:{fixture.server_port}','HTTP_PROXY':f'http://127.0.0.1:{fixture.server_port}','NO_COLOR':'1'}
  process=None;client=None;reader=None
  try:
   with open(home/'daemon.log','wb') as log:
    process=subprocess.Popen([str(binary),'--no-update','--no-selfdev','--provider','claude','--model','claude-sonnet-4-6','--socket',str(native),'serve','--temporary-server','--owner-pid',str(os.getpid()),'--temp-idle-timeout-secs','60'],cwd=home,env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=log)
   deadline=time.monotonic()+20
   while not native.exists():
    assert process.poll() is None,'daemon exited before readiness';assert time.monotonic()<deadline,'socket readiness deadline';select.select([],[],[],.01)
   assert Path(f'/proc/{process.pid}/exe').resolve()==binary,'wrong serving executable'
   client=socket.socket(socket.AF_UNIX);client.settimeout(20);client.connect(str(native));reader=client.makefile('rb')
   def send(value):client.sendall((json.dumps(value)+'\n').encode())
   def until(predicate):
    deadline=time.monotonic()+20
    while True:
     assert time.monotonic()<deadline,'public frame deadline'
     raw=reader.readline();assert raw,'public EOF';frame=json.loads(raw);frames.append(frame)
     if predicate(frame):return frame
   send({'type':'ping','id':1});identity=until(lambda f:f.get('type')=='pong')['server_runtime']
   assert Path(identity['binary_path']).resolve()==binary and source.startswith(identity['git_hash']),'binary/source binding mismatch'
   send({'type':'subscribe','id':2,'working_dir':str(home),'system_prompt':'Synthetic isolated auth test. No tools.'});until(lambda f:f.get('type')=='done' and f.get('id')==2)
   send({'type':'message','id':3,'content':'Synthetic OAuth taxonomy acceptance.','plan_only':True})
   terminal=until(lambda f:f.get('type') in ['error','done'] and f.get('id')==3)
   errors='\n'.join(f.get('message','') for f in frames if f.get('type')=='error')
   assert not faults,faults
   expected=(2,1) if case=='refresh_once' else (1,0) if case=='quota' else (1,1)
   assert (counts['messages'],counts['refresh'])==expected,(case,counts,terminal,errors)
   expected_kind={'refresh_once':'AuthExpired:','revoked':'AuthRevoked:','quota':'QuotaExhausted:','refresh_transport':'Transient:'}[case]
   assert expected_kind in errors,(case,terminal,errors)
   if case=='revoked':assert 'jcode login --provider claude' in errors
   if case=='refresh_transport':assert 'AuthRevoked:' not in errors and 'jcode login' not in errors
   stored=json.loads(auth.read_text());assert stored['active_anthropic_account']=='claude-otter',stored
   alternate=next(a for a in stored['anthropic_accounts'] if a['label']=='claude-fox');assert alternate==initial['anthropic_accounts'][1]
   selected=next(a for a in stored['anthropic_accounts'] if a['label']=='claude-otter')
   assert selected['refresh']==('synthetic-refresh-new' if case=='refresh_once' else 'synthetic-refresh-old')
   print(json.dumps({'case':case,'source_head':source,'binary_sha256':sha(binary.read_bytes()),'server_runtime':identity,'requests':counts,'kind':expected_kind,'same_route':True,'same_identity':True,'isolated_network':True}),flush=True)
  except BaseException:
   print('synthetic owned daemon log:\n'+(home/'daemon.log').read_text(errors='replace'),file=sys.stderr);raise
  finally:
   if reader:reader.close()
   if client:client.close()
   if process:
    process.terminate()
    try:process.wait(timeout=5)
    except subprocess.TimeoutExpired:process.kill();process.wait(timeout=5)
   fixture.shutdown();fixture.server_close();thread.join(timeout=2)
print('cleanup: owned daemons and fixtures reaped; private synthetic homes removed',flush=True)
"###)
        .arg(root)
        .arg(env!("CARGO_BIN_EXE_jcode"))
        .arg(namespace)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", root.join("scratch"))
        .output().expect("launch isolated OAuth binary acceptance");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(output.status.success(), "private OAuth acceptance failed: {}\n{}", output.status, String::from_utf8_lossy(&output.stderr));
}
