//! Telegram bot credential loading.
//!
//! The token follows Jcode's existing provider-secret path: process
//! environment first, then the hardened env file under the Jcode config
//! directory. The value is never serialized and its `Debug` implementation
//! is always redacted.

use std::fmt;

pub const BOT_TOKEN_ENV: &str = "TELEGRAM_BOT_TOKEN";
pub const BOT_ENV_FILE: &str = "telegram.env";

/// A validated Telegram bot token whose value cannot appear in `Debug` output.
///
/// Deliberately does not implement `Serialize`, `Display`, or `AsRef<str>`.
/// The narrow [`BotToken::expose`] method makes every use of the credential
/// explicit at the HTTP boundary.
#[derive(Clone, PartialEq, Eq)]
pub struct BotToken(String);

impl BotToken {
    pub fn new(value: impl Into<String>) -> Result<Self, CredentialError> {
        let value = value.into();
        if !valid_token_shape(&value) {
            return Err(CredentialError::InvalidFormat);
        }
        Ok(Self(value))
    }

    /// Expose the value only at the outbound HTTP authorization boundary.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for BotToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BotToken([REDACTED])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialError {
    Missing,
    InvalidFormat,
}

impl fmt::Display for CredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => formatter.write_str(
                "Telegram bot credential is not configured; set TELEGRAM_BOT_TOKEN or save it in telegram.env",
            ),
            Self::InvalidFormat => formatter.write_str(
                "Telegram bot credential has an invalid format; the value was not logged",
            ),
        }
    }
}

impl std::error::Error for CredentialError {}

/// Load from the canonical Jcode credential path without logging the value.
pub fn load_bot_token() -> Result<BotToken, CredentialError> {
    let value = jcode_provider_env::load_api_key_from_env_or_config(BOT_TOKEN_ENV, BOT_ENV_FILE)
        .ok_or(CredentialError::Missing)?;
    BotToken::new(value)
}

fn valid_token_shape(value: &str) -> bool {
    let Some((id, secret)) = value.split_once(':') else {
        return false;
    };
    (8..=10).contains(&id.len())
        && id.bytes().all(|byte| byte.is_ascii_digit())
        && secret.len() == 35
        && secret
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM";

    #[test]
    fn valid_token_can_be_exposed_only_explicitly() {
        let token = BotToken::new(TOKEN).expect("valid token");
        assert_eq!(token.expose(), TOKEN);
    }

    #[test]
    fn debug_output_never_contains_the_token() {
        let token = BotToken::new(TOKEN).expect("valid token");
        let output = format!("{token:?}");
        assert_eq!(output, "BotToken([REDACTED])");
        assert!(!output.contains(TOKEN));
    }

    #[test]
    fn invalid_format_error_never_contains_the_submitted_value() {
        let submitted = "not-a-real-secret-but-still-private";
        let error = BotToken::new(submitted).expect_err("invalid format");
        let output = format!("{error:?}: {error}");
        assert!(!output.contains(submitted));
    }

    #[test]
    fn malformed_values_are_rejected() {
        for value in [
            "",
            "123:nope",
            "abcdefgh:abcdefghijklmnopqrstuvwxyz123456789",
        ] {
            assert_eq!(BotToken::new(value), Err(CredentialError::InvalidFormat));
        }
    }
}
