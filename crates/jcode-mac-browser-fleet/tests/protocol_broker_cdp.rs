use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use jcode_mac_browser_fleet::{
    Action, Broker, BrokerConfig, BrowserKind, Capability, CdpAdapter, CdpEndpoint, FleetErrorKind,
    FleetRequest, FleetResponse, InventoryUpdate, ManagedCdpSource, MutationReplayGuard,
    ProtocolCodec, ProtocolSession, TargetRef,
};
use tempfile::tempdir;

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
