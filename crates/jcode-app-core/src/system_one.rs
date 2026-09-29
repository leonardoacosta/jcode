//! Adapts the shared JCode route config to the System One HTTP client.

use anyhow::Result;
use jcode_system_one::{Provider, ServiceConfig};

pub fn service_config() -> Result<ServiceConfig> {
    let resolved = crate::systemone::resolve()?;
    let provider = match resolved.provider {
        crate::systemone::SystemOneProvider::NineRouter => Provider::NineRouter,
        crate::systemone::SystemOneProvider::Omni => Provider::Omni,
        crate::systemone::SystemOneProvider::TypeSafe => Provider::TypeSafe,
        crate::systemone::SystemOneProvider::OpenRouter => Provider::OpenRouter,
    };
    Ok(ServiceConfig {
        provider,
        endpoint_url: resolved.endpoint_url,
        api_key: resolved.api_key,
        default_model: resolved.default_model,
    })
}
