use super::{BrowserInput, BrowserProvider, ToolContext, ToolOutput, attach_browser_metadata};
use anyhow::{Context, Result};
use async_trait::async_trait;
use jcode_mac_browser_fleet::{FleetEnvelope, WireAction};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};

pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

pub struct MacFleetProvider;

#[derive(Clone, Debug)]
pub struct MacFleetConfig {
    pub socket_path: PathBuf,
    pub secret: String,
}

impl MacFleetConfig {
    pub fn from_env() -> Result<Self> {
        let home = std::env::var_os("JCODE_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|path| path.join(".jcode")))
            .context("JCODE_HOME or a home directory is required for the Mac browser fleet")?;
        let socket_path = std::env::var_os("JCODE_MAC_BROWSER_FLEET_SOCKET")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("browser/mac-fleet.sock"));
        let secret_file = home.join("browser/mac-fleet.secret");
        let secret = match std::env::var("JCODE_MAC_BROWSER_FLEET_SECRET") {
            Ok(value) if !value.trim().is_empty() => value,
            _ => std::fs::read_to_string(&secret_file)
                .unwrap_or_default()
                .trim()
                .to_string(),
        };
        Ok(Self {
            socket_path,
            secret,
        })
    }
}

pub fn build_request(
    id: &str,
    secret: String,
    action: &str,
    input: &BrowserInput,
) -> Result<FleetEnvelope> {
    let wire_action = match action {
        "status" | "list_tabs" | "get_active_tab" => WireAction::ListBrowsers,
        "open" | "new_tab" => WireAction::Navigate,
        "type" | "fill_form" | "select" => WireAction::Type,
        "press" => WireAction::Press,
        "click" | "upload" | "scroll" => WireAction::Click,
        other => anyhow::bail!("Mac browser fleet does not support action '{other}' yet"),
    };
    let generation = input.generation.unwrap_or(0);
    let target = json!({
        "browser_id": input.browser_ref,
        "window_id": input.window_ref,
        "tab_id": input.tab_ref,
        "generation": generation,
    });
    Ok(FleetEnvelope {
        version: 1,
        auth: secret,
        id: id.to_string(),
        deadline_ms: input.timeout_ms.unwrap_or(30_000).clamp(1, 120_000),
        target_generation: generation,
        action: wire_action,
        payload: json!({
            "target": target,
            "url": input.url,
            "selector": input.selector,
            "text": input.text,
            "key": input.key,
            "x": input.x,
            "y": input.y,
        }),
    })
}

pub fn encode_request_line(request: &FleetEnvelope) -> Result<Vec<u8>> {
    let mut encoded = serde_json::to_vec(request)?;
    if encoded.len() > MAX_REQUEST_BYTES {
        anyhow::bail!("Mac browser fleet request exceeded size limit");
    }
    encoded.push(b'\n');
    Ok(encoded)
}

pub fn tool_error_from_wire(value: Value) -> Result<Value> {
    if value.get("ok").and_then(Value::as_bool) != Some(false) {
        return Ok(value);
    }
    let kind = value
        .pointer("/error/kind")
        .and_then(Value::as_str)
        .unwrap_or("providerError");
    match kind {
        "approvalRequired" => anyhow::bail!("Mac browser fleet approval required on the Mac"),
        "staleGeneration" => anyhow::bail!("Mac browser fleet stale generation; refresh inventory"),
        "hardDenied" => anyhow::bail!("Mac browser fleet action is blocked by Mac policy"),
        "emergencyStop" => anyhow::bail!("Mac browser fleet emergency stop is active"),
        _ => anyhow::bail!("Mac browser fleet request failed"),
    }
}

pub fn normalize_fleet_result(action: &str, value: Value) -> Value {
    let value = unwrap_fleet_result(value);
    match action {
        "list_tabs" | "new_tab" => normalize_targets(value),
        "get_active_tab" => normalize_active_target(value),
        _ => value,
    }
}

fn unwrap_fleet_result(mut value: Value) -> Value {
    if value.get("ok").and_then(Value::as_bool) == Some(true)
        && let Some(result) = value.get_mut("result")
    {
        return result.take();
    }
    value
}

fn normalize_targets(value: Value) -> Value {
    let targets = value
        .get("targets")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| match value {
            Value::Array(items) => items,
            _ => Vec::new(),
        });

    Value::Array(
        targets
            .into_iter()
            .filter_map(|target| {
                let browser_ref = target
                    .get("browser_id")
                    .or_else(|| target.get("browserId"))?;
                let window_ref = target.get("window_id").or_else(|| target.get("windowId"))?;
                let tab_ref = target.get("tab_id").or_else(|| target.get("tabId"))?;
                let generation = target.get("generation").cloned().unwrap_or(json!(0));
                Some(json!({
                    "id": tab_ref,
                    "browser": "mac",
                    "browser_ref": browser_ref,
                    "window_ref": window_ref,
                    "tab_ref": tab_ref,
                    "generation": generation,
                }))
            })
            .collect(),
    )
}

fn normalize_active_target(value: Value) -> Value {
    let targets = normalize_targets(value);
    targets
        .as_array()
        .and_then(|items| items.first())
        .cloned()
        .unwrap_or(targets)
}

async fn request(action: &str, input: &BrowserInput) -> Result<Value> {
    let config = MacFleetConfig::from_env()?;
    if config.secret.is_empty() {
        anyhow::bail!("Mac browser fleet peer secret is not configured; run setup on the Mac");
    }
    let id = format!(
        "jcode-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let request = build_request(&id, config.secret, action, input)?;
    let encoded = encode_request_line(&request)?;
    let deadline = Duration::from_millis(request.deadline_ms);
    timeout(deadline, async {
        let mut stream = UnixStream::connect(&config.socket_path)
            .await
            .with_context(|| {
                format!(
                    "Mac browser fleet socket is unavailable at {}",
                    config.socket_path.display()
                )
            })?;
        stream.write_all(&encoded).await?;
        stream.flush().await?;
        let reader = BufReader::new(stream);
        let mut response = Vec::new();
        reader
            .take(MAX_RESPONSE_BYTES as u64)
            .read_until(b'\n', &mut response)
            .await?;
        if response.is_empty() || response.len() >= MAX_RESPONSE_BYTES {
            anyhow::bail!("Mac browser fleet returned missing or oversized output");
        }
        let value: Value = serde_json::from_slice(&response)
            .context("Mac browser fleet returned malformed JSON")?;
        tool_error_from_wire(value).map(|value| normalize_fleet_result(action, value))
    })
    .await
    .context("Mac browser fleet request timed out")?
}

#[async_trait]
impl BrowserProvider for MacFleetProvider {
    fn id(&self) -> &'static str {
        "mac_browser_fleet"
    }
    fn supported_browsers(&self) -> &'static [&'static str] {
        &["mac"]
    }

    async fn status(&self, _ctx: &ToolContext) -> Result<ToolOutput> {
        let config = MacFleetConfig::from_env()?;
        let ready = !config.secret.is_empty() && config.socket_path.exists();
        Ok(attach_browser_metadata(
            ToolOutput::new(if ready { "Mac browser fleet is ready." } else { "Mac browser fleet is not ready. Run setup on the Mac and connect the reverse SSH Unix-socket forward." })
                .with_title("browser status")
                .with_metadata(json!({"ready": ready, "socket": config.socket_path.file_name().and_then(|v| v.to_str())})),
            self.id(),
            "mac",
        ))
    }

    async fn setup(&self) -> Result<ToolOutput> {
        Ok(attach_browser_metadata(
            ToolOutput::new(
                "Install the Mac broker with jcode-mac-browser-setup, enable the Chrome or Edge extension, and reconnect the reverse SSH Unix-socket forward.",
            ),
            self.id(),
            "mac",
        ))
    }

    async fn ensure_ready(&self) -> Result<Option<String>> {
        let config = MacFleetConfig::from_env()?;
        if config.secret.is_empty() || !config.socket_path.exists() {
            anyhow::bail!(
                "Mac browser fleet is not connected; run browser status with browser='mac'"
            );
        }
        Ok(None)
    }

    async fn execute(
        &self,
        action: &str,
        input: &BrowserInput,
        _ctx: &ToolContext,
    ) -> Result<ToolOutput> {
        let value = request(action, input).await?;
        Ok(attach_browser_metadata(
            ToolOutput::new(serde_json::to_string_pretty(&value)?)
                .with_title(format!("browser {action}"))
                .with_metadata(json!({"result": value})),
            self.id(),
            "mac",
        ))
    }
}
