use super::*;

#[tokio::test]
async fn expired_bootstrap_is_rejected_and_remote_cookie_is_secure() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Mutex::new(
        Store::open(dir.path().join("state.json")).unwrap(),
    ));
    let state = app_state(
        store,
        WebConfig {
            local_origin: "http://127.0.0.1:8123".into(),
            tailnet_origin: Some("https://node.test.ts.net:8443".into()),
            control_token: "test-control".into(),
            default_working_dir: dir.path().into(),
            default_provider: None,
            default_model: None,
        },
    );
    state.bootstrap.lock().await.insert(
        "expired".into(),
        (
            state.config.local_origin.clone(),
            Utc::now() - Duration::seconds(1),
        ),
    );
    assert_eq!(
        login(
            State(state.clone()),
            axum::Form(Login {
                token: "expired".into()
            })
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    assert!(state.sessions.lock().await.is_empty());
    state.bootstrap.lock().await.insert(
        "remote".into(),
        (
            state.config.tailnet_origin.clone().unwrap(),
            Utc::now() + Duration::seconds(60),
        ),
    );
    let response = login(
        State(state.clone()),
        axum::Form(Login {
            token: "remote".into(),
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    for cookie in response.headers().get_all(header::SET_COOKIE) {
        let cookie = cookie.to_str().unwrap();
        assert!(cookie.contains("; Secure"));
        assert!(cookie.contains("SameSite=Strict"));
        assert!(!cookie.contains("Domain="));
    }
    assert!(
        state
            .sessions
            .lock()
            .await
            .values()
            .all(|(_, _, remote)| *remote)
    );
}

#[tokio::test]
async fn real_http_pairing_requires_origin_session_and_csrf() {
    let dir = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    let store = Arc::new(Mutex::new(
        Store::open(dir.path().join("state.json")).unwrap(),
    ));
    let config = WebConfig {
        local_origin: origin.clone(),
        tailnet_origin: Some("https://node.test.ts.net:8443".into()),
        control_token: "control-test".into(),
        default_working_dir: dir.path().into(),
        default_provider: None,
        default_model: None,
    };
    let cancel = CancellationToken::new();
    let task = tokio::spawn(serve(listener, store, config, cancel.clone()));
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    assert_eq!(
        client
            .get(format!("{origin}/api/status"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let status = client
        .get(format!("{origin}/api/status"))
        .bearer_auth("control-test")
        .send()
        .await
        .unwrap();
    assert_eq!(status.status(), 200);
    let status: serde_json::Value = status.json().await.unwrap();
    assert_eq!(status["service"], "jcode-automation-bulletin");
    assert_eq!(status["persistence_ready"], true);
    assert_eq!(
        client
            .get(format!("{origin}/api/status"))
            .header("Host", "node.test.ts.net:8443")
            .bearer_auth("control-test")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .get(format!("{origin}/api/runs"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .get(format!("{origin}/api/runs"))
            .header("Host", "evil.test")
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(format!("{origin}/api/bootstrap"))
            .header("Origin", &origin)
            .bearer_auth("wrong")
            .json(&serde_json::json!({"target":"local"}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let response = client
        .post(format!("{origin}/api/bootstrap"))
        .header("Origin", &origin)
        .bearer_auth("control-test")
        .json(&serde_json::json!({"target":"local"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let value: serde_json::Value = response.json().await.unwrap();
    let login_url = value["url"].as_str().unwrap();
    let parsed = url::Url::parse(login_url).unwrap();
    let token = parsed
        .query_pairs()
        .find(|(k, _)| k == "token")
        .unwrap()
        .1
        .into_owned();
    let login = client
        .post(format!("{origin}/login"))
        .header("Origin", &origin)
        .form(&[("token", &token)])
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 303);
    let cookies: Vec<String> = login
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().split(';').next().unwrap().to_owned())
        .collect();
    let cookie = cookies.join("; ");
    let csrf = cookies
        .iter()
        .find_map(|c| c.strip_prefix("bulletin_csrf="))
        .unwrap();
    assert_eq!(
        client
            .post(format!("{origin}/login"))
            .header("Origin", &origin)
            .form(&[("token", &token)])
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .get(format!("{origin}/api/runs"))
            .header("Cookie", &cookie)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .get(format!("{origin}/api/runs"))
            .header("Cookie", &cookie)
            .header("Host", "node.test.ts.net:8443")
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .post(format!("{origin}/logout"))
            .header("Origin", &origin)
            .header("Cookie", &cookie)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(format!("{origin}/logout"))
            .header("Origin", "https://evil.test")
            .header("Cookie", &cookie)
            .form(&[("csrf", csrf)])
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(format!("{origin}/logout"))
            .header("Origin", &origin)
            .header("Cookie", &cookie)
            .form(&[("csrf", csrf)])
            .send()
            .await
            .unwrap()
            .status(),
        303
    );
    assert_eq!(
        client
            .get(format!("{origin}/api/runs"))
            .header("Cookie", &cookie)
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let oversized = client
        .post(format!("{origin}/api/bootstrap"))
        .header("Origin", &origin)
        .body("x".repeat(MAX_BODY + 1))
        .send()
        .await
        .unwrap();
    assert_eq!(oversized.status(), 413);
    cancel.cancel();
    tokio::time::timeout(std::time::Duration::from_secs(3), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
