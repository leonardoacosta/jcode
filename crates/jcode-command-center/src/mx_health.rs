//! Secret-safe, server-side adapter for the pinned MX `mx.health.v1` contract.
//!
//! The browser-facing projection intentionally contains no connection details or
//! upstream error text. The adapter owns bounded I/O, strict validation, and one
//! in-memory last-known-good value.

use std::collections::HashSet;
use std::fmt;
use std::sync::Arc;
use std::time::Duration as StdDuration;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use futures_util::StreamExt;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Notify};

pub const MX_HEALTH_VERSION: &str = "mx.health.v1";
pub const MX_HEALTH_PROVENANCE_ID: &str =
    "mx:6f9ac51a419807a3636b17f5e697ae23c37cacff:mx.health.v1";
pub const MX_HEALTH_REPOSITORY: &str = "https://github.com/leonardoacosta/mx.git";
pub const MX_HEALTH_COMMIT: &str = "6f9ac51a419807a3636b17f5e697ae23c37cacff";
pub const MX_HEALTH_IMPLEMENTATION_SHA256: &str =
    "35da7eae62b10732beeb27c828b3c9418d93482d3dd78f5a0edda296cf0d82c4";
pub const MX_HEALTH_SPEC_SHA256: &str =
    "ca17e036c7ba5becedf4f6463779d1f78116c90870fb353d51dbf9264c1e4f1e";
pub const MX_HEALTH_OPENAPI_SHA256: &str =
    "a8af2d00c7e62c24dd8b303329ed483198b875b67b3221f461866519cab258f6";
pub const MX_HEALTH_TEST_SHA256: &str =
    "6788bf1a60a964e2ec0a7c1db5c911d5270b85c9220924848f98683693074d23";

const DEFAULT_TIMEOUT: StdDuration = StdDuration::from_secs(3);
const DEFAULT_MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const DEFAULT_REFRESH_WINDOW: StdDuration = StdDuration::from_secs(5);
const DEFAULT_STALE_LIMIT: StdDuration = StdDuration::from_secs(300);

#[derive(Clone)]
pub struct MxHealthConfig {
    endpoint: Option<Url>,
    token: Option<String>,
    timeout: StdDuration,
    max_response_bytes: usize,
    refresh_window: StdDuration,
    stale_limit: StdDuration,
    invalid_endpoint: bool,
}

impl fmt::Debug for MxHealthConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MxHealthConfig")
            .field("configured", &self.is_configured())
            .field("endpoint", &self.endpoint.as_ref().map(|_| "<configured>"))
            .field("token", &self.token.as_ref().map(|_| "<redacted>"))
            .field("timeout", &self.timeout)
            .field("max_response_bytes", &self.max_response_bytes)
            .field("refresh_window", &self.refresh_window)
            .field("stale_limit", &self.stale_limit)
            .field("invalid_endpoint", &self.invalid_endpoint)
            .finish()
    }
}

impl Default for MxHealthConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            token: None,
            timeout: DEFAULT_TIMEOUT,
            max_response_bytes: DEFAULT_MAX_RESPONSE_BYTES,
            refresh_window: DEFAULT_REFRESH_WINDOW,
            stale_limit: DEFAULT_STALE_LIMIT,
            invalid_endpoint: false,
        }
    }
}

impl MxHealthConfig {
    /// Loads daemon-only configuration. Values are never included in the
    /// returned projection or in error messages.
    pub fn from_env() -> Self {
        let mut config = Self::default();
        let endpoint_value = std::env::var("JCODE_MX_HEALTH_BASE_URL")
            .ok()
            .or_else(|| std::env::var("JCODE_MX_HEALTH_URL").ok())
            .filter(|value| !value.trim().is_empty());
        if let Some(value) = endpoint_value {
            match Url::parse(value.trim()) {
                Ok(url)
                    if matches!(url.scheme(), "http" | "https")
                        && url.username().is_empty()
                        && url.password().is_none() =>
                {
                    config.endpoint = Some(url);
                }
                _ => config.invalid_endpoint = true,
            }
        }
        config.token = std::env::var("JCODE_MX_HEALTH_TOKEN")
            .ok()
            .filter(|value| !value.is_empty());
        config.timeout = duration_from_env("JCODE_MX_HEALTH_TIMEOUT_MS", DEFAULT_TIMEOUT);
        config.max_response_bytes = usize_from_env(
            "JCODE_MX_HEALTH_MAX_RESPONSE_BYTES",
            DEFAULT_MAX_RESPONSE_BYTES,
        )
        .max(1);
        config.refresh_window =
            duration_from_env("JCODE_MX_HEALTH_REFRESH_WINDOW_MS", DEFAULT_REFRESH_WINDOW);
        config.stale_limit =
            duration_from_env("JCODE_MX_HEALTH_STALE_LIMIT_MS", DEFAULT_STALE_LIMIT);
        config
    }

    pub fn for_tests(endpoint: impl Into<String>, token: impl Into<String>) -> Self {
        Self::with_options(
            Some(endpoint.into()),
            Some(token.into()),
            DEFAULT_TIMEOUT,
            DEFAULT_MAX_RESPONSE_BYTES,
            DEFAULT_REFRESH_WINDOW,
            DEFAULT_STALE_LIMIT,
        )
    }

    pub fn with_options(
        endpoint: Option<String>,
        token: Option<String>,
        timeout: StdDuration,
        max_response_bytes: usize,
        refresh_window: StdDuration,
        stale_limit: StdDuration,
    ) -> Self {
        let (endpoint, invalid_endpoint) = if let Some(value) = endpoint {
            match Url::parse(value.trim()) {
                Ok(url)
                    if matches!(url.scheme(), "http" | "https")
                        && url.username().is_empty()
                        && url.password().is_none() =>
                {
                    (Some(url), false)
                }
                _ => (None, true),
            }
        } else {
            (None, false)
        };
        Self {
            endpoint,
            token: token.filter(|value| !value.is_empty()),
            timeout: timeout.max(StdDuration::from_millis(1)),
            max_response_bytes: max_response_bytes.max(1),
            refresh_window,
            stale_limit,
            invalid_endpoint,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.invalid_endpoint && self.endpoint.is_some() && self.token.is_some()
    }

    pub fn timeout(&self) -> StdDuration {
        self.timeout
    }

    pub fn max_response_bytes(&self) -> usize {
        self.max_response_bytes
    }
}

fn duration_from_env(name: &str, default: StdDuration) -> StdDuration {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(StdDuration::from_millis)
        .filter(|value| !value.is_zero())
        .unwrap_or(default)
}

fn usize_from_env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MxOverallStatus {
    Ok,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MxCheckStatus {
    Ok,
    Degraded,
    Down,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MxHealthCheck {
    pub id: String,
    pub layer: String,
    pub status: MxCheckStatus,
    pub reason_code: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MxHealthSnapshot {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub overall: MxOverallStatus,
    pub redacted: bool,
    pub checks: Vec<MxHealthCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MxHealthProvenance {
    pub id: String,
    pub repository: String,
    pub commit: String,
    pub implementation_sha256: String,
    pub specification_sha256: String,
    pub openapi_sha256: String,
    pub tests_sha256: String,
}

impl Default for MxHealthProvenance {
    fn default() -> Self {
        Self {
            id: MX_HEALTH_PROVENANCE_ID.to_owned(),
            repository: MX_HEALTH_REPOSITORY.to_owned(),
            commit: MX_HEALTH_COMMIT.to_owned(),
            implementation_sha256: MX_HEALTH_IMPLEMENTATION_SHA256.to_owned(),
            specification_sha256: MX_HEALTH_SPEC_SHA256.to_owned(),
            openapi_sha256: MX_HEALTH_OPENAPI_SHA256.to_owned(),
            tests_sha256: MX_HEALTH_TEST_SHA256.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MxAdapterState {
    Live,
    Stale,
    Unconfigured,
    Unauthorized,
    Unreachable,
    Timeout,
    InvalidContract,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MxFailureCategory {
    Unauthorized,
    UnexpectedStatus,
    Timeout,
    Unreachable,
    Oversized,
    InvalidContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MxStaleMetadata {
    pub cached_fetched_at: DateTime<Utc>,
    pub cached_generated_at: DateTime<Utc>,
    pub age_seconds: i64,
    pub current_failure: MxFailureCategory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MxHealthProjection {
    pub provenance: MxHealthProvenance,
    pub adapter_state: MxAdapterState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_category: Option<MxFailureCategory>,
    pub fetched_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<MxHealthSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<MxStaleMetadata>,
}

impl MxHealthProjection {
    pub fn unconfigured(now: DateTime<Utc>) -> Self {
        Self {
            provenance: MxHealthProvenance::default(),
            adapter_state: MxAdapterState::Unconfigured,
            failure_category: None,
            fetched_at: now,
            health: None,
            stale: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CachedSnapshot {
    snapshot: MxHealthSnapshot,
    fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
struct CacheState {
    cached: Option<CachedSnapshot>,
    last_attempt_at: Option<DateTime<Utc>>,
    last_projection: Option<MxHealthProjection>,
    in_flight: Option<Arc<Notify>>,
}

#[async_trait]
pub trait MxHealthSource: Send + Sync {
    async fn read(&self) -> MxHealthProjection;
}

#[derive(Clone)]
pub struct MxHealthClient {
    config: MxHealthConfig,
    http: Client,
    state: Arc<Mutex<CacheState>>,
}

impl fmt::Debug for MxHealthClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MxHealthClient")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl MxHealthClient {
    pub fn from_env() -> Self {
        Self::new(MxHealthConfig::from_env())
    }

    pub fn new(config: MxHealthConfig) -> Self {
        let http = Client::builder()
            .connect_timeout(config.timeout)
            .timeout(config.timeout)
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            config,
            http,
            state: Arc::new(Mutex::new(CacheState::default())),
        }
    }

    pub fn config(&self) -> &MxHealthConfig {
        &self.config
    }

    async fn fetch_uncached(&self) -> Result<MxHealthSnapshot, MxFailureCategory> {
        let endpoint = self
            .config
            .endpoint
            .as_ref()
            .ok_or(MxFailureCategory::Unreachable)?;
        let token = self
            .config
            .token
            .as_deref()
            .ok_or(MxFailureCategory::Unauthorized)?;
        let mut health_url = endpoint.clone();
        health_url.set_path("/health/v1");
        health_url.set_query(None);
        health_url.set_fragment(None);

        let response = self
            .http
            .get(health_url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|error| {
                if error.is_timeout() {
                    MxFailureCategory::Timeout
                } else {
                    MxFailureCategory::Unreachable
                }
            })?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            return Err(MxFailureCategory::Unauthorized);
        }
        if status != StatusCode::OK && status != StatusCode::SERVICE_UNAVAILABLE {
            return Err(MxFailureCategory::UnexpectedStatus);
        }

        let mut body = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| MxFailureCategory::Unreachable)?;
            if body.len().saturating_add(chunk.len()) > self.config.max_response_bytes {
                return Err(MxFailureCategory::Oversized);
            }
            body.extend_from_slice(&chunk);
        }
        let raw: UpstreamHealthResponse =
            serde_json::from_slice(&body).map_err(|_| MxFailureCategory::InvalidContract)?;
        validate_upstream(raw)
    }

    async fn read_uncached(&self, now: DateTime<Utc>) -> MxHealthProjection {
        let result = tokio::time::timeout(self.config.timeout, self.fetch_uncached()).await;
        match result {
            Ok(Ok(snapshot)) => {
                let cached = CachedSnapshot {
                    snapshot: snapshot.clone(),
                    fetched_at: now,
                };
                let projection = MxHealthProjection {
                    provenance: MxHealthProvenance::default(),
                    adapter_state: MxAdapterState::Live,
                    failure_category: None,
                    fetched_at: now,
                    health: Some(snapshot),
                    stale: None,
                };
                let mut state = self.state.lock().await;
                state.cached = Some(cached);
                state.last_projection = Some(projection.clone());
                projection
            }
            Ok(Err(failure)) => self.failure_projection(now, failure).await,
            Err(_) => {
                self.failure_projection(now, MxFailureCategory::Timeout)
                    .await
            }
        }
    }

    async fn failure_projection(
        &self,
        now: DateTime<Utc>,
        failure: MxFailureCategory,
    ) -> MxHealthProjection {
        let state = self.state.lock().await;
        if let Some(cached) = &state.cached {
            let age_duration = now - cached.fetched_at;
            let age = age_duration.num_seconds().max(0);
            let stale_limit = Duration::from_std(self.config.stale_limit)
                .unwrap_or_else(|_| Duration::seconds(0));
            if age_duration <= stale_limit {
                return MxHealthProjection {
                    provenance: MxHealthProvenance::default(),
                    adapter_state: MxAdapterState::Stale,
                    failure_category: Some(failure),
                    fetched_at: now,
                    health: Some(cached.snapshot.clone()),
                    stale: Some(MxStaleMetadata {
                        cached_fetched_at: cached.fetched_at,
                        cached_generated_at: cached.snapshot.generated_at,
                        age_seconds: age,
                        current_failure: failure,
                    }),
                };
            }
        }
        MxHealthProjection {
            provenance: MxHealthProvenance::default(),
            adapter_state: if failure == MxFailureCategory::Unauthorized {
                MxAdapterState::Unauthorized
            } else if failure == MxFailureCategory::Timeout {
                MxAdapterState::Timeout
            } else if failure == MxFailureCategory::Unreachable {
                MxAdapterState::Unreachable
            } else if failure == MxFailureCategory::InvalidContract {
                MxAdapterState::InvalidContract
            } else {
                MxAdapterState::Unavailable
            },
            failure_category: Some(failure),
            fetched_at: now,
            health: None,
            stale: None,
        }
    }
}

#[async_trait]
impl MxHealthSource for MxHealthClient {
    async fn read(&self) -> MxHealthProjection {
        if !self.config.is_configured() {
            return MxHealthProjection::unconfigured(Utc::now());
        }
        loop {
            let now = Utc::now();
            let (wait_for, should_fetch, cached_projection) = {
                let mut state = self.state.lock().await;
                if let Some(in_flight) = &state.in_flight {
                    (Some(in_flight.clone()), false, None)
                } else if let Some(last_attempt) = state.last_attempt_at {
                    if now - last_attempt
                        < Duration::from_std(self.config.refresh_window)
                            .unwrap_or_else(|_| Duration::seconds(0))
                    {
                        (None, false, state.last_projection.clone())
                    } else {
                        let notify = Arc::new(Notify::new());
                        state.in_flight = Some(notify);
                        state.last_attempt_at = Some(now);
                        (None, true, None)
                    }
                } else {
                    let notify = Arc::new(Notify::new());
                    state.in_flight = Some(notify);
                    state.last_attempt_at = Some(now);
                    (None, true, None)
                }
            };
            if let Some(notify) = wait_for {
                notify.notified().await;
                continue;
            }
            if !should_fetch {
                return cached_projection.unwrap_or_else(|| MxHealthProjection::unconfigured(now));
            }
            let projection = self.read_uncached(now).await;
            let notify = {
                let mut state = self.state.lock().await;
                state.last_projection = Some(projection.clone());
                state.in_flight.take()
            };
            if let Some(notify) = notify {
                notify.notify_waiters();
            }
            return projection;
        }
    }
}

#[derive(Debug, Deserialize)]
struct UpstreamHealthResponse {
    version: String,
    generated_at: String,
    overall: String,
    redacted: bool,
    checks: Vec<UpstreamHealthCheck>,
}

#[derive(Debug, Deserialize)]
struct UpstreamHealthCheck {
    id: String,
    layer: String,
    status: String,
    reason_code: String,
    summary: String,
    #[serde(default)]
    depends_on: Vec<String>,
}

fn validate_upstream(raw: UpstreamHealthResponse) -> Result<MxHealthSnapshot, MxFailureCategory> {
    if raw.version != MX_HEALTH_VERSION || !raw.redacted {
        return Err(MxFailureCategory::InvalidContract);
    }
    let generated_at = DateTime::parse_from_rfc3339(&raw.generated_at)
        .map_err(|_| MxFailureCategory::InvalidContract)?
        .with_timezone(&Utc);
    let overall = match raw.overall.as_str() {
        "ok" => MxOverallStatus::Ok,
        "degraded" => MxOverallStatus::Degraded,
        "down" => MxOverallStatus::Down,
        _ => return Err(MxFailureCategory::InvalidContract),
    };
    let mut ids = HashSet::with_capacity(raw.checks.len());
    let mut checks = Vec::with_capacity(raw.checks.len());
    for check in raw.checks {
        if !safe_text(&check.id)
            || !safe_text(&check.layer)
            || !safe_text(&check.reason_code)
            || !safe_text(&check.summary)
            || !ids.insert(check.id.clone())
        {
            return Err(MxFailureCategory::InvalidContract);
        }
        let status = match check.status.as_str() {
            "ok" => MxCheckStatus::Ok,
            "degraded" => MxCheckStatus::Degraded,
            "down" => MxCheckStatus::Down,
            "blocked" => MxCheckStatus::Blocked,
            _ => return Err(MxFailureCategory::InvalidContract),
        };
        let mut dependencies = HashSet::with_capacity(check.depends_on.len());
        for dependency in &check.depends_on {
            if !safe_text(dependency)
                || dependency == &check.id
                || !dependencies.insert(dependency.clone())
            {
                return Err(MxFailureCategory::InvalidContract);
            }
        }
        checks.push(MxHealthCheck {
            id: check.id,
            layer: check.layer,
            status,
            reason_code: check.reason_code,
            summary: check.summary,
            depends_on: check.depends_on,
        });
    }
    for check in &checks {
        if check
            .depends_on
            .iter()
            .any(|dependency| !ids.contains(dependency))
        {
            return Err(MxFailureCategory::InvalidContract);
        }
    }
    Ok(MxHealthSnapshot {
        version: MX_HEALTH_VERSION.to_string(),
        generated_at,
        overall,
        redacted: true,
        checks,
    })
}

fn safe_text(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed.len() <= 512 && !trimmed.chars().any(char::is_control)
}

pub fn generated_typescript_contract() -> &'static str {
    include_str!("../../../apps/command-center/src/generated/mx-health-contract.ts")
}

pub fn write_typescript_contract(out_dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(
        out_dir.join("mx-health-contract.ts"),
        generated_typescript_contract(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum::{Json, Router};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::net::TcpListener;
    use tokio::time::{Duration as TokioDuration, sleep};

    fn raw(checks: serde_json::Value, status: &str, redacted: bool) -> serde_json::Value {
        serde_json::json!({
            "version": MX_HEALTH_VERSION,
            "generated_at": "2026-08-17T18:00:00Z",
            "overall": status,
            "redacted": redacted,
            "checks": checks,
        })
    }

    fn checks() -> serde_json::Value {
        serde_json::json!([
            {"id":"source.gmail","layer":"source","status":"ok","reason_code":"source_serving","summary":"Source is serving"},
            {"id":"persistence","layer":"persistence","status":"down","reason_code":"persistence_unavailable","summary":"Persistence is unavailable"},
            {"id":"workflows","layer":"workflow","status":"blocked","reason_code":"dependency_unavailable","summary":"Persistence-backed workflows are blocked","depends_on":["persistence"]}
        ])
    }

    async fn server<F>(handler: F) -> (String, tokio::task::JoinHandle<()>)
    where
        F: Fn(HeaderMap) -> axum::response::Response + Clone + Send + Sync + 'static,
    {
        let app = Router::new().route(
            "/health/v1",
            get(move |headers: HeaderMap| async move { handler(headers) }),
        );
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (format!("http://{address}"), task)
    }

    #[test]
    fn strict_validator_accepts_additive_fields_and_preserves_dependencies() {
        let mut value = raw(checks(), "down", true);
        value["additive"] = serde_json::json!({"ignored": true});
        let snapshot: MxHealthSnapshot =
            validate_upstream(serde_json::from_value(value).unwrap()).unwrap();
        assert_eq!(snapshot.overall, MxOverallStatus::Down);
        assert_eq!(snapshot.checks[2].depends_on, vec!["persistence"]);
    }

    #[test]
    fn strict_validator_rejects_incompatible_payloads() {
        let cases = [
            ("version", serde_json::json!("wrong")),
            ("redacted", serde_json::json!(false)),
            ("overall", serde_json::json!("mystery")),
        ];
        for (field, value) in cases {
            let mut payload = raw(checks(), "ok", true);
            payload[field] = value;
            assert_eq!(
                validate_upstream(serde_json::from_value(payload).unwrap()),
                Err(MxFailureCategory::InvalidContract)
            );
        }
    }

    #[test]
    fn strict_validator_rejects_missing_empty_duplicate_and_dangling_fields() {
        let mut missing = raw(checks(), "ok", true);
        missing["checks"][0]
            .as_object_mut()
            .unwrap()
            .remove("summary");
        assert!(serde_json::from_value::<UpstreamHealthResponse>(missing).is_err());

        let mut empty = raw(checks(), "ok", true);
        empty["checks"][0]["summary"] = serde_json::json!("");
        assert_eq!(
            validate_upstream(serde_json::from_value(empty).unwrap()),
            Err(MxFailureCategory::InvalidContract)
        );

        let mut duplicate = raw(checks(), "ok", true);
        let duplicate_check = duplicate["checks"][0].clone();
        duplicate["checks"]
            .as_array_mut()
            .unwrap()
            .push(duplicate_check);
        assert_eq!(
            validate_upstream(serde_json::from_value(duplicate).unwrap()),
            Err(MxFailureCategory::InvalidContract)
        );

        let mut dangling = raw(checks(), "ok", true);
        dangling["checks"][2]["depends_on"] = serde_json::json!(["missing"]);
        assert_eq!(
            validate_upstream(serde_json::from_value(dangling).unwrap()),
            Err(MxFailureCategory::InvalidContract)
        );
    }

    #[tokio::test]
    async fn client_accepts_503_as_authoritative_down() {
        let (base, task) = server(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(raw(checks(), "down", true)),
            )
                .into_response()
        })
        .await;
        let client = MxHealthClient::new(MxHealthConfig::with_options(
            Some(base),
            Some("secret-sentinel".into()),
            StdDuration::from_secs(2),
            DEFAULT_MAX_RESPONSE_BYTES,
            StdDuration::from_millis(1),
            StdDuration::from_secs(30),
        ));
        let projection = client.read().await;
        assert_eq!(projection.adapter_state, MxAdapterState::Live);
        assert_eq!(projection.health.unwrap().overall, MxOverallStatus::Down);
        task.abort();
    }

    #[tokio::test]
    async fn client_maps_auth_status_unexpected_status_malformed_and_oversized_safely() {
        let cases = [
            (
                StatusCode::UNAUTHORIZED,
                "not parsed",
                DEFAULT_MAX_RESPONSE_BYTES,
                MxAdapterState::Unauthorized,
                MxFailureCategory::Unauthorized,
            ),
            (
                StatusCode::NOT_FOUND,
                "not parsed",
                DEFAULT_MAX_RESPONSE_BYTES,
                MxAdapterState::Unavailable,
                MxFailureCategory::UnexpectedStatus,
            ),
            (
                StatusCode::OK,
                "malformed",
                DEFAULT_MAX_RESPONSE_BYTES,
                MxAdapterState::InvalidContract,
                MxFailureCategory::InvalidContract,
            ),
            (
                StatusCode::OK,
                "0123456789",
                4,
                MxAdapterState::Unavailable,
                MxFailureCategory::Oversized,
            ),
        ];
        for (status, body, cap, expected_state, expected_failure) in cases {
            let body = body.to_owned();
            let (base, task) = server(move |_| (status, body.clone()).into_response()).await;
            let client = MxHealthClient::new(MxHealthConfig::with_options(
                Some(base),
                Some("token".to_owned()),
                StdDuration::from_secs(2),
                cap,
                StdDuration::from_millis(1),
                StdDuration::from_secs(30),
            ));
            let projection = client.read().await;
            assert_eq!(projection.adapter_state, expected_state);
            assert_eq!(projection.failure_category, Some(expected_failure));
            assert!(projection.health.is_none());
            assert!(
                !serde_json::to_string(&projection)
                    .unwrap()
                    .contains("malformed")
            );
            task.abort();
        }
    }

    #[tokio::test]
    async fn client_maps_timeout_and_unreachable_without_upstream_details() {
        let app = Router::new().route(
            "/health/v1",
            get(|| async {
                sleep(TokioDuration::from_millis(50)).await;
                (StatusCode::OK, Json(raw(serde_json::json!([]), "ok", true)))
            }),
        );
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let timeout_client = MxHealthClient::new(MxHealthConfig::with_options(
            Some(format!("http://{address}")),
            Some("token".to_owned()),
            StdDuration::from_millis(5),
            DEFAULT_MAX_RESPONSE_BYTES,
            StdDuration::from_millis(1),
            StdDuration::from_secs(30),
        ));
        let timed_out = timeout_client.read().await;
        assert_eq!(timed_out.adapter_state, MxAdapterState::Timeout);
        assert_eq!(timed_out.failure_category, Some(MxFailureCategory::Timeout));
        task.abort();

        let unreachable_client = MxHealthClient::new(MxHealthConfig::with_options(
            Some("http://127.0.0.1:1".to_owned()),
            Some("token".to_owned()),
            StdDuration::from_millis(100),
            DEFAULT_MAX_RESPONSE_BYTES,
            StdDuration::from_millis(1),
            StdDuration::from_secs(30),
        ));
        let unreachable = unreachable_client.read().await;
        assert_eq!(unreachable.adapter_state, MxAdapterState::Unreachable);
        assert_eq!(
            unreachable.failure_category,
            Some(MxFailureCategory::Unreachable)
        );
    }

    #[tokio::test]
    async fn client_coalesces_concurrent_reads_and_redacts_debug() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_for_server = calls.clone();
        let (base, task) = server(move |headers| {
            assert_eq!(
                headers.get("authorization").unwrap(),
                "Bearer secret-sentinel"
            );
            calls_for_server.fetch_add(1, Ordering::SeqCst);
            (StatusCode::OK, Json(raw(serde_json::json!([]), "ok", true))).into_response()
        })
        .await;
        let config = MxHealthConfig::with_options(
            Some(base),
            Some("secret-sentinel".into()),
            StdDuration::from_secs(2),
            DEFAULT_MAX_RESPONSE_BYTES,
            StdDuration::from_secs(30),
            StdDuration::from_secs(30),
        );
        let client = Arc::new(MxHealthClient::new(config));
        let (a, b) = tokio::join!(client.read(), client.read());
        assert_eq!(a.adapter_state, MxAdapterState::Live);
        assert_eq!(b.adapter_state, MxAdapterState::Live);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let debug = format!("{client:?}");
        assert!(!debug.contains("secret-sentinel"));
        task.abort();
    }

    #[tokio::test]
    async fn client_returns_eligible_stale_cache_and_expires_it() {
        let responses = Arc::new(AtomicUsize::new(0));
        let responses_for_server = responses.clone();
        let (base, task) = server(move |_| {
            let call = responses_for_server.fetch_add(1, Ordering::SeqCst);
            if call == 0 {
                (
                    StatusCode::OK,
                    Json(raw(serde_json::json!([]), "degraded", true)),
                )
                    .into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "not-health").into_response()
            }
        })
        .await;
        let config = MxHealthConfig::with_options(
            Some(base),
            Some("token".into()),
            StdDuration::from_secs(2),
            DEFAULT_MAX_RESPONSE_BYTES,
            StdDuration::from_millis(1),
            StdDuration::from_millis(30),
        );
        let client = MxHealthClient::new(config);
        let live = client.read().await;
        assert_eq!(live.adapter_state, MxAdapterState::Live);
        sleep(TokioDuration::from_millis(3)).await;
        let stale = client.read().await;
        assert_eq!(stale.adapter_state, MxAdapterState::Stale);
        assert_eq!(stale.health.unwrap().overall, MxOverallStatus::Degraded);
        sleep(TokioDuration::from_millis(40)).await;
        let unavailable = client.read().await;
        assert_eq!(unavailable.adapter_state, MxAdapterState::Unavailable);
        assert!(unavailable.health.is_none());
        task.abort();
    }

    #[test]
    fn configuration_and_projection_do_not_serialize_credentials() {
        let config = MxHealthConfig::for_tests("http://127.0.0.1:8799", "secret-sentinel");
        assert!(!format!("{config:?}").contains("secret-sentinel"));
        let projection = MxHealthProjection::unconfigured(Utc::now());
        let value = serde_json::to_string(&projection).unwrap();
        assert!(!value.contains("secret-sentinel"));
        assert!(!value.contains("127.0.0.1:8799"));
    }
}
