use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub enabled: bool,
    pub socket_path: PathBuf,
    pub port: u16,
    pub tailnet_origin: Option<String>,
    pub control_token: String,
    pub working_dir: PathBuf,
    pub provider: Option<String>,
    pub model: Option<String>,
}

impl Config {
    pub fn new(working_dir: PathBuf, socket_path: PathBuf, port: u16) -> Self {
        Self {
            version: 1,
            enabled: true,
            socket_path,
            port,
            tailnet_origin: None,
            control_token: format!(
                "{}{}",
                uuid::Uuid::new_v4().simple(),
                uuid::Uuid::new_v4().simple()
            ),
            working_dir,
            provider: None,
            model: None,
        }
    }

    pub fn local_origin(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1,
            "unsupported automation configuration version"
        );
        ensure!(self.port != 0, "bulletin port must be nonzero");
        ensure!(
            self.socket_path.is_absolute(),
            "automation socket must be absolute"
        );
        ensure!(
            self.working_dir.is_absolute() && self.working_dir.is_dir(),
            "automation working directory must exist and be absolute"
        );
        ensure!(
            self.control_token.len() == 64
                && self.control_token.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid bulletin control token"
        );
        if let Some(origin) = &self.tailnet_origin {
            let url = url::Url::parse(origin).context("invalid tailnet origin")?;
            ensure!(
                url.scheme() == "https"
                    && url.host_str().is_some_and(|host| host.ends_with(".ts.net"))
                    && url.username().is_empty()
                    && url.password().is_none()
                    && url.path() == "/"
                    && url.query().is_none()
                    && url.fragment().is_none()
                    && url.origin().ascii_serialization() == *origin,
                "tailnet origin must be a canonical HTTPS .ts.net origin without credentials, path or query"
            );
        }
        Ok(())
    }
}

pub fn directory() -> Result<PathBuf> {
    Ok(crate::storage::jcode_dir()?.join("automations"))
}

pub fn load(directory: &Path) -> Result<Option<Config>> {
    let path = directory.join("config.json");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("read automation configuration"),
    };
    let config: Config =
        serde_json::from_slice(&bytes).context("invalid automation configuration")?;
    config.validate()?;
    Ok(Some(config))
}

pub fn save(directory: &Path, config: &Config) -> Result<()> {
    config.validate()?;
    crate::storage::write_json_secret(&directory.join("config.json"), config)
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod tests;
