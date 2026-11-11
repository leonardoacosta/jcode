use super::*;

struct AccountSandbox {
    _home: tempfile::TempDir,
    previous_home: Option<std::ffi::OsString>,
    previous_runtime: Option<std::ffi::OsString>,
    previous_account: Option<String>,
}

impl AccountSandbox {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let previous_home = std::env::var_os("JCODE_HOME");
        let previous_runtime = std::env::var_os("JCODE_RUNTIME_PROVIDER");
        let previous_account = auth::claude::get_active_account_override();
        jcode_base::env::set_var("JCODE_HOME", home.path());
        jcode_base::env::set_var("JCODE_RUNTIME_PROVIDER", "claude-oauth");
        auth::claude::set_active_account_override(None);
        Self {
            _home: home,
            previous_home,
            previous_runtime,
            previous_account,
        }
    }
}

impl Drop for AccountSandbox {
    fn drop(&mut self) {
        auth::claude::set_active_account_override(self.previous_account.take());
        for (key, value) in [
            ("JCODE_HOME", self.previous_home.take()),
            ("JCODE_RUNTIME_PROVIDER", self.previous_runtime.take()),
        ] {
            match value {
                Some(value) => jcode_base::env::set_var(key, value),
                None => jcode_base::env::remove_var(key),
            }
        }
    }
}

fn save_account(label: &str, token: &str, refresh: &str) -> String {
    auth::claude::upsert_account(auth::claude::AnthropicAccount {
        label: label.into(),
        access: token.into(),
        refresh: refresh.into(),
        expires: chrono::Utc::now().timestamp_millis() + 3_600_000,
        email: None,
        subscription_type: Some("max".into()),
        scopes: vec!["user:inference".into()],
    })
    .unwrap()
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // Serialize process-global test environment.
async fn accepted_refresh_commits_disk_and_memory_after_caller_cancellation() {
    let _lock = jcode_base::storage::lock_test_env();
    let _sandbox = AccountSandbox::new();
    let personal = save_account("cancel-personal", "old-access", "old-refresh");
    let work = save_account("cancel-work", "work-access", "work-refresh");
    auth::claude::set_active_account(&work).unwrap();
    let credentials = Arc::new(RwLock::new(Some(CachedCredentials {
        access_token: "old-access".into(),
        refresh_token: "old-refresh".into(),
        expires_at: 1,
    })));
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let caller_credentials = Arc::clone(&credentials);
    let personal_task = personal.clone();
    let caller = tokio::spawn(async move {
        refresh_and_commit_claude_tokens(caller_credentials, async move {
            started_tx.send(()).unwrap();
            release_rx.await.unwrap();
            let tokens = oauth::OAuthTokens {
                access_token: "rotated-access".into(),
                refresh_token: "rotated-refresh".into(),
                expires_at: chrono::Utc::now().timestamp_millis() + 3_600_000,
                id_token: None,
                scopes: vec![],
            };
            oauth::save_claude_tokens_for_account(&tokens, &personal_task)?;
            Ok(tokens)
        })
        .await
    });
    started_rx.await.unwrap();
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    release_tx.send(()).unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            if credentials.read().await.as_ref().unwrap().refresh_token == "rotated-refresh" {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        auth::claude::load_credentials_for_account(&personal)
            .unwrap()
            .access_token,
        "rotated-access"
    );
    assert_eq!(
        auth::claude::load_credentials_for_account(&work)
            .unwrap()
            .access_token,
        "work-access"
    );
    assert_eq!(
        auth::claude::active_account_label().as_deref(),
        Some(work.as_str())
    );
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn cloud_import_replaces_live_daemon_cached_bearer() {
    let _lock = jcode_base::storage::lock_test_env();
    let _sandbox = AccountSandbox::new();
    let label = save_account("cloud-account", "old-access", "old-refresh");
    let provider = AnthropicProvider::new();
    provider.set_oauth_account(Some(&label)).unwrap();
    provider
        .set_credential_mode(AnthropicCredentialMode::OAuth)
        .unwrap();
    assert_eq!(provider.get_access_token().await.unwrap().0, "old-access");
    save_account(&label, "imported-access", "imported-refresh");
    assert_eq!(
        provider.get_access_token().await.unwrap().0,
        "imported-access"
    );
}

#[test]
fn auth_rejection_refreshes_once_without_route_or_identity_substitution() {
    use std::io::{Read, Write};
    let _lock = jcode_base::storage::lock_test_env();
    let _sandbox = AccountSandbox::new();
    let personal = save_account("retry-personal", "rotated-access", "rotated-refresh");
    let work = save_account("retry-work", "work-access", "work-refresh");
    auth::claude::set_active_account(&work).unwrap();
    // Joining an already-rotated stored account exercises the real refresh
    // transaction without ever calling a live token endpoint.
    let credentials = Arc::new(RwLock::new(Some(CachedCredentials {
        access_token: "old-access".into(),
        refresh_token: "old-refresh".into(),
        expires_at: 1,
    })));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/v1/messages", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        for token in ["old-access", "rotated-access"] {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "same-route retry missing"
                        );
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                    Err(error) => panic!("{error}"),
                }
            };
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(2)))
                .unwrap();
            let mut bytes = Vec::new();
            loop {
                let mut buffer = [0; 4096];
                let count = socket.read(&mut buffer).unwrap();
                assert!(count > 0);
                bytes.extend_from_slice(&buffer[..count]);
                if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    if bytes.len() >= end + 4 + length {
                        assert!(headers.starts_with("POST /v1/messages"));
                        assert!(
                            headers
                                .to_ascii_lowercase()
                                .contains(&format!("authorization: bearer {token}"))
                        );
                        let request: serde_json::Value =
                            serde_json::from_slice(&bytes[end + 4..end + 4 + length]).unwrap();
                        assert_eq!(request["model"], "claude-sonnet-4-6");
                        break;
                    }
                }
            }
            let body = r#"{"error":{"type":"authentication_error","message":"token expired"}}"#;
            write!(socket, "HTTP/1.1 401 Unauthorized\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
    });
    // Construct outside Tokio to avoid background usage work.
    let provider = AnthropicProvider::new();
    let retry_settings = reasoning_request::RetrySettings::from_provider(&provider);
    let mut transport = DirectTransportConfig::from_env();
    transport.oauth_api_url = Some(endpoint);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let (tx, mut rx) = mpsc::channel(64);
        let task = tokio::spawn(run_stream_with_retries(
            Client::builder().no_proxy().build().unwrap(),
            "old-access".into(),
            true,
            ApiRequest {
                model: "claude-sonnet-4-6".into(),
                max_tokens: 16,
                system: None,
                messages: vec![],
                tools: None,
                metadata: None,
                thinking: None,
                output_config: None,
                temperature: None,
                service_tier: None,
                stream: true,
            },
            tx,
            Arc::clone(&credentials),
            Some(personal.clone()),
            "claude-sonnet-4-6".into(),
            "fixture-session".into(),
            Arc::new(std::sync::RwLock::new("claude-sonnet-4-6".into())),
            transport,
            retry_settings,
        ));
        let error = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while let Some(event) = rx.recv().await {
                if let Err(error) = event {
                    return error;
                }
            }
            panic!("second authentication rejection must stop the turn");
        })
        .await
        .unwrap();
        assert!(error.to_string().contains("AuthExpired:"), "{error:#}");
        assert_eq!(
            jcode_provider_core::classify_failover_error_message(&error.to_string()),
            jcode_provider_core::FailoverDecision::None
        );
        task.await.unwrap();
        assert_eq!(
            credentials.read().await.as_ref().unwrap().access_token,
            "rotated-access"
        );
    });
    server.join().unwrap();
    assert_eq!(
        auth::claude::load_credentials_for_account(&work)
            .unwrap()
            .access_token,
        "work-access"
    );
    assert_eq!(
        auth::claude::active_account_label().as_deref(),
        Some(work.as_str())
    );
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn active_oauth_email_follows_cached_unbound_credentials_not_global_account() {
    let _lock = jcode_base::storage::lock_test_env();
    let _sandbox = AccountSandbox::new();
    let personal = save_account("new-personal", "personal-access", "personal-refresh");
    let work = save_account("new-work", "work-access", "work-refresh");
    auth::claude::update_account_profile(&personal, Some("personal@example.com".into())).unwrap();
    auth::claude::update_account_profile(&work, Some("work@example.com".into())).unwrap();
    auth::claude::set_active_account(&personal).unwrap();
    let provider = AnthropicProvider::new();
    provider
        .set_credential_mode(AnthropicCredentialMode::Auto)
        .unwrap();
    assert_eq!(provider.oauth_account_label(), None);
    assert_eq!(provider.active_oauth_account_email(), None);
    provider.get_access_token().await.unwrap();
    auth::claude::set_active_account(&work).unwrap();
    assert_eq!(
        provider.active_oauth_account_email().as_deref(),
        Some("personal@example.com")
    );

    // A concurrently rotated access token still matches the held refresh identity.
    auth::claude::update_account_tokens(
        &personal,
        "rotated-access",
        "personal-refresh",
        chrono::Utc::now().timestamp_millis() + 3_600_000,
    )
    .unwrap();
    assert_eq!(
        provider.active_oauth_account_email().as_deref(),
        Some("personal@example.com")
    );

    // Auto falling back to a direct key must not expose the old OAuth cache.
    provider.request_credential.store(2, Ordering::Release);
    assert_eq!(provider.active_oauth_account_email(), None);
    provider.request_credential.store(0, Ordering::Release);
    assert_eq!(provider.active_oauth_account_email(), None);
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn oauth_account_binding_survives_global_switch_refresh_and_fork() {
    let _lock = jcode_base::storage::lock_test_env();
    let _sandbox = AccountSandbox::new();
    let personal = save_account("new-personal", "personal-access", "personal-refresh-old");
    let work = save_account("new-work", "work-access", "work-refresh");
    auth::claude::set_active_account(&personal).unwrap();
    let provider = AnthropicProvider::new();
    provider.set_oauth_account(Some(&personal)).unwrap();
    provider
        .set_credential_mode(AnthropicCredentialMode::OAuth)
        .unwrap();
    assert_eq!(
        provider.get_access_token().await.unwrap(),
        ("personal-access".into(), true)
    );

    auth::claude::set_active_account(&work).unwrap();
    provider.invalidate_credentials().await;
    assert_eq!(
        provider.get_access_token().await.unwrap(),
        ("personal-access".into(), true)
    );
    assert_eq!(
        provider.catalog_scope_for_route(true),
        format!("oauth::{personal}")
    );
    assert_eq!(
        provider.oauth_refresh_label().as_deref(),
        Some(personal.as_str())
    );

    // Another request already rotated personal's refresh token. Forced refresh
    // must join that account's transaction, not the globally active work login.
    save_account(&personal, "personal-access-rotated", "personal-refresh-new");
    let token = force_refresh_oauth_token(
        Arc::clone(&provider.credentials),
        &provider.oauth_refresh_label(),
    )
    .await
    .unwrap();
    assert_eq!(token, "personal-access-rotated");
    assert_eq!(
        auth::claude::load_credentials_for_account(&work)
            .unwrap()
            .access_token,
        "work-access"
    );
    assert_eq!(
        auth::claude::active_account_label().as_deref(),
        Some(work.as_str())
    );

    let fork = provider.fork();
    assert_eq!(
        fork.oauth_account_label().as_deref(),
        Some(personal.as_str())
    );
    fork.set_oauth_account(Some(&work)).unwrap();
    assert_eq!(
        provider.oauth_account_label().as_deref(),
        Some(personal.as_str())
    );
    assert_eq!(fork.oauth_account_label().as_deref(), Some(work.as_str()));
    assert!(fork.set_oauth_account(Some("missing-account")).is_err());
    assert_eq!(fork.oauth_account_label().as_deref(), Some(work.as_str()));

    provider.invalidate_credentials().await;
    assert_eq!(
        provider.get_access_token().await.unwrap(),
        ("personal-access-rotated".into(), true)
    );
    provider.set_oauth_account(Some(&work)).unwrap();
    assert_eq!(
        provider.get_access_token().await.unwrap(),
        ("work-access".into(), true)
    );
    auth::claude::remove_account(&work).unwrap();
    assert!(
        provider.get_access_token().await.is_err(),
        "removed pinned account must not use cached bearer"
    );
}
