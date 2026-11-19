use super::*;

#[test]
fn service_tier_headers_follow_serialized_tier_on_both_transports() {
    let client = Client::new();
    for mode in 0..3 {
        let oauth = mode != 0;
        let creds = CodexCredentials {
            access_token: "fixture-token".into(),
            refresh_token: if mode == 1 {
                "refresh".into()
            } else {
                String::new()
            },
            id_token: (mode == 2).then(|| "id-token".into()),
            account_id: Some("fixture-account".into()),
            expires_at: None,
        };
        for tier in [
            None,
            Some("default"),
            Some("auto"),
            Some("priority"),
            Some("fast"),
            Some("flex"),
            Some("ultrafast"),
        ] {
            let http = response_http_request(
                &client,
                "https://example.test/responses",
                &creds.access_token,
                oauth,
                None,
                None,
                Some("gpt-6.1-sol"),
                tier,
            )
            .json(&serde_json::json!({"service_tier": tier}))
            .build()
            .unwrap();
            let ws = openai_websocket_prewarm::websocket_request(
                &creds,
                &creds.access_token,
                None,
                Some("gpt-6.1-sol"),
                tier,
            )
            .unwrap();
            if !oauth {
                for header in [
                    "originator",
                    "chatgpt-account-id",
                    "session-id",
                    "x-codex-routing-hint",
                ] {
                    assert!(!http.headers().contains_key(header));
                    assert!(!ws.headers().contains_key(header));
                }
            }
            let expected = (!oauth && tier == Some("ultrafast")).then_some("ultrafast");
            let routing_hint = oauth.then(|| match tier {
                Some(tier) => format!("model=gpt-6.1-sol;tier={tier}"),
                None => "model=gpt-6.1-sol".to_string(),
            });
            assert_eq!(
                http.headers()
                    .get("x-codex-routing-hint")
                    .map(|v| v.to_str().unwrap()),
                routing_hint.as_deref()
            );
            assert_eq!(
                ws.headers()
                    .get("x-codex-routing-hint")
                    .map(|v| v.to_str().unwrap()),
                routing_hint.as_deref()
            );
            assert_eq!(
                http.headers()
                    .get("OpenAI-Service-Tier")
                    .map(|v| v.to_str().unwrap()),
                expected
            );
            assert_eq!(
                ws.headers()
                    .get("OpenAI-Service-Tier")
                    .map(|v| v.to_str().unwrap()),
                expected
            );
        }
    }
}

fn dummy_credentials() -> Arc<RwLock<CodexCredentials>> {
    Arc::new(RwLock::new(CodexCredentials {
        access_token: "at_old".into(),
        refresh_token: "rt_old".into(),
        id_token: None,
        account_id: Some("acct_dummy".into()),
        expires_at: Some(1),
    }))
}

fn rotated_tokens() -> oauth::OAuthTokens {
    oauth::OAuthTokens {
        access_token: "at_new".into(),
        refresh_token: "rt_new".into(),
        expires_at: 42,
        id_token: None,
        scopes: Vec::new(),
    }
}

/// Issue #1757: once rotation has started, a cancelled caller must not
/// leave the consumed refresh token in memory.
#[tokio::test]
async fn refresh_commit_survives_caller_cancellation() {
    let credentials = dummy_credentials();
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
    let caller_credentials = Arc::clone(&credentials);
    let caller = tokio::spawn(async move {
        refresh_and_commit_openai_tokens(&caller_credentials, async move {
            started_tx.send(()).unwrap();
            release_rx.await.unwrap();
            Ok(rotated_tokens())
        })
        .await
    });
    started_rx.await.unwrap();
    caller.abort();
    assert!(caller.await.unwrap_err().is_cancelled());
    release_tx.send(()).unwrap();

    tokio::time::timeout(Duration::from_secs(2), async {
        while credentials.read().await.refresh_token != "rt_new" {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("rotated tokens must be committed in memory after caller cancellation");
    let creds = credentials.read().await;
    assert_eq!(creds.access_token, "at_new");
    assert_eq!(creds.account_id.as_deref(), Some("acct_dummy"));
    assert_eq!(creds.expires_at, Some(42));
}

#[tokio::test]
async fn refresh_failure_leaves_in_memory_credentials_untouched() {
    let credentials = dummy_credentials();
    let err = refresh_and_commit_openai_tokens(&credentials, async {
        anyhow::bail!("OpenAI token refresh timed out after 30s waiting for response headers")
    })
    .await
    .unwrap_err();
    assert!(err.to_string().contains("timed out"));
    let creds = credentials.read().await;
    assert_eq!(creds.access_token, "at_old");
    assert_eq!(creds.refresh_token, "rt_old");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // Serialize process-global test environment.
async fn cancelled_accepted_refresh_persists_commits_and_skips_transport() {
    let _lock = jcode_base::storage::lock_test_env();
    let home = tempfile::tempdir().unwrap();
    struct RestoreHome(Option<std::ffi::OsString>);
    impl Drop for RestoreHome {
        fn drop(&mut self) {
            match self.0.take() {
                Some(value) => jcode_base::env::set_var("JCODE_HOME", value),
                None => jcode_base::env::remove_var("JCODE_HOME"),
            }
        }
    }
    let _restore = RestoreHome(std::env::var_os("JCODE_HOME"));
    jcode_base::env::set_var("JCODE_HOME", home.path());
    let credentials = dummy_credentials();
    let original = oauth::OAuthTokens {
        access_token: "at_old".into(),
        refresh_token: "rt_old".into(),
        expires_at: 1,
        id_token: None,
        scopes: vec![],
    };
    oauth::save_openai_tokens_for_account(&original, "cancel-fixture").unwrap();
    let label = jcode_base::auth::codex::list_accounts()
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .label;
    let task_label = label.clone();
    let (tx, rx) = mpsc::channel(1);
    let (started_tx, started_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    let caller_credentials = Arc::clone(&credentials);
    let caller = tokio::spawn(async move {
        super::super::openai_stream_cancellation::after_authentication(
            &tx,
            refresh_and_commit_openai_tokens(&caller_credentials, async move {
                started_tx.send(()).unwrap();
                release_rx.await.unwrap();
                let tokens = rotated_tokens();
                oauth::save_openai_tokens_for_account(&tokens, &task_label)?;
                Ok(tokens)
            }),
            |_| async { panic!("cancelled consumer must never start inference transport") },
        )
        .await
    });
    started_rx.await.unwrap();
    drop(rx);
    assert!(caller.await.unwrap().is_err());
    release_tx.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(2), async {
        while credentials.read().await.refresh_token != "rt_new" {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let stored = jcode_base::auth::codex::load_credentials_for_account(&label).unwrap();
    assert_eq!(stored.access_token, "at_new");
    assert_eq!(stored.refresh_token, "rt_new");
    assert_eq!(
        credentials.read().await.account_id.as_deref(),
        Some("acct_dummy")
    );
}

#[test]
fn terminal_http_errors_render_kind_specific_remedies() {
    let expired = format_http_error(StatusCode::UNAUTHORIZED, "token expired");
    assert!(expired.contains("AuthExpired:"), "{expired}");
    assert!(!expired.contains("jcode login"), "{expired}");
    let quota = format_http_error(StatusCode::PAYMENT_REQUIRED, "insufficient_quota");
    assert!(quota.contains("QuotaExhausted:"), "{quota}");
    let revoked = format_http_error(StatusCode::BAD_REQUEST, "invalid_grant");
    assert!(
        revoked.contains("jcode login --provider openai"),
        "{revoked}"
    );
    assert_eq!(
        format_http_error(StatusCode::BAD_REQUEST, "invalid_request_error"),
        "OpenAI API error 400 Bad Request: invalid_request_error"
    );
}

#[test]
fn terminal_stream_error_events_get_shared_remedy_once() {
    let render = |message: &str| match with_failure_remedy(StreamEvent::Error {
        message: message.to_string(),
        retry_after_secs: Some(7),
    }) {
        StreamEvent::Error {
            message,
            retry_after_secs,
        } => {
            assert_eq!(retry_after_secs, Some(7));
            message
        }
        other => panic!("{other:?}"),
    };
    let quota = render("The usage limit has been reached");
    assert!(
        quota.ends_with("QuotaExhausted: Check this account's usage and billing before retrying."),
        "{quota}"
    );
    assert_eq!(render(&quota), quota, "remedy must not be appended twice");
    assert!(render("Invalid token").contains("AuthExpired:"));
    assert_eq!(render("Invalid tool schema."), "Invalid tool schema.");
}

#[test]
fn unauthorized_triggers_token_refresh() {
    assert!(should_refresh_token(StatusCode::UNAUTHORIZED, ""));
}

#[test]
fn forbidden_triggers_refresh_only_for_token_bodies() {
    assert!(should_refresh_token(
        StatusCode::FORBIDDEN,
        "access token expired"
    ));
    assert!(!should_refresh_token(
        StatusCode::FORBIDDEN,
        "region not allowed"
    ));
}

#[test]
fn refreshed_token_marker_is_retryable() {
    // After a 401/403 we force-refresh the OpenAI token and surface this
    // marker so the retry loop reconnects with the new credentials.
    assert!(is_retryable_error(
        "openai token refreshed, retrying: 401 unauthorized"
    ));
}

#[test]
fn missing_or_failed_refresh_is_not_retryable() {
    assert!(!is_retryable_error(
        "openai rejected the access token and no refresh token is available; run /login to re-authenticate: 401"
    ));
    assert!(!is_retryable_error(
        "openai token refresh failed; run /login to re-authenticate: network error"
    ));
}

#[test]
fn tls_transient_errors_are_retryable() {
    // Regression for issue #338: transient TLS faults must be retried on
    // the OpenAI path, matching every other provider. Callers pass the
    // error string already lowercased.
    assert!(is_retryable_error(
        "stream error: io error: received fatal alert: badrecordmac"
    ));
    assert!(is_retryable_error("received fatal alert: badrecordmac"));
    assert!(is_retryable_error("decryption failed or bad record mac"));
    assert!(is_retryable_error("tls handshake eof"));
    assert!(is_retryable_error("connection aborted"));
    assert!(is_retryable_error("temporary failure in name resolution"));
    assert!(is_retryable_error("no route to host"));
    assert!(is_retryable_error("network is unreachable"));
    // A send-level cause that callers now surface via the full anyhow
    // chain ({:#}) instead of the masked top-level context alone.
    assert!(is_retryable_error(
        "failed to send request to openai api: error sending request: received fatal alert: badrecordmac"
    ));
}

#[test]
fn rate_limit_is_retryable() {
    // Regression for issue #338 (gap #2): 429s should be retried, unifying
    // behavior with Anthropic/Copilot.
    assert!(is_retryable_error("429 too many requests"));
    assert!(is_retryable_error("rate limit exceeded"));
    assert!(is_retryable_error("rate_limit_exceeded"));
}

#[test]
fn auth_errors_remain_non_retryable() {
    assert!(!is_retryable_error("401 unauthorized"));
    assert!(!is_retryable_error("invalid api key"));
}
#[test]
fn cache_affinity_https_matches_websocket_session_header() {
    let client = Client::new();
    let mut credentials = CodexCredentials {
        access_token: "fixture-token".into(),
        refresh_token: "fixture-refresh".into(),
        id_token: None,
        account_id: Some("fixture-account".into()),
        expires_at: None,
    };
    for oauth in [true, false] {
        if !oauth {
            credentials.refresh_token.clear();
        }
        let http = response_http_request(
            &client,
            "https://example.test/responses",
            &credentials.access_token,
            oauth,
            credentials.account_id.as_deref(),
            Some("persisted-session"),
            None,
            None,
        )
        .json(&serde_json::json!({"prompt_cache_key":"persisted-session"}))
        .build()
        .unwrap();
        let ws = openai_websocket_prewarm::websocket_request(
            &credentials,
            &credentials.access_token,
            Some("persisted-session"),
            None,
            None,
        )
        .unwrap();
        assert_eq!(
            http.headers().get("session-id"),
            ws.headers().get("session-id")
        );
        assert_eq!(http.headers().contains_key("session-id"), oauth);
        assert_eq!(
            http.headers().get("chatgpt-account-id"),
            ws.headers().get("chatgpt-account-id")
        );
        assert_eq!(http.headers()["authorization"], "Bearer fixture-token");
        if oauth {
            assert_eq!(http.headers()["session-id"], "persisted-session");
        }
    }
}

#[test]
fn taxonomy_blocks_quota_revocation_and_permanent_retries() {
    for error in [
        "429 too many requests: insufficient_quota",
        "429 billing exhausted, retry after timeout",
        "invalid_grant: refresh token revoked, connection reset",
        "403 forbidden rate limit",
    ] {
        assert!(!is_retryable_error(error), "{error}");
    }
    assert!(!is_retryable_error(
        "openai token refresh failed: 503 service unavailable"
    ));
    assert_eq!(
        jcode_provider_core::classify_provider_failure(
            "OpenAI token refresh failed: 503 service unavailable"
        ),
        jcode_provider_core::ProviderFailureKind::Transient
    );
    assert!(!should_refresh_token(
        StatusCode::UNAUTHORIZED,
        "invalid_grant"
    ));
    assert!(!should_refresh_token(
        StatusCode::UNAUTHORIZED,
        "insufficient_quota"
    ));
}

#[test]
fn auth_refresh_is_claimed_once_across_request_retries() {
    let mut attempted = false;
    assert!(!claim_auth_refresh(
        StatusCode::SERVICE_UNAVAILABLE,
        "timeout",
        &mut attempted
    ));
    assert!(!attempted);
    assert!(claim_auth_refresh(
        StatusCode::UNAUTHORIZED,
        "",
        &mut attempted
    ));
    assert!(attempted);
    assert!(!claim_auth_refresh(
        StatusCode::UNAUTHORIZED,
        "",
        &mut attempted
    ));
    assert!(!claim_auth_refresh(
        StatusCode::FORBIDDEN,
        "token expired",
        &mut attempted
    ));
}
