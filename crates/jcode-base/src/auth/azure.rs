use anyhow::Result;

use crate::provider_catalog::{
    load_api_key_from_env_or_config, load_env_value_from_env_or_config, normalize_api_base,
};

pub const ENV_FILE: &str = "azure-openai.env";
pub const ENDPOINT_ENV: &str = "AZURE_OPENAI_ENDPOINT";
pub const API_KEY_ENV: &str = "AZURE_OPENAI_API_KEY";
pub const MODEL_ENV: &str = "AZURE_OPENAI_MODEL";
pub const MODELS_ENV: &str = "AZURE_OPENAI_MODELS";
pub const USE_ENTRA_ENV: &str = "AZURE_OPENAI_USE_ENTRA";
pub const COGNITIVE_SCOPE: &str = "https://cognitiveservices.azure.com/.default";

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

pub fn normalize_endpoint(raw: &str) -> Option<String> {
    let mut endpoint = normalize_api_base(raw)?;
    if endpoint.ends_with("/openai/v1") {
        return Some(endpoint);
    }
    endpoint.push_str("/openai/v1");
    Some(endpoint)
}

pub fn load_endpoint() -> Option<String> {
    let raw = load_env_value_from_env_or_config(ENDPOINT_ENV, ENV_FILE)?;
    normalize_endpoint(&raw)
}

pub fn load_model() -> Option<String> {
    load_env_value_from_env_or_config(MODEL_ENV, ENV_FILE)
}

fn parse_model_list(raw: &str) -> Vec<String> {
    let mut models = Vec::new();
    for model in raw.split([',', ';', '\n', '\r']) {
        let model = model.trim();
        if !model.is_empty() && !models.iter().any(|existing| existing == model) {
            models.push(model.to_string());
        }
    }
    models
}

pub fn load_models() -> Vec<String> {
    let mut models = load_env_value_from_env_or_config(MODELS_ENV, ENV_FILE)
        .map(|raw| parse_model_list(&raw))
        .unwrap_or_default();

    if let Some(model) = load_model()
        && !model.trim().is_empty()
        && !models.iter().any(|existing| existing == model.trim())
    {
        models.insert(0, model.trim().to_string());
    }

    models
}

pub fn has_api_key() -> bool {
    load_api_key_from_env_or_config(API_KEY_ENV, ENV_FILE).is_some()
}

pub fn uses_entra_id() -> bool {
    load_env_value_from_env_or_config(USE_ENTRA_ENV, ENV_FILE)
        .and_then(|value| parse_bool(&value))
        .unwrap_or(false)
}

pub fn has_configuration() -> bool {
    load_endpoint().is_some() && (has_api_key() || uses_entra_id())
}

pub fn method_detail() -> String {
    let mut parts = Vec::new();
    if has_api_key() {
        parts.push(format!("API key (`{API_KEY_ENV}`)"));
    }
    if uses_entra_id() {
        parts.push("Microsoft Entra ID (DefaultAzureCredential)".to_string());
    }
    if parts.is_empty() {
        "not configured".to_string()
    } else {
        parts.join(" + ")
    }
}

pub fn apply_runtime_env() -> Result<()> {
    let endpoint = load_endpoint().ok_or_else(|| {
        anyhow::anyhow!(
            "{} not found in environment or ~/.config/jcode/{}",
            ENDPOINT_ENV,
            ENV_FILE
        )
    })?;

    crate::env::set_var("JCODE_OPENROUTER_API_BASE", endpoint);
    crate::env::set_var("JCODE_OPENROUTER_API_KEY_NAME", API_KEY_ENV);
    crate::env::set_var("JCODE_OPENROUTER_ENV_FILE", ENV_FILE);
    crate::env::set_var("JCODE_OPENROUTER_CACHE_NAMESPACE", "azure-openai");
    crate::env::set_var("JCODE_OPENROUTER_PROVIDER_FEATURES", "0");
    crate::env::set_var("JCODE_OPENROUTER_TRANSPORT_STATE", "direct-api-key");
    crate::env::set_var("JCODE_OPENROUTER_MODEL_CATALOG", "0");
    let models = load_models();
    if models.is_empty() {
        crate::env::remove_var("JCODE_OPENROUTER_STATIC_MODELS");
    } else {
        crate::env::set_var("JCODE_OPENROUTER_STATIC_MODELS", models.join("\n"));
    }

    if uses_entra_id() {
        crate::env::set_var("JCODE_OPENROUTER_AUTH_HEADER", "authorization-bearer");
        crate::env::set_var("JCODE_OPENROUTER_DYNAMIC_BEARER_PROVIDER", "azure");
    } else {
        crate::env::set_var("JCODE_OPENROUTER_AUTH_HEADER", "api-key");
        crate::env::remove_var("JCODE_OPENROUTER_DYNAMIC_BEARER_PROVIDER");
    }

    Ok(())
}

pub async fn get_bearer_token() -> Result<String> {
    jcode_azure_auth::get_bearer_token(COGNITIVE_SCOPE).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvGuard {
        keys: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn new(keys: &[&'static str]) -> Self {
            let keys = keys
                .iter()
                .map(|key| {
                    let saved = std::env::var(key).ok();
                    crate::env::remove_var(key);
                    (*key, saved)
                })
                .collect();
            Self { keys }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.keys.drain(..) {
                if let Some(value) = value {
                    crate::env::set_var(key, value);
                } else {
                    crate::env::remove_var(key);
                }
            }
        }
    }

    #[test]
    fn normalize_endpoint_appends_openai_v1() {
        assert_eq!(
            normalize_endpoint("https://example.openai.azure.com"),
            Some("https://example.openai.azure.com/openai/v1".to_string())
        );
    }

    #[test]
    fn normalize_endpoint_preserves_existing_openai_v1() {
        assert_eq!(
            normalize_endpoint("https://example.openai.azure.com/openai/v1/"),
            Some("https://example.openai.azure.com/openai/v1".to_string())
        );
    }

    #[test]
    fn normalize_endpoint_rejects_insecure_remote_http() {
        assert_eq!(normalize_endpoint("http://example.com"), None);
    }

    #[test]
    fn parse_model_list_dedupes_common_separators() {
        assert_eq!(
            parse_model_list("gpt-5.5, claude-sonnet-5\nFW-Kimi-K3;gpt-5.5"),
            vec!["gpt-5.5", "claude-sonnet-5", "FW-Kimi-K3"]
        );
    }

    #[test]
    fn apply_runtime_env_exports_static_deployment_list() {
        let _lock = crate::storage::lock_test_env();
        let _guard = EnvGuard::new(&[
            ENDPOINT_ENV,
            MODEL_ENV,
            MODELS_ENV,
            API_KEY_ENV,
            USE_ENTRA_ENV,
            "JCODE_OPENROUTER_API_BASE",
            "JCODE_OPENROUTER_API_KEY_NAME",
            "JCODE_OPENROUTER_ENV_FILE",
            "JCODE_OPENROUTER_CACHE_NAMESPACE",
            "JCODE_OPENROUTER_PROVIDER_FEATURES",
            "JCODE_OPENROUTER_TRANSPORT_STATE",
            "JCODE_OPENROUTER_MODEL_CATALOG",
            "JCODE_OPENROUTER_STATIC_MODELS",
            "JCODE_OPENROUTER_AUTH_HEADER",
            "JCODE_OPENROUTER_DYNAMIC_BEARER_PROVIDER",
        ]);

        crate::env::set_var(ENDPOINT_ENV, "https://example.openai.azure.com");
        crate::env::set_var(MODEL_ENV, "FW-Kimi-K3");
        crate::env::set_var(MODELS_ENV, "gpt-5.5,claude-sonnet-5,FW-Kimi-K3");
        crate::env::set_var(API_KEY_ENV, "test-key");
        crate::env::set_var(USE_ENTRA_ENV, "0");

        apply_runtime_env().expect("apply Azure runtime env");

        assert_eq!(
            std::env::var("JCODE_OPENROUTER_STATIC_MODELS").as_deref(),
            Ok("gpt-5.5\nclaude-sonnet-5\nFW-Kimi-K3")
        );
    }
}
