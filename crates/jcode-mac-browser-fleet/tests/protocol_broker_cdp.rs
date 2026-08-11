use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
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
    "test-secret-with-enough-entropy".to_string()
}
fn target(generation: u64) -> TargetRef {
    TargetRef {
        browser_id: "chrome:stable".into(),
        window_id: "win-1".into(),
        tab_id: "tab-1".into(),
        generation,
    }
}

#[test]
fn protocol_fails_closed_for_bad_inputs_without_leaking_secret() {
    let codec = ProtocolCodec::new(vec![1], 1024, secret());
    let err = codec
        .decode_request("x".repeat(2048).as_bytes())
        .unwrap_err();
    assert_eq!(err.kind(), FleetErrorKind::Oversized);
    assert!(!err.to_string().contains("test-secret"));
    let unsupported = serde_json::json!({"version":99,"auth":"test-secret-with-enough-entropy","id":"r1","deadline_ms":1000,"target_generation":1,"action":"fleetHealth","payload":{}});
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

    let valid = serde_json::json!({"version":1,"auth":"test-secret-with-enough-entropy","id":"dup","deadline_ms":1000,"target_generation":1,"action":"fleetHealth","payload":{}});
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
