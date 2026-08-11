use std::collections::BTreeSet;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use jcode_mac_browser_fleet::{
    Action, Broker, BrokerConfig, BrowserKind, Capability, CdpAdapter, CdpEndpoint, FleetErrorKind,
    FleetRequest, FleetResponse, InventoryUpdate, MutationReplayGuard, ProtocolCodec,
    ProtocolSession, TargetRef,
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
