//! Provider and credential resolver for the System One service.
//!
//! Resolves endpoint URL, API key, and default model from environment variables.
//! Prefers OpenRouter when its credential is available, with TypeSafe as fallback.
//! Credentials come only from environment variables — never from config files,
//! arguments, or serialized state.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Which System One provider to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    /// Direct TypeSafe System One API (api.typesafe.ai).
    TypeSafe,
    /// OpenRouter Decisions endpoint with `~typesafe/jev-latest` model alias.
    OpenRouter,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::TypeSafe => write!(f, "typesafe"),
            Provider::OpenRouter => write!(f, "openrouter"),
        }
    }
}

/// Resolved configuration for a System One service call.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// The provider selected for this configuration.
    pub provider: Provider,
    /// Full endpoint URL (e.g. `https://api.typesafe.ai/v1/systemone`).
    pub endpoint_url: String,
    /// Bearer token for the selected provider.
    pub api_key: String,
    /// Default model for this provider when no override is given.
    pub default_model: String,
}

const TYPESAFE_URL: &str = "https://api.typesafe.ai/v1/systemone";
const OPENROUTER_URL: &str = "https://openrouter.ai/api/alpha/decisions";
const TYPESAFE_DEFAULT_MODEL: &str = "jev-latest";
const OPENROUTER_DEFAULT_MODEL: &str = "~typesafe/jev-latest";

/// Resolve the System One service configuration from environment variables.
///
/// When `provider` is `None`, auto-detection prefers OpenRouter then falls
/// back to TypeSafe. When a provider is explicitly chosen, only its key is
/// used and an error is returned if it is missing.
///
/// Empty or whitespace-only environment values are treated as absent.
pub fn resolve_service(provider: Option<Provider>) -> Result<ServiceConfig> {
    let typesafe_key = std::env::var("TYPESAFE_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty());
    let openrouter_key = std::env::var("OPENROUTER_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty());

    match provider {
        Some(Provider::TypeSafe) => {
            let key = typesafe_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "System One service: TypeSafe provider selected but TYPESAFE_API_KEY is not set or is empty"
                )
            })?;
            Ok(ServiceConfig {
                provider: Provider::TypeSafe,
                endpoint_url: TYPESAFE_URL.to_string(),
                api_key: key,
                default_model: TYPESAFE_DEFAULT_MODEL.to_string(),
            })
        }
        Some(Provider::OpenRouter) => {
            let key = openrouter_key.ok_or_else(|| {
                anyhow::anyhow!(
                    "System One service: OpenRouter provider selected but OPENROUTER_API_KEY is not set or is empty"
                )
            })?;
            Ok(ServiceConfig {
                provider: Provider::OpenRouter,
                endpoint_url: OPENROUTER_URL.to_string(),
                api_key: key,
                default_model: OPENROUTER_DEFAULT_MODEL.to_string(),
            })
        }
        None => {
            if let Some(key) = openrouter_key {
                Ok(ServiceConfig {
                    provider: Provider::OpenRouter,
                    endpoint_url: OPENROUTER_URL.to_string(),
                    api_key: key,
                    default_model: OPENROUTER_DEFAULT_MODEL.to_string(),
                })
            } else if let Some(key) = typesafe_key {
                Ok(ServiceConfig {
                    provider: Provider::TypeSafe,
                    endpoint_url: TYPESAFE_URL.to_string(),
                    api_key: key,
                    default_model: TYPESAFE_DEFAULT_MODEL.to_string(),
                })
            } else {
                anyhow::bail!(
                    "System One service: no credential available. Set OPENROUTER_API_KEY or TYPESAFE_API_KEY."
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Global lock to serialize environment-variable tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_keys(typesafe: Option<&str>, openrouter: Option<&str>, test: impl FnOnce()) {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::remove_var("TYPESAFE_API_KEY");
            std::env::remove_var("OPENROUTER_API_KEY");
            if let Some(val) = typesafe {
                std::env::set_var("TYPESAFE_API_KEY", val);
            }
            if let Some(val) = openrouter {
                std::env::set_var("OPENROUTER_API_KEY", val);
            }
        }
        test();
    }

    // --- auto-detection (no explicit provider) ---

    #[test]
    fn auto_openrouter_preferred() {
        with_keys(None, Some("orkey"), || {
            let cfg = resolve_service(None).unwrap();
            assert_eq!(cfg.provider, Provider::OpenRouter);
            assert_eq!(cfg.endpoint_url, OPENROUTER_URL);
            assert_eq!(cfg.api_key, "orkey");
        });
    }

    #[test]
    fn auto_typesafe_fallback() {
        with_keys(Some("tfkey"), None, || {
            let cfg = resolve_service(None).unwrap();
            assert_eq!(cfg.provider, Provider::TypeSafe);
            assert_eq!(cfg.endpoint_url, TYPESAFE_URL);
            assert_eq!(cfg.api_key, "tfkey");
        });
    }

    #[test]
    fn auto_openrouter_wins_over_typesafe() {
        with_keys(Some("tfkey"), Some("orkey"), || {
            let cfg = resolve_service(None).unwrap();
            assert_eq!(cfg.provider, Provider::OpenRouter);
            assert_eq!(cfg.api_key, "orkey");
        });
    }

    #[test]
    fn auto_empty_keys_are_absent() {
        with_keys(None, Some("  "), || {
            let err = resolve_service(None).unwrap_err();
            assert!(err.to_string().contains("no credential"));
        });
    }

    #[test]
    fn auto_missing_both_keys() {
        with_keys(None, None, || {
            let err = resolve_service(None).unwrap_err();
            assert!(err.to_string().contains("OPENROUTER_API_KEY"));
            assert!(err.to_string().contains("TYPESAFE_API_KEY"));
        });
    }

    // --- explicit provider selection ---

    #[test]
    fn explicit_typesafe_uses_its_key_only() {
        with_keys(Some("tfkey"), Some("orkey"), || {
            let cfg = resolve_service(Some(Provider::TypeSafe)).unwrap();
            assert_eq!(cfg.provider, Provider::TypeSafe);
            assert_eq!(cfg.api_key, "tfkey");
            assert_eq!(cfg.endpoint_url, TYPESAFE_URL);
        });
    }

    #[test]
    fn explicit_openrouter_uses_its_key_only() {
        with_keys(Some("tfkey"), Some("orkey"), || {
            let cfg = resolve_service(Some(Provider::OpenRouter)).unwrap();
            assert_eq!(cfg.provider, Provider::OpenRouter);
            assert_eq!(cfg.api_key, "orkey");
            assert_eq!(cfg.endpoint_url, OPENROUTER_URL);
        });
    }

    #[test]
    fn explicit_typesafe_missing_key() {
        with_keys(None, Some("orkey"), || {
            let err = resolve_service(Some(Provider::TypeSafe)).unwrap_err();
            assert!(err.to_string().contains("TYPESAFE_API_KEY"));
        });
    }

    #[test]
    fn explicit_openrouter_missing_key() {
        with_keys(Some("tfkey"), None, || {
            let err = resolve_service(Some(Provider::OpenRouter)).unwrap_err();
            assert!(err.to_string().contains("OPENROUTER_API_KEY"));
        });
    }
}