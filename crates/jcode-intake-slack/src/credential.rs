use std::fmt;

pub const APP_TOKEN_ENV: &str = "SLACK_APP_TOKEN";
pub const BOT_TOKEN_ENV: &str = "SLACK_BOT_TOKEN";
pub const SLACK_ENV_FILE: &str = "slack.env";

#[derive(Clone, PartialEq, Eq)]
pub struct AppToken(String);

#[derive(Clone, PartialEq, Eq)]
pub struct BotToken(String);

macro_rules! secret_token {
    ($name:ident, $prefix:literal) => {
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, CredentialError> {
                let value = value.into();
                if !value.starts_with($prefix) || value.len() <= $prefix.len() + 8 {
                    return Err(CredentialError::InvalidFormat);
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn expose(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([REDACTED])"))
            }
        }
    };
}

secret_token!(AppToken, "xapp-");
secret_token!(BotToken, "xoxb-");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialError {
    Missing(&'static str),
    InvalidFormat,
}

impl fmt::Display for CredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(
                formatter,
                "Slack credential {name} is not configured; use the process environment or slack.env"
            ),
            Self::InvalidFormat => formatter
                .write_str("Slack credential has an invalid format; the value was not logged"),
        }
    }
}

impl std::error::Error for CredentialError {}

pub fn load_tokens() -> Result<(AppToken, BotToken), CredentialError> {
    let app = jcode_provider_env::load_api_key_from_env_or_config(APP_TOKEN_ENV, SLACK_ENV_FILE)
        .ok_or(CredentialError::Missing(APP_TOKEN_ENV))?;
    let bot = jcode_provider_env::load_api_key_from_env_or_config(BOT_TOKEN_ENV, SLACK_ENV_FILE)
        .ok_or(CredentialError::Missing(BOT_TOKEN_ENV))?;
    Ok((AppToken::new(app)?, BotToken::new(bot)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_errors_never_expose_slack_tokens() {
        let app = AppToken::new("xapp-1-A1234567890").unwrap();
        let bot = BotToken::new("xoxb-1234567890-secret").unwrap();
        assert!(!format!("{app:?} {bot:?}").contains("1234567890"));
        let submitted = "private-value";
        let error = AppToken::new(submitted).unwrap_err();
        assert!(!format!("{error:?}: {error}").contains(submitted));
    }
}
