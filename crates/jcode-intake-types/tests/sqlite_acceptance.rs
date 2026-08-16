//! Independent acceptance tests for the durable SQLite intake store.
//!
//! These are intentionally outside `sqlite.rs`: the implementation agent's
//! own tests cannot independently establish that the public API satisfies the
//! OpenSpec durability and restart requirements.

use chrono::Utc;
use jcode_intake_types::{Classification, Envelope, ProposalState, SqliteIntakeStore};
use serde_json::json;
use tempfile::tempdir;

const TOKEN: &str = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM";

fn envelope(content: &str) -> Envelope {
    Envelope {
        adapter: "telegram".into(),
        sender_identity: "op:leo".into(),
        conversation: "tg:555".into(),
        content: Some(content.into()),
        attachments: Vec::new(),
        received_at: Utc::now(),
    }
}

/// Public acceptance path: message -> durable record -> proposal, no work.
#[test]
fn message_and_proposal_survive_a_real_close_and_reopen() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("intake.sqlite");
    {
        let mut store = SqliteIntakeStore::open_with_classifier(&path, None, |_| {
            Ok(Classification::WorkRequest)
        })
        .unwrap();
        store
            .receive(
                envelope("build a dashboard"),
                json!({"update_id": 1}),
                Some("op:leo".into()),
            )
            .unwrap();
        assert_eq!(store.records().unwrap().len(), 1);
        assert_eq!(store.proposals().unwrap().len(), 1);
        assert!(store.tracked_work().unwrap().is_empty());
    }

    let reopened = SqliteIntakeStore::open(&path, None).unwrap();
    assert_eq!(reopened.records().unwrap().len(), 1);
    let proposals = reopened.proposals().unwrap();
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].state, ProposalState::AwaitingApproval);
    assert!(reopened.tracked_work().unwrap().is_empty());
}

/// Public acceptance path: approval -> linked work -> durable audit metadata.
#[test]
fn approval_and_linked_work_survive_restart_and_conversation_loss() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("intake.sqlite");
    let record;
    {
        let mut store = SqliteIntakeStore::open_with_classifier(&path, None, |_| {
            Ok(Classification::WorkRequest)
        })
        .unwrap();
        record = store
            .receive(envelope("build it"), json!({}), Some("op:leo".into()))
            .unwrap();
        let proposal = store.proposals().unwrap()[0].id;
        store
            .approve(proposal, "op:leo".into(), Utc::now(), "telegram".into())
            .unwrap();
    }

    // The upstream conversation may now be deleted. Nothing in this reopen
    // contacts Telegram or reads conversation history.
    let reopened = SqliteIntakeStore::open(&path, None).unwrap();
    let proposal = &reopened.proposals().unwrap()[0];
    assert_eq!(proposal.state, ProposalState::Approved);
    assert_eq!(proposal.approved_by.as_deref(), Some("op:leo"));
    assert_eq!(proposal.approved_channel.as_deref(), Some("telegram"));
    assert!(proposal.approved_at.is_some());
    let work = reopened.tracked_work().unwrap();
    assert_eq!(work.len(), 1);
    assert_eq!(work[0].from_record, record);
    assert_eq!(reopened.records().unwrap().len(), 1);
}

/// Ingress redaction must occur before the first SQLite INSERT. Checking the
/// file bytes is stronger than checking the deserialized public record.
#[test]
fn credential_never_reaches_the_database_or_wal_bytes() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("intake.sqlite");
    {
        let mut store = SqliteIntakeStore::open(&path, None).unwrap();
        store
            .receive(
                envelope(&format!("token {TOKEN}")),
                json!({"nested": {"secret": TOKEN}}),
                Some("op:leo".into()),
            )
            .unwrap();
    }

    for candidate in [
        &path,
        &path.with_extension("sqlite-wal"),
        &path.with_extension("sqlite-shm"),
    ] {
        if let Ok(bytes) = std::fs::read(candidate) {
            assert!(
                !bytes
                    .windows(TOKEN.len())
                    .any(|window| window == TOKEN.as_bytes()),
                "credential reached SQLite storage at {}",
                candidate.display()
            );
        }
    }
}

/// The first transaction is committed before classification. A panic in the
/// classifier cannot erase the inbound record.
#[test]
fn classifier_panic_still_leaves_a_restart_recoverable_record() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("intake.sqlite");
    {
        let mut store = SqliteIntakeStore::open_with_classifier(&path, None, |_| {
            panic!("synthetic classifier failure")
        })
        .unwrap();
        store
            .receive(envelope("ambiguous"), json!({}), Some("op:leo".into()))
            .unwrap();
    }

    let reopened = SqliteIntakeStore::open(&path, None).unwrap();
    let records = reopened.records().unwrap();
    assert_eq!(records.len(), 1);
    assert!(records[0].classification_error.is_some());
    assert!(!records[0].executed);
}

#[test]
fn classifier_error_still_leaves_a_restart_recoverable_record() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("intake.sqlite");
    {
        let mut store = SqliteIntakeStore::open_with_classifier(&path, None, |_| {
            Err("synthetic classification error".to_owned())
        })
        .unwrap();
        store
            .receive(envelope("ambiguous"), json!({}), Some("op:leo".into()))
            .unwrap();
    }

    let reopened = SqliteIntakeStore::open(&path, None).unwrap();
    let records = reopened.records().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0].classification_error.as_deref(),
        Some("synthetic classification error")
    );
    assert!(!records[0].executed);
}
