use std::{collections::VecDeque, path::PathBuf};

use jcode_intake_slack::{ApiError, RunOutcome, RunnerConfig, SlackIntakeRunner, SlackTransport};
use jcode_intake_types::{DecisionInboxStatus, SqliteIntakeStore};
use serde_json::{Value, json};
use tempfile::tempdir;

#[derive(Default)]
struct FakeSlackTransport {
    envelopes: VecDeque<Result<Option<Value>, ApiError>>,
    acknowledgements: Vec<String>,
    sent: Vec<(String, String)>,
    fail_ack: bool,
}

impl SlackTransport for FakeSlackTransport {
    fn next_envelope(&mut self) -> Result<Option<Value>, ApiError> {
        self.envelopes.pop_front().unwrap_or(Ok(None))
    }

    fn acknowledge(&mut self, envelope_id: &str) -> Result<(), ApiError> {
        self.acknowledgements.push(envelope_id.to_owned());
        if self.fail_ack {
            return Err(ApiError::Network);
        }
        Ok(())
    }

    fn send_message(&mut self, conversation: &str, text: &str) -> Result<Value, ApiError> {
        self.sent.push((conversation.to_owned(), text.to_owned()));
        Ok(json!({"ok": true, "ts": "1.0"}))
    }
}

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/decision-inbox/slack-socket-envelope.json"
    ))
    .unwrap()
}

fn config(path: PathBuf) -> RunnerConfig {
    RunnerConfig::from_values("T123", "U123", "<@B123>", path).unwrap()
}

#[test]
fn socket_envelope_is_recorded_before_acknowledgement() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("decision-inbox.sqlite");
    let mut transport = FakeSlackTransport {
        fail_ack: true,
        ..Default::default()
    };
    transport.envelopes.push_back(Ok(Some(fixture())));
    let mut runner = SlackIntakeRunner::open(config(db.clone()), transport).unwrap();

    assert!(matches!(
        runner.run_once(),
        Err(jcode_intake_slack::RunnerError::Api(ApiError::Network))
    ));
    drop(runner);

    let reopened = SqliteIntakeStore::open(&db, None).unwrap();
    assert_eq!(reopened.records().unwrap().len(), 1);
    assert_eq!(reopened.proposals().unwrap().len(), 1);
}

#[test]
fn reconnect_redelivery_is_idempotent_and_backfills_the_missed_ack() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("decision-inbox.sqlite");
    let mut first_transport = FakeSlackTransport {
        fail_ack: true,
        ..Default::default()
    };
    first_transport.envelopes.push_back(Ok(Some(fixture())));
    let mut first = SlackIntakeRunner::open(config(db.clone()), first_transport).unwrap();
    let _ = first.run_once();
    drop(first);

    let mut retry_transport = FakeSlackTransport::default();
    retry_transport.envelopes.push_back(Ok(Some(fixture())));
    let mut retry = SlackIntakeRunner::open(config(db), retry_transport).unwrap();
    assert_eq!(
        retry.run_once().unwrap(),
        RunOutcome {
            envelopes: 1,
            acknowledgements: 1
        }
    );
    assert_eq!(retry.transport().acknowledgements, vec!["env-1"]);
    assert_eq!(retry.store().records().unwrap().len(), 2);
    assert_eq!(retry.store().proposals().unwrap().len(), 1);
    let items = retry.store().decision_inbox_items().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].duplicate_deliveries, 1);
    assert_eq!(items[0].status, DecisionInboxStatus::AwaitingApproval);
}

#[test]
fn workspace_and_sender_authorization_are_part_of_source_identity() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("decision-inbox.sqlite");
    let mut event = fixture();
    event["payload"]["team_id"] = json!("T999");
    let mut transport = FakeSlackTransport::default();
    transport.envelopes.push_back(Ok(Some(event)));
    let mut runner = SlackIntakeRunner::open(config(db), transport).unwrap();

    runner.run_once().unwrap();
    let items = runner.store().decision_inbox_items().unwrap();
    assert_eq!(items[0].status, DecisionInboxStatus::Unauthorized);
    assert_eq!(items[0].source.adapter, "slack");
    assert_eq!(items[0].source.sender_identity, "sl:U123");
}
