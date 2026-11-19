use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn fixture() -> (tokio::net::TcpListener, MultiProvider) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind capture endpoint");
    let api_base = format!("http://{}/v1", listener.local_addr().unwrap());
    let mut config = String::from("[provider]\ndefault_provider = \"alpha\"\n");
    for (name, affinity) in [("alpha", true), ("beta", true), ("plain", false)] {
        config.push_str(&format!(
            "\n[providers.{name}]\ntype = \"openai-compatible\"\nbase_url = \"{api_base}\"\nauth = \"none\"\ndefault_model = \"vendor/cache-model\"\nmodel_catalog = false\n"
        ));
        if affinity {
            config.push_str("session_cache_affinity = true\n");
        }
        config.push_str(&format!(
            "[[providers.{name}.models]]\nid = \"vendor/cache-model\"\n"
        ));
    }
    let home = std::path::PathBuf::from(std::env::var_os("JCODE_HOME").unwrap());
    std::fs::write(home.join("config.toml"), config).expect("write isolated profile config");
    crate::config::invalidate_config_cache();
    let router = MultiProvider::new_with_auth_status(crate::auth::AuthStatus::default());
    assert_eq!(router.active_provider(), ActiveProvider::OpenRouter);
    router
        .set_model("alpha:vendor/cache-model")
        .expect("select named profile");
    (listener, router)
}

async fn capture(
    listener: &tokio::net::TcpListener,
    provider: &dyn Provider,
    split: bool,
) -> (serde_json::Value, Option<String>) {
    let request = async {
        let (mut socket, _) = listener.accept().await.expect("accept local request");
        let mut bytes = Vec::new();
        let (header_end, body_length) = loop {
            let mut buffer = [0; 4096];
            let count = socket.read(&mut buffer).await.expect("read request");
            assert_ne!(count, 0, "request ended before headers");
            bytes.extend_from_slice(&buffer[..count]);
            assert!(bytes.len() < 1_048_576, "request unexpectedly large");
            if let Some(offset) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                let headers = std::str::from_utf8(&bytes[..offset]).unwrap();
                assert!(headers.starts_with("POST /v1/chat/completions "));
                let length = headers
                    .lines()
                    .filter_map(|line| line.split_once(':'))
                    .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
                    .expect("request content length")
                    .1
                    .trim()
                    .parse::<usize>()
                    .unwrap();
                assert!(length < 1_048_576, "request body unexpectedly large");
                break (offset + 4, length);
            }
        };
        while bytes.len() < header_end + body_length {
            let mut buffer = [0; 4096];
            let count = socket.read(&mut buffer).await.expect("read request body");
            assert_ne!(count, 0, "request body truncated");
            bytes.extend_from_slice(&buffer[..count]);
        }
        let headers = std::str::from_utf8(&bytes[..header_end]).unwrap();
        let session_header = headers
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.eq_ignore_ascii_case("x-session-id"))
            .map(|(_, value)| value.trim().to_owned());
        let body = serde_json::from_slice(&bytes[header_end..header_end + body_length])
            .expect("parse captured JSON");
        let response = "data: [DONE]\n\n";
        socket
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                    response.len()
                )
                .as_bytes(),
            )
            .await
            .expect("write response");
        (body, session_header)
    };
    let completion = async {
        let messages = [Message {
            role: crate::message::Role::User,
            content: vec![crate::message::ContentBlock::Text {
                text: "cache lifecycle regression".into(),
                cache_control: None,
            }]
            .into(),
            timestamp: None,
            tool_duration_ms: None,
        }];
        let mut stream = if split {
            provider
                .complete_split(&messages, &[], "static", "dynamic", None)
                .await
        } else {
            provider.complete(&messages, &[], "static", None).await
        }
        .expect("start real MultiProvider completion");
        while let Some(event) = futures::StreamExt::next(&mut stream).await {
            event.expect("parse completion event");
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let (captured, ()) = tokio::join!(request, completion);
        captured
    })
    .await
    .expect("local completion must finish")
}

async fn cache_key(
    listener: &tokio::net::TcpListener,
    provider: &dyn Provider,
    split: bool,
) -> String {
    let (body, header) = capture(listener, provider, split).await;
    assert_eq!(body["model"], "vendor/cache-model");
    let key = body["prompt_cache_key"]
        .as_str()
        .expect("enabled profile must emit prompt_cache_key");
    assert!(!key.is_empty());
    assert_eq!(
        header.as_deref(),
        Some(key),
        "body/header identity must agree"
    );
    assert!(
        !key.contains("persisted-session"),
        "raw session ID must not leak"
    );
    key.to_owned()
}

#[test]
fn session_cache_affinity_survives_reuse_restore_replacement_and_session_forks() {
    with_clean_provider_test_env(|| {
        let sandbox = tempfile::tempdir().unwrap();
        let _home = OrEnvVarGuard::set("HOME", sandbox.path());
        let _appdata = OrEnvVarGuard::set("APPDATA", sandbox.path());
        let _env = isolate_openrouter_autodetect_env_or();
        enter_test_runtime().block_on(async {
            let (listener, router) = fixture().await;
            router.set_session_id("persisted-session-parent");
            let parent = cache_key(&listener, &router, false).await;
            assert_eq!(cache_key(&listener, &router, true).await, parent);
            router.set_session_id("persisted-session-parent");
            assert_eq!(cache_key(&listener, &router, false).await, parent);

            let previous = router.active_openrouter_execution_provider().unwrap();
            router.openai_compatible_profiles.write().unwrap().clear();
            router.set_model("alpha:vendor/cache-model").unwrap();
            assert!(!Arc::ptr_eq(
                &previous,
                &router.active_openrouter_execution_provider().unwrap()
            ));
            assert_eq!(cache_key(&listener, &router, true).await, parent);

            let restored = MultiProvider::new_with_auth_status(crate::auth::AuthStatus::default());
            restored.set_session_id("persisted-session-parent");
            restored.set_model("alpha:vendor/cache-model").unwrap();
            assert_eq!(cache_key(&listener, &restored, false).await, parent);

            let resumed = router.fork();
            assert_eq!(cache_key(&listener, resumed.as_ref(), true).await, parent);
            let fresh = router.fork_for_new_session();
            let provisional = cache_key(&listener, fresh.as_ref(), false).await;
            assert_ne!(provisional, parent);
            assert_eq!(
                cache_key(&listener, fresh.as_ref(), true).await,
                provisional
            );
            let another_fresh = router.fork_for_new_session();
            assert_ne!(
                cache_key(&listener, another_fresh.as_ref(), false).await,
                provisional,
                "independent unbound sessions need separate provisional identities"
            );
            fresh.set_session_id("persisted-session-child");
            let child = cache_key(&listener, fresh.as_ref(), false).await;
            assert_ne!(child, parent);
            assert_ne!(child, provisional);
            assert_eq!(cache_key(&listener, &router, false).await, parent);
            assert_eq!(cache_key(&listener, resumed.as_ref(), false).await, parent);
            router.set_session_id("persisted-session-rebound");
            assert_ne!(cache_key(&listener, &router, false).await, parent);
            assert_eq!(cache_key(&listener, resumed.as_ref(), false).await, parent);
            assert_eq!(cache_key(&listener, fresh.as_ref(), false).await, child);
        });
    });
}

#[test]
fn session_cache_affinity_is_profile_scoped_and_opt_in() {
    with_clean_provider_test_env(|| {
        let sandbox = tempfile::tempdir().unwrap();
        let _home = OrEnvVarGuard::set("HOME", sandbox.path());
        let _appdata = OrEnvVarGuard::set("APPDATA", sandbox.path());
        let _env = isolate_openrouter_autodetect_env_or();
        enter_test_runtime().block_on(async {
            let (listener, router) = fixture().await;
            router.set_session_id("persisted-session-shared");
            let alpha = cache_key(&listener, &router, false).await;
            router.set_model("beta:vendor/cache-model").unwrap();
            let beta = cache_key(&listener, &router, true).await;
            assert_ne!(alpha, beta, "profiles must not share a cache identity");
            router.set_model("alpha:vendor/cache-model").unwrap();
            assert_eq!(cache_key(&listener, &router, false).await, alpha);
            router.set_model("plain:vendor/cache-model").unwrap();
            let (body, header) = capture(&listener, &router, false).await;
            assert!(body.get("prompt_cache_key").is_none());
            assert!(
                header.is_none(),
                "default profile must not emit x-session-id"
            );
        });
    });
}
