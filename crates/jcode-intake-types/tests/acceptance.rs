//! Independent acceptance tests for the factory-intake capability.
//!
//! These exercise the public API only, and were written against the spec
//! scenarios rather than against the implementation. They deliberately
//! duplicate intent from the unit tests: those were written by the same
//! agents that wrote the code, so they cannot independently confirm it.

use chrono::Utc;
use jcode_intake_types::{Classification, Envelope, IntakeEvent, IntakeStore, Redactor};
use serde_json::json;

const TG_TOKEN: &str = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM";
const TG_TOKEN_2: &str = "987654321:BBHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM";

fn envelope(adapter: &str, sender: &str, conversation: &str, content: &str) -> Envelope {
    Envelope {
        adapter: adapter.to_string(),
        sender_identity: sender.to_string(),
        conversation: conversation.to_string(),
        content: Some(content.to_string()),
        attachments: Vec::new(),
        received_at: Utc::now(),
    }
}

/// Scenario: Durable record before interpretation / An operator inspects history.
#[test]
fn every_message_is_retained_permanently() {
    let mut store = IntakeStore::with_classifier(Some(1), |_| Ok(Classification::StatusRequest));
    for i in 0..4 {
        store.receive(
            envelope("telegram", "op:leo", "c1", &format!("message {i}")),
            json!({}),
            Some("op:leo".into()),
        );
    }
    // duplicate of the first
    store.receive(
        envelope("telegram", "op:leo", "c1", "message 0"),
        json!({}),
        Some("op:leo".into()),
    );
    assert_eq!(
        store.records().len(),
        5,
        "duplicates and deferred messages must all be retained"
    );
}

/// Scenario: A credential is pasted into a message.
/// The strongest form: the token must appear nowhere in the entire serialized store.
#[test]
fn no_unredacted_credential_survives_anywhere_in_the_store() {
    let mut store = IntakeStore::new(None);
    store.receive(
        envelope(
            "telegram",
            "op:leo",
            "c1",
            &format!("my token is {TG_TOKEN}"),
        ),
        json!({
            "message": {"text": format!("my token is {TG_TOKEN}")},
            "nested": {"deep": {"deeper": [{"leaked": TG_TOKEN}]}},
        }),
        Some("op:leo".into()),
    );

    let serialized = serde_json::to_string(store.records()).expect("records serialize");
    assert!(
        !serialized.contains(TG_TOKEN),
        "credential survived in the record store"
    );

    let events = serde_json::to_string(store.events()).expect("events serialize");
    assert!(
        !events.contains(TG_TOKEN),
        "credential survived in the event log"
    );

    assert!(
        store
            .events()
            .iter()
            .any(|e| matches!(e, IntakeEvent::Redaction { .. })),
        "a redaction event must be visible to the operator"
    );
}

/// Scenario: Two messages containing different credentials arrive.
/// Regression guard for the defect where post-redaction keying collapsed them.
#[test]
fn different_credentials_do_not_collapse_into_one_record() {
    let mut store = IntakeStore::new(None);
    let a = store.receive(
        envelope("telegram", "op:leo", "c1", &format!("token {TG_TOKEN}")),
        json!({}),
        Some("op:leo".into()),
    );
    let b = store.receive(
        envelope("telegram", "op:leo", "c1", &format!("token {TG_TOKEN_2}")),
        json!({}),
        Some("op:leo".into()),
    );
    let rec_b = store
        .records()
        .iter()
        .find(|r| r.id == b)
        .expect("second record exists");
    assert_ne!(a, b);
    assert!(
        rec_b.duplicate_of.is_none(),
        "two different credentials must not be treated as the same message"
    );
}

/// Scenario: Distinct senders send identical content.
#[test]
fn identity_and_conversation_participate_in_the_key() {
    let mut store = IntakeStore::new(None);
    store.receive(
        envelope("telegram", "op:leo", "c1", "ok"),
        json!({}),
        Some("op:leo".into()),
    );
    let other_sender = store.receive(
        envelope("telegram", "op:sam", "c1", "ok"),
        json!({}),
        Some("op:sam".into()),
    );
    let other_conv = store.receive(
        envelope("telegram", "op:leo", "c2", "ok"),
        json!({}),
        Some("op:leo".into()),
    );

    for id in [other_sender, other_conv] {
        let rec = store.records().iter().find(|r| r.id == id).unwrap();
        assert!(
            rec.duplicate_of.is_none(),
            "identical text from a different sender or conversation must not collide"
        );
    }
}

/// Scenario: A deferred message is resent.
/// This is the defect that made throttling permanent.
#[test]
fn resending_a_throttled_message_is_not_swallowed() {
    let mut store = IntakeStore::with_classifier(Some(0), |_| Ok(Classification::WorkRequest));
    let first = store.receive(
        envelope("telegram", "op:leo", "c1", "do the thing"),
        json!({}),
        Some("op:leo".into()),
    );
    let first_rec = store.records().iter().find(|r| r.id == first).unwrap();
    assert!(first_rec.deferred, "budget of zero must defer");

    let second = store.receive(
        envelope("telegram", "op:leo", "c1", "do the thing"),
        json!({}),
        Some("op:leo".into()),
    );
    let second_rec = store.records().iter().find(|r| r.id == second).unwrap();
    assert!(
        second_rec.duplicate_of.is_none(),
        "a resend of a never-executed message must not be a duplicate"
    );
    assert_eq!(
        second_rec.retry_of,
        Some(first),
        "the resend must be recorded as a retry of the deferred message"
    );
}

/// Scenario: A transport replays a delivery.
/// The complement of the retry rule: a genuine replay is still deduped.
#[test]
fn replay_of_an_executed_message_is_still_a_duplicate() {
    let mut store = IntakeStore::with_classifier(None, |_| Ok(Classification::StatusRequest));
    let first = store.receive(
        envelope("telegram", "op:leo", "c1", "ping"),
        json!({}),
        Some("op:leo".into()),
    );
    let second = store.receive(
        envelope("telegram", "op:leo", "c1", "ping"),
        json!({}),
        Some("op:leo".into()),
    );
    let rec = store.records().iter().find(|r| r.id == second).unwrap();
    assert_eq!(rec.duplicate_of, Some(first));
}

/// Scenario: Admission control is applied across message classes.
#[test]
fn read_only_traffic_cannot_starve_work_proposals() {
    let mut store = IntakeStore::with_classifier(Some(1), |text| {
        if text.starts_with("status") {
            Ok(Classification::StatusRequest)
        } else {
            Ok(Classification::WorkRequest)
        }
    });
    store.receive(
        envelope("telegram", "op:leo", "c1", "status please"),
        json!({}),
        Some("op:leo".into()),
    );
    let work = store.receive(
        envelope("telegram", "op:leo", "c1", "build a dashboard"),
        json!({}),
        Some("op:leo".into()),
    );
    let rec = store.records().iter().find(|r| r.id == work).unwrap();
    assert!(
        !rec.deferred,
        "a status request must not consume the work-request budget"
    );
    assert_eq!(store.proposals().len(), 1);
}

/// Scenario: A message requests new work / A proposal is approved.
#[test]
fn work_is_created_only_by_explicit_approval() {
    let mut store = IntakeStore::with_classifier(None, |_| Ok(Classification::WorkRequest));
    let record = store.receive(
        envelope("telegram", "op:leo", "c1", "build a dashboard"),
        json!({}),
        Some("op:leo".into()),
    );
    assert_eq!(store.proposals().len(), 1);
    assert!(
        store.tracked_work().is_empty(),
        "no tracked work may exist before approval"
    );

    let proposal = store.proposals()[0].id;
    let at = Utc::now();
    store
        .approve(proposal, "op:leo".into(), at, "telegram".into())
        .expect("approval succeeds");

    assert_eq!(store.tracked_work().len(), 1);
    assert_eq!(store.tracked_work()[0].from_record, record);
    let approved = &store.proposals()[0];
    assert_eq!(approved.approved_by.as_deref(), Some("op:leo"));
    assert_eq!(approved.approved_channel.as_deref(), Some("telegram"));
    assert!(approved.approved_at.is_some());
}

/// Scenario: Interpretation fails.
#[test]
fn classification_failure_leaves_an_inspectable_record() {
    let mut store = IntakeStore::with_classifier(None, |_| Err("classifier exploded".to_string()));
    let id = store.receive(
        envelope("telegram", "op:leo", "c1", "ambiguous"),
        json!({}),
        Some("op:leo".into()),
    );
    let rec = store.records().iter().find(|r| r.id == id).unwrap();
    assert!(rec.classification_error.is_some());
    assert_eq!(store.records().len(), 1);
}

/// Scenario: Conversation history is unavailable.
#[test]
fn records_and_work_survive_loss_of_conversation_context() {
    let mut store = IntakeStore::with_classifier(None, |_| Ok(Classification::WorkRequest));
    let record = store.receive(
        envelope("telegram", "op:leo", "c1", "build a dashboard"),
        json!({}),
        Some("op:leo".into()),
    );
    let proposal = store.proposals()[0].id;
    store
        .approve(proposal, "op:leo".into(), Utc::now(), "telegram".into())
        .expect("approval succeeds");

    // Authority derives from the recorded approver, not from the conversation.
    let approved = &store.proposals()[0];
    assert_eq!(approved.approved_by.as_deref(), Some("op:leo"));
    assert_eq!(store.tracked_work()[0].from_record, record);
    assert_eq!(store.records().len(), 1);
}

/// Scenario: A second transport is added.
#[test]
fn two_transports_share_one_core_without_collision() {
    let mut store = IntakeStore::with_classifier(None, |_| Ok(Classification::WorkRequest));
    let tg = store.receive(
        envelope("telegram", "op:leo", "tg:1", "do the thing"),
        json!({"update_id": 1}),
        Some("op:leo".into()),
    );
    let sl = store.receive(
        envelope("slack", "op:leo", "sl:D01", "do the thing"),
        json!({"event": {"thread_ts": "1.2"}}),
        Some("op:leo".into()),
    );
    let tg_rec = store.records().iter().find(|r| r.id == tg).unwrap();
    let sl_rec = store.records().iter().find(|r| r.id == sl).unwrap();

    assert_eq!(tg_rec.adapter, "telegram");
    assert_eq!(sl_rec.adapter, "slack");
    assert!(
        sl_rec.duplicate_of.is_none(),
        "identical text on a different transport is not a duplicate"
    );
    assert_eq!(store.proposals().len(), 2, "both promote through one path");
}

/// Scenario: Ordinary content arrives.
#[test]
fn ordinary_content_is_stored_verbatim() {
    let mut store = IntakeStore::new(None);
    let text = "deploy the staging environment at 5pm, contact leo@example.com";
    let id = store.receive(
        envelope("telegram", "op:leo", "c1", text),
        json!({}),
        Some("op:leo".into()),
    );
    let rec = store.records().iter().find(|r| r.id == id).unwrap();
    assert_eq!(rec.content.as_deref(), Some(text), "no spurious redaction");
    assert!(store.events().is_empty());
}

/// The redactor is scoped to credential-shaped strings only.
#[test]
fn redactor_scope_is_narrow() {
    let redactor = Redactor::new();
    for benign in [
        "https://example.com/path?query=value",
        "leo@example.com",
        "a perfectly ordinary sentence",
        "1234567890",
    ] {
        let outcome = redactor.scrub(benign);
        assert_eq!(outcome.count, 0, "benign string was redacted: {benign}");
        assert_eq!(outcome.text, benign);
    }
    assert!(redactor.scrub(TG_TOKEN).count > 0, "real token must redact");
}
