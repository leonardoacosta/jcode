//! The Telegram adapter: gating, authorization, and intake handoff.
//!
//! This is the only component that decides whether an update reaches
//! intake. Transport vocabulary is confined to [`crate::mapping`]; what
//! crosses the intake boundary here is the provider-neutral envelope.

use jcode_intake_types::{IntakeEvent, IntakeStore, RecordId};
use serde_json::Value;

use crate::{
    allowlist::{Allowlist, unauthorized_hint},
    mapping::{ParseOutcome, ParsedMessage, parse, to_envelope},
};

/// A message the adapter wants delivered back to a conversation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outbound {
    pub conversation: String,
    pub text: String,
}

/// What the adapter did with one inbound update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Handled {
    /// Forwarded to intake and recorded.
    Recorded(RecordId),
    /// Recorded, but the sender is not authorized.
    Unauthorized(RecordId),
    /// Recorded as an unhandled transport variant, never dropped.
    UnhandledVariant { record: RecordId, variant: String },
    /// Not forwarded: a group message that did not address the bot.
    Ignored,
}

pub struct TelegramAdapter {
    allowlist: Allowlist,
    bot_handle: String,
    outbound: Vec<Outbound>,
}

impl TelegramAdapter {
    #[must_use]
    pub fn new(allowlist: Allowlist, bot_handle: impl Into<String>) -> Self {
        Self {
            allowlist,
            bot_handle: bot_handle.into(),
            outbound: Vec::new(),
        }
    }

    #[must_use]
    pub fn outbound(&self) -> &[Outbound] {
        &self.outbound
    }

    /// Queue a message for delivery to a conversation.
    pub fn deliver(&mut self, conversation: impl Into<String>, text: impl Into<String>) {
        self.outbound.push(Outbound {
            conversation: conversation.into(),
            text: text.into(),
        });
    }

    /// Process one inbound update.
    ///
    /// Group messages that do not address the bot are dropped before any
    /// record exists, so unrelated group traffic never enters permanent
    /// storage. Everything else is recorded, including unauthorized senders
    /// and update variants this adapter cannot interpret.
    pub fn handle(&mut self, update: &Value, store: &mut IntakeStore) -> Handled {
        let parsed = match parse(update, &self.bot_handle) {
            ParseOutcome::Message(parsed) => parsed,
            ParseOutcome::Unhandled { variant } => {
                let record = self.record_unhandled(update, store, &variant);
                return Handled::UnhandledVariant { record, variant };
            }
        };

        if parsed.is_group && !parsed.addresses_bot {
            return Handled::Ignored;
        }

        match self.allowlist.resolve(&parsed.sender) {
            Some(operator) => {
                let operator = operator.to_owned();
                self.forward(&parsed, update, store, operator)
            }
            None => self.record_unauthorized(&parsed, update, store),
        }
    }

    fn forward(
        &mut self,
        parsed: &ParsedMessage,
        update: &Value,
        store: &mut IntakeStore,
        operator: String,
    ) -> Handled {
        let envelope = to_envelope(parsed, &operator);
        let record = store.receive(envelope, update.clone(), Some(operator));
        self.notify_redaction(record, parsed, store);
        Handled::Recorded(record)
    }

    fn record_unauthorized(
        &mut self,
        parsed: &ParsedMessage,
        update: &Value,
        store: &mut IntakeStore,
    ) -> Handled {
        // The sender identity is retained so the operator can read their own
        // identifier from this first attempt and configure the allowlist.
        let envelope = to_envelope(parsed, &parsed.sender);
        let record = store.receive_unauthorized(envelope, update.clone());
        self.deliver(&parsed.conversation, unauthorized_hint(&parsed.sender));
        Handled::Unauthorized(record)
    }

    fn record_unhandled(
        &mut self,
        update: &Value,
        store: &mut IntakeStore,
        variant: &str,
    ) -> RecordId {
        let envelope = jcode_intake_types::Envelope {
            adapter: "telegram".to_owned(),
            sender_identity: format!("unhandled:{variant}"),
            conversation: format!("unhandled:{variant}"),
            content: None,
            attachments: Vec::new(),
            received_at: chrono::Utc::now(),
        };
        store.receive(envelope, update.clone(), None)
    }

    fn notify_redaction(&mut self, record: RecordId, parsed: &ParsedMessage, store: &IntakeStore) {
        let redacted = store.events().iter().any(
            |event| matches!(event, IntakeEvent::Redaction { record: id, .. } if *id == record),
        );
        if redacted {
            // The notice deliberately does not restate the redacted value.
            self.deliver(
                &parsed.conversation,
                "Note: credential-shaped content was redacted before storage.",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_intake_types::Classification;
    use serde_json::json;

    const TOKEN: &str = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM";

    fn allowlist() -> Allowlist {
        let mut list = Allowlist::new("op:leo");
        list.allow("tg:7");
        list
    }

    fn adapter() -> TelegramAdapter {
        TelegramAdapter::new(allowlist(), "@jcode")
    }

    fn private(text: &str, user: u64) -> Value {
        json!({
            "update_id": 1,
            "message": {
                "chat": {"id": 555, "type": "private"},
                "from": {"id": user},
                "text": text,
            }
        })
    }

    fn group(text: &str, user: u64) -> Value {
        json!({
            "update_id": 2,
            "message": {
                "chat": {"id": -100, "type": "group"},
                "from": {"id": user},
                "text": text,
            }
        })
    }

    #[test]
    fn direct_message_from_allowlisted_sender_is_forwarded() {
        let mut store = IntakeStore::new(None);
        let mut adapter = adapter();
        let outcome = adapter.handle(&private("hello", 7), &mut store);
        assert!(matches!(outcome, Handled::Recorded(_)));
        assert_eq!(store.records().len(), 1);
        assert_eq!(store.records()[0].operator.as_deref(), Some("op:leo"));
    }

    #[test]
    fn group_message_without_mention_creates_no_record() {
        let mut store = IntakeStore::new(None);
        let mut adapter = adapter();
        let outcome = adapter.handle(&group("unrelated chatter", 7), &mut store);
        assert_eq!(outcome, Handled::Ignored);
        assert!(store.records().is_empty());
    }

    #[test]
    fn group_message_addressing_the_bot_is_forwarded() {
        let mut store = IntakeStore::new(None);
        let mut adapter = adapter();
        let outcome = adapter.handle(&group("@jcode status?", 7), &mut store);
        assert!(matches!(outcome, Handled::Recorded(_)));
        assert_eq!(store.records().len(), 1);
    }

    #[test]
    fn unauthorized_sender_is_recorded_but_not_promoted() {
        let mut store = IntakeStore::with_classifier(None, |_| Ok(Classification::WorkRequest));
        let mut adapter = adapter();
        let outcome = adapter.handle(&private("deploy prod", 999), &mut store);
        assert!(matches!(outcome, Handled::Unauthorized(_)));
        assert_eq!(store.records().len(), 1, "the attempt is retained");
        assert!(store.records()[0].operator.is_none());
        assert!(
            store.proposals().is_empty(),
            "an unauthorized sender must not create work"
        );
    }

    #[test]
    fn unauthorized_reply_carries_the_sender_id_and_no_repository_content() {
        let mut store = IntakeStore::new(None);
        let mut adapter = adapter();
        adapter.handle(&private("hello", 999), &mut store);
        let reply = &adapter.outbound()[0].text;
        assert!(reply.contains("tg:999"), "operator can self-configure");
        assert!(!reply.contains('/'), "no paths or repository content");
    }

    #[test]
    fn unhandled_variant_is_recorded_with_its_name() {
        let mut store = IntakeStore::new(None);
        let mut adapter = adapter();
        let update = json!({"update_id": 3, "callback_query": {"id": "cb1"}});
        let outcome = adapter.handle(&update, &mut store);
        match outcome {
            Handled::UnhandledVariant { variant, .. } => assert_eq!(variant, "callback_query"),
            other => panic!("expected an unhandled variant, got {other:?}"),
        }
        assert_eq!(
            store.records().len(),
            1,
            "unhandled updates are not dropped"
        );
    }

    #[test]
    fn redaction_notice_is_sent_without_restating_the_value() {
        let mut store = IntakeStore::new(None);
        let mut adapter = adapter();
        adapter.handle(&private(&format!("my token {TOKEN}"), 7), &mut store);
        let notice = &adapter.outbound()[0].text;
        assert!(notice.to_lowercase().contains("redact"));
        assert!(
            !notice.contains(TOKEN),
            "the notice must not leak the value"
        );
    }

    #[test]
    fn randomized_delivery_sequence_does_not_defeat_deduplication() {
        let mut store = IntakeStore::with_classifier(None, |_| Ok(Classification::StatusRequest));
        let mut adapter = adapter();
        adapter.handle(&private("deploy", 7), &mut store);

        // Telegram may randomize its delivery counter after idle periods.
        let mut later = private("deploy", 7);
        later["update_id"] = json!(987_654_321_u64);
        adapter.handle(&later, &mut store);

        assert_eq!(store.records().len(), 2, "both deliveries are retained");
        assert!(
            store.records()[1].duplicate_of.is_some(),
            "dedupe must survive a randomized delivery sequence"
        );
    }
}
