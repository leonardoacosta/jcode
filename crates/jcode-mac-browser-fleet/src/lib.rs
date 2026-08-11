use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UnixListener, UnixStream};
use url::Url;

pub use jcode_mac_browser_policy::Action;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FleetErrorKind {
    Malformed,
    Oversized,
    Unauthenticated,
    UnsupportedVersion,
    DuplicateId,
    DuplicateMutation,
    StaleGeneration,
    DeadlineExceeded,
    ApprovalRequired,
    UnsupportedCapability,
    UntrustedEndpoint,
    Io,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FleetError {
    kind: FleetErrorKind,
    message: String,
}

impl FleetError {
    pub fn new(kind: FleetErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> FleetErrorKind {
        self.kind
    }
}

impl fmt::Display for FleetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FleetError {}

type Result<T> = std::result::Result<T, FleetError>;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowserKind {
    Chrome,
    Edge,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    Inventory,
    Navigate,
    Click,
    RichInspection,
    Evaluate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TargetRef {
    pub browser_id: String,
    pub window_id: String,
    pub tab_id: String,
    pub generation: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FleetEnvelope {
    pub version: u16,
    pub auth: String,
    pub id: String,
    pub deadline_ms: u64,
    pub target_generation: u64,
    pub action: WireAction,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WireAction {
    FleetHealth,
    ListBrowsers,
    Navigate,
    Click,
    Type,
    Press,
}

#[derive(Debug)]
pub struct ProtocolCodec {
    versions: Vec<u16>,
    max_payload_bytes: usize,
    secret: String,
}

impl ProtocolCodec {
    pub fn new(versions: Vec<u16>, max_payload_bytes: usize, secret: String) -> Self {
        Self {
            versions,
            max_payload_bytes,
            secret,
        }
    }

    pub fn decode_request(&self, bytes: &[u8]) -> Result<FleetEnvelope> {
        if bytes.len() > self.max_payload_bytes {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "fleet request exceeded size limit",
            ));
        }
        let env: FleetEnvelope = serde_json::from_slice(bytes).map_err(|_| {
            FleetError::new(FleetErrorKind::Malformed, "fleet request was malformed")
        })?;
        if !self.versions.contains(&env.version) {
            return Err(FleetError::new(
                FleetErrorKind::UnsupportedVersion,
                "unsupported fleet protocol version",
            ));
        }
        if env.auth != self.secret {
            return Err(FleetError::new(
                FleetErrorKind::Unauthenticated,
                "fleet authentication failed",
            ));
        }
        if env.id.trim().is_empty() {
            return Err(FleetError::new(
                FleetErrorKind::Malformed,
                "fleet request id is required",
            ));
        }
        Ok(env)
    }
}

#[derive(Debug)]
pub struct ProtocolSession {
    codec: ProtocolCodec,
    seen_ids: BTreeSet<String>,
}

impl ProtocolSession {
    pub fn new(codec: ProtocolCodec) -> Self {
        Self {
            codec,
            seen_ids: BTreeSet::new(),
        }
    }

    pub fn decode_unique_request(&mut self, bytes: &[u8]) -> Result<FleetEnvelope> {
        let env = self.codec.decode_request(bytes)?;
        if !self.seen_ids.insert(env.id.clone()) {
            return Err(FleetError::new(
                FleetErrorKind::DuplicateId,
                "duplicate fleet request id rejected",
            ));
        }
        Ok(env)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityLease {
    pub lease_id: String,
    pub target: TargetRef,
    pub capabilities: BTreeSet<Capability>,
    pub expires_at_monotonic_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApprovalRequest {
    pub request_id: String,
    pub target: TargetRef,
    pub action: WireAction,
    pub declared_sensitivity: String,
    pub deadline_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuditMetadata {
    pub request_id: String,
    pub action: WireAction,
    pub target_generation: u64,
    pub decision: String,
}

#[derive(Clone, Debug)]
pub struct BrokerConfig {
    pub socket_path: PathBuf,
    pub secret: String,
    pub max_payload_bytes: usize,
    pub max_in_flight: usize,
}

#[derive(Clone, Debug)]
pub enum FleetRequest {
    Health {
        id: String,
        auth: String,
        target_generation: u64,
        deadline: Duration,
    },
    Action {
        id: String,
        auth: String,
        target: TargetRef,
        action: Action,
        deadline: Duration,
    },
}

impl FleetRequest {
    pub fn health(
        id: impl Into<String>,
        auth: String,
        target_generation: u64,
        deadline: Duration,
    ) -> Self {
        Self::Health {
            id: id.into(),
            auth,
            target_generation,
            deadline,
        }
    }

    pub fn action(
        id: impl Into<String>,
        auth: String,
        target: TargetRef,
        action: Action,
        deadline: Duration,
    ) -> Self {
        Self::Action {
            id: id.into(),
            auth,
            target,
            action,
            deadline,
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::Health { id, .. } | Self::Action { id, .. } => id,
        }
    }

    fn auth(&self) -> &str {
        match self {
            Self::Health { auth, .. } | Self::Action { auth, .. } => auth,
        }
    }

    fn deadline(&self) -> Duration {
        match self {
            Self::Health { deadline, .. } | Self::Action { deadline, .. } => *deadline,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum FleetResponse {
    Health {
        generation: u64,
        connected_targets: usize,
        targets: Vec<TargetRef>,
    },
    Accepted,
}

#[derive(Clone, Debug)]
pub struct InventoryUpdate {
    browser: BrowserKind,
    profile: String,
    targets: Vec<TargetRef>,
}

impl InventoryUpdate {
    pub fn connected(
        browser: BrowserKind,
        profile: impl Into<String>,
        targets: Vec<TargetRef>,
    ) -> Self {
        Self {
            browser,
            profile: profile.into(),
            targets,
        }
    }

    pub fn targets(&self) -> &[TargetRef] {
        &self.targets
    }
}

#[derive(Default)]
pub struct MutationReplayGuard {
    mutations: BTreeSet<String>,
}

impl MutationReplayGuard {
    pub fn observe(&mut self, id: &str, action: Action) -> Result<()> {
        if is_read_only(action) {
            return Ok(());
        }
        if !self.mutations.insert(id.to_string()) {
            return Err(FleetError::new(
                FleetErrorKind::DuplicateMutation,
                "duplicate mutation request rejected",
            ));
        }
        Ok(())
    }
}

fn is_read_only(action: Action) -> bool {
    matches!(
        action,
        Action::FleetHealth
            | Action::PolicyStatus
            | Action::ListBrowsers
            | Action::ListWindows
            | Action::ListTabs
    )
}

pub struct Broker {
    config: BrokerConfig,
    listener: UnixListener,
    generation: u64,
    targets: BTreeMap<String, TargetRef>,
    replay_guard: MutationReplayGuard,
}

impl Broker {
    pub async fn bind(config: BrokerConfig) -> Result<Self> {
        if let Some(parent) = config.socket_path.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not create socket directory")
            })?;
        }
        let _ = fs::remove_file(&config.socket_path);
        let listener = UnixListener::bind(&config.socket_path)
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not bind fleet socket"))?;
        fs::set_permissions(&config.socket_path, fs::Permissions::from_mode(0o600)).map_err(
            |_| {
                FleetError::new(
                    FleetErrorKind::Io,
                    "could not restrict fleet socket permissions",
                )
            },
        )?;
        Ok(Self {
            config,
            listener,
            generation: 0,
            targets: BTreeMap::new(),
            replay_guard: MutationReplayGuard::default(),
        })
    }

    pub fn apply_inventory(&mut self, update: InventoryUpdate) -> Result<()> {
        self.generation = self.generation.saturating_add(1);
        let prefix = format!("{:?}:{}", update.browser, update.profile);
        let prefix_with_separator = format!("{prefix}:");
        self.targets
            .retain(|key, _| !key.starts_with(&prefix_with_separator));
        for mut target in update.targets {
            target.generation = self.generation;
            self.targets.insert(
                format!("{}:{}:{}", prefix, target.window_id, target.tab_id),
                target,
            );
        }
        for target in self.targets.values_mut() {
            target.generation = self.generation;
        }
        Ok(())
    }

    pub async fn handle(&mut self, request: FleetRequest) -> Result<FleetResponse> {
        if request.auth() != self.config.secret {
            return Err(FleetError::new(
                FleetErrorKind::Unauthenticated,
                "fleet authentication failed",
            ));
        }
        if request.deadline().is_zero() {
            return Err(FleetError::new(
                FleetErrorKind::DeadlineExceeded,
                "fleet request deadline elapsed",
            ));
        }
        match &request {
            FleetRequest::Health {
                target_generation, ..
            } => {
                if *target_generation > self.generation {
                    return Err(FleetError::new(
                        FleetErrorKind::StaleGeneration,
                        "requested generation is not available",
                    ));
                }
                Ok(FleetResponse::Health {
                    generation: self.generation,
                    connected_targets: self.targets.len(),
                    targets: self.targets.values().cloned().collect(),
                })
            }
            FleetRequest::Action { target, action, .. } => {
                self.replay_guard.observe(request.id(), *action)?;
                if target.generation != self.generation {
                    return Err(FleetError::new(
                        FleetErrorKind::StaleGeneration,
                        "target generation is stale",
                    ));
                }
                if is_read_only(*action) {
                    return Ok(FleetResponse::Accepted);
                }
                Err(FleetError::new(
                    FleetErrorKind::ApprovalRequired,
                    "local approval is required",
                ))
            }
        }
    }

    pub async fn serve(mut self) -> Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await.map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not accept fleet connection")
            })?;
            self.serve_connection(stream).await?;
        }
    }

    pub async fn serve_connection(&mut self, stream: UnixStream) -> Result<()> {
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half).take((self.config.max_payload_bytes + 1) as u64);
        let mut line = Vec::new();
        reader
            .read_until(b'\n', &mut line)
            .await
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not read fleet request"))?;
        if line.is_empty() {
            return Err(FleetError::new(
                FleetErrorKind::Malformed,
                "fleet request was empty",
            ));
        }
        if line.len() > self.config.max_payload_bytes {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "fleet request exceeded size limit",
            ));
        }

        let codec = ProtocolCodec::new(
            vec![1],
            self.config.max_payload_bytes,
            self.config.secret.clone(),
        );
        let result = match codec.decode_request(&line).and_then(envelope_to_request) {
            Ok(request) => self.handle(request).await,
            Err(error) => Err(error),
        };
        let response = match result {
            Ok(result) => serde_json::json!({"ok": true, "result": result}),
            Err(error) => serde_json::json!({
                "ok": false,
                "error": {
                    "kind": error_kind_name(error.kind()),
                    "message": error.to_string(),
                }
            }),
        };
        let mut encoded = serde_json::to_vec(&response)
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode fleet response"))?;
        encoded.push(b'\n');
        write_half
            .write_all(&encoded)
            .await
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not write fleet response"))?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ManagedCdpSource {
    endpoint: Url,
    browser: BrowserKind,
    max_targets: usize,
    max_response_bytes: usize,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CdpHttpTarget {
    id: String,
    #[serde(rename = "type")]
    target_type: String,
    web_socket_debugger_url: Option<String>,
}

impl ManagedCdpSource {
    pub fn new(
        endpoint: impl AsRef<str>,
        browser: BrowserKind,
        max_targets: usize,
        max_response_bytes: usize,
    ) -> Result<Self> {
        let endpoint = Url::parse(endpoint.as_ref()).map_err(|_| {
            FleetError::new(
                FleetErrorKind::UntrustedEndpoint,
                "CDP endpoint URL is invalid",
            )
        })?;
        if endpoint.scheme() != "http"
            || !matches!(endpoint.host_str(), Some("127.0.0.1" | "::1"))
            || endpoint.port_or_known_default().is_none()
        {
            return Err(FleetError::new(
                FleetErrorKind::UntrustedEndpoint,
                "managed CDP discovery endpoint must use loopback HTTP",
            ));
        }
        if max_targets == 0 || max_response_bytes == 0 {
            return Err(FleetError::new(
                FleetErrorKind::Malformed,
                "managed CDP bounds must be non-zero",
            ));
        }
        Ok(Self {
            endpoint,
            browser,
            max_targets,
            max_response_bytes,
        })
    }

    pub async fn discover(&self) -> Result<InventoryUpdate> {
        let host = self.endpoint.host_str().expect("validated host");
        let port = self
            .endpoint
            .port_or_known_default()
            .expect("validated port");
        let mut stream = TcpStream::connect((host, port)).await.map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "managed CDP endpoint is unavailable")
        })?;
        let base = self.endpoint.path().trim_end_matches('/');
        let path = if base.is_empty() {
            "/json/list".to_string()
        } else {
            format!("{base}/json/list")
        };
        let authority = if host.contains(':') {
            format!("[{host}]:{port}")
        } else {
            format!("{host}:{port}")
        };
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {authority}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).await.map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "could not query managed CDP endpoint")
        })?;
        let mut response = Vec::new();
        stream
            .take((self.max_response_bytes + 1) as u64)
            .read_to_end(&mut response)
            .await
            .map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not read managed CDP inventory")
            })?;
        if response.len() > self.max_response_bytes {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "managed CDP inventory exceeded size limit",
            ));
        }
        let separator = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or_else(|| {
                FleetError::new(
                    FleetErrorKind::Malformed,
                    "managed CDP response was malformed",
                )
            })?;
        if !response.starts_with(b"HTTP/1.1 200") && !response.starts_with(b"HTTP/1.0 200") {
            return Err(FleetError::new(
                FleetErrorKind::Io,
                "managed CDP endpoint returned an error",
            ));
        }
        let targets: Vec<CdpHttpTarget> = serde_json::from_slice(&response[separator + 4..])
            .map_err(|_| {
                FleetError::new(
                    FleetErrorKind::Malformed,
                    "managed CDP inventory was malformed",
                )
            })?;
        let browser_id = match self.browser {
            BrowserKind::Chrome => "managed-chrome",
            BrowserKind::Edge => "managed-edge",
        };
        let targets = targets
            .into_iter()
            .filter(|target| {
                target.target_type == "page"
                    && target
                        .web_socket_debugger_url
                        .as_deref()
                        .is_some_and(is_loopback_websocket_url)
            })
            .take(self.max_targets)
            .map(|target| TargetRef {
                browser_id: browser_id.to_string(),
                window_id: "managed-cdp".to_string(),
                tab_id: target.id,
                generation: 0,
            })
            .collect();
        Ok(InventoryUpdate::connected(
            self.browser,
            "managed-cdp",
            targets,
        ))
    }
}

fn is_loopback_websocket_url(raw: &str) -> bool {
    Url::parse(raw).is_ok_and(|url| {
        matches!(url.scheme(), "ws" | "wss")
            && matches!(url.host_str(), Some("127.0.0.1" | "::1"))
            && url.port_or_known_default().is_some()
    })
}

fn envelope_to_request(envelope: FleetEnvelope) -> Result<FleetRequest> {
    let deadline = Duration::from_millis(envelope.deadline_ms);
    match envelope.action {
        WireAction::FleetHealth | WireAction::ListBrowsers => Ok(FleetRequest::health(
            envelope.id,
            envelope.auth,
            envelope.target_generation,
            deadline,
        )),
        WireAction::Navigate | WireAction::Click | WireAction::Type | WireAction::Press => {
            let target =
                serde_json::from_value(envelope.payload.get("target").cloned().ok_or_else(
                    || FleetError::new(FleetErrorKind::Malformed, "target is required"),
                )?)
                .map_err(|_| FleetError::new(FleetErrorKind::Malformed, "target was malformed"))?;
            let action = match envelope.action {
                WireAction::Navigate => Action::Navigate,
                WireAction::Click => Action::Click,
                WireAction::Type | WireAction::Press => Action::Type,
                _ => unreachable!(),
            };
            Ok(FleetRequest::action(
                envelope.id,
                envelope.auth,
                target,
                action,
                deadline,
            ))
        }
    }
}

fn error_kind_name(kind: FleetErrorKind) -> &'static str {
    match kind {
        FleetErrorKind::Malformed => "malformed",
        FleetErrorKind::Oversized => "oversized",
        FleetErrorKind::Unauthenticated => "unauthenticated",
        FleetErrorKind::UnsupportedVersion => "unsupportedVersion",
        FleetErrorKind::DuplicateId => "duplicateId",
        FleetErrorKind::DuplicateMutation => "duplicateMutation",
        FleetErrorKind::StaleGeneration => "staleGeneration",
        FleetErrorKind::DeadlineExceeded => "deadlineExceeded",
        FleetErrorKind::ApprovalRequired => "approvalRequired",
        FleetErrorKind::UnsupportedCapability => "unsupportedCapability",
        FleetErrorKind::UntrustedEndpoint => "untrustedEndpoint",
        FleetErrorKind::Io => "io",
    }
}

#[derive(Clone, Debug)]
pub struct CdpEndpoint {
    pub id: String,
    pub browser: BrowserKind,
    pub websocket_url: String,
    pub capabilities: BTreeSet<Capability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CdpTarget {
    pub id: String,
    pub browser: BrowserKind,
    pub capabilities: BTreeSet<Capability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CdpInventory {
    pub generation: u64,
    pub targets: Vec<CdpTarget>,
}

#[derive(Debug)]
pub struct CdpAdapter {
    endpoints: Vec<CdpEndpoint>,
    max_output_bytes: usize,
}

impl CdpAdapter {
    pub fn new(endpoints: Vec<CdpEndpoint>, max_output_bytes: usize) -> Result<Self> {
        for endpoint in &endpoints {
            validate_managed_endpoint(endpoint)?;
        }
        Ok(Self {
            endpoints,
            max_output_bytes,
        })
    }

    pub async fn discover(&self) -> Result<CdpInventory> {
        Ok(CdpInventory {
            generation: 1,
            targets: self
                .endpoints
                .iter()
                .map(|endpoint| CdpTarget {
                    id: endpoint.id.clone(),
                    browser: endpoint.browser,
                    capabilities: endpoint.capabilities.clone(),
                })
                .collect(),
        })
    }

    pub async fn inspect(&self, id: &str, content: &str) -> Result<String> {
        if !self.endpoints.iter().any(|endpoint| endpoint.id == id) {
            return Err(FleetError::new(
                FleetErrorKind::UntrustedEndpoint,
                "unknown managed CDP endpoint",
            ));
        }
        Ok(content.chars().take(self.max_output_bytes).collect())
    }
}

fn validate_managed_endpoint(endpoint: &CdpEndpoint) -> Result<()> {
    if !endpoint.id.starts_with("managed-") {
        return Err(FleetError::new(
            FleetErrorKind::UntrustedEndpoint,
            "CDP endpoint is not explicitly managed",
        ));
    }
    let url = Url::parse(&endpoint.websocket_url).map_err(|_| {
        FleetError::new(
            FleetErrorKind::UntrustedEndpoint,
            "CDP endpoint URL is invalid",
        )
    })?;
    match (url.scheme(), url.host_str()) {
        ("ws" | "wss", Some("127.0.0.1" | "::1")) => Ok(()),
        _ => Err(FleetError::new(
            FleetErrorKind::UntrustedEndpoint,
            "CDP endpoint must be loopback and managed",
        )),
    }
}
