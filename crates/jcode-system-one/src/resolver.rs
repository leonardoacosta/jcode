//! Types used by the System One service client.
//!
//! Route and credential selection lives in `jcode_base::systemone`, which reads
//! JCode's loaded `config.toml` and provider credential files.

use serde::{Deserialize, Serialize};

/// Selected System One provider, for display and diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    NineRouter,
    TypeSafe,
    OpenRouter,
    JcodeSubscription,
    Omni,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::NineRouter => f.write_str("9router"),
            Provider::TypeSafe => f.write_str("typesafe"),
            Provider::OpenRouter => f.write_str("openrouter"),
            Provider::JcodeSubscription => f.write_str("jcode-subscription"),
            Provider::Omni => f.write_str("omni"),
        }
    }
}

/// Resolved endpoint and credential for one System One call.
#[derive(Clone)]
pub struct ServiceConfig {
    pub provider: Provider,
    pub endpoint_url: String,
    pub api_key: String,
    pub default_model: String,
}
