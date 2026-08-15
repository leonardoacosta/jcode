use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A provider-neutral inbound message accepted by intake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub adapter: String,
    pub sender_identity: String,
    pub conversation: String,
    pub content: Option<String>,
    pub attachments: Vec<Attachment>,
    pub received_at: DateTime<Utc>,
}

/// Metadata for an attachment whose identifier is assigned by intake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub media_type: String,
    pub byte_len: u64,
    pub content_hash: String,
}

#[cfg(test)]
mod tests {
    use super::{Attachment, Envelope};
    use chrono::{TimeZone, Utc};

    #[test]
    fn envelope_round_trips_through_json() {
        let envelope = Envelope {
            adapter: "example".to_owned(),
            sender_identity: "person:7".to_owned(),
            conversation: "conversation:9".to_owned(),
            content: Some("hello".to_owned()),
            attachments: vec![Attachment {
                id: "attachment:1".to_owned(),
                media_type: "text/plain".to_owned(),
                byte_len: 5,
                content_hash: "abc123".to_owned(),
            }],
            received_at: Utc.with_ymd_and_hms(2026, 8, 15, 18, 46, 0).unwrap(),
        };

        let encoded = serde_json::to_string(&envelope).unwrap();
        let decoded: Envelope = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.adapter, envelope.adapter);
        assert_eq!(decoded.sender_identity, envelope.sender_identity);
        assert_eq!(decoded.conversation, envelope.conversation);
        assert_eq!(decoded.content, envelope.content);
        assert_eq!(decoded.attachments.len(), 1);
        assert_eq!(decoded.attachments[0].id, "attachment:1");
        assert_eq!(decoded.received_at, envelope.received_at);
    }

    #[test]
    fn envelope_supports_contentless_messages() {
        let envelope = Envelope {
            adapter: "example".to_owned(),
            sender_identity: "person:7".to_owned(),
            conversation: "conversation:9".to_owned(),
            content: None,
            attachments: Vec::new(),
            received_at: Utc.with_ymd_and_hms(2026, 8, 15, 18, 46, 0).unwrap(),
        };

        let encoded = serde_json::to_value(envelope).unwrap();
        assert!(encoded["content"].is_null());
        assert_eq!(encoded["attachments"], serde_json::json!([]));
    }
}
