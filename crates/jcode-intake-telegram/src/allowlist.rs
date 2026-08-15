use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Resolves configured Telegram sender identities to the single operator identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allowlist {
    allowed_senders: HashSet<String>,
    operator_identity: String,
}

impl Allowlist {
    /// Creates an empty allowlist for the given operator identity.
    pub fn new(operator_identity: impl Into<String>) -> Self {
        Self {
            allowed_senders: HashSet::new(),
            operator_identity: operator_identity.into(),
        }
    }

    /// Adds a sender identity, such as `tg:12345`, to the allowlist.
    pub fn allow(&mut self, sender: impl Into<String>) {
        self.allowed_senders.insert(sender.into());
    }

    /// Returns the operator identity when the sender is authorized.
    pub fn resolve(&self, sender: &str) -> Option<&str> {
        self.allowed_senders
            .contains(sender)
            .then_some(self.operator_identity.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.allowed_senders.is_empty()
    }
}

/// Formats a configuration-only response that lets an unauthorized sender recover their identity.
pub fn unauthorized_hint(sender: &str) -> String {
    format!("Unauthorized sender {sender}. To authorize, add \"{sender}\" to the intake allowlist.")
}

#[cfg(test)]
mod tests {
    use super::{Allowlist, unauthorized_hint};

    #[test]
    fn fresh_allowlist_authorizes_nobody() {
        let allowlist = Allowlist::new("operator");

        assert!(allowlist.is_empty());
        assert_eq!(allowlist.resolve("tg:12345"), None);
    }

    #[test]
    fn allowed_sender_resolves_to_operator_identity() {
        let mut allowlist = Allowlist::new("operator");

        allowlist.allow("tg:12345");

        assert!(!allowlist.is_empty());
        assert_eq!(allowlist.resolve("tg:12345"), Some("operator"));
    }

    #[test]
    fn different_sender_remains_unauthorized() {
        let mut allowlist = Allowlist::new("operator");
        allowlist.allow("tg:12345");

        assert_eq!(allowlist.resolve("tg:67890"), None);
    }

    #[test]
    fn unauthorized_hint_is_safe_and_copy_pasteable() {
        let hint = unauthorized_hint("tg:12345");
        let lowercase = hint.to_lowercase();

        assert!(hint.contains("tg:12345"));
        assert!(hint.contains("\"tg:12345\""));
        assert!(!hint.contains('/'));
        assert!(!hint.contains('\\'));
        assert!(!lowercase.contains("repository"));
        assert!(!lowercase.contains("crate"));
        assert!(!lowercase.contains("system"));
    }

    #[test]
    fn allowlist_round_trips_through_json() {
        let mut allowlist = Allowlist::new("operator");
        allowlist.allow("tg:12345");

        let json = serde_json::to_string(&allowlist).expect("allowlist should serialize");
        let restored: Allowlist =
            serde_json::from_str(&json).expect("allowlist should deserialize");

        assert_eq!(restored, allowlist);
        assert_eq!(restored.resolve("tg:12345"), Some("operator"));
    }
}
