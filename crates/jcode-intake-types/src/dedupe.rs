use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A stable content-and-identity key computed before content redaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DedupeKey(String);

impl DedupeKey {
    /// Derives a key from sender, conversation, and pre-redaction content.
    #[must_use]
    pub fn new(sender_identity: &str, conversation: &str, content: Option<&str>) -> Self {
        let mut hasher = Sha256::new();
        for part in [sender_identity, conversation, content.unwrap_or("")] {
            hasher.update(part.as_bytes());
            hasher.update([0x1f]);
        }
        Self(format!("{digest:x}", digest = hasher.finalize()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for DedupeKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::DedupeKey;

    #[test]
    fn identical_content_from_same_sender_has_same_key() {
        let first = DedupeKey::new("person:1", "conversation:1", Some("hello"));
        let second = DedupeKey::new("person:1", "conversation:1", Some("hello"));

        assert_eq!(first, second);
        assert_eq!(first.as_str().len(), 64);
    }

    #[test]
    fn identical_content_from_different_senders_has_different_keys() {
        let first = DedupeKey::new("person:1", "conversation:1", Some("hello"));
        let second = DedupeKey::new("person:2", "conversation:1", Some("hello"));

        assert_ne!(first, second);
    }

    #[test]
    fn identical_content_in_different_conversations_has_different_keys() {
        let first = DedupeKey::new("person:1", "conversation:1", Some("hello"));
        let second = DedupeKey::new("person:1", "conversation:2", Some("hello"));

        assert_ne!(first, second);
    }

    #[test]
    fn external_delivery_counters_cannot_affect_the_key() {
        let reused_counter = 17_u64;
        let randomized_counter = 9_481_516_u64;

        let first = DedupeKey::new("person:1", "conversation:1", Some("hello"));
        let second = DedupeKey::new("person:1", "conversation:1", Some("hello"));

        assert_ne!(reused_counter, randomized_counter);
        assert_eq!(first, second);
    }

    #[test]
    fn different_pre_redaction_credentials_have_different_keys() {
        let first = DedupeKey::new(
            "person:1",
            "conversation:1",
            Some("credential sk-AAAAAAAAAAAAAAAAAAAA"),
        );
        let second = DedupeKey::new(
            "person:1",
            "conversation:1",
            Some("credential sk-BBBBBBBBBBBBBBBBBBBB"),
        );

        assert_ne!(first, second);
    }

    #[test]
    fn field_separator_prevents_concatenation_aliases() {
        let first = DedupeKey::new("ab", "c", Some("d"));
        let second = DedupeKey::new("a", "bc", Some("d"));

        assert_ne!(first, second);
    }
}
