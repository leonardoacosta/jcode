use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use jcode_mac_browser_fleet::{
    Action, AuthorityAction, AuthorityEnvelope, Broker, BrokerConfig, BrowserKind, Capability,
    CdpAdapter, CdpEndpoint, FleetErrorKind, FleetRequest, FleetResponse, InventoryUpdate,
    ManagedCdpSource, ManagedTarget, MutationReplayGuard, NativeHostBridgeConfig, ProtocolCodec,
    ProtocolSession, TargetRef, forward_native_payload, read_native_message, serve_native_host,
    write_native_message,
};
use tempfile::tempdir;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

fn secret() -> String {
    "test-peer-secret-with-enough-entropy".to_string()
}

fn native_secret() -> String {
    "test-native-secret-with-enough-entropy".to_string()
}

fn target(generation: u64) -> TargetRef {
    TargetRef {
        browser_id: "chrome:stable".into(),
        window_id: "win-1".into(),
        tab_id: "tab-1".into(),
        generation,
    }
}

async fn broker_line(socket: &Path, request: serde_json::Value) -> serde_json::Value {
    let mut stream = tokio::net::UnixStream::connect(socket).await.unwrap();
    let mut encoded = serde_json::to_vec(&request).unwrap();
    encoded.push(b'\n');
    stream.write_all(&encoded).await.unwrap();
    stream.flush().await.unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = Vec::new();
    reader.read_until(b'\n', &mut line).await.unwrap();
    serde_json::from_slice(&line).unwrap()
}

async fn peer_list(socket: &Path, auth: String, generation: u64) -> serde_json::Value {
    broker_line(
        socket,
        serde_json::json!({
            "version": 1,
            "auth": auth,
            "id": format!("list-{generation}"),
            "deadline_ms": 1000,
            "target_generation": generation,
            "action": "listBrowsers",
            "payload": {}
        }),
    )
    .await
}

async fn internal_snapshot(
    socket: &Path,
    browser: &str,
    profile: &str,
    native_tab: u64,
) -> serde_json::Value {
    broker_line(
        socket,
        serde_json::json!({
            "version": 1,
            "auth": native_secret(),
            "id": format!("snapshot-{browser}-{native_tab}"),
            "action": "extensionInventorySnapshot",
            "payload": {
                "snapshot": {
                    "browserKind": browser,
                    "displayName": if browser == "edge" { "Microsoft Edge" } else { "Google Chrome" },
                    "profileLabel": profile,
                    "generation": 1,
                    "capabilities": ["navigate"],
                    "windows": [{
                        "windowRef": format!("win-{browser}"),
                        "nativeWindowId": native_tab + 1000,
                        "focused": true,
                        "tabs": [{
                            "tabRef": format!("tab-{browser}"),
                            "windowRef": format!("win-{browser}"),
                            "nativeWindowId": native_tab + 1000,
                            "nativeTabId": native_tab,
                            "active": true,
                            "controllable": true,
                            "capabilities": ["navigate"],
                            "title": "Example",
                            "url": "https://example.test/path"
                        }]
                    }]
                }
            }
        }),
    )
    .await
}

#[test]
fn protocol_fails_closed_for_bad_inputs_without_leaking_secret() {
    let codec = ProtocolCodec::new(vec![1], 1024, secret());
    let err = codec
        .decode_request("x".repeat(2048).as_bytes())
        .unwrap_err();
    assert_eq!(err.kind(), FleetErrorKind::Oversized);
    assert!(!err.to_string().contains("test-secret"));
    let unsupported = serde_json::json!({"version":99,"auth": secret(),"id":"r1","deadline_ms":1000,"target_generation":1,"action":"fleetHealth","payload":{}});
    assert_eq!(
        codec
            .decode_request(unsupported.to_string().as_bytes())
            .unwrap_err()
            .kind(),
        FleetErrorKind::UnsupportedVersion
    );
    let unauth = serde_json::json!({"version":1,"auth":"wrong","id":"r1","deadline_ms":1000,"target_generation":1,"action":"fleetHealth","payload":{}});
    assert_eq!(
        codec
            .decode_request(unauth.to_string().as_bytes())
            .unwrap_err()
            .kind(),
        FleetErrorKind::Unauthenticated
    );
    assert_eq!(
        codec.decode_request(b"{not json").unwrap_err().kind(),
        FleetErrorKind::Malformed
    );

    let valid = serde_json::json!({"version":1,"auth": secret(),"id":"dup","deadline_ms":1000,"target_generation":1,"action":"fleetHealth","payload":{}});
    let mut session = ProtocolSession::new(codec);
    session
        .decode_unique_request(valid.to_string().as_bytes())
        .unwrap();
    assert_eq!(
        session
            .decode_unique_request(valid.to_string().as_bytes())
            .unwrap_err()
            .kind(),
        FleetErrorKind::DuplicateId
    );
}

#[tokio::test]
async fn broker_uses_0600_socket_auth_generations_deadlines_and_no_mutation_replay() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("fleet.sock");
    let mut broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: None,
        secret: secret(),
        native_secret: None,
        max_payload_bytes: 4096,
        max_in_flight: 1,
    })
    .await
    .unwrap();
    assert_eq!(
        std::fs::metadata(&socket).unwrap().permissions().mode() & 0o777,
        0o600
    );
    broker
        .apply_inventory(InventoryUpdate::connected(
            BrowserKind::Chrome,
            "stable",
            vec![target(1)],
        ))
        .unwrap();
    assert!(matches!(
        broker
            .handle(FleetRequest::health(
                "health-1",
                secret(),
                1,
                Duration::from_secs(1)
            ))
            .await
            .unwrap(),
        FleetResponse::Health { generation: 1, .. }
    ));
    assert_eq!(
        broker
            .handle(FleetRequest::action(
                "stale",
                secret(),
                target(0),
                Action::Click,
                Duration::from_secs(1)
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::StaleGeneration
    );
    assert!(matches!(
        broker
            .handle(FleetRequest::health(
                "health-1",
                secret(),
                1,
                Duration::from_secs(1)
            ))
            .await
            .unwrap(),
        FleetResponse::Health { .. }
    ));
    let mutation = FleetRequest::action(
        "mut-1",
        secret(),
        target(1),
        Action::Navigate,
        Duration::from_secs(1),
    );
    assert_eq!(
        broker.handle(mutation.clone()).await.unwrap_err().kind(),
        FleetErrorKind::ApprovalRequired
    );
    assert_eq!(
        broker.handle(mutation).await.unwrap_err().kind(),
        FleetErrorKind::DuplicateMutation
    );
    assert_eq!(
        broker
            .handle(FleetRequest::health(
                "late",
                secret(),
                1,
                Duration::from_millis(0)
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::DeadlineExceeded
    );
    broker
        .apply_inventory(InventoryUpdate::connected(
            BrowserKind::Chrome,
            "stable",
            vec![target(2)],
        ))
        .unwrap();
    assert_eq!(
        broker
            .handle(FleetRequest::action(
                "mut-2",
                secret(),
                target(1),
                Action::Click,
                Duration::from_secs(1)
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::StaleGeneration
    );
}

#[test]
fn mutation_replay_guard_allows_readonly_duplicates_but_not_mutations() {
    let mut guard = MutationReplayGuard::default();
    assert!(guard.observe("same", Action::FleetHealth).is_ok());
    assert!(guard.observe("same", Action::FleetHealth).is_ok());
    assert!(guard.observe("m1", Action::Click).is_ok());
    assert_eq!(
        guard.observe("m1", Action::Click).unwrap_err().kind(),
        FleetErrorKind::DuplicateMutation
    );
}

#[tokio::test]
async fn cdp_adapter_trusts_only_managed_endpoints_and_bounds_output() {
    let mut caps = BTreeSet::new();
    caps.insert(Capability::RichInspection);
    caps.insert(Capability::Evaluate);
    let endpoint = CdpEndpoint {
        id: "managed-1".into(),
        browser: BrowserKind::Chrome,
        websocket_url: "ws://127.0.0.1:9222/devtools/browser/abc".into(),
        capabilities: caps,
    };
    let adapter = CdpAdapter::new(vec![endpoint], 32).unwrap();
    let inventory = adapter.discover().await.unwrap();
    assert_eq!(inventory.generation, 1);
    assert!(
        inventory.targets[0]
            .capabilities
            .contains(&Capability::RichInspection)
    );
    assert_eq!(
        adapter
            .inspect("managed-1", "abcdefghijklmnopqrstuvwxyz0123456789")
            .await
            .unwrap()
            .len(),
        32
    );
    let untrusted = CdpEndpoint {
        id: "daily".into(),
        browser: BrowserKind::Chrome,
        websocket_url: "ws://localhost:9222/devtools/browser/daily-profile".into(),
        capabilities: BTreeSet::new(),
    };
    assert_eq!(
        CdpAdapter::new(vec![untrusted], 32).unwrap_err().kind(),
        FleetErrorKind::UntrustedEndpoint
    );
}

#[tokio::test]
async fn managed_cdp_source_discovers_loopback_targets_and_rejects_remote_hosts() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await.unwrap();
        let body = serde_json::json!([
            {
                "id": "page-1",
                "type": "page",
                "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/page/page-1"
            },
            {
                "id": "remote-page",
                "type": "page",
                "webSocketDebuggerUrl": "ws://192.0.2.10:9222/devtools/page/remote-page"
            },
            {
                "id": "worker-1",
                "type": "service_worker",
                "webSocketDebuggerUrl": "ws://127.0.0.1:9222/devtools/page/worker-1"
            }
        ])
        .to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    });

    let source = ManagedCdpSource::new(
        format!("http://{address}"),
        BrowserKind::Chrome,
        16,
        64 * 1024,
    )
    .unwrap();
    let update = tokio::time::timeout(Duration::from_millis(250), source.discover())
        .await
        .expect("discovery must respect Content-Length without waiting for connection close")
        .unwrap();
    assert_eq!(update.targets().len(), 1);
    assert_eq!(update.targets()[0].tab_id, "page-1");
    server.await.unwrap();

    assert_eq!(
        ManagedCdpSource::new("http://192.0.2.10:9222", BrowserKind::Chrome, 16, 64 * 1024,)
            .unwrap_err()
            .kind(),
        FleetErrorKind::UntrustedEndpoint
    );
}

#[tokio::test]
async fn broker_merges_multiple_browser_sources_and_reports_current_generation() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("fleet.sock");
    let mut broker = Broker::bind(BrokerConfig {
        socket_path: socket,
        authority_socket_path: None,
        secret: secret(),
        native_secret: None,
        max_payload_bytes: 4096,
        max_in_flight: 4,
    })
    .await
    .unwrap();

    broker
        .apply_inventory(InventoryUpdate::connected(
            BrowserKind::Chrome,
            "managed-cdp",
            vec![TargetRef {
                browser_id: "managed-chrome".into(),
                window_id: "managed-cdp".into(),
                tab_id: "chrome-page".into(),
                generation: 0,
            }],
        ))
        .unwrap();
    broker
        .apply_inventory(InventoryUpdate::connected(
            BrowserKind::Edge,
            "managed-cdp",
            vec![TargetRef {
                browser_id: "managed-edge".into(),
                window_id: "managed-cdp".into(),
                tab_id: "edge-page".into(),
                generation: 0,
            }],
        ))
        .unwrap();

    let FleetResponse::Health {
        generation,
        connected_targets,
        targets,
    } = broker
        .handle(FleetRequest::health(
            "merged-health",
            secret(),
            0,
            Duration::from_secs(1),
        ))
        .await
        .unwrap()
    else {
        panic!("health request should return inventory");
    };

    assert_eq!(generation, 2);
    assert_eq!(connected_targets, 2);
    assert_eq!(targets.len(), 2);
    assert!(targets.iter().any(|target| target.tab_id == "chrome-page"));
    assert!(targets.iter().any(|target| target.tab_id == "edge-page"));
    assert!(
        targets.iter().all(|target| target.generation == generation),
        "all advertised targets must carry the current broker generation: {targets:?}"
    );
}

async fn authority_round_trip(
    broker: &mut Broker,
    request: AuthorityEnvelope,
) -> serde_json::Value {
    let (mut client, server) = tokio::net::UnixStream::pair().unwrap();
    let server = async move { broker.serve_authority_connection(server).await.unwrap() };
    let client = async move {
        let mut encoded = serde_json::to_vec(&request).unwrap();
        encoded.push(b'\n');
        client.write_all(&encoded).await.unwrap();
        let mut reader = BufReader::new(client);
        let mut line = Vec::new();
        reader.read_until(b'\n', &mut line).await.unwrap();
        serde_json::from_slice::<serde_json::Value>(&line).unwrap()
    };
    let (_, response) = tokio::join!(server, client);
    response
}

fn authority_grant(
    target: TargetRef,
    capabilities: Vec<Capability>,
    duration_seconds: u64,
) -> AuthorityEnvelope {
    AuthorityEnvelope {
        version: 1,
        action: AuthorityAction::GrantLease,
        lease_id: None,
        target: Some(target),
        capabilities,
        duration_seconds: Some(duration_seconds),
    }
}

fn managed_target(generation: u64, websocket_url: String, url: &str) -> ManagedTarget {
    ManagedTarget::new(target(generation), websocket_url, url.to_string()).unwrap()
}

#[tokio::test]
async fn broker_requires_mac_local_authority_and_enforces_lease_scope_revocation_expiry_and_stop() {
    let dir = tempdir().unwrap();
    let peer_socket = dir.path().join("peer.sock");
    let authority_socket = dir.path().join("authority.sock");
    let mut broker = Broker::bind(BrokerConfig {
        socket_path: peer_socket.clone(),
        authority_socket_path: Some(authority_socket.clone()),
        secret: secret(),
        native_secret: None,
        max_payload_bytes: 4096,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    assert_eq!(
        std::fs::metadata(&authority_socket)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    broker
        .apply_inventory(InventoryUpdate::managed(
            BrowserKind::Chrome,
            "managed-cdp",
            vec![managed_target(
                0,
                "ws://127.0.0.1:9/devtools/page/page-1".to_string(),
                "https://example.com/page",
            )],
        ))
        .unwrap();
    let current = target(1);

    let peer_mutation = FleetRequest::action_with_payload(
        "peer-mut-unauthorized",
        secret(),
        current.clone(),
        Action::Navigate,
        Duration::from_secs(1),
        serde_json::json!({"url":"https://example.com/next"}),
    );
    assert_eq!(
        broker.handle(peer_mutation).await.unwrap_err().kind(),
        FleetErrorKind::ApprovalRequired
    );

    let grant = authority_round_trip(
        &mut broker,
        authority_grant(current.clone(), vec![Capability::Navigate], 60),
    )
    .await;
    assert_eq!(
        grant.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
    let lease_id = grant
        .pointer("/result/leaseId")
        .and_then(|value| value.as_str())
        .expect("grant returns lease id")
        .to_string();

    assert_eq!(
        broker
            .handle(FleetRequest::action_with_payload(
                "wrong-capability",
                secret(),
                current.clone(),
                Action::Click,
                Duration::from_secs(1),
                serde_json::json!({"x":1,"y":1}),
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::ApprovalRequired
    );
    assert_eq!(
        broker
            .handle(FleetRequest::action_with_payload(
                "wrong-generation",
                secret(),
                target(0),
                Action::Navigate,
                Duration::from_secs(1),
                serde_json::json!({"url":"https://example.com/next"}),
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::StaleGeneration
    );

    let revoke = authority_round_trip(
        &mut broker,
        AuthorityEnvelope {
            version: 1,
            action: AuthorityAction::RevokeLease,
            lease_id: Some(lease_id),
            target: None,
            capabilities: Vec::new(),
            duration_seconds: None,
        },
    )
    .await;
    assert_eq!(
        revoke.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        broker
            .handle(FleetRequest::action_with_payload(
                "after-revoke",
                secret(),
                current.clone(),
                Action::Navigate,
                Duration::from_secs(1),
                serde_json::json!({"url":"https://example.com/next"}),
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::ApprovalRequired
    );

    let expiring = authority_round_trip(
        &mut broker,
        authority_grant(current.clone(), vec![Capability::Navigate], 0),
    )
    .await;
    assert_eq!(
        expiring.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        broker
            .handle(FleetRequest::action_with_payload(
                "after-expiry",
                secret(),
                current.clone(),
                Action::Navigate,
                Duration::from_secs(1),
                serde_json::json!({"url":"https://example.com/next"}),
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::ApprovalRequired
    );

    let _ = authority_round_trip(
        &mut broker,
        authority_grant(current.clone(), vec![Capability::Navigate], 60),
    )
    .await;
    let stop = authority_round_trip(
        &mut broker,
        AuthorityEnvelope {
            version: 1,
            action: AuthorityAction::EmergencyStop,
            lease_id: None,
            target: None,
            capabilities: Vec::new(),
            duration_seconds: None,
        },
    )
    .await;
    assert_eq!(stop.get("ok").and_then(|value| value.as_bool()), Some(true));
    assert!(matches!(
        broker
            .handle(FleetRequest::health(
                "health-during-stop",
                secret(),
                1,
                Duration::from_secs(1)
            ))
            .await
            .unwrap(),
        FleetResponse::Health { .. }
    ));
    assert_eq!(
        broker
            .handle(FleetRequest::action_with_payload(
                "during-stop",
                secret(),
                current,
                Action::Navigate,
                Duration::from_secs(1),
                serde_json::json!({"url":"https://example.com/next"}),
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::EmergencyStop
    );
}

#[tokio::test]
async fn hard_denies_survive_local_leases_and_cdp_websocket_urls_stay_loopback_internal() {
    let dir = tempdir().unwrap();
    let mut broker = Broker::bind(BrokerConfig {
        socket_path: dir.path().join("peer.sock"),
        authority_socket_path: None,
        secret: secret(),
        native_secret: None,
        max_payload_bytes: 4096,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    broker
        .apply_inventory(InventoryUpdate::managed(
            BrowserKind::Chrome,
            "managed-cdp",
            vec![managed_target(
                0,
                "ws://127.0.0.1:9/devtools/page/settings".to_string(),
                "chrome://settings/passwords",
            )],
        ))
        .unwrap();
    let FleetResponse::Health { targets, .. } = broker
        .handle(FleetRequest::health(
            "hard-deny-health",
            secret(),
            0,
            Duration::from_secs(1),
        ))
        .await
        .unwrap()
    else {
        panic!("health should return targets");
    };
    let serialized = serde_json::to_string(&targets).unwrap();
    assert!(!serialized.contains("webSocketDebuggerUrl"));
    assert!(!serialized.contains("devtools/page"));

    let denied = target(1);
    let grant = authority_round_trip(
        &mut broker,
        authority_grant(denied.clone(), vec![Capability::Navigate], 60),
    )
    .await;
    assert_eq!(
        grant.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        broker
            .handle(FleetRequest::action_with_payload(
                "hard-denied-mutation",
                secret(),
                denied,
                Action::Navigate,
                Duration::from_secs(1),
                serde_json::json!({"url":"https://example.com/next"}),
            ))
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::HardDenied
    );

    assert_eq!(
        ManagedTarget::new(
            target(0),
            "ws://192.0.2.10:9222/devtools/page/remote".to_string(),
            "https://example.com".to_string(),
        )
        .unwrap_err()
        .kind(),
        FleetErrorKind::UntrustedEndpoint
    );
}

async fn fake_cdp_server() -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut headers = Vec::new();
        loop {
            let mut byte = [0_u8; 1];
            stream.read_exact(&mut byte).await.unwrap();
            headers.push(byte[0]);
            if headers.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        stream
            .write_all(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: test\r\n\r\n")
            .await
            .unwrap();
        let mut seen = Vec::new();
        for id in 1..=3 {
            let text = read_client_ws_text(&mut stream).await;
            let method = serde_json::from_str::<serde_json::Value>(&text)
                .unwrap()
                .get("method")
                .and_then(|value| value.as_str())
                .unwrap()
                .to_string();
            seen.push(method);
            write_server_ws_text(
                &mut stream,
                &serde_json::json!({"id": id, "result": {}}).to_string(),
            )
            .await;
        }
        seen
    });
    (format!("ws://{addr}/devtools/page/page-1"), handle)
}

async fn read_client_ws_text(stream: &mut tokio::net::TcpStream) -> String {
    let mut head = [0_u8; 2];
    stream.read_exact(&mut head).await.unwrap();
    assert_eq!(head[0] & 0x0f, 1);
    assert_ne!(head[1] & 0x80, 0, "client frames must be masked");
    let mut len = usize::from(head[1] & 0x7f);
    if len == 126 {
        let mut ext = [0_u8; 2];
        stream.read_exact(&mut ext).await.unwrap();
        len = u16::from_be_bytes(ext) as usize;
    }
    let mut mask = [0_u8; 4];
    stream.read_exact(&mut mask).await.unwrap();
    let mut payload = vec![0_u8; len];
    stream.read_exact(&mut payload).await.unwrap();
    for (index, byte) in payload.iter_mut().enumerate() {
        *byte ^= mask[index % 4];
    }
    String::from_utf8(payload).unwrap()
}

async fn write_server_ws_text(stream: &mut tokio::net::TcpStream, text: &str) {
    let bytes = text.as_bytes();
    let mut frame = vec![0x81];
    frame.push(bytes.len() as u8);
    frame.extend_from_slice(bytes);
    stream.write_all(&frame).await.unwrap();
}

#[tokio::test]
async fn approved_managed_cdp_mutation_executes_against_fake_loopback_websocket() {
    let (websocket_url, server) = fake_cdp_server().await;
    let dir = tempdir().unwrap();
    let mut broker = Broker::bind(BrokerConfig {
        socket_path: dir.path().join("peer.sock"),
        authority_socket_path: None,
        secret: secret(),
        native_secret: None,
        max_payload_bytes: 4096,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    broker
        .apply_inventory(InventoryUpdate::managed(
            BrowserKind::Chrome,
            "managed-cdp",
            vec![managed_target(0, websocket_url, "https://example.com")],
        ))
        .unwrap();
    let current = target(1);
    let grant = authority_round_trip(
        &mut broker,
        authority_grant(current.clone(), vec![Capability::Navigate], 60),
    )
    .await;
    assert_eq!(
        grant.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );

    assert_eq!(
        broker
            .handle(FleetRequest::action_with_payload(
                "approved-navigate",
                secret(),
                current,
                Action::Navigate,
                Duration::from_secs(1),
                serde_json::json!({"url":"https://example.com/next"}),
            ))
            .await
            .unwrap(),
        FleetResponse::Accepted
    );
    assert_eq!(
        server.await.unwrap(),
        vec!["Page.enable", "Page.navigate", "Page.disable"]
    );
}

#[tokio::test]
async fn native_host_uses_chromium_framing_bounds_payloads_and_hides_secret() {
    let request = serde_json::json!({"version":1,"id":"native-health","deadline_ms":1000,"target_generation":0,"action":"fleetHealth","payload":{}});
    let payload = serde_json::to_vec(&request).unwrap();
    let (mut client, mut host) = tokio::io::duplex(4096);
    write_native_message(&mut client, &payload, 4096)
        .await
        .unwrap();
    let decoded = read_native_message(&mut host, 4096).await.unwrap().unwrap();
    assert_eq!(decoded, payload);
    assert!(!String::from_utf8(decoded).unwrap().contains("test-secret"));

    let oversized = (4097_u32).to_le_bytes();
    let (mut client, mut host) = tokio::io::duplex(8);
    client.write_all(&oversized).await.unwrap();
    assert_eq!(
        read_native_message(&mut host, 4096)
            .await
            .unwrap_err()
            .kind(),
        FleetErrorKind::Oversized
    );
}

#[tokio::test]
async fn native_host_forwards_inventory_requests_to_broker_with_internal_secret() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("fleet.sock");
    let mut broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: None,
        secret: secret(),
        native_secret: None,
        max_payload_bytes: 4096,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    broker
        .apply_inventory(InventoryUpdate::connected(
            BrowserKind::Chrome,
            "ordinary",
            vec![target(1)],
        ))
        .unwrap();
    let server = tokio::spawn(async move { broker.serve().await });

    let request = serde_json::to_vec(&serde_json::json!({"version":1,"id":"native-list","deadline_ms":1000,"target_generation":1,"action":"listBrowsers","payload":{}})).unwrap();
    let response = forward_native_payload(
        &NativeHostBridgeConfig {
            socket_path: socket,
            secret: secret(),
            max_payload_bytes: 4096,
        },
        &request,
    )
    .await
    .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&response).unwrap();
    assert_eq!(response["ok"], true);
    assert_eq!(response["result"]["connected_targets"], 1);
    assert!(!String::from_utf8(request).unwrap().contains("test-secret"));
    server.abort();
}

#[tokio::test]
async fn native_host_stream_returns_bounded_secret_safe_errors_when_broker_is_absent() {
    let dir = tempdir().unwrap();
    let (mut extension, mut host_input) = tokio::io::duplex(8192);
    let (mut host_output, mut extension_reader) = tokio::io::duplex(8192);
    let config = NativeHostBridgeConfig {
        socket_path: dir.path().join("missing.sock"),
        secret: secret(),
        max_payload_bytes: 4096,
    };
    let host = tokio::spawn(async move {
        serve_native_host(config, &mut host_input, &mut host_output)
            .await
            .unwrap();
    });
    let request = serde_json::to_vec(&serde_json::json!({"version":1,"id":"native-missing-broker","deadline_ms":1000,"target_generation":0,"action":"fleetHealth","payload":{}})).unwrap();
    write_native_message(&mut extension, &request, 4096)
        .await
        .unwrap();
    drop(extension);
    let response = read_native_message(&mut extension_reader, 4096)
        .await
        .unwrap()
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&response).unwrap();
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["kind"], "io");
    assert!(!response.to_string().contains("test-secret"));
    host.await.unwrap();
}

#[tokio::test]
async fn native_host_accepts_the_extension_hello_shape_and_returns_hello_ack() {
    let dir = tempdir().unwrap();
    let (mut extension, mut host_input) = tokio::io::duplex(8192);
    let (mut host_output, mut extension_reader) = tokio::io::duplex(8192);
    let config = NativeHostBridgeConfig {
        socket_path: dir.path().join("missing.sock"),
        secret: secret(),
        max_payload_bytes: 4096,
    };
    let host = tokio::spawn(async move {
        serve_native_host(config, &mut host_input, &mut host_output)
            .await
            .unwrap();
    });
    let hello = serde_json::to_vec(&serde_json::json!({
        "type": "hello",
        "protocolVersion": 1,
        "browserKind": "chrome",
        "extensionVersion": "0.1.0",
        "sessionId": "extension-session"
    }))
    .unwrap();
    write_native_message(&mut extension, &hello, 4096)
        .await
        .unwrap();
    drop(extension);

    let response = read_native_message(&mut extension_reader, 4096)
        .await
        .unwrap()
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&response).unwrap();
    assert_eq!(
        response,
        serde_json::json!({
            "type": "hello_ack",
            "protocolVersion": 1,
            "sessionId": "extension-session"
        })
    );
    host.await.unwrap();
}

#[tokio::test]
async fn extension_snapshot_creates_ordinary_targets_and_public_output_strips_host_only_fields() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("fleet.sock");
    let broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: None,
        secret: secret(),
        native_secret: Some(native_secret()),
        max_payload_bytes: 8192,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    let server = tokio::spawn(async move { broker.serve().await });

    let synced = internal_snapshot(&socket, "chrome", "Default", 424242).await;
    assert_eq!(synced["ok"], true);

    let listed = peer_list(&socket, secret(), 0).await;
    assert_eq!(listed["ok"], true);
    let targets = listed["result"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["browser_id"], "ordinary-chrome");
    assert_ne!(targets[0]["browser_id"], "managed-chrome");
    assert_eq!(targets[0]["window_id"], "win-chrome");
    assert_eq!(targets[0]["tab_id"], "tab-chrome");

    let public = listed.to_string();
    assert!(!public.contains("nativeWindowId"));
    assert!(!public.contains("nativeTabId"));
    assert!(!public.contains("424242"));
    assert!(!public.contains("test-peer-secret"));
    assert!(!public.contains("test-native-secret"));
    server.abort();
}

#[tokio::test]
async fn extension_sources_stay_separated_and_disconnect_cleans_only_that_browser() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("fleet.sock");
    let broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: None,
        secret: secret(),
        native_secret: Some(native_secret()),
        max_payload_bytes: 8192,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    let server = tokio::spawn(async move { broker.serve().await });

    assert_eq!(
        internal_snapshot(&socket, "chrome", "Default", 111).await["ok"],
        true
    );
    assert_eq!(
        internal_snapshot(&socket, "edge", "Default", 222).await["ok"],
        true
    );

    let listed = peer_list(&socket, secret(), 0).await;
    let mut browser_refs = listed["result"]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|target| target["browser_id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    browser_refs.sort();
    assert_eq!(browser_refs, vec!["ordinary-chrome", "ordinary-edge"]);

    let disconnected = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": native_secret(),
            "id": "disconnect-chrome",
            "action": "extensionDisconnect",
            "payload": {"browserKind": "chrome", "profileLabel": "Default"}
        }),
    )
    .await;
    assert_eq!(disconnected["ok"], true);

    let listed = peer_list(&socket, secret(), 0).await;
    let targets = listed["result"]["targets"].as_array().unwrap();
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0]["browser_id"], "ordinary-edge");
    server.abort();
}

#[tokio::test]
async fn approved_extension_navigate_round_trips_through_poll_and_response_without_replay() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("fleet.sock");
    let authority_socket = dir.path().join("authority.sock");
    let broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: Some(authority_socket.clone()),
        secret: secret(),
        native_secret: Some(native_secret()),
        max_payload_bytes: 8192,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    let server = tokio::spawn(async move { broker.serve().await });

    assert_eq!(
        internal_snapshot(&socket, "chrome", "Default", 333333).await["ok"],
        true
    );
    let listed = peer_list(&socket, secret(), 0).await;
    let target: TargetRef = serde_json::from_value(listed["result"]["targets"][0].clone()).unwrap();

    let denied = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": secret(),
            "id": "extension-nav-denied",
            "deadline_ms": 1000,
            "target_generation": target.generation,
            "action": "navigate",
            "payload": {"target": target, "url": "https://denied.test"}
        }),
    )
    .await;
    assert_eq!(denied["ok"], false);
    assert_eq!(denied["error"]["kind"], "approvalRequired");

    let grant = broker_line(
        &authority_socket,
        serde_json::json!({
            "version": 1,
            "action": "grantLease",
            "target": target,
            "capabilities": ["navigate"],
            "durationSeconds": 60
        }),
    )
    .await;
    assert_eq!(grant["ok"], true);

    let accepted = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": secret(),
            "id": "extension-nav-approved",
            "deadline_ms": 5000,
            "target_generation": target.generation,
            "action": "navigate",
            "payload": {"target": target, "url": "https://approved.test/path"}
        }),
    )
    .await;
    assert_eq!(accepted["ok"], true);
    assert_eq!(accepted["result"]["kind"], "accepted");

    let duplicate = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": secret(),
            "id": "extension-nav-approved",
            "deadline_ms": 5000,
            "target_generation": target.generation,
            "action": "navigate",
            "payload": {"target": target, "url": "https://approved.test/path"}
        }),
    )
    .await;
    assert_eq!(duplicate["ok"], false);
    assert_eq!(duplicate["error"]["kind"], "duplicateMutation");

    let polled = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": native_secret(),
            "id": "poll-1",
            "action": "extensionActionPoll",
            "payload": {"browserKind": "chrome", "profileLabel": "Default"}
        }),
    )
    .await;
    assert_eq!(polled["ok"], true);
    assert_eq!(polled["result"]["type"], "action_request");
    assert_eq!(polled["result"]["requestId"], "extension-nav-approved");
    assert_eq!(polled["result"]["action"], "navigate");
    assert_eq!(polled["result"]["target"]["tabId"], 333333);
    assert_eq!(polled["result"]["target"]["windowId"], 334333);

    let result = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": native_secret(),
            "id": "result-1",
            "action": "extensionActionResult",
            "payload": {"requestId": "extension-nav-approved", "ok": true, "result": {}}
        }),
    )
    .await;
    assert_eq!(result["ok"], true);

    let second_poll = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": native_secret(),
            "id": "poll-2",
            "action": "extensionActionPoll",
            "payload": {"browserKind": "chrome", "profileLabel": "Default"}
        }),
    )
    .await;
    assert_eq!(second_poll["ok"], true);
    assert_eq!(
        second_poll["result"],
        serde_json::json!({"type": "action_idle"})
    );
    server.abort();
}

#[tokio::test]
async fn peer_and_native_secrets_are_not_interchangeable_between_wire_surfaces() {
    let dir = tempdir().unwrap();
    let socket = dir.path().join("fleet.sock");
    let broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: None,
        secret: secret(),
        native_secret: Some(native_secret()),
        max_payload_bytes: 8192,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    let server = tokio::spawn(async move { broker.serve().await });

    let native_as_peer = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": native_secret(),
            "id": "native-as-peer-action",
            "deadline_ms": 1000,
            "target_generation": 0,
            "action": "navigate",
            "payload": {"target": {"browser_id":"ordinary-chrome","window_id":"win","tab_id":"tab","generation":0}, "url":"https://example.test"}
        }),
    )
    .await;
    assert_eq!(native_as_peer["ok"], false);
    assert_eq!(native_as_peer["error"]["kind"], "unauthenticated");

    let peer_as_internal = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": secret(),
            "id": "peer-as-internal",
            "action": "extensionActionPoll",
            "payload": {"browserKind": "chrome", "profileLabel": "Default"}
        }),
    )
    .await;
    assert_eq!(peer_as_internal["ok"], false);
    assert_eq!(peer_as_internal["error"]["kind"], "unauthenticated");
    assert!(!native_as_peer.to_string().contains("test-native-secret"));
    assert!(!peer_as_internal.to_string().contains("test-peer-secret"));
    server.abort();
}

#[test]
fn native_host_defaults_match_the_installed_broker_socket_layout() {
    // The native host is launched by Chrome with no arguments, so its defaults
    // must match what the setup tool installs. A stale default silently breaks
    // every ordinary-profile connection while all files still look installed.
    let home = std::path::Path::new("/Users/test");

    let socket = jcode_mac_browser_fleet::default_broker_socket_path(home);
    let secret = jcode_mac_browser_fleet::default_native_secret_path(home);

    assert_eq!(
        socket,
        std::path::PathBuf::from("/Users/test/.jcode/mac-fleet/broker.sock")
    );
    assert!(socket.to_string_lossy().len() <= 103);
    assert_eq!(
        secret,
        std::path::PathBuf::from(
            "/Users/test/Library/Application Support/Jcode/MacBrowserFleet/native.secret"
        )
    );
}

#[test]
fn chrome_style_native_host_launch_is_not_a_usage_error() {
    use jcode_mac_browser_fleet::{Invocation, classify_invocation};

    // Chrome/Edge launch the host as:
    //   <binary> /path/to/host-manifest.json chrome-extension://<id>/
    // Treating that as an unknown subcommand makes the host exit immediately,
    // so ordinary profiles can never attach even though every file is installed.
    assert_eq!(
        classify_invocation([
            "/Users/test/Library/Application Support/Google/Chrome/NativeMessagingHosts/dev.jcode.mac_browser_fleet.json",
            "chrome-extension://mlgjaoahakdijgckgjpmpkafccgffpgd/",
        ]),
        Invocation::NativeHost
    );

    let empty: [&str; 0] = [];
    assert_eq!(classify_invocation(empty), Invocation::NativeHost);
    assert_eq!(classify_invocation(["native-host"]), Invocation::NativeHost);
    assert_eq!(classify_invocation(["broker"]), Invocation::Broker);
    assert_eq!(classify_invocation(["authority"]), Invocation::Authority);
}

#[test]
fn navigation_destination_is_hard_denied_even_under_an_active_lease() {
    use jcode_mac_browser_fleet::navigation_hard_deny;

    // Hard-deny classifies the tab's *current* context. A navigate also has a
    // destination, and privileged destinations must be refused even when a
    // valid Mac-issued lease covers the target: otherwise an approved lease on
    // an ordinary tab becomes a way to drive the browser into chrome://settings
    // or chrome://extensions.
    for denied in [
        "chrome://settings/",
        "chrome://extensions/",
        "edge://settings/profiles",
        "about:config",
        "https://myaccount.google.com/security/passwords",
    ] {
        assert!(
            navigation_hard_deny(denied).is_some(),
            "expected {denied} to be hard denied as a navigation destination"
        );
    }

    for allowed in ["https://example.com/", "https://docs.rs/serde"] {
        assert!(
            navigation_hard_deny(allowed).is_none(),
            "expected {allowed} to remain navigable"
        );
    }
}

#[tokio::test]
async fn one_bad_connection_does_not_terminate_the_broker() {
    // A single malformed or oversized request must not take the whole broker
    // down: it is shared by every browser, and launchd restarting it drops
    // every extension connection at once. Observed live before this fix, a
    // 3 MiB request killed the process.
    let dir = tempdir().unwrap();
    let socket = dir.path().join("resilient.sock");
    let broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: None,
        secret: secret(),
        native_secret: None,
        max_payload_bytes: 4096,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    let handle = tokio::spawn(async move { broker.serve().await });

    // Oversized, never newline-framed request, then an abrupt disconnect.
    {
        let mut stream = tokio::net::UnixStream::connect(&socket).await.unwrap();
        let flood = vec![b'x'; 64 * 1024];
        let _ = stream.write_all(&flood).await;
        drop(stream);
    }
    tokio::time::sleep(Duration::from_millis(50)).await;

    // The broker must still answer a well-formed request afterwards.
    let mut stream = tokio::net::UnixStream::connect(&socket)
        .await
        .expect("broker must still be listening after a bad connection");
    let request = serde_json::json!({
        "version": 1,
        "auth": secret(),
        "id": "after-flood",
        "deadline_ms": 5_000,
        "target_generation": 0,
        "action": "listBrowsers",
        "payload": {},
    });
    let mut line = serde_json::to_vec(&request).unwrap();
    line.push(b'\n');
    stream.write_all(&line).await.unwrap();
    let mut reader = BufReader::new(stream);
    let mut response = Vec::new();
    reader.read_until(b'\n', &mut response).await.unwrap();
    let value: serde_json::Value = serde_json::from_slice(&response).unwrap();
    assert_eq!(
        value.get("ok").and_then(serde_json::Value::as_bool),
        Some(true),
        "broker should keep serving after a bad connection: {value}"
    );

    handle.abort();
}

#[tokio::test]
async fn action_poll_from_an_unknown_source_requests_re_registration() {
    // The broker holds extension inventory in memory. After it restarts (or is
    // relaunched by launchd), every extension still has a live native host but
    // the broker no longer knows that source, so the browser silently vanishes
    // from the fleet. The poll the extension already sends every second is the
    // natural liveness signal: an unknown source must be told to re-register
    // rather than answered with a plain idle.
    let dir = tempdir().unwrap();
    let socket = dir.path().join("resync.sock");
    let mut broker = Broker::bind(BrokerConfig {
        socket_path: socket.clone(),
        authority_socket_path: None,
        secret: secret(),
        native_secret: Some(native_secret()),
        max_payload_bytes: 65_536,
        max_in_flight: 4,
    })
    .await
    .unwrap();
    let handle = tokio::spawn(async move { broker.serve().await });

    let response = broker_line(
        &socket,
        serde_json::json!({
            "version": 1,
            "auth": native_secret(),
            "id": "poll-unknown",
            "action": "extensionActionPoll",
            "payload": {"browserKind": "chrome", "profileLabel": "profile-unknown"}
        }),
    )
    .await;

    let result = response.get("result").cloned().unwrap_or_default();
    assert_eq!(
        result.get("type").and_then(serde_json::Value::as_str),
        Some("resync_request"),
        "unknown source must be asked to re-register: {response}"
    );

    handle.abort();
}
