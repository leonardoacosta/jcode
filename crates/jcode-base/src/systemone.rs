//! Shared System One endpoint and credential resolver.
//!
//! This is the only module that selects a System One route or reads its key.

use anyhow::{Context, Result};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/alpha/decisions";
const TYPESAFE_URL: &str = "https://api.typesafe.ai/v1/systemone";
const OPENROUTER_MODEL: &str = "~typesafe/jev-latest";
const TYPESAFE_MODEL: &str = "jev-latest";
const NINEROUTER_MODEL: &str = "openrouter/typesafe/jev-1.13";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemOneProvider {
    NineRouter,
    OpenRouter,
    TypeSafe,
}

impl std::fmt::Display for SystemOneProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NineRouter => f.write_str("9router"),
            Self::OpenRouter => f.write_str("openrouter"),
            Self::TypeSafe => f.write_str("typesafe"),
        }
    }
}

#[derive(Clone)]
pub struct ResolvedSystemOneConfig {
    pub provider: SystemOneProvider,
    pub endpoint_url: String,
    pub api_key: String,
    pub default_model: String,
}

/// Resolve the selected System One route from the loaded config.toml.
///
/// `systemone_url` accepts `9router`, `openrouter`, `typesafe`, or a full URL.
/// For a full URL, the matching configured provider profile supplies the key.
pub fn resolve() -> Result<ResolvedSystemOneConfig> {
    let config = crate::config::config();
    let selection = if config.systemone_url.trim().is_empty() {
        "9router"
    } else {
        config.systemone_url.trim()
    };
    let selection_lower = selection.to_ascii_lowercase();
    let model_override = config
        .systemone_model
        .as_deref()
        .map(str::trim)
        .filter(|model| !model.is_empty());

    let (provider, endpoint_url, key_env, env_file, default_model) =
        match selection_lower.as_str() {
            "9router" => {
                let profile = config
                    .providers
                    .get("9router")
                    .context("System One: systemone_url is 9router but [providers.9router] is missing")?;
                let endpoint = append_systemone_path(&profile.base_url)?;
                let key_env = profile.api_key_env.as_deref().ok_or_else(|| {
                    anyhow::anyhow!("System One: [providers.9router] has no api_key_env")
                })?;
                let env_file = profile.env_file.as_deref().unwrap_or("provider-9router.env");
                (
                    SystemOneProvider::NineRouter,
                    endpoint,
                    key_env.to_string(),
                    env_file.to_string(),
                    NINEROUTER_MODEL,
                )
            }
            "openrouter" => (
                SystemOneProvider::OpenRouter,
                OPENROUTER_URL.to_string(),
                "OPENROUTER_API_KEY".to_string(),
                "openrouter.env".to_string(),
                OPENROUTER_MODEL,
            ),
            "typesafe" => (
                SystemOneProvider::TypeSafe,
                TYPESAFE_URL.to_string(),
                "TYPESAFE_API_KEY".to_string(),
                "typesafe.env".to_string(),
                TYPESAFE_MODEL,
            ),
            _ => resolve_full_url(config, selection)?,
        };

    let api_key = crate::provider_catalog::load_api_key_from_env_or_config(&key_env, &env_file)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "System One: no credential for {} route; set {} or add it to {}",
                provider,
                key_env,
                env_file
            )
        })?;

    Ok(ResolvedSystemOneConfig {
        provider,
        endpoint_url,
        api_key,
        default_model: model_override.unwrap_or(default_model).to_string(),
    })
}

fn append_systemone_path(base_url: &str) -> Result<String> {
    let base = base_url.trim().trim_end_matches('/');
    let parsed = url::Url::parse(base).context("System One: invalid 9Router base_url")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("System One: 9Router base_url must use HTTP or HTTPS");
    }
    Ok(format!("{base}/systemone"))
}

fn resolve_full_url(
    config: &crate::config::Config,
    endpoint_url: &str,
) -> Result<(SystemOneProvider, String, String, String, &'static str)> {
    let parsed = url::Url::parse(endpoint_url).context("System One: systemone_url must be a URL or provider name")?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("System One: systemone_url must use HTTP or HTTPS");
    }
    let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();
    if host == "openrouter.ai" {
        return Ok((
            SystemOneProvider::OpenRouter,
            endpoint_url.to_string(),
            "OPENROUTER_API_KEY".to_string(),
            "openrouter.env".to_string(),
            OPENROUTER_MODEL,
        ));
    }
    if host == "api.typesafe.ai" {
        return Ok((
            SystemOneProvider::TypeSafe,
            endpoint_url.to_string(),
            "TYPESAFE_API_KEY".to_string(),
            "typesafe.env".to_string(),
            TYPESAFE_MODEL,
        ));
    }

    if let Some(profile) = config.providers.get("9router")
        && let Ok(base_url) = url::Url::parse(profile.base_url.trim())
        && parsed.origin() == base_url.origin()
    {
        let base_path = base_url.path().trim_end_matches('/');
        let endpoint_path = parsed.path();
        let same_api_root = endpoint_path == base_path
            || endpoint_path
                .strip_prefix(base_path)
                .is_some_and(|suffix| suffix.starts_with('/'));
        if !base_path.is_empty() && same_api_root {
            let key_env = profile.api_key_env.as_deref().ok_or_else(|| {
                anyhow::anyhow!("System One: provider profile [9router] has no api_key_env")
            })?;
            return Ok((
                SystemOneProvider::NineRouter,
                endpoint_url.to_string(),
                key_env.to_string(),
                profile
                    .env_file
                    .clone()
                    .unwrap_or_else(|| "provider-9router.env".to_string()),
                NINEROUTER_MODEL,
            ));
        }
    }

    anyhow::bail!("System One: no credential mapping for configured URL host '{host}'")
}
