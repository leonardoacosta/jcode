use super::*;
use crate::tests::{ENV_LOCK, EnvVarGuard};
use std::io::{Read, Write};
use std::time::Duration;

fn profile(enabled: bool) -> jcode_base::config::NamedProviderConfig {
    serde_json::from_value(serde_json::json!({
        "base_url": "http://127.0.0.1:1/v1", "auth": "none",
        "default_model": "test-model", "session_cache_affinity": enabled
    }))
    .unwrap()
}

fn provider(name: &str, enabled: bool) -> OpenRouterProvider {
    OpenRouterProvider::new_named_openai_compatible(name, &profile(enabled)).unwrap()
}

// Exercise the real transport, including protocol conversion and retry reconstruction.
fn capture(
    provider: &mut OpenRouterProvider,
    responses: bool,
    retry: bool,
) -> Vec<(Value, String)> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for attempt in 0..if retry { 2 } else { 1 } {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0; 4096];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                assert!(count > 0);
                bytes.extend_from_slice(&buffer[..count]);
                if let Some(split) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..split]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.split_once(':')
                                .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
                                .map(|(_, value)| value.trim().parse().unwrap())
                        })
                        .unwrap();
                    if bytes.len() >= split + 4 + length {
                        break;
                    }
                }
            }
            let wire = String::from_utf8(bytes).unwrap();
            tx.send(wire).unwrap();
            let body = if responses {
                "data: {\"type\":\"response.completed\",\"response\":{\"status\":\"completed\",\"output\":[]}}\n\n"
            } else {
                "data: {\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n"
            };
            let (status, body) = if retry && attempt == 0 {
                ("503 Service Unavailable", "temporarily unavailable")
            } else {
                ("200 OK", body)
            };
            write!(stream, "HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
    });
    provider.api_base = format!("http://{addr}/v1");
    // Preserve inferred OpenCode Responses routing when the explicit selector is off.
    provider.responses_api = responses && provider.profile_id.as_deref() != Some("opencode-go");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let mut events = provider
            .complete(&[Message::user("hello")], &[], "system", None)
            .await
            .unwrap();
        while let Some(event) = events.next().await {
            event.unwrap();
        }
    });
    (0..if retry { 2 } else { 1 })
        .map(|_| {
            let wire = rx.recv_timeout(Duration::from_secs(5)).unwrap();
            let (headers, body) = wire.split_once("\r\n\r\n").unwrap();
            assert!(headers.starts_with(if responses {
                "POST /v1/responses "
            } else {
                "POST /v1/chat/completions "
            }));
            (serde_json::from_str(body).unwrap(), headers.to_string())
        })
        .collect()
}

#[test]
fn affinity_config_defaults_off_and_round_trips_opt_in() {
    let default = serde_json::to_value(jcode_base::config::NamedProviderConfig::default()).unwrap();
    assert_eq!(default["session_cache_affinity"], false);
    let enabled = serde_json::to_value(profile(true)).unwrap();
    assert_eq!(enabled["session_cache_affinity"], true);
    let omitted: jcode_base::config::NamedProviderConfig =
        toml::from_str("base_url = 'http://localhost/v1'").unwrap();
    assert!(!omitted.session_cache_affinity);
    let enabled: jcode_base::config::NamedProviderConfig =
        toml::from_str("session_cache_affinity = true").unwrap();
    assert!(
        toml::from_str::<jcode_base::config::NamedProviderConfig>(
            &toml::to_string(&enabled).unwrap()
        )
        .unwrap()
        .session_cache_affinity
    );
}

#[test]
fn affinity_wire_default_off_and_both_protocols_opt_in() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    for responses in [false, true] {
        for enabled in [false, true] {
            let mut provider = provider("affinity-test", enabled);
            provider.set_session_id("persisted-session-private");
            let wire = capture(&mut provider, responses, false);
            let (body, headers) = &wire[0];
            let header = headers.lines().find_map(|line| {
                line.split_once(':')
                    .filter(|(name, _)| name.eq_ignore_ascii_case("x-session-id"))
                    .map(|(_, value)| value.trim())
            });
            if enabled {
                let key = body["prompt_cache_key"].as_str().expect("affinity key");
                assert_eq!(header, Some(key));
                assert!(!key.contains("persisted-session-private"));
                assert!(!key.contains("affinity-test"));
            } else {
                assert!(body.get("prompt_cache_key").is_none());
                assert!(header.is_none());
            }
        }
    }
}

#[test]
fn affinity_lifecycle_is_stable_scoped_opaque_and_forks_rebind() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let mut first = provider("private-profile", true);
    first.set_session_id("private-session");
    let original = capture(&mut first, false, false)[0].0["prompt_cache_key"].clone();
    first.set_session_id("");
    first.set_session_id("private-session");
    assert_eq!(
        capture(&mut first, false, false)[0].0["prompt_cache_key"],
        original
    );
    let mut rebuilt = provider("private-profile", true);
    rebuilt.set_session_id("private-session");
    assert_eq!(
        capture(&mut rebuilt, false, false)[0].0["prompt_cache_key"],
        original
    );
    rebuilt.set_session_id("other-session");
    assert_ne!(
        capture(&mut rebuilt, false, false)[0].0["prompt_cache_key"],
        original
    );
    let mut other_profile = provider("other-profile", true);
    other_profile.set_session_id("private-session");
    assert_ne!(
        capture(&mut other_profile, false, false)[0].0["prompt_cache_key"],
        original
    );
    assert!(!original.as_str().unwrap().contains("private"));
    let affinity = first.session_cache_affinity.as_ref().unwrap();
    let forked = first.fork();
    assert_ne!(affinity.key(), first.conversation_id);
    // A concrete independent state follows the same fork constructor contract.
    let fresh = affinity.fork();
    assert_ne!(fresh.key(), affinity.key());
    fresh.bind("private-session");
    assert_eq!(fresh.key(), affinity.key());
    forked.set_session_id("private-session");
}

#[test]
fn affinity_explicit_overrides_and_retries_remain_coherent() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    for responses in [false, true] {
        let mut config = profile(true);
        config.extra_body = Some(
            serde_json::json!({"prompt_cache_key":"explicit-key", "metadata":{"custom":true}}),
        );
        config
            .headers
            .insert("X-Session-ID".into(), "explicit-key".into());
        let mut provider =
            OpenRouterProvider::new_named_openai_compatible("overrides", &config).unwrap();
        let wire = capture(&mut provider, responses, true);
        assert_eq!(wire[0].0, wire[1].0);
        for (body, headers) in wire {
            assert_eq!(body["prompt_cache_key"], "explicit-key");
            assert_eq!(body["metadata"]["custom"], true);
            assert!(
                headers
                    .to_ascii_lowercase()
                    .contains("x-session-id: explicit-key")
            );
        }
    }
}

#[test]
fn affinity_conflicts_and_invalid_explicit_keys_fail_before_transport() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    for key in [
        serde_json::json!("different"),
        serde_json::json!(null),
        serde_json::json!(false),
        serde_json::json!(""),
        serde_json::json!("bad\nheader"),
    ] {
        let mut config = profile(true);
        config.extra_body = Some(serde_json::json!({"prompt_cache_key":key}));
        config
            .headers
            .insert("x-session-id".into(), "configured".into());
        let provider =
            OpenRouterProvider::new_named_openai_compatible("conflict", &config).unwrap();
        let error = runtime
            .block_on(provider.complete(&[], &[], "", None))
            .err()
            .expect("must reject invalid affinity");
        assert!(error.to_string().contains("session cache affinity"));
    }
}

#[test]
fn affinity_rejects_duplicate_and_auth_header_conflicts() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let mut config = profile(true);
    config.headers.insert("x-session-id".into(), "first".into());
    config
        .headers
        .insert("X-Session-ID".into(), "second".into());
    assert!(OpenRouterProvider::new_named_openai_compatible("conflict", &config).is_err());
    config.headers.clear();
    config.auth = jcode_base::config::NamedProviderAuth::Header;
    config.auth_header = Some("X-Session-ID".into());
    config.api_key = Some("private-token".into());
    assert!(OpenRouterProvider::new_named_openai_compatible("auth-conflict", &config).is_err());
}

#[test]
fn affinity_strict_profile_stays_off_and_explicit_body_is_untouched() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let mut config = profile(false);
    config.extra_body = Some(serde_json::json!({"prompt_cache_key":"manual-key"}));
    let mut provider = OpenRouterProvider::new_named_openai_compatible("mistral", &config).unwrap();
    provider.set_session_id("strict-session");
    for responses in [false, true] {
        let wire = capture(&mut provider, responses, false);
        assert_eq!(wire[0].0["prompt_cache_key"], "manual-key");
        assert!(!wire[0].1.to_ascii_lowercase().contains("x-session-id:"));
    }
}

#[test]
fn affinity_inferred_responses_preserves_final_key() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let mut config = profile(true);
    config.default_model = Some("gpt-6-luna".into());
    config.extra_body = Some(serde_json::json!({"prompt_cache_key":"inferred-override"}));
    let mut provider =
        OpenRouterProvider::new_named_openai_compatible("opencode-go", &config).unwrap();
    provider.set_session_id("inferred-session");
    let wire = capture(&mut provider, true, false);
    assert_eq!(wire[0].0["prompt_cache_key"], "inferred-override");
    assert!(
        wire[0]
            .1
            .to_ascii_lowercase()
            .contains("x-session-id: inferred-override")
    );
    assert!(wire[0].0.get("messages").is_none());
}

#[test]
fn affinity_header_only_override_sets_matching_body() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let mut config = profile(true);
    config
        .headers
        .insert("X-SESSION-ID".into(), "header-only".into());
    let mut provider =
        OpenRouterProvider::new_named_openai_compatible("header-only", &config).unwrap();
    provider.set_session_id("ignored-by-explicit-header");
    let wire = capture(&mut provider, false, false);
    assert_eq!(wire[0].0["prompt_cache_key"], "header-only");
    assert!(
        wire[0]
            .1
            .to_ascii_lowercase()
            .contains("x-session-id: header-only")
    );
}

#[test]
fn affinity_generated_identity_is_stable_across_retries_and_model_changes() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let mut provider = provider("retry-generated", true);
    provider.set_session_id("persisted-retry");
    let initial = capture(&mut provider, false, true);
    let key = initial[0].0["prompt_cache_key"].as_str().unwrap();
    assert_eq!(initial[1].0["prompt_cache_key"], key);
    for (_, headers) in &initial {
        assert!(headers.contains(&format!("x-session-id: {key}")));
    }
    provider.set_model("another-model").unwrap();
    let later = capture(&mut provider, true, false);
    assert_eq!(later[0].0["prompt_cache_key"], key);
}

#[test]
fn affinity_review_rejects_padded_overrides_before_any_http() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        for responses in [false, true] {
            for header_only in [false, true] {
                for padded in [" explicit-key", "explicit-key ", "\texplicit-key", "explicit-key\t"] {
                    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                    let mut config = profile(true);
                    config.base_url = format!("http://{}/v1", listener.local_addr().unwrap());
                    config.api = responses.then(|| "responses".into());
                    if header_only {
                        config.headers.insert("X-SESSION-ID".into(), padded.into());
                    } else {
                        config.extra_body = Some(serde_json::json!({"prompt_cache_key":padded}));
                    }
                    let provider = OpenRouterProvider::new_named_openai_compatible("padded-review", &config).unwrap();
                    tokio::select! {
                        result = provider.complete(&[], &[], "", None) => {
                            let error = result.err().expect("padded affinity must fail before transport");
                            assert!(error.to_string().contains("session cache affinity"));
                        }
                        _ = listener.accept() => panic!("padded affinity reached HTTP"),
                        _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("padded affinity did not fail promptly"),
                    }
                    assert!(tokio::time::timeout(Duration::from_millis(5), listener.accept()).await.is_err());
                }
            }
        }
    });
}

#[test]
fn affinity_review_validates_before_cold_catalog_and_routing_http() {
    let _lock = ENV_LOCK.lock();
    let _namespace = EnvVarGuard::remove("JCODE_OPENROUTER_CACHE_NAMESPACE");
    let _extra = EnvVarGuard::remove("JCODE_OPENAI_EXTRA_BODY");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        for catalog in [false, true] {
            for responses in [false, true] {
                // Responses profiles cannot use provider routing.
                if responses && !catalog { continue; }
                for key in [serde_json::json!(null), serde_json::json!(false), serde_json::json!(""), serde_json::json!("bad\nheader"), serde_json::json!("different")] {
                    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                    let mut config = profile(true);
                    config.base_url = format!("http://{}/v1", listener.local_addr().unwrap());
                    config.api = responses.then(|| "responses".into());
                    config.model_catalog = catalog;
                    config.provider_routing = !catalog;
                    let id = uuid::Uuid::new_v4().to_string();
                    config.default_model = Some(id.clone());
                    config.extra_body = Some(serde_json::json!({"prompt_cache_key":key}));
                    config.headers.insert("x-session-id".into(), "configured".into());
                    let provider = OpenRouterProvider::new_named_openai_compatible(&id, &config).unwrap();
                    tokio::select! {
                        result = provider.complete(&[], &[], "", None) => {
                            let error = result.err().expect("invalid affinity must fail before transport");
                            assert!(error.to_string().contains("session cache affinity"));
                        }
                        _ = listener.accept() => panic!("invalid affinity reached catalog/routing HTTP (catalog={catalog}, responses={responses})"),
                        _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("invalid affinity did not fail promptly"),
                    }
                    assert!(tokio::time::timeout(Duration::from_millis(5), listener.accept()).await.is_err());
                }
            }
        }
    });
}
