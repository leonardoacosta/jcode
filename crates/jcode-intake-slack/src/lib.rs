//! A second transport adapter, used as the neutrality conformance test.
//!
//! Task 5.5 of `add-factory-intake-capability`: the design claims a second
//! adapter requires no change to the intake core. This crate exists to make
//! that claim falsifiable in code rather than in prose. Slack's payload
//! shape differs from Telegram's in every structural respect (different
//! nesting, different identifier fields, inverted conversation-type
//! detection, different unhandled-variant vocabulary), so if the core had
//! absorbed any Telegram assumption, this crate could not compile against
//! it unchanged.

use chrono::Utc;
use jcode_intake_types::{Envelope, IntakeStore, RecordId};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMessage {
    pub sender: String,
    pub conversation: String,
    pub text: String,
    pub is_group: bool,
    pub addresses_bot: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOutcome {
    Message(ParsedMessage),
    Unhandled { variant: String },
}

/// Parse a Slack event into provider-neutral message data.
#[must_use]
pub fn parse(event: &Value, bot_handle: &str) -> ParseOutcome {
    let inner = event.get("event");
    let Some(inner) = inner else {
        return ParseOutcome::Unhandled {
            variant: "missing_event".to_owned(),
        };
    };
    let variant = inner
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    if variant != "message" {
        return ParseOutcome::Unhandled { variant };
    }
    let Some(text) = inner.get("text").and_then(Value::as_str) else {
        return ParseOutcome::Unhandled { variant };
    };
    let Some(channel) = inner.get("channel").and_then(Value::as_str) else {
        return ParseOutcome::Unhandled { variant };
    };
    let user = inner.get("user").and_then(Value::as_str).unwrap_or("");

    // Slack marks direct messages with a channel id beginning with `D`, the
    // inverse of Telegram's explicit chat type field.
    let is_group = !channel.starts_with('D');
    let addresses_bot = text.contains(bot_handle) || inner.get("thread_ts").is_some();

    ParseOutcome::Message(ParsedMessage {
        sender: format!("sl:{user}"),
        conversation: format!("sl:{channel}"),
        text: text.to_owned(),
        is_group,
        addresses_bot,
    })
}

#[must_use]
pub fn to_envelope(parsed: &ParsedMessage, operator_identity: &str) -> Envelope {
    Envelope {
        adapter: "slack".to_owned(),
        sender_identity: operator_identity.to_owned(),
        conversation: parsed.conversation.clone(),
        content: Some(parsed.text.clone()),
        attachments: Vec::new(),
        received_at: Utc::now(),
    }
}

/// Forward an event through the same gating rules the Telegram adapter uses,
/// against the same unmodified core.
pub fn handle(
    event: &Value,
    bot_handle: &str,
    operator: Option<&str>,
    store: &mut IntakeStore,
) -> Option<RecordId> {
    match parse(event, bot_handle) {
        ParseOutcome::Message(parsed) => {
            if parsed.is_group && !parsed.addresses_bot {
                return None;
            }
            match operator {
                Some(operator) => {
                    let envelope = to_envelope(&parsed, operator);
                    Some(store.receive(envelope, event.clone(), Some(operator.to_owned())))
                }
                None => {
                    let envelope = to_envelope(&parsed, &parsed.sender);
                    Some(store.receive_unauthorized(envelope, event.clone()))
                }
            }
        }
        ParseOutcome::Unhandled { variant } => {
            let envelope = Envelope {
                adapter: "slack".to_owned(),
                sender_identity: format!("unhandled:{variant}"),
                conversation: format!("unhandled:{variant}"),
                content: None,
                attachments: Vec::new(),
                received_at: Utc::now(),
            };
            Some(store.receive_unauthorized(envelope, event.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_intake_types::Classification;
    use serde_json::json;

    fn dm(text: &str) -> Value {
        json!({"event": {"type": "message", "channel": "D01", "user": "U7", "text": text}})
    }

    fn channel(text: &str) -> Value {
        json!({"event": {"type": "message", "channel": "C01", "user": "U7", "text": text}})
    }

    #[test]
    fn direct_message_is_forwarded_without_a_mention() {
        let mut store = IntakeStore::new(None);
        let id = handle(&dm("hello"), "@jcode", Some("op:leo"), &mut store);
        assert!(id.is_some());
        assert_eq!(store.records().len(), 1);
        assert_eq!(store.records()[0].adapter, "slack");
    }

    #[test]
    fn channel_message_without_mention_is_ignored() {
        let mut store = IntakeStore::new(None);
        let id = handle(&channel("unrelated"), "@jcode", Some("op:leo"), &mut store);
        assert!(id.is_none());
        assert!(store.records().is_empty());
    }

    #[test]
    fn channel_message_with_mention_is_forwarded() {
        let mut store = IntakeStore::new(None);
        let id = handle(
            &channel("@jcode status"),
            "@jcode",
            Some("op:leo"),
            &mut store,
        );
        assert!(id.is_some());
    }

    #[test]
    fn non_message_events_are_recorded_as_unhandled() {
        let mut store = IntakeStore::new(None);
        let event = json!({"event": {"type": "reaction_added", "user": "U7"}});
        handle(&event, "@jcode", Some("op:leo"), &mut store);
        assert_eq!(store.records().len(), 1, "never dropped");
        assert!(
            store.records()[0]
                .sender_identity
                .contains("reaction_added")
        );
    }

    /// The conformance assertion: both transports promote through one path,
    /// and identical text on different transports is not a false duplicate.
    #[test]
    fn two_transports_share_one_unmodified_core() {
        let mut store = IntakeStore::with_classifier(None, |_| Ok(Classification::WorkRequest));

        let telegram_envelope = Envelope {
            adapter: "telegram".to_owned(),
            sender_identity: "op:leo".to_owned(),
            conversation: "tg:555".to_owned(),
            content: Some("do the thing".to_owned()),
            attachments: Vec::new(),
            received_at: Utc::now(),
        };
        store.receive(
            telegram_envelope,
            json!({"update_id": 1}),
            Some("op:leo".to_owned()),
        );

        handle(&dm("do the thing"), "@jcode", Some("op:leo"), &mut store);

        assert_eq!(store.records().len(), 2);
        assert!(
            store.records()[1].duplicate_of.is_none(),
            "identical text on a different transport is not a duplicate"
        );
        assert_eq!(
            store.proposals().len(),
            2,
            "both transports promote through the same approval path"
        );
    }
}
