use super::*;
use std::future;
use std::io::{BufRead, Read, Write};

type ScriptedHttpResponse = (u16, Vec<(&'static str, &'static str)>, String);

fn accept_token_request(listener: &std::net::TcpListener) -> std::net::TcpStream {
    loop {
        let (mut stream, _) = listener.accept().expect("accept test request");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set request read timeout");
        let mut request = std::io::BufReader::new(&mut stream);
        let mut headers = request.by_ref().take(64 * 1024);
        let mut line = String::new();
        assert_ne!(headers.read_line(&mut line).expect("read request line"), 0);
        let is_token_request = line == "POST /v1/auth/token HTTP/1.1\r\n";
        let mut content_length = 0;
        loop {
            line.clear();
            assert_ne!(headers.read_line(&mut line).expect("read header"), 0);
            if line == "\r\n" {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                content_length = value.trim().parse().expect("request content length");
            }
        }
        assert!(
            content_length <= 64 * 1024,
            "request body exceeds fixture limit"
        );
        let mut body = vec![0; content_length];
        request.read_exact(&mut body).expect("read request body");
        if is_token_request {
            return stream;
        }
        // Local HTTP probes must not consume a scripted token response.
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    }
}

fn spawn_scripted_http_server(responses: Vec<ScriptedHttpResponse>) -> String {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    std::thread::spawn(move || {
        for (status, headers, body) in responses {
            let mut stream = accept_token_request(&listener);
            let headers = headers
                .into_iter()
                .map(|(name, value)| format!("{name}: {value}\r\n"))
                .collect::<String>();
            let response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
    format!("http://127.0.0.1:{}/v1", addr.port())
}

#[test]
fn token_fixture_rejects_oversized_headers_and_bodies() {
    for request in [
        format!(
            "POST /v1/auth/token HTTP/1.1\r\nX-Padding: {}\r\nContent-Length: 0\r\n\r\n",
            "x".repeat(64 * 1024)
        ),
        format!(
            "POST /v1/auth/token HTTP/1.1\r\nContent-Length: 65537\r\n\r\n{}",
            "x".repeat(65537)
        ),
    ] {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind fixture");
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || accept_token_request(&listener));
        let mut stream = std::net::TcpStream::connect(address).expect("connect fixture");
        let _ = stream.write_all(request.as_bytes());
        let _ = stream.shutdown(std::net::Shutdown::Write);
        assert!(
            server.join().is_err(),
            "oversized request must fail fixture"
        );
    }
}

#[tokio::test]
async fn scripted_responses_are_not_consumed_by_unrelated_http_probes() {
    let body = r#"{"error":"authorization_pending"}"#;
    let base = spawn_scripted_http_server(vec![(428, vec![], body.to_string())]);
    let client = test_client();
    let probe = client
        .get(base.trim_end_matches("/v1"))
        .header("User-Agent", "Go-http-client/1.1")
        .send()
        .await
        .expect("unrelated HTTP probe");
    assert_eq!(probe.status(), 404);
    let response = client
        .post(format!("{base}/auth/token"))
        .json(&serde_json::json!({ "device_code": "x".repeat(8192) }))
        .send()
        .await
        .expect("token request after probe");
    assert_eq!(response.status(), 428);
    assert_eq!(response.text().await.expect("scripted response body"), body);
}

fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("build test client")
}

#[tokio::test]
async fn polling_pending_slow_down_then_approval() {
    let base = spawn_scripted_http_server(vec![
        (428, vec![], r#"{"error":"authorization_pending"}"#.to_string()),
        (
            429,
            vec![("Retry-After", "1")],
            r#"{"error":"slow_down"}"#.to_string(),
        ),
        (
            200,
            vec![],
            r#"{"api_key":"jck_live_test","account_id":"acct_42","email":"user@example.com","tier":"none","status":"active"}"#.to_string(),
        ),
    ]);
    let approved = poll_for_api_key(
        &test_client(),
        &base,
        "device-secret",
        1,
        10,
        future::pending(),
    )
    .await
    .expect("approval");
    let KeyPollCompletion::Approved(approved) = approved else {
        panic!("approval was unexpectedly canceled");
    };
    assert_eq!(approved.account_id, "acct_42");
    assert_eq!(approved.email, "user@example.com");
    assert_eq!(approved.tier, "none");
}

#[tokio::test]
async fn polling_denied_has_clear_redacted_error() {
    let base = spawn_scripted_http_server(vec![(
        400,
        vec![],
        r#"{"error":"access_denied","message":"device-secret-must-not-appear"}"#.to_string(),
    )]);
    let error = poll_for_api_key(
        &test_client(),
        &base,
        "device-secret",
        1,
        3,
        future::pending(),
    )
    .await
    .expect_err("denied");
    let message = error.to_string();
    assert!(message.contains("canceled or denied"), "{message}");
    assert!(!message.contains("device-secret"), "{message}");
}

#[tokio::test]
async fn polling_timeout_is_deterministic_before_first_request() {
    let error = poll_for_api_key(
        &test_client(),
        "http://127.0.0.1:9/v1",
        "device-secret",
        2,
        1,
        future::pending(),
    )
    .await
    .expect_err("timeout");
    assert!(error.to_string().contains("timed out"));
}

#[tokio::test]
async fn cancellation_during_consumed_exchange_finishes_and_returns_the_key() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    std::thread::spawn(move || {
        let mut stream = accept_token_request(&listener);
        std::thread::sleep(Duration::from_millis(300));
        let body = r#"{"api_key":"jck_live_test","account_id":"acct_42","email":"user@example.com","tier":"pro","status":"active"}"#;
        let response = format!(
            "HTTP/1.1 200 Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write exchange");
    });
    let base = format!("http://127.0.0.1:{}/v1", addr.port());
    let cancel = async {
        tokio::time::sleep(Duration::from_millis(1100)).await;
        Ok(())
    };
    let outcome = poll_for_api_key(&test_client(), &base, "device-secret", 1, 5, cancel)
        .await
        .expect("exchange must finish");
    assert!(matches!(outcome, KeyPollCompletion::Approved(_)));
}

#[test]
fn approved_key_persistence_is_owner_only_and_clear_is_deterministic() {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::TempDir::new().expect("temp dir");
    let previous_home = std::env::var_os("JCODE_HOME");
    let previous_key = std::env::var_os(crate::subscription_catalog::JCODE_API_KEY_ENV);
    crate::env::set_var("JCODE_HOME", temp.path());
    crate::env::remove_var(crate::subscription_catalog::JCODE_API_KEY_ENV);

    let approved = ApprovedAccountKey {
        api_key: "jck_live_test".to_string(),
        account_id: "acct_42".to_string(),
        email: "user@example.com".to_string(),
        tier: "none".to_string(),
        status: "active".to_string(),
    };
    persist_approved_key(&approved).expect("persist");
    let path = crate::subscription_catalog::account_credential_path().expect("path");
    let content = std::fs::read_to_string(&path).expect("read");
    assert!(content.contains("JCODE_API_KEY=jck_live_test"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    crate::subscription_catalog::clear_account_credentials().expect("clear");
    assert!(crate::subscription_catalog::configured_api_key().is_none());
    let cleared = std::fs::read_to_string(&path).expect("read cleared");
    assert!(!cleared.contains("jck_live_test"));
    assert!(!cleared.contains("acct_42"));
    assert!(!cleared.contains("user@example.com"));

    match previous_home {
        Some(value) => crate::env::set_var("JCODE_HOME", value),
        None => crate::env::remove_var("JCODE_HOME"),
    }
    match previous_key {
        Some(value) => crate::env::set_var(crate::subscription_catalog::JCODE_API_KEY_ENV, value),
        None => crate::env::remove_var(crate::subscription_catalog::JCODE_API_KEY_ENV),
    }
}
