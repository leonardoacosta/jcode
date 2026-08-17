use chrono::{TimeZone, Utc};
use jcode_intake_types::{
    Classification, DecisionInboxStatus, Envelope, ProposalState, SqliteIntakeStore,
};
use serde_json::json;
use tempfile::tempdir;

fn envelope(adapter: &str, sender: &str, conversation: &str, content: &str) -> Envelope {
    Envelope {
        adapter: adapter.to_owned(),
        sender_identity: sender.to_owned(),
        conversation: conversation.to_owned(),
        content: Some(content.to_owned()),
        attachments: Vec::new(),
        received_at: Utc.with_ymd_and_hms(2026, 8, 17, 5, 0, 0).unwrap(),
    }
}

#[test]
fn durable_inbox_projection_preserves_provider_provenance_and_category() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("decision-inbox.sqlite");
    {
        let mut store = SqliteIntakeStore::open(&path, None).unwrap();
        store
            .receive(
                envelope("telegram", "operator", "tg:555", "implement the inbox"),
                json!({"update_id": 41, "message": {"from": {"id": 7}}}),
                Some("operator".to_owned()),
            )
            .unwrap();
    }

    let reopened = SqliteIntakeStore::open(&path, None).unwrap();
    let items = reopened.decision_inbox_items().unwrap();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    assert_eq!(item.source.adapter, "telegram");
    assert_eq!(item.source.sender_identity, "operator");
    assert_eq!(item.source.conversation, "tg:555");
    assert_eq!(item.category, Some(Classification::WorkRequest));
    assert_eq!(item.status, DecisionInboxStatus::AwaitingApproval);
    assert_eq!(
        item.proposal.as_ref().unwrap().state,
        ProposalState::AwaitingApproval
    );
    assert!(item.raw_payload_retained);
    assert_eq!(
        item.received_at,
        Utc.with_ymd_and_hms(2026, 8, 17, 5, 0, 0).unwrap()
    );
}

#[test]
fn replayed_provider_delivery_is_one_canonical_item_with_visible_duplicate_evidence() {
    let mut store = SqliteIntakeStore::open_in_memory(None).unwrap();
    let inbound = envelope(
        "slack",
        "operator",
        "sl:D123",
        "build the release dashboard",
    );
    let raw = json!({"envelope_id": "env-1", "payload": {"event_id": "Ev1"}});
    store
        .receive(inbound.clone(), raw.clone(), Some("operator".to_owned()))
        .unwrap();
    store
        .receive(inbound, raw, Some("operator".to_owned()))
        .unwrap();

    assert_eq!(
        store.records().unwrap().len(),
        2,
        "every delivery remains auditable"
    );
    assert_eq!(
        store.proposals().unwrap().len(),
        1,
        "redelivery cannot create more work"
    );
    let items = store.decision_inbox_items().unwrap();
    assert_eq!(
        items.len(),
        1,
        "the Decision Inbox shows one canonical decision"
    );
    assert_eq!(items[0].duplicate_deliveries, 1);
    assert_eq!(items[0].status, DecisionInboxStatus::AwaitingApproval);
}

#[test]
fn category_mapping_and_approval_state_are_visible_without_provider_history() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("decision-inbox.sqlite");
    {
        let mut store = SqliteIntakeStore::open(&path, None).unwrap();
        store
            .receive(
                envelope("slack", "operator", "sl:D123", "status for command center"),
                json!({"event_id": "status-event"}),
                Some("operator".to_owned()),
            )
            .unwrap();
        store
            .receive(
                envelope("telegram", "operator", "tg:555", "implement approval flow"),
                json!({"update_id": 8}),
                Some("operator".to_owned()),
            )
            .unwrap();
        let proposal = store.proposals().unwrap()[0].id;
        store
            .approve(
                proposal,
                "operator".to_owned(),
                Utc.with_ymd_and_hms(2026, 8, 17, 5, 1, 0).unwrap(),
                "telegram".to_owned(),
            )
            .unwrap();
    }

    let reopened = SqliteIntakeStore::open(&path, None).unwrap();
    let items = reopened.decision_inbox_items().unwrap();
    assert_eq!(items[0].category, Some(Classification::StatusRequest));
    assert_eq!(items[0].status, DecisionInboxStatus::ReadOnly);
    assert_eq!(items[1].status, DecisionInboxStatus::Approved);
    assert_eq!(
        items[1]
            .proposal
            .as_ref()
            .unwrap()
            .approved_channel
            .as_deref(),
        Some("telegram")
    );
    assert!(items[1].tracked_work.is_some());
}
