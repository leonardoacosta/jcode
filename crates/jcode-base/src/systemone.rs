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

    let (provider, endpoint_url, key_env, env_file, default_model) = match selection_lower.as_str()
    {
        "9router" => {
            let profile = config.providers.get("9router").context(
                "System One: systemone_url is 9router but [providers.9router] is missing",
            )?;
            let endpoint = append_systemone_path(&profile.base_url)?;
            let key_env = profile.api_key_env.as_deref().ok_or_else(|| {
                anyhow::anyhow!("System One: [providers.9router] has no api_key_env")
            })?;
            let env_file = profile
                .env_file
                .as_deref()
                .unwrap_or("provider-9router.env");
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
    let parsed = url::Url::parse(endpoint_url)
        .context("System One: systemone_url must be a URL or provider name")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn with_config(contents: &str, test: impl FnOnce()) {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("JCODE_HOME", dir.path()) };
        std::fs::write(dir.path().join("config.toml"), contents).unwrap();
        unsafe { std::env::set_var("JCODE_PROVIDER_TEST_SYSTEMONE_KEY", "systemone-test-key") };
        crate::config::invalidate_config_cache();
        test();
        unsafe {
            std::env::remove_var("JCODE_PROVIDER_TEST_SYSTEMONE_KEY");
            std::env::remove_var("JCODE_HOME");
        }
        crate::config::invalidate_config_cache();
    }

    #[test]
    fn defaults_to_configured_ninerouter_profile_endpoint() {
        with_config(
            r#"[providers.9router]
type = "openai-compatible"
base_url = "https://router.example/v1"
api_key_env = "JCODE_PROVIDER_TEST_SYSTEMONE_KEY"
"#,
            || {
                let resolved = resolve().unwrap();
                assert_eq!(resolved.provider, SystemOneProvider::NineRouter);
                assert_eq!(resolved.endpoint_url, "https://router.example/v1/systemone");
                assert_eq!(resolved.default_model, NINEROUTER_MODEL);
                assert_eq!(resolved.api_key, "systemone-test-key");
            },
        );
    }

    #[test]
    fn requires_key_for_selected_direct_provider() {
        with_config("systemone_url = \"typesafe\"\n", || {
            let error = resolve().err().unwrap().to_string();
            assert!(error.contains("TYPESAFE_API_KEY"));
        });
    }

    #[test]
    fn rejects_unmapped_url() {
        let config = crate::config::Config::default();
        let error = resolve_full_url(&config, "https://unknown.example/systemone")
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("no credential mapping"));
    }

    #[test]
    fn accepts_url_and_rejects_unmapped_host() {
        let config = crate::config::Config::default();
        let result = resolve_full_url(&config, "https://openrouter.ai/api/alpha/decisions").unwrap();
        assert_eq!(result.0, SystemOneProvider::OpenRouter);
        assert_eq!(result.4, OPENROUTER_MODEL);
        assert!(resolve_full_url(&config, "https://unknown.example/systemone").is_err());
    }
}
