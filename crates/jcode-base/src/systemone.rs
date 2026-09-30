//! Shared System One endpoint and credential resolver.
//!
//! This is the only module that selects a System One route or reads its key.

use anyhow::{Context, Result};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/alpha/decisions";
const TYPESAFE_URL: &str = "https://api.typesafe.ai/v1/systemone";
const OPENROUTER_MODEL: &str = "~typesafe/jev-latest";
const TYPESAFE_MODEL: &str = "jev-latest";
const NINEROUTER_MODEL: &str = "openrouter/typesafe/jev-1.13";
const OMNI_MODEL: &str = "jev-latest";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemOneProvider {
    NineRouter,
    OpenRouter,
    TypeSafe,
    JcodeSubscription,
    Omni,
}

impl std::fmt::Display for SystemOneProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NineRouter => f.write_str("9router"),
            Self::OpenRouter => f.write_str("openrouter"),
            Self::TypeSafe => f.write_str("typesafe"),
            Self::JcodeSubscription => f.write_str("jcode-subscription"),
            Self::Omni => f.write_str("omni"),
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
/// `systemone_url` accepts `9router`, `omni`, `openrouter`, `typesafe`, or a full URL.
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
        "jcode" | "subscription" | "jcode-subscription" => {
            if crate::subscription_catalog::configured_api_key().is_none() {
                anyhow::bail!(
                    "System One: Jcode subscription route selected but no JCODE_API_KEY configured"
                );
            }
            (
                SystemOneProvider::JcodeSubscription,
                crate::subscription_catalog::configured_api_base()
                    .map(|base| {
                        let base = base.trim_end_matches('/');
                        if base.ends_with("/systemone") {
                            base.to_string()
                        } else {
                            format!("{base}/systemone")
                        }
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "{}/systemone",
                            crate::subscription_catalog::DEFAULT_JCODE_API_BASE
                        )
                    }),
                crate::subscription_catalog::JCODE_API_KEY_ENV.to_string(),
                crate::subscription_catalog::JCODE_ENV_FILE.to_string(),
                TYPESAFE_MODEL,
            )
        }
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
        "omni" => {
            let profile = config
                .providers
                .get("omni")
                .context("System One: systemone_url is omni but [providers.omni] is missing")?;
            let endpoint = append_systemone_path(&profile.base_url)?;
            let key_env = profile.api_key_env.as_deref().ok_or_else(|| {
                anyhow::anyhow!("System One: [providers.omni] has no api_key_env")
            })?;
            let env_file = profile.env_file.as_deref().unwrap_or("provider-omni.env");
            (
                SystemOneProvider::Omni,
                endpoint,
                key_env.to_string(),
                env_file.to_string(),
                OMNI_MODEL,
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

    for (name, provider, default_model) in [
        ("9router", SystemOneProvider::NineRouter, NINEROUTER_MODEL),
        ("omni", SystemOneProvider::Omni, OMNI_MODEL),
    ] {
        if let Some(profile) = config.providers.get(name)
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
                    anyhow::anyhow!("System One: provider profile [{name}] has no api_key_env")
                })?;
                return Ok((
                    provider,
                    endpoint_url.to_string(),
                    key_env.to_string(),
                    profile
                        .env_file
                        .clone()
                        .unwrap_or_else(|| format!("provider-{name}.env")),
                    default_model,
                ));
            }
        }
    }

    anyhow::bail!("System One: no credential mapping for configured URL host '{host}'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn omni_full_url_uses_exact_matching_profile_credentials() {
        with_config(
            r#"systemone_url = "https://omni.example.test/v1/systemone/typesafe"

[providers.omni]
base_url = "https://omni.example.test/v1"
api_key_env = "JCODE_PROVIDER_OMNI_API_KEY"
env_file = "provider-omni.env"
"#,
            || {
                crate::env::set_var("JCODE_PROVIDER_OMNI_API_KEY", "omni-test-key");
                let resolved = resolve().unwrap();
                assert_eq!(resolved.provider, SystemOneProvider::Omni);
                assert_eq!(
                    resolved.endpoint_url,
                    "https://omni.example.test/v1/systemone/typesafe"
                );
                assert_eq!(resolved.api_key, "omni-test-key");
                assert_eq!(resolved.default_model, "jev-latest");
            },
        );
    }

    #[test]
    fn omni_full_url_rejects_different_origin() {
        with_config(
            r#"systemone_url = "https://other.example.test/v1/systemone/typesafe"

[providers.omni]
base_url = "https://omni.example.test/v1"
api_key_env = "JCODE_PROVIDER_OMNI_API_KEY"
"#,
            || {
                let error = resolve().err().unwrap().to_string();
                assert!(
                    error.contains("no credential mapping"),
                    "unexpected error: {error}"
                );
            },
        );
    }

    #[test]
    fn omni_full_url_rejects_sibling_path_and_effective_port_mismatch() {
        for endpoint in [
            "https://omni.example.test/v10/systemone/typesafe",
            "https://omni.example.test:444/v1/systemone/typesafe",
            "http://omni.example.test/v1/systemone/typesafe",
            "https://omni.example.test.evil.test/v1/systemone/typesafe",
        ] {
            with_config(
                &format!(
                    "systemone_url = \"{endpoint}\"\n\n[providers.omni]\nbase_url = \"https://omni.example.test/v1\"\napi_key_env = \"JCODE_PROVIDER_OMNI_API_KEY\"\n"
                ),
                || {
                    let error = resolve().err().unwrap().to_string();
                    assert!(
                        error.contains("no credential mapping"),
                        "unexpected error: {error}"
                    );
                },
            );
        }
    }

    #[test]
    fn omni_full_url_accepts_trailing_slash_and_equivalent_default_port() {
        with_config(
            "systemone_url = \"https://omni.example.test:443/v1/\"\n\n[providers.omni]\nbase_url = \"https://omni.example.test/v1\"\napi_key_env = \"JCODE_PROVIDER_OMNI_API_KEY\"\n",
            || {
                crate::env::set_var("JCODE_PROVIDER_OMNI_API_KEY", "omni-test-key");
                assert_eq!(resolve().unwrap().api_key, "omni-test-key");
            },
        );
    }

    fn with_config(contents: &str, test: impl FnOnce()) {
        let _guard = crate::storage::lock_test_env();
        let previous_home = std::env::var_os("JCODE_HOME");
        let omni_key = std::env::var_os("JCODE_PROVIDER_OMNI_API_KEY");
        struct RestoreEnv(Option<std::ffi::OsString>, Option<std::ffi::OsString>);
        impl Drop for RestoreEnv {
            fn drop(&mut self) {
                match self.0.take() {
                    Some(value) => crate::env::set_var("JCODE_HOME", value),
                    None => crate::env::remove_var("JCODE_HOME"),
                }
                match self.1.take() {
                    Some(value) => crate::env::set_var("JCODE_PROVIDER_OMNI_API_KEY", value),
                    None => crate::env::remove_var("JCODE_PROVIDER_OMNI_API_KEY"),
                }
                crate::env::remove_var("JCODE_PROVIDER_TEST_SYSTEMONE_KEY");
                crate::config::invalidate_config_cache();
            }
        }
        let _restore = RestoreEnv(previous_home, omni_key);
        let dir = tempfile::tempdir().unwrap();
        crate::env::set_var("JCODE_HOME", dir.path());
        std::fs::write(dir.path().join("config.toml"), contents).unwrap();
        unsafe { std::env::set_var("JCODE_PROVIDER_TEST_SYSTEMONE_KEY", "systemone-test-key") };
        crate::config::invalidate_config_cache();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        crate::config::invalidate_config_cache();
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
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
        let result =
            resolve_full_url(&config, "https://openrouter.ai/api/alpha/decisions").unwrap();
        assert_eq!(result.0, SystemOneProvider::OpenRouter);
        assert_eq!(result.4, OPENROUTER_MODEL);
        assert!(resolve_full_url(&config, "https://unknown.example/systemone").is_err());
    }
}
