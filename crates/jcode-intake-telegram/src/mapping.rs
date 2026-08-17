use chrono::Utc;
use jcode_intake_types::Envelope;
use serde_json::Value;

/// Provider-neutral message data extracted from an inbound Telegram update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMessage {
    pub sender: String,
    pub conversation: String,
    pub text: String,
    pub is_group: bool,
    pub addresses_bot: bool,
}

/// The result of classifying and parsing an inbound update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOutcome {
    Message(ParsedMessage),
    Unhandled { variant: String },
}

/// Parse a supported text-bearing update into provider-neutral message data.
pub fn parse(update: &Value, bot_handle: &str) -> ParseOutcome {
    let variant = variant_of(update);
    let Some(message) = update.get("message").or_else(|| update.get("channel_post")) else {
        return ParseOutcome::Unhandled { variant };
    };
    let Some(text) = message.get("text").and_then(Value::as_str) else {
        return ParseOutcome::Unhandled { variant };
    };
    let Some(sender_id) = message
        .get("from")
        .and_then(|sender| sender.get("id"))
        .and_then(identifier_text)
    else {
        return ParseOutcome::Unhandled { variant };
    };
    let Some(chat) = message.get("chat") else {
        return ParseOutcome::Unhandled { variant };
    };
    let Some(conversation_id) = chat.get("id").and_then(identifier_text) else {
        return ParseOutcome::Unhandled { variant };
    };

    let is_group = matches!(
        chat.get("type").and_then(Value::as_str),
        Some("group" | "supergroup")
    );
    let replies_to_bot = message
        .get("reply_to_message")
        .and_then(|reply| reply.get("from"))
        .and_then(|sender| sender.get("is_bot"))
        .and_then(Value::as_bool)
        == Some(true);

    ParseOutcome::Message(ParsedMessage {
        sender: format!("tg:{sender_id}"),
        conversation: format!("tg:{conversation_id}"),
        text: text.to_owned(),
        is_group,
        addresses_bot: text.contains(bot_handle) || replies_to_bot,
    })
}

/// Convert parsed message data into the fixed provider-neutral intake envelope.
pub fn to_envelope(parsed: &ParsedMessage, operator_identity: &str) -> Envelope {
    Envelope {
        adapter: "telegram".to_owned(),
        sender_identity: operator_identity.to_owned(),
        conversation: parsed.conversation.clone(),
        content: Some(parsed.text.clone()),
        attachments: Vec::new(),
        received_at: Utc::now(),
    }
}

fn variant_of(update: &Value) -> String {
    update
        .as_object()
        .and_then(|object| object.keys().find(|key| key.as_str() != "update_id"))
        .cloned()
        .unwrap_or_else(|| "unknown".to_owned())
}

fn identifier_text(value: &Value) -> Option<String> {
    match value {
        Value::Number(number) => Some(number.to_string()),
        Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseOutcome, ParsedMessage, parse, to_envelope};
    use serde_json::json;

    fn parsed(update: serde_json::Value) -> ParsedMessage {
        match parse(&update, "@jcode_bot") {
            ParseOutcome::Message(message) => message,
            outcome => panic!("expected parsed message, got {outcome:?}"),
        }
    }

    #[test]
    fn private_message_is_not_a_group_message() {
        let message = parsed(json!({
            "update_id": 1,
            "message": {
                "from": { "id": 7 },
                "chat": { "id": 555, "type": "private" },
                "text": "hello"
            }
        }));

        assert_eq!(message.sender, "tg:7");
        assert_eq!(message.conversation, "tg:555");
        assert!(!message.is_group);
    }

    #[test]
    fn group_message_with_handle_addresses_bot() {
        let message = parsed(json!({
            "message": {
                "from": { "id": 7 },
                "chat": { "id": -100, "type": "group" },
                "text": "hello @jcode_bot"
            }
        }));

        assert!(message.is_group);
        assert!(message.addresses_bot);
    }

    #[test]
    fn group_message_without_handle_does_not_address_bot() {
        let message = parsed(json!({
            "message": {
                "from": { "id": 7 },
                "chat": { "id": -100, "type": "supergroup" },
                "text": "hello everyone"
            }
        }));

        assert!(message.is_group);
        assert!(!message.addresses_bot);
    }

    #[test]
    fn reply_to_bot_addresses_bot_without_handle() {
        let message = parsed(json!({
            "message": {
                "from": { "id": 7 },
                "chat": { "id": -100, "type": "group" },
                "text": "following up",
                "reply_to_message": { "from": { "is_bot": true } }
            }
        }));

        assert!(message.addresses_bot);
    }

    #[test]
    fn callback_query_is_unhandled_with_variant_name() {
        let outcome = parse(
            &json!({ "update_id": 2, "callback_query": { "id": "cb1" } }),
            "@jcode_bot",
        );

        assert_eq!(
            outcome,
            ParseOutcome::Unhandled {
                variant: "callback_query".to_owned()
            }
        );
    }

    #[test]
    fn message_without_text_is_unhandled() {
        let outcome = parse(
            &json!({
                "update_id": 3,
                "message": {
                    "from": { "id": 7 },
                    "chat": { "id": 555, "type": "private" }
                }
            }),
            "@jcode_bot",
        );

        assert_eq!(
            outcome,
            ParseOutcome::Unhandled {
                variant: "message".to_owned()
            }
        );
    }

    #[test]
    fn envelope_is_neutral_and_carries_message_content() {
        let message = ParsedMessage {
            sender: "tg:7".to_owned(),
            conversation: "tg:555".to_owned(),
            text: "hello".to_owned(),
            is_group: false,
            addresses_bot: false,
        };

        let envelope = to_envelope(&message, &message.sender);

        assert_eq!(envelope.adapter, "telegram");
        assert_eq!(envelope.sender_identity, "tg:7");
        assert_eq!(envelope.conversation, "tg:555");
        assert_eq!(envelope.content.as_deref(), Some("hello"));
        assert!(envelope.attachments.is_empty());
    }
}
