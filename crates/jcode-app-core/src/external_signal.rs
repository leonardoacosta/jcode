//! Durable private-network ingress for provider-neutral external signals.
//!
//! Admission is intentionally unauthenticated. Its security boundary is the
//! explicitly configured loopback/private bind address and private routing.

use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result, bail};
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::ambient::{AmbientManager, Priority, ScheduleRequest, ScheduleTarget};

pub const INGRESS_PATH: &str = "/v1/external-signals/grafana";
pub const READY_PATH: &str = "/readyz";
pub const ADAPTER_VERSION: &str = "grafana/v1";
pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_MAX_BODY_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone)]
pub struct ExternalSignalConfig {
    pub enabled: bool,
    pub bind_addr: SocketAddr,
    pub source_id: String,
    pub projects: BTreeMap<String, PathBuf>,
    pub max_body_bytes: usize,
    pub wakes_enabled: bool,
}

impl ExternalSignalConfig {
    pub fn from_env() -> Result<Self> {
        let enabled = env_flag("JCODE_EXTERNAL_SIGNAL_ENABLED");
        let bind_addr = std::env::var("JCODE_EXTERNAL_SIGNAL_BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:0".to_string())
            .parse()
            .context("invalid JCODE_EXTERNAL_SIGNAL_BIND_ADDR")?;
        let source_id = std::env::var("JCODE_EXTERNAL_SIGNAL_SOURCE_ID")
            .unwrap_or_else(|_| "grafana-homelab".to_string());
        let projects =
            parse_projects(&std::env::var("JCODE_EXTERNAL_SIGNAL_PROJECTS").unwrap_or_default())?;
        let max_body_bytes = std::env::var("JCODE_EXTERNAL_SIGNAL_MAX_BODY_BYTES")
            .ok()
            .map(|value| value.parse())
            .transpose()
            .context("invalid JCODE_EXTERNAL_SIGNAL_MAX_BODY_BYTES")?
            .unwrap_or(DEFAULT_MAX_BODY_BYTES);
        let config = Self {
            enabled,
            bind_addr,
            source_id,
            projects,
            max_body_bytes,
            wakes_enabled: env_flag("JCODE_EXTERNAL_SIGNAL_WAKES_ENABLED"),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        if !is_provably_private(self.bind_addr.ip()) {
            bail!(
                "external signal ingress bind must be loopback, RFC1918, or Tailscale CGNAT (100.64.0.0/10)"
            );
        }
        if self.source_id.trim().is_empty() || self.source_id.len() > 128 {
            bail!("external signal source ID must contain 1..=128 bytes");
        }
        if self.projects.is_empty() {
            bail!("external signal ingress requires an explicit project registry");
        }
        if self.max_body_bytes == 0 || self.max_body_bytes > 1024 * 1024 {
            bail!("external signal body limit must be within 1..=1048576 bytes");
        }
        Ok(())
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
}

fn parse_projects(value: &str) -> Result<BTreeMap<String, PathBuf>> {
    let mut projects = BTreeMap::new();
    for mapping in value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        let (key, path) = mapping
            .split_once('=')
            .context("project mappings must use key=/absolute/path")?;
        if key.is_empty() || key.len() > 128 || !Path::new(path).is_absolute() {
            bail!("project mappings require a bounded key and absolute path");
        }
        if projects
            .insert(key.to_string(), PathBuf::from(path))
            .is_some()
        {
            bail!("duplicate external signal project key: {key}");
        }
    }
    Ok(projects)
}

fn is_provably_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => ip.is_loopback() || ip.is_private() || is_tailscale_cgnat(ip),
        IpAddr::V6(ip) => ip.is_loopback(),
    }
}

fn is_tailscale_cgnat(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (64..=127).contains(&octets[1])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrafanaWebhook {
    version: String,
    group_key: String,
    status: String,
    receiver: String,
    #[serde(default)]
    group_labels: BTreeMap<String, String>,
    #[serde(default)]
    common_labels: BTreeMap<String, String>,
    #[serde(default)]
    common_annotations: BTreeMap<String, String>,
    alerts: Vec<GrafanaAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrafanaAlert {
    status: String,
    labels: BTreeMap<String, String>,
    #[serde(default)]
    annotations: BTreeMap<String, String>,
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    #[serde(default)]
    #[serde(rename = "generatorURL")]
    generator_url: String,
    fingerprint: String,
    #[serde(default)]
    #[serde(rename = "silenceURL")]
    silence_url: String,
    #[serde(default)]
    #[serde(rename = "dashboardURL")]
    dashboard_url: String,
    #[serde(default)]
    #[serde(rename = "panelURL")]
    panel_url: String,
    #[serde(default)]
    values: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    value_string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEnvelope {
    pub schema_version: u32,
    pub receipt_id: String,
    pub delivery_key: String,
    pub source_id: String,
    pub adapter_version: String,
    pub project_key: String,
    pub received_at: DateTime<Utc>,
    pub content_sha256: String,
    pub raw_json: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Firing,
    Resolved,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SignalSeverity {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSignal {
    pub schema_version: u32,
    pub signal_id: String,
    pub receipt_id: String,
    pub project_key: String,
    pub working_dir: PathBuf,
    pub source_id: String,
    pub provider_event_id: String,
    pub fingerprint: String,
    pub lifecycle: LifecycleState,
    pub severity: SignalSeverity,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleAggregate {
    pub lifecycle_key: String,
    pub project_key: String,
    pub source_id: String,
    pub fingerprint: String,
    pub state: LifecycleState,
    pub severity: SignalSeverity,
    pub generation: u64,
    pub occurrence_count: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub last_transition_at: DateTime<Utc>,
    pub title: String,
    pub attention_id: Option<String>,
    pub scheduled_item_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionEvidence {
    pub attention_id: String,
    pub lifecycle_key: String,
    pub project_key: String,
    pub kind: String,
    pub priority: String,
    pub summary: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingStage {
    Pending,
    Projected,
    DeadLetter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRecord {
    pub receipt_id: String,
    pub stage: ProcessingStage,
    pub attempts: u32,
    pub next_attempt_at: DateTime<Utc>,
    pub last_error: Option<String>,
    pub failure_stage: Option<String>,
    #[serde(default)]
    pub terminal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterRecord {
    pub receipt_id: String,
    pub failed_at: DateTime<Utc>,
    pub attempts: u32,
    pub failure_stage: String,
    pub error: String,
    pub replay_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExternalSignalStore {
    pub schema_version: u32,
    pub envelopes: BTreeMap<String, ProviderEnvelope>,
    pub delivery_receipts: BTreeMap<String, String>,
    pub processing: BTreeMap<String, ProcessingRecord>,
    pub signals: BTreeMap<String, ExternalSignal>,
    pub lifecycles: BTreeMap<String, LifecycleAggregate>,
    pub attention: BTreeMap<String, AttentionEvidence>,
    #[serde(default)]
    pub dead_letters: BTreeMap<String, DeadLetterRecord>,
    pub accepted_count: u64,
    pub deduplicated_count: u64,
    pub rejected_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSignalCommandCenterProjection {
    pub readiness: ExternalSignalReadinessProjection,
    pub accepted_count: u64,
    pub rejected_count: u64,
    pub deduplicated_count: u64,
    pub lifecycles: Vec<LifecycleAggregate>,
    pub processing: Vec<ProcessingRecord>,
    pub dead_letters: Vec<DeadLetterRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSignalReadinessProjection {
    pub enabled: bool,
    pub bind_addr: String,
    pub source_id: String,
    pub adapter_version: String,
    pub wakes_enabled: bool,
}

/// Build the redacted, daemon-owned Command Center view. Raw provider bodies and
/// annotations are intentionally absent from this DTO.
pub fn command_center_projection(
    path: &Path,
    config: &ExternalSignalConfig,
) -> Result<ExternalSignalCommandCenterProjection> {
    let store = load_store(path)?;
    Ok(ExternalSignalCommandCenterProjection {
        readiness: ExternalSignalReadinessProjection {
            enabled: config.enabled,
            bind_addr: config.bind_addr.to_string(),
            source_id: config.source_id.clone(),
            adapter_version: ADAPTER_VERSION.to_string(),
            wakes_enabled: config.wakes_enabled,
        },
        accepted_count: store.accepted_count,
        rejected_count: store.rejected_count,
        deduplicated_count: store.deduplicated_count,
        lifecycles: store.lifecycles.into_values().collect(),
        processing: store.processing.into_values().collect(),
        dead_letters: store.dead_letters.into_values().collect(),
    })
}

#[derive(Clone)]
struct IngressState {
    config: ExternalSignalConfig,
    path: PathBuf,
    gate: Arc<Mutex<()>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReceiptResponse {
    receipt_id: String,
    outcome: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

pub struct ExternalSignalHttpHost {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}

impl ExternalSignalHttpHost {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
    pub async fn shutdown(mut self) -> std::io::Result<()> {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        self.task.await.expect("external signal host task panicked")
    }
}

pub(crate) async fn spawn_managed_http_host(runtime: &crate::server::runtime::ServerRuntime) {
    let config = match ExternalSignalConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            crate::logging::error(&format!(
                "External signal ingress configuration rejected: {error:#}"
            ));
            return;
        }
    };
    if !config.enabled {
        return;
    }
    let host = match spawn_external_signal_http_host(config).await {
        Ok(Some(host)) => host,
        Ok(None) => return,
        Err(error) => {
            crate::logging::error(&format!(
                "External signal ingress failed to start: {error:#}"
            ));
            return;
        }
    };
    crate::logging::info(&format!(
        "External signal ingress listening on http://{}{}",
        host.addr(),
        INGRESS_PATH
    ));
    let spawned = runtime
        .spawn_cancellable_background_task(move |cancellation| async move {
            cancellation.cancelled().await;
            if let Err(error) = host.shutdown().await {
                crate::logging::warn(&format!("External signal ingress shutdown failed: {error}"));
            }
        })
        .await;
    if !spawned {
        crate::logging::warn("External signal ingress lifecycle task rejected during shutdown");
    }
}

pub async fn spawn_external_signal_http_host(
    config: ExternalSignalConfig,
) -> Result<Option<ExternalSignalHttpHost>> {
    config.validate()?;
    if !config.enabled {
        return Ok(None);
    }
    let path = crate::storage::jcode_dir()?
        .join("external-signals")
        .join("state.json");
    if let Some(parent) = path.parent() {
        crate::storage::ensure_dir(parent)?;
    }
    let listener = TcpListener::bind(config.bind_addr).await?;
    let addr = listener.local_addr()?;
    let state = IngressState {
        config: config.clone(),
        path,
        gate: Arc::new(Mutex::new(())),
    };
    let app = Router::new()
        .route(READY_PATH, get(ready))
        .route(INGRESS_PATH, post(admit))
        .layer(DefaultBodyLimit::max(config.max_body_bytes))
        .with_state(state);
    let (tx, rx) = oneshot::channel();
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await
    });
    Ok(Some(ExternalSignalHttpHost {
        addr,
        shutdown: Some(tx),
        task,
    }))
}

async fn ready(State(state): State<IngressState>) -> impl IntoResponse {
    Json(
        serde_json::json!({"ready": true, "sourceId": state.config.source_id, "adapterVersion": ADAPTER_VERSION}),
    )
}

async fn admit(State(state): State<IngressState>, headers: HeaderMap, body: Bytes) -> Response {
    match admit_inner(&state, &headers, &body) {
        Ok((status, receipt)) => (status, Json(receipt)).into_response(),
        Err((status, error)) => (status, Json(ErrorResponse { error })).into_response(),
    }
}

fn admit_inner(
    state: &IngressState,
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(StatusCode, ReceiptResponse), (StatusCode, &'static str)> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if content_type != "application/json" && content_type != "application/json; charset=utf-8" {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_content_type",
        ));
    }
    if headers.contains_key(header::CONTENT_ENCODING) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_content_encoding",
        ));
    }
    let webhook: GrafanaWebhook = serde_json::from_slice(body)
        .map_err(|_| (StatusCode::UNPROCESSABLE_ENTITY, "invalid_grafana_payload"))?;
    validate_webhook(&webhook, &state.config)?;
    let project_key =
        project_key(&webhook).ok_or((StatusCode::UNPROCESSABLE_ENTITY, "missing_project_key"))?;
    let digest = sha256(body);
    let delivery_key = headers
        .get("x-jcode-delivery-id")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty() && v.len() <= 256)
        .map(str::to_string)
        .unwrap_or_else(|| digest.clone());
    let _guard = state
        .gate
        .lock()
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "store_unavailable"))?;
    let mut store =
        load_store(&state.path).map_err(|error| store_unavailable(&state.path, "load", error))?;
    if let Some(receipt_id) = store.delivery_receipts.get(&delivery_key).cloned() {
        store.deduplicated_count += 1;
        save_store(&state.path, &store)
            .map_err(|error| store_unavailable(&state.path, "deduplication save", error))?;
        return Ok((
            StatusCode::ACCEPTED,
            ReceiptResponse {
                receipt_id,
                outcome: "deduplicated",
            },
        ));
    }
    let receipt_id = format!("rcpt_{}", &digest[..24]);
    let now = Utc::now();
    let envelope = ProviderEnvelope {
        schema_version: SCHEMA_VERSION,
        receipt_id: receipt_id.clone(),
        delivery_key: delivery_key.clone(),
        source_id: state.config.source_id.clone(),
        adapter_version: ADAPTER_VERSION.to_string(),
        project_key: project_key.to_string(),
        received_at: now,
        content_sha256: digest,
        raw_json: String::from_utf8_lossy(body).into_owned(),
    };
    store.schema_version = SCHEMA_VERSION;
    store.envelopes.insert(receipt_id.clone(), envelope);
    store
        .delivery_receipts
        .insert(delivery_key, receipt_id.clone());
    store.processing.insert(
        receipt_id.clone(),
        ProcessingRecord {
            receipt_id: receipt_id.clone(),
            stage: ProcessingStage::Pending,
            attempts: 0,
            next_attempt_at: now,
            last_error: None,
            failure_stage: None,
            terminal: false,
        },
    );
    store.accepted_count += 1;
    save_store(&state.path, &store)
        .map_err(|error| store_unavailable(&state.path, "admission save", error))?;
    drop(_guard);
    let path = state.path.clone();
    let gate = Arc::clone(&state.gate);
    let config = state.config.clone();
    let processing_receipt_id = receipt_id.clone();
    tokio::spawn(async move {
        let _ = process_receipt(&path, &gate, &config, &processing_receipt_id).await;
    });
    Ok((
        StatusCode::ACCEPTED,
        ReceiptResponse {
            receipt_id,
            outcome: "accepted",
        },
    ))
}

const MAX_PROCESSING_ATTEMPTS: u32 = 5;

/// Process one accepted receipt. The durable processing record is updated before
/// work is attempted, so a crash or restart can safely resume it.
pub async fn process_receipt(
    path: &Path,
    gate: &Arc<Mutex<()>>,
    config: &ExternalSignalConfig,
    receipt_id: &str,
) -> Result<ProcessingRecord> {
    let (envelope, attempt) = {
        let _guard = gate
            .lock()
            .map_err(|_| anyhow::anyhow!("store lock poisoned"))?;
        let mut store = load_store(path)?;
        let record = store
            .processing
            .get_mut(receipt_id)
            .context("unknown receipt")?;
        if record.terminal || matches!(record.stage, ProcessingStage::Projected) {
            return Ok(record.clone());
        }
        record.attempts += 1;
        record.next_attempt_at = Utc::now();
        let attempt = record.attempts;
        let envelope = store
            .envelopes
            .get(receipt_id)
            .cloned()
            .context("missing envelope")?;
        save_store(path, &store)?;
        (envelope, attempt)
    };
    let result = (|| -> Result<()> {
        let webhook: GrafanaWebhook =
            serde_json::from_str(&envelope.raw_json).context("decode provider envelope")?;
        let _guard = gate
            .lock()
            .map_err(|_| anyhow::anyhow!("store lock poisoned"))?;
        let mut store = load_store(path)?;
        project_webhook(
            &mut store,
            config,
            receipt_id,
            &webhook,
            envelope.received_at,
        );
        save_store(path, &store)?;
        Ok(())
    })();
    let _guard = gate
        .lock()
        .map_err(|_| anyhow::anyhow!("store lock poisoned"))?;
    let mut store = load_store(path)?;
    let mut terminal_error = None;
    {
        let record = store
            .processing
            .get_mut(receipt_id)
            .context("unknown receipt")?;
        match result {
            Ok(()) => {
                record.stage = ProcessingStage::Projected;
                record.last_error = None;
                record.failure_stage = None;
                record.terminal = false;
            }
            Err(error) => {
                let message = error.to_string();
                record.last_error = Some(message.clone());
                record.failure_stage = Some("adapt_and_project".to_string());
                record.terminal = attempt >= MAX_PROCESSING_ATTEMPTS;
                if record.terminal {
                    record.stage = ProcessingStage::DeadLetter;
                    terminal_error = Some(message);
                } else {
                    let delay = 2_i64.pow(attempt.min(10));
                    record.next_attempt_at = Utc::now() + chrono::Duration::seconds(delay);
                }
            }
        }
    }
    if let Some(error) = terminal_error {
        let replay_count = store
            .dead_letters
            .get(receipt_id)
            .map_or(0, |d| d.replay_count);
        store.dead_letters.insert(
            receipt_id.to_string(),
            DeadLetterRecord {
                receipt_id: receipt_id.to_string(),
                failed_at: Utc::now(),
                attempts: attempt,
                failure_stage: "adapt_and_project".to_string(),
                error,
                replay_count,
            },
        );
    }
    let result_record = store
        .processing
        .get(receipt_id)
        .cloned()
        .context("unknown receipt")?;
    save_store(path, &store)?;
    if config.wakes_enabled && result_record.stage == ProcessingStage::Projected {
        let wake_path = path.to_path_buf();
        let wake_gate = Arc::clone(gate);
        tokio::task::spawn_blocking(move || {
            let _ = project_wakes(&wake_path, &wake_gate);
        });
    }
    Ok(result_record)
}

/// Clear terminal state and process a receipt again. Reprojection is idempotent.
pub async fn replay_receipt(
    path: &Path,
    gate: &Arc<Mutex<()>>,
    config: &ExternalSignalConfig,
    receipt_id: &str,
) -> Result<ProcessingRecord> {
    {
        let _guard = gate
            .lock()
            .map_err(|_| anyhow::anyhow!("store lock poisoned"))?;
        let mut store = load_store(path)?;
        let record = store
            .processing
            .get_mut(receipt_id)
            .context("unknown receipt")?;
        record.stage = ProcessingStage::Pending;
        record.terminal = false;
        record.next_attempt_at = Utc::now();
        if let Some(dlq) = store.dead_letters.get_mut(receipt_id) {
            dlq.replay_count += 1;
        }
        save_store(path, &store)?;
    }
    process_receipt(path, gate, config, receipt_id).await
}

fn validate_webhook(
    webhook: &GrafanaWebhook,
    config: &ExternalSignalConfig,
) -> Result<(), (StatusCode, &'static str)> {
    if webhook.version != "1" {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "unsupported_schema_version",
        ));
    }
    if webhook.alerts.is_empty() || webhook.alerts.len() > 100 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid_alert_count"));
    }
    if !matches!(webhook.status.as_str(), "firing" | "resolved") {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "unsupported_lifecycle"));
    }
    let mut projects = BTreeSet::new();
    for alert in &webhook.alerts {
        if !matches!(alert.status.as_str(), "firing" | "resolved")
            || alert.fingerprint.is_empty()
            || alert.fingerprint.len() > 256
        {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid_alert"));
        }
        if alert.labels.len() > 128
            || alert.annotations.len() > 128
            || alert
                .labels
                .values()
                .chain(alert.annotations.values())
                .any(|v| v.len() > 8192)
        {
            return Err((StatusCode::UNPROCESSABLE_ENTITY, "payload_limits_exceeded"));
        }
        if let Some(project) = alert
            .labels
            .get("jcode_project")
            .or_else(|| webhook.common_labels.get("jcode_project"))
        {
            projects.insert(project.as_str());
        }
    }
    if projects.len() != 1 {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "ambiguous_project"));
    }
    if !config
        .projects
        .contains_key(*projects.iter().next().expect("one project"))
    {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "unknown_project"));
    }
    Ok(())
}

fn project_key(webhook: &GrafanaWebhook) -> Option<&str> {
    webhook
        .alerts
        .first()?
        .labels
        .get("jcode_project")
        .or_else(|| webhook.common_labels.get("jcode_project"))
        .map(String::as_str)
}

fn project_webhook(
    store: &mut ExternalSignalStore,
    config: &ExternalSignalConfig,
    receipt_id: &str,
    webhook: &GrafanaWebhook,
    now: DateTime<Utc>,
) {
    let project_key = project_key(webhook).expect("validated project");
    for alert in &webhook.alerts {
        let lifecycle = if alert.status == "resolved" {
            LifecycleState::Resolved
        } else {
            LifecycleState::Firing
        };
        let severity = severity(
            alert
                .labels
                .get("severity")
                .or_else(|| webhook.common_labels.get("severity"))
                .map(String::as_str),
        );
        let lifecycle_key = sha256(
            format!(
                "{}\0{}\0{}",
                config.source_id, project_key, alert.fingerprint
            )
            .as_bytes(),
        );
        let occurred_at = if lifecycle == LifecycleState::Resolved && alert.ends_at.timestamp() > 0
        {
            alert.ends_at
        } else {
            alert.starts_at
        };
        let signal_id = format!(
            "sig_{}",
            &sha256(format!("{receipt_id}\0{}", alert.fingerprint).as_bytes())[..24]
        );
        let title = alert
            .annotations
            .get("summary")
            .or_else(|| alert.labels.get("alertname"))
            .cloned()
            .unwrap_or_else(|| "Grafana alert".to_string());
        store
            .signals
            .entry(signal_id.clone())
            .or_insert(ExternalSignal {
                schema_version: SCHEMA_VERSION,
                signal_id,
                receipt_id: receipt_id.to_string(),
                project_key: project_key.to_string(),
                working_dir: config.projects[project_key].clone(),
                source_id: config.source_id.clone(),
                provider_event_id: webhook.group_key.clone(),
                fingerprint: alert.fingerprint.clone(),
                lifecycle,
                severity,
                title: title.clone(),
                occurred_at,
                observed_at: now,
            });
        reduce_lifecycle(
            store,
            lifecycle_key,
            project_key,
            &config.source_id,
            alert,
            lifecycle,
            severity,
            title,
            occurred_at,
            now,
        );
    }
    if let Some(record) = store.processing.get_mut(receipt_id) {
        record.stage = ProcessingStage::Projected;
        record.attempts = 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn reduce_lifecycle(
    store: &mut ExternalSignalStore,
    key: String,
    project_key: &str,
    source_id: &str,
    alert: &GrafanaAlert,
    incoming: LifecycleState,
    severity: SignalSeverity,
    title: String,
    occurred_at: DateTime<Utc>,
    now: DateTime<Utc>,
) {
    let aggregate = store
        .lifecycles
        .entry(key.clone())
        .or_insert_with(|| LifecycleAggregate {
            lifecycle_key: key.clone(),
            project_key: project_key.to_string(),
            source_id: source_id.to_string(),
            fingerprint: alert.fingerprint.clone(),
            state: incoming,
            severity,
            generation: u64::from(incoming == LifecycleState::Firing),
            occurrence_count: 0,
            first_seen: occurred_at,
            last_seen: occurred_at,
            last_transition_at: occurred_at,
            title: title.clone(),
            attention_id: None,
            scheduled_item_id: None,
        });
    if occurred_at < aggregate.last_transition_at && incoming != aggregate.state {
        return;
    }
    aggregate.occurrence_count += 1;
    aggregate.last_seen = aggregate.last_seen.max(occurred_at);
    aggregate.severity = aggregate.severity.max(severity);
    aggregate.title = title;
    if incoming != aggregate.state {
        if incoming == LifecycleState::Firing {
            aggregate.generation += 1;
        }
        aggregate.state = incoming;
        aggregate.last_transition_at = occurred_at;
    }
    let attention_id = aggregate
        .attention_id
        .get_or_insert_with(|| format!("attn_{}", &key[..24]))
        .clone();
    let evidence = store
        .attention
        .entry(attention_id.clone())
        .or_insert(AttentionEvidence {
            attention_id,
            lifecycle_key: key,
            project_key: project_key.to_string(),
            kind: "notify".to_string(),
            priority: priority_name(aggregate.severity).to_string(),
            summary: aggregate.title.clone(),
            created_at: now,
            updated_at: now,
            resolved_at: None,
        });
    evidence.updated_at = now;
    evidence.priority = priority_name(aggregate.severity).to_string();
    evidence.summary = format!(
        "{} ({} occurrence{})",
        aggregate.title,
        aggregate.occurrence_count,
        if aggregate.occurrence_count == 1 {
            ""
        } else {
            "s"
        }
    );
    evidence.resolved_at = (aggregate.state == LifecycleState::Resolved).then_some(now);
}

fn project_wakes(path: &Path, gate: &Mutex<()>) -> Result<()> {
    let _guard = gate
        .lock()
        .map_err(|_| anyhow::anyhow!("store lock poisoned"))?;
    let mut store = load_store(path)?;
    let mut manager = AmbientManager::new()?;
    let mut changed = false;
    for aggregate in store.lifecycles.values_mut() {
        if aggregate.state != LifecycleState::Firing || aggregate.scheduled_item_id.is_some() {
            continue;
        }
        let wake_in_minutes = match aggregate.severity {
            SignalSeverity::Critical => 0,
            SignalSeverity::High => 1,
            SignalSeverity::Normal => 15,
            SignalSeverity::Low => 60,
        };
        let id = manager.schedule(ScheduleRequest {
            wake_in_minutes: Some(wake_in_minutes),
            wake_at: None,
            context: format!(
                "External signal: {}. Evidence ID: {}",
                aggregate.title,
                aggregate.attention_id.as_deref().unwrap_or("unknown")
            ),
            priority: match aggregate.severity {
                SignalSeverity::Critical | SignalSeverity::High => Priority::High,
                SignalSeverity::Normal => Priority::Normal,
                SignalSeverity::Low => Priority::Low,
            },
            target: ScheduleTarget::Ambient,
            created_by_session: "external-signal-ingress".to_string(),
            working_dir: Some(
                store
                    .signals
                    .values()
                    .find(|signal| {
                        signal.fingerprint == aggregate.fingerprint
                            && signal.project_key == aggregate.project_key
                    })
                    .map(|signal| signal.working_dir.display().to_string())
                    .unwrap_or_default(),
            ),
            task_description: Some(aggregate.title.clone()),
            relevant_files: Vec::new(),
            git_branch: None,
            additional_context: Some(format!(
                "lifecycle_key={} occurrences={}",
                aggregate.lifecycle_key, aggregate.occurrence_count
            )),
        })?;
        aggregate.scheduled_item_id = Some(id);
        changed = true;
    }
    if changed {
        save_store(path, &store)?;
    }
    Ok(())
}

fn severity(value: Option<&str>) -> SignalSeverity {
    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "critical" | "p0" | "sev0" => SignalSeverity::Critical,
        "high" | "warning" | "p1" | "sev1" => SignalSeverity::High,
        "low" | "info" => SignalSeverity::Low,
        _ => SignalSeverity::Normal,
    }
}
fn priority_name(value: SignalSeverity) -> &'static str {
    match value {
        SignalSeverity::Critical => "critical",
        SignalSeverity::High => "high",
        SignalSeverity::Normal => "normal",
        SignalSeverity::Low => "low",
    }
}
fn sha256(bytes: impl AsRef<[u8]>) -> String {
    Sha256::digest(bytes.as_ref())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn load_store(path: &Path) -> Result<ExternalSignalStore> {
    if path.exists() {
        crate::storage::read_json(path)
    } else {
        Ok(ExternalSignalStore::default())
    }
}
fn save_store(path: &Path, store: &ExternalSignalStore) -> Result<()> {
    crate::storage::write_json(path, store)
}

fn store_unavailable(
    path: &Path,
    operation: &str,
    error: anyhow::Error,
) -> (StatusCode, &'static str) {
    crate::logging::warn(&format!(
        "External signal store {operation} failed for {}: {error:#}",
        path.display()
    ));
    (StatusCode::SERVICE_UNAVAILABLE, "store_unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    fn config() -> ExternalSignalConfig {
        ExternalSignalConfig {
            enabled: true,
            bind_addr: "127.0.0.1:0".parse().unwrap(),
            source_id: "grafana-test".into(),
            projects: BTreeMap::from([("jcode".into(), PathBuf::from("/repo/jcode"))]),
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
            wakes_enabled: false,
        }
    }
    fn payload(status: &str, severity: &str, starts: &str, ends: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "version":"1", "groupKey":"group", "status":status, "receiver":"jcode", "groupLabels":{},
            "commonLabels":{"jcode_project":"jcode","severity":severity}, "commonAnnotations":{},
            "alerts":[{"status":status,"labels":{"alertname":"DiskFull","jcode_project":"jcode","severity":severity},"annotations":{"summary":"Disk is full"},"startsAt":starts,"endsAt":ends,"generatorURL":"","fingerprint":"abc123"}]
        })).unwrap()
    }

    #[test]
    fn rejects_public_and_wildcard_binds() {
        for ip in [
            IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1)),
            IpAddr::V6(Ipv6Addr::UNSPECIFIED),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            IpAddr::V6(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 1)),
        ] {
            if ip.is_loopback() {
                continue;
            }
            let mut cfg = config();
            cfg.bind_addr = SocketAddr::new(ip, 8080);
            assert!(cfg.validate().is_err());
        }
    }

    #[test]
    fn permits_loopback_rfc1918_and_tailscale_binds() {
        for ip in [
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V4(Ipv4Addr::new(10, 1, 2, 3)),
            IpAddr::V4(Ipv4Addr::new(172, 16, 2, 3)),
            IpAddr::V4(Ipv4Addr::new(192, 168, 2, 3)),
            IpAddr::V4(Ipv4Addr::new(100, 64, 2, 3)),
            IpAddr::V4(Ipv4Addr::new(100, 127, 255, 254)),
            IpAddr::V6(Ipv6Addr::LOCALHOST),
        ] {
            let mut cfg = config();
            cfg.bind_addr = SocketAddr::new(ip, 8080);
            assert!(cfg.validate().is_ok(), "{ip} should be accepted");
        }
    }

    #[test]
    fn coalesces_repeats_and_ignores_stale_resolution() {
        let cfg = config();
        let mut store = ExternalSignalStore::default();
        let firing: GrafanaWebhook = serde_json::from_slice(&payload(
            "firing",
            "high",
            "2026-08-17T04:00:00Z",
            "0001-01-01T00:00:00Z",
        ))
        .unwrap();
        project_webhook(&mut store, &cfg, "r1", &firing, Utc::now());
        project_webhook(&mut store, &cfg, "r2", &firing, Utc::now());
        let aggregate = store.lifecycles.values().next().unwrap();
        assert_eq!(aggregate.occurrence_count, 2);
        assert_eq!(aggregate.state, LifecycleState::Firing);
        assert_eq!(store.attention.len(), 1);
        let stale: GrafanaWebhook = serde_json::from_slice(&payload(
            "resolved",
            "high",
            "2026-08-17T03:00:00Z",
            "2026-08-17T03:30:00Z",
        ))
        .unwrap();
        project_webhook(&mut store, &cfg, "r3", &stale, Utc::now());
        assert_eq!(
            store.lifecycles.values().next().unwrap().state,
            LifecycleState::Firing
        );
    }

    #[test]
    fn resolved_before_firing_reopens_a_new_generation() {
        let cfg = config();
        let mut store = ExternalSignalStore::default();
        let resolved: GrafanaWebhook = serde_json::from_slice(&payload(
            "resolved",
            "normal",
            "2026-08-17T04:00:00Z",
            "2026-08-17T04:05:00Z",
        ))
        .unwrap();
        project_webhook(&mut store, &cfg, "r1", &resolved, Utc::now());
        let firing: GrafanaWebhook = serde_json::from_slice(&payload(
            "firing",
            "critical",
            "2026-08-17T04:10:00Z",
            "0001-01-01T00:00:00Z",
        ))
        .unwrap();
        project_webhook(&mut store, &cfg, "r2", &firing, Utc::now());
        let aggregate = store.lifecycles.values().next().unwrap();
        assert_eq!(aggregate.state, LifecycleState::Firing);
        assert_eq!(aggregate.generation, 1);
        assert_eq!(aggregate.severity, SignalSeverity::Critical);
    }

    #[test]
    fn accepts_native_grafana_envelope_with_unconsumed_fields() {
        let cfg = config();
        let webhook: GrafanaWebhook = serde_json::from_value(serde_json::json!({
            "version":"1",
            "groupKey":"group",
            "status":"firing",
            "receiver":"jcode",
            "externalURL":"http://grafana.local/",
            "truncatedAlerts":0,
            "orgId":1,
            "title":"[FIRING:1] DiskFull",
            "state":"alerting",
            "message":"Disk is full",
            "groupLabels":{},
            "commonLabels":{"jcode_project":"jcode","severity":"critical"},
            "commonAnnotations":{},
            "alerts":[{
                "status":"firing",
                "labels":{"alertname":"DiskFull","jcode_project":"jcode","severity":"critical"},
                "annotations":{"summary":"Disk is full"},
                "startsAt":"2026-08-17T04:00:00Z",
                "endsAt":"0001-01-01T00:00:00Z",
                "generatorURL":"http://grafana.local/alerting/list",
                "fingerprint":"abc123",
                "silenceURL":"http://grafana.local/silence/new",
                "dashboardURL":"http://grafana.local/d/hash",
                "panelURL":"http://grafana.local/d/hash?viewPanel=1",
                "values":{"A":1},
                "valueString":"A=1",
                "extraNativeField":"ignored"
            }]
        }))
        .unwrap();

        assert!(validate_webhook(&webhook, &cfg).is_ok());
    }

    #[tokio::test]
    async fn failed_processing_reaches_dlq_and_replay_is_idempotent_after_reload() {
        let cfg = config();
        let path = std::env::temp_dir().join(format!(
            "jcode-external-signal-dlq-{}.json",
            uuid::Uuid::new_v4()
        ));
        let receipt_id = "rcpt_dlq";
        let mut store = ExternalSignalStore::default();
        let now = Utc::now();
        store.envelopes.insert(
            receipt_id.to_string(),
            ProviderEnvelope {
                schema_version: SCHEMA_VERSION,
                receipt_id: receipt_id.to_string(),
                delivery_key: "delivery-dlq".to_string(),
                source_id: cfg.source_id.clone(),
                adapter_version: ADAPTER_VERSION.to_string(),
                project_key: "jcode".to_string(),
                received_at: now,
                content_sha256: "deadbeef".to_string(),
                raw_json: "{}".to_string(),
            },
        );
        store.processing.insert(
            receipt_id.to_string(),
            ProcessingRecord {
                receipt_id: receipt_id.to_string(),
                stage: ProcessingStage::Pending,
                attempts: 0,
                next_attempt_at: now,
                last_error: None,
                failure_stage: None,
                terminal: false,
            },
        );
        save_store(&path, &store).unwrap();
        let gate = Arc::new(Mutex::new(()));
        for _ in 0..MAX_PROCESSING_ATTEMPTS {
            process_receipt(&path, &gate, &cfg, receipt_id)
                .await
                .unwrap();
        }
        let persisted = load_store(&path).unwrap();
        assert_eq!(
            persisted.processing[receipt_id].stage,
            ProcessingStage::DeadLetter
        );
        assert_eq!(
            persisted.processing[receipt_id].attempts,
            MAX_PROCESSING_ATTEMPTS
        );
        assert_eq!(persisted.dead_letters[receipt_id].replay_count, 0);

        let replayed = replay_receipt(&path, &gate, &cfg, receipt_id)
            .await
            .unwrap();
        assert_eq!(replayed.stage, ProcessingStage::DeadLetter);
        let after_replay = load_store(&path).unwrap();
        assert_eq!(after_replay.dead_letters[receipt_id].replay_count, 1);
        assert_eq!(
            after_replay.processing[receipt_id].attempts,
            MAX_PROCESSING_ATTEMPTS + 1
        );
        std::fs::remove_file(path).ok();
    }

    #[tokio::test]
    async fn admission_persists_when_existing_store_has_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let cfg = config();
        let mut existing = ExternalSignalStore::default();
        existing.schema_version = SCHEMA_VERSION;
        save_store(&path, &existing).unwrap();
        let mut prior = existing.clone();
        prior.accepted_count = 1;
        save_store(&path, &prior).unwrap();

        let state = IngressState {
            config: cfg,
            path,
            gate: Arc::new(Mutex::new(())),
        };
        let headers = HeaderMap::from_iter([
            (header::CONTENT_TYPE, "application/json".parse().unwrap()),
            (
                header::HeaderName::from_static("x-jcode-delivery-id"),
                "regression-existing-store".parse().unwrap(),
            ),
        ]);
        let body = payload(
            "firing",
            "high",
            "2026-08-17T04:00:00Z",
            "0001-01-01T00:00:00Z",
        );

        let (status, receipt) = admit_inner(&state, &headers, &body).unwrap();
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(receipt.outcome, "accepted");
        let persisted = load_store(&state.path).unwrap();
        assert_eq!(persisted.accepted_count, 2);
        assert!(state.path.with_extension("bak").exists());
    }

    #[tokio::test]
    async fn admission_migrates_legacy_store_without_dead_letters_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let cfg = config();
        let mut legacy = serde_json::to_value(ExternalSignalStore::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("dead_letters");
        std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();

        let state = IngressState {
            config: cfg,
            path,
            gate: Arc::new(Mutex::new(())),
        };
        let headers = HeaderMap::from_iter([(
            header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        )]);
        let body = payload(
            "firing",
            "high",
            "2026-08-17T04:00:00Z",
            "0001-01-01T00:00:00Z",
        );

        let (status, receipt) = admit_inner(&state, &headers, &body).unwrap();
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(receipt.outcome, "accepted");
        let persisted = load_store(&state.path).unwrap();
        assert_eq!(persisted.accepted_count, 1);
        assert!(persisted.dead_letters.is_empty());
    }

    #[tokio::test]
    async fn admission_loads_legacy_primary_and_backup_without_terminal_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        let cfg = config();
        let mut legacy = serde_json::to_value(ExternalSignalStore::default()).unwrap();
        let processing = legacy
            .as_object_mut()
            .unwrap()
            .get_mut("processing")
            .unwrap()
            .as_object_mut()
            .unwrap();
        processing.insert(
            "rcpt_legacy".to_string(),
            serde_json::json!({
                "receipt_id": "rcpt_legacy",
                "stage": "pending",
                "attempts": 0,
                "next_attempt_at": "2026-08-17T04:00:00Z",
                "last_error": null,
                "failure_stage": null
            }),
        );
        legacy.as_object_mut().unwrap().remove("dead_letters");
        let legacy_bytes = serde_json::to_vec(&legacy).unwrap();
        std::fs::write(&path, &legacy_bytes).unwrap();
        std::fs::write(path.with_extension("bak"), &legacy_bytes).unwrap();
        assert!(!load_store(&path).unwrap().processing["rcpt_legacy"].terminal);
        std::fs::write(&path, b"{\"broken\":").unwrap();

        let state = IngressState {
            config: cfg,
            path,
            gate: Arc::new(Mutex::new(())),
        };
        let headers = HeaderMap::from_iter([
            (header::CONTENT_TYPE, "application/json".parse().unwrap()),
            (
                header::HeaderName::from_static("x-jcode-delivery-id"),
                "legacy-primary-backup".parse().unwrap(),
            ),
        ]);
        let body = payload(
            "firing",
            "high",
            "2026-08-17T04:00:00Z",
            "0001-01-01T00:00:00Z",
        );

        let (status, receipt) = admit_inner(&state, &headers, &body).unwrap();
        assert_eq!(status, StatusCode::ACCEPTED);
        assert_eq!(receipt.outcome, "accepted");
        let persisted = load_store(&state.path).unwrap();
        assert!(!persisted.processing["rcpt_legacy"].terminal);
        assert_eq!(persisted.accepted_count, 1);
        assert_eq!(persisted.delivery_receipts.len(), 1);
    }
}
