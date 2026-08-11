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
use jcode_mac_browser_policy::{
    Context, Decision, Denial, Lease, PolicyEngine, Scope, Target as PolicyTarget,
};

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
    HardDenied,
    EmergencyStop,
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
    Type,
    Press,
    RichInspection,
    Evaluate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorityAction {
    GrantLease,
    RevokeLease,
    EmergencyStop,
    ReleaseEmergencyStop,
    Status,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityEnvelope {
    pub version: u16,
    pub action: AuthorityAction,
    #[serde(default)]
    pub lease_id: Option<String>,
    #[serde(default)]
    pub target: Option<TargetRef>,
    #[serde(default)]
    pub capabilities: Vec<Capability>,
    #[serde(default)]
    pub duration_seconds: Option<u64>,
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NativeHostEnvelope {
    pub version: u16,
    pub id: String,
    pub deadline_ms: u64,
    pub target_generation: u64,
    pub action: WireAction,
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl NativeHostEnvelope {
    fn into_fleet(self, auth: String) -> FleetEnvelope {
        FleetEnvelope {
            version: self.version,
            auth,
            id: self.id,
            deadline_ms: self.deadline_ms,
            target_generation: self.target_generation,
            action: self.action,
            payload: self.payload,
        }
    }
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

#[derive(Clone, Debug)]
pub struct NativeHostBridgeConfig {
    pub socket_path: PathBuf,
    pub secret: String,
    pub max_payload_bytes: usize,
}

pub async fn read_native_message<R>(
    reader: &mut R,
    max_payload_bytes: usize,
) -> Result<Option<Vec<u8>>>
where
    R: AsyncReadExt + Unpin,
{
    let mut length = [0_u8; 4];
    let mut read = 0;
    while read < length.len() {
        let n = reader.read(&mut length[read..]).await.map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "could not read native message length")
        })?;
        if n == 0 {
            if read == 0 {
                return Ok(None);
            }
            return Err(FleetError::new(
                FleetErrorKind::Malformed,
                "native message length prefix was truncated",
            ));
        }
        read += n;
    }
    let length = u32::from_le_bytes(length) as usize;
    if length > max_payload_bytes {
        return Err(FleetError::new(
            FleetErrorKind::Oversized,
            "native message exceeded size limit",
        ));
    }
    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload).await.map_err(|_| {
        FleetError::new(
            FleetErrorKind::Malformed,
            "native message payload was truncated",
        )
    })?;
    Ok(Some(payload))
}

pub async fn write_native_message<W>(
    writer: &mut W,
    payload: &[u8],
    max_payload_bytes: usize,
) -> Result<()>
where
    W: AsyncWriteExt + Unpin,
{
    if payload.len() > max_payload_bytes || payload.len() > u32::MAX as usize {
        return Err(FleetError::new(
            FleetErrorKind::Oversized,
            "native response exceeded size limit",
        ));
    }
    writer
        .write_all(&(payload.len() as u32).to_le_bytes())
        .await
        .map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "could not write native message length")
        })?;
    writer.write_all(payload).await.map_err(|_| {
        FleetError::new(FleetErrorKind::Io, "could not write native message payload")
    })?;
    writer
        .flush()
        .await
        .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not flush native message"))?;
    Ok(())
}

pub async fn forward_native_payload(
    config: &NativeHostBridgeConfig,
    payload: &[u8],
) -> Result<Vec<u8>> {
    if payload.len() > config.max_payload_bytes {
        return Err(FleetError::new(
            FleetErrorKind::Oversized,
            "native request exceeded size limit",
        ));
    }
    let request: NativeHostEnvelope = serde_json::from_slice(payload)
        .map_err(|_| FleetError::new(FleetErrorKind::Malformed, "native request was malformed"))?;
    if request.version != 1 {
        return Err(FleetError::new(
            FleetErrorKind::UnsupportedVersion,
            "unsupported native host protocol version",
        ));
    }
    if request.id.trim().is_empty() {
        return Err(FleetError::new(
            FleetErrorKind::Malformed,
            "native request id is required",
        ));
    }

    let mut broker_payload = serde_json::to_vec(&request.into_fleet(config.secret.clone()))
        .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode broker request"))?;
    if broker_payload.len() > config.max_payload_bytes {
        return Err(FleetError::new(
            FleetErrorKind::Oversized,
            "broker request exceeded size limit",
        ));
    }
    broker_payload.push(b'\n');

    let mut stream = UnixStream::connect(&config.socket_path)
        .await
        .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not connect to fleet broker"))?;
    stream
        .write_all(&broker_payload)
        .await
        .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not write broker request"))?;
    let mut reader = BufReader::new(stream).take((config.max_payload_bytes + 1) as u64);
    let mut response = Vec::new();
    reader
        .read_until(b'\n', &mut response)
        .await
        .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not read broker response"))?;
    if response.len() > config.max_payload_bytes {
        return Err(FleetError::new(
            FleetErrorKind::Oversized,
            "broker response exceeded size limit",
        ));
    }
    if response.ends_with(b"\n") {
        response.pop();
    }
    Ok(response)
}

pub async fn serve_native_host<R, W>(
    config: NativeHostBridgeConfig,
    reader: &mut R,
    writer: &mut W,
) -> Result<()>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    while let Some(payload) = read_native_message(reader, config.max_payload_bytes).await? {
        let response = match forward_native_payload(&config, &payload).await {
            Ok(response) => response,
            Err(error) => serde_json::to_vec(&serde_json::json!({
                "ok": false,
                "error": {"kind": error_kind_name(error.kind()), "message": error.to_string()}
            }))
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode native error"))?,
        };
        write_native_message(writer, &response, config.max_payload_bytes).await?;
    }
    Ok(())
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
    pub authority_socket_path: Option<PathBuf>,
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
        payload: serde_json::Value,
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
            payload: serde_json::Value::Null,
        }
    }

    pub fn action_with_payload(
        id: impl Into<String>,
        auth: String,
        target: TargetRef,
        action: Action,
        deadline: Duration,
        payload: serde_json::Value,
    ) -> Self {
        Self::Action {
            id: id.into(),
            auth,
            target,
            action,
            deadline,
            payload,
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
    targets: Vec<ManagedTarget>,
}

#[derive(Clone, Debug)]
pub struct ManagedTarget {
    reference: TargetRef,
    websocket_url: Option<String>,
    origin: String,
    context: Context,
}

impl ManagedTarget {
    pub fn new(reference: TargetRef, websocket_url: String, url: String) -> Result<Self> {
        if !is_loopback_websocket_url(&websocket_url) {
            return Err(FleetError::new(
                FleetErrorKind::UntrustedEndpoint,
                "managed target websocket must be loopback",
            ));
        }
        Ok(Self::with_optional_websocket(
            reference,
            Some(websocket_url),
            url,
        ))
    }

    fn from_reference(reference: TargetRef) -> Self {
        Self {
            reference,
            websocket_url: None,
            origin: "local://unmanaged".to_string(),
            context: Context::Ordinary,
        }
    }

    fn with_optional_websocket(
        reference: TargetRef,
        websocket_url: Option<String>,
        url: String,
    ) -> Self {
        Self {
            reference,
            websocket_url,
            origin: origin_for_url(&url),
            context: context_for_url(&url),
        }
    }

    pub fn reference(&self) -> &TargetRef {
        &self.reference
    }
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
            targets: targets
                .into_iter()
                .map(ManagedTarget::from_reference)
                .collect(),
        }
    }

    pub fn managed(
        browser: BrowserKind,
        profile: impl Into<String>,
        targets: Vec<ManagedTarget>,
    ) -> Self {
        Self {
            browser,
            profile: profile.into(),
            targets,
        }
    }

    pub fn targets(&self) -> Vec<TargetRef> {
        self.targets
            .iter()
            .map(|target| target.reference.clone())
            .collect()
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
    authority_listener: Option<UnixListener>,
    generation: u64,
    targets: BTreeMap<String, ManagedTarget>,
    replay_guard: MutationReplayGuard,
    policy: PolicyEngine,
    lease_counter: u64,
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
        let authority_listener = if let Some(path) = &config.authority_socket_path {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|_| {
                    FleetError::new(
                        FleetErrorKind::Io,
                        "could not create authority socket directory",
                    )
                })?;
            }
            let _ = fs::remove_file(path);
            let listener = UnixListener::bind(path).map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not bind authority socket")
            })?;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|_| {
                FleetError::new(
                    FleetErrorKind::Io,
                    "could not restrict authority socket permissions",
                )
            })?;
            Some(listener)
        } else {
            None
        };
        Ok(Self {
            config,
            listener,
            authority_listener,
            generation: 0,
            targets: BTreeMap::new(),
            replay_guard: MutationReplayGuard::default(),
            policy: PolicyEngine::new(),
            lease_counter: 0,
        })
    }

    pub fn apply_inventory(&mut self, update: InventoryUpdate) -> Result<()> {
        self.generation = self.generation.saturating_add(1);
        let prefix = format!("{:?}:{}", update.browser, update.profile);
        let prefix_with_separator = format!("{prefix}:");
        self.targets
            .retain(|key, _| !key.starts_with(&prefix_with_separator));
        for mut target in update.targets {
            target.reference.generation = self.generation;
            self.targets.insert(
                format!(
                    "{}:{}:{}",
                    prefix, target.reference.window_id, target.reference.tab_id
                ),
                target,
            );
        }
        for target in self.targets.values_mut() {
            target.reference.generation = self.generation;
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
                    targets: self
                        .targets
                        .values()
                        .map(|target| target.reference.clone())
                        .collect(),
                })
            }
            FleetRequest::Action {
                target,
                action,
                payload,
                ..
            } => {
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
                let state = self.target_state(target).ok_or_else(|| {
                    FleetError::new(FleetErrorKind::StaleGeneration, "target is not connected")
                })?;
                let policy_target = policy_target_for(&state);
                match self
                    .policy
                    .authorize(request.id(), *action, &policy_target, now_seconds())
                {
                    Decision::Allow => self.execute_cdp_action(state, *action, payload).await,
                    Decision::RequireLocalApproval => Err(FleetError::new(
                        FleetErrorKind::ApprovalRequired,
                        "local approval is required",
                    )),
                    Decision::Deny(Denial::HardDenied(_)) => Err(FleetError::new(
                        FleetErrorKind::HardDenied,
                        "target is blocked by hard-deny policy",
                    )),
                    Decision::Deny(Denial::EmergencyStop) => Err(FleetError::new(
                        FleetErrorKind::EmergencyStop,
                        "Mac browser fleet emergency stop is active",
                    )),
                    Decision::Deny(_) => Err(FleetError::new(
                        FleetErrorKind::ApprovalRequired,
                        "local approval is required",
                    )),
                }
            }
        }
    }

    fn target_state(&self, target: &TargetRef) -> Option<ManagedTarget> {
        self.targets
            .values()
            .find(|candidate| {
                candidate.reference.browser_id == target.browser_id
                    && candidate.reference.window_id == target.window_id
                    && candidate.reference.tab_id == target.tab_id
                    && candidate.reference.generation == target.generation
            })
            .cloned()
    }

    async fn execute_cdp_action(
        &self,
        target: ManagedTarget,
        action: Action,
        payload: &serde_json::Value,
    ) -> Result<FleetResponse> {
        let websocket_url = target.websocket_url.ok_or_else(|| {
            FleetError::new(
                FleetErrorKind::UnsupportedCapability,
                "target does not expose a managed CDP websocket",
            )
        })?;
        let mut client = CdpClient::connect(&websocket_url).await?;
        match action {
            Action::Navigate => {
                let url = payload
                    .get("url")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        FleetError::new(FleetErrorKind::Malformed, "navigate requires url")
                    })?;
                client.call("Page.enable", serde_json::json!({})).await?;
                client
                    .call("Page.navigate", serde_json::json!({ "url": url }))
                    .await?;
                client.call("Page.disable", serde_json::json!({})).await?;
            }
            Action::Click => {
                let x = payload
                    .get("x")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                let y = payload
                    .get("y")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                client.call("Input.dispatchMouseEvent", serde_json::json!({"type":"mousePressed","x":x,"y":y,"button":"left","clickCount":1})).await?;
                client.call("Input.dispatchMouseEvent", serde_json::json!({"type":"mouseReleased","x":x,"y":y,"button":"left","clickCount":1})).await?;
            }
            Action::Type => {
                if let Some(text) = payload.get("text").and_then(serde_json::Value::as_str) {
                    client
                        .call("Input.insertText", serde_json::json!({ "text": text }))
                        .await?;
                } else if let Some(key) = payload.get("key").and_then(serde_json::Value::as_str) {
                    client
                        .call(
                            "Input.dispatchKeyEvent",
                            serde_json::json!({"type":"keyDown","key":key}),
                        )
                        .await?;
                    client
                        .call(
                            "Input.dispatchKeyEvent",
                            serde_json::json!({"type":"keyUp","key":key}),
                        )
                        .await?;
                } else {
                    return Err(FleetError::new(
                        FleetErrorKind::Malformed,
                        "type requires text or key",
                    ));
                }
            }
            _ => {
                return Err(FleetError::new(
                    FleetErrorKind::UnsupportedCapability,
                    "action is not supported by managed CDP execution",
                ));
            }
        }
        Ok(FleetResponse::Accepted)
    }

    pub async fn serve(mut self) -> Result<()> {
        loop {
            enum Accepted {
                Peer(UnixStream),
                Authority(UnixStream),
            }
            let accepted = if let Some(authority_listener) = &self.authority_listener {
                tokio::select! {
                    peer = self.listener.accept() => Accepted::Peer(peer.map_err(|_| FleetError::new(FleetErrorKind::Io, "could not accept fleet connection"))?.0),
                    authority = authority_listener.accept() => Accepted::Authority(authority.map_err(|_| FleetError::new(FleetErrorKind::Io, "could not accept authority connection"))?.0),
                }
            } else {
                let (stream, _) = self.listener.accept().await.map_err(|_| {
                    FleetError::new(FleetErrorKind::Io, "could not accept fleet connection")
                })?;
                Accepted::Peer(stream)
            };
            match accepted {
                Accepted::Peer(stream) => self.serve_connection(stream).await?,
                Accepted::Authority(stream) => self.serve_authority_connection(stream).await?,
            }
        }
    }

    pub async fn serve_authority_connection(&mut self, stream: UnixStream) -> Result<()> {
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half).take((self.config.max_payload_bytes + 1) as u64);
        let mut line = Vec::new();
        reader
            .read_until(b'\n', &mut line)
            .await
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not read authority request"))?;
        let response = match self.handle_authority_bytes(&line) {
            Ok(result) => serde_json::json!({"ok": true, "result": result}),
            Err(error) => serde_json::json!({
                "ok": false,
                "error": {"kind": error_kind_name(error.kind()), "message": error.to_string()}
            }),
        };
        let mut encoded = serde_json::to_vec(&response).map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "could not encode authority response")
        })?;
        encoded.push(b'\n');
        write_half.write_all(&encoded).await.map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "could not write authority response")
        })?;
        Ok(())
    }

    fn handle_authority_bytes(&mut self, bytes: &[u8]) -> Result<serde_json::Value> {
        if bytes.len() > self.config.max_payload_bytes {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "authority request exceeded size limit",
            ));
        }
        let request: AuthorityEnvelope = serde_json::from_slice(bytes).map_err(|_| {
            FleetError::new(FleetErrorKind::Malformed, "authority request was malformed")
        })?;
        if request.version != 1 {
            return Err(FleetError::new(
                FleetErrorKind::UnsupportedVersion,
                "unsupported authority protocol version",
            ));
        }
        match request.action {
            AuthorityAction::GrantLease => {
                let target = request.target.ok_or_else(|| {
                    FleetError::new(FleetErrorKind::Malformed, "grant requires target")
                })?;
                if target.generation != self.generation {
                    return Err(FleetError::new(
                        FleetErrorKind::StaleGeneration,
                        "lease target generation is stale",
                    ));
                }
                let state = self.target_state(&target).ok_or_else(|| {
                    FleetError::new(
                        FleetErrorKind::StaleGeneration,
                        "lease target is not connected",
                    )
                })?;
                let mut actions = BTreeSet::new();
                for capability in request.capabilities {
                    actions.insert(action_for_capability(capability)?);
                }
                if actions.is_empty() {
                    return Err(FleetError::new(
                        FleetErrorKind::UnsupportedCapability,
                        "grant requires at least one mutation capability",
                    ));
                }
                self.lease_counter = self.lease_counter.saturating_add(1);
                let lease_id = format!("mac-lease-{}-{}", self.generation, self.lease_counter);
                let lease = Lease::with_id(
                    lease_id.clone(),
                    Scope::for_target(&policy_target_for(&state)),
                    actions,
                    request.duration_seconds.unwrap_or(60),
                );
                self.policy
                    .issue_lease(lease, now_seconds())
                    .map_err(fleet_error_for_denial)?;
                Ok(serde_json::json!({"leaseId": lease_id, "generation": self.generation}))
            }
            AuthorityAction::RevokeLease => {
                let lease_id = request.lease_id.ok_or_else(|| {
                    FleetError::new(FleetErrorKind::Malformed, "revoke requires leaseId")
                })?;
                Ok(serde_json::json!({"revoked": self.policy.revoke_lease(&lease_id)}))
            }
            AuthorityAction::EmergencyStop => {
                self.policy.activate_emergency_stop();
                Ok(serde_json::json!({"emergencyStop": true}))
            }
            AuthorityAction::ReleaseEmergencyStop => {
                self.policy.release_emergency_stop_locally();
                Ok(serde_json::json!({"emergencyStop": false}))
            }
            AuthorityAction::Status => Ok(serde_json::json!({"generation": self.generation})),
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
    #[serde(default)]
    url: Option<String>,
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
        let mut reader = BufReader::new(stream);
        let mut headers = Vec::new();
        loop {
            if headers.len() >= 16 * 1024 {
                return Err(FleetError::new(
                    FleetErrorKind::Oversized,
                    "managed CDP response headers exceeded size limit",
                ));
            }
            let read = reader.read_until(b'\n', &mut headers).await.map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not read managed CDP inventory")
            })?;
            if read == 0 || headers.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        if !headers.starts_with(b"HTTP/1.1 200") && !headers.starts_with(b"HTTP/1.0 200") {
            return Err(FleetError::new(
                FleetErrorKind::Io,
                "managed CDP endpoint returned an error",
            ));
        }
        let header_text = std::str::from_utf8(&headers).map_err(|_| {
            FleetError::new(
                FleetErrorKind::Malformed,
                "managed CDP response headers were malformed",
            )
        })?;
        let content_length = header_text
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .ok_or_else(|| {
                FleetError::new(
                    FleetErrorKind::Malformed,
                    "managed CDP response omitted Content-Length",
                )
            })?;
        if content_length > self.max_response_bytes {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "managed CDP inventory exceeded size limit",
            ));
        }
        let mut body = vec![0; content_length];
        reader.read_exact(&mut body).await.map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "could not read managed CDP inventory")
        })?;
        let targets: Vec<CdpHttpTarget> = serde_json::from_slice(&body).map_err(|_| {
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
            .filter_map(|target| {
                ManagedTarget::new(
                    TargetRef {
                        browser_id: browser_id.to_string(),
                        window_id: "managed-cdp".to_string(),
                        tab_id: target.id,
                        generation: 0,
                    },
                    target.web_socket_debugger_url?,
                    target.url.unwrap_or_else(|| "about:blank".to_string()),
                )
                .ok()
            })
            .collect();
        Ok(InventoryUpdate::managed(
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

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn policy_target_for(target: &ManagedTarget) -> PolicyTarget {
    PolicyTarget {
        browser: target.reference.browser_id.clone(),
        profile: target.reference.window_id.clone(),
        tab: target.reference.tab_id.clone(),
        origin: target.origin.clone(),
        generation: target.reference.generation,
        context: target.context,
    }
}

fn origin_for_url(raw: &str) -> String {
    Url::parse(raw)
        .map(|url| url.origin().ascii_serialization())
        .unwrap_or_else(|_| raw.split('/').next().unwrap_or("about:blank").to_string())
}

fn context_for_url(raw: &str) -> Context {
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("chrome://") || lower.starts_with("edge://") || lower.starts_with("about:")
    {
        return Context::PrivilegedBrowserUrl;
    }
    if lower.contains("/password") || lower.contains("passwords") {
        return Context::PasswordManager;
    }
    Context::Ordinary
}

fn action_for_capability(capability: Capability) -> Result<Action> {
    match capability {
        Capability::Navigate => Ok(Action::Navigate),
        Capability::Click => Ok(Action::Click),
        Capability::Type | Capability::Press => Ok(Action::Type),
        Capability::Inventory | Capability::RichInspection | Capability::Evaluate => {
            Err(FleetError::new(
                FleetErrorKind::UnsupportedCapability,
                "authority grants only support mutation capabilities",
            ))
        }
    }
}

fn fleet_error_for_denial(denial: Denial) -> FleetError {
    match denial {
        Denial::LeaseDurationExceeded => FleetError::new(
            FleetErrorKind::UnsupportedCapability,
            "lease duration exceeds local policy maximum",
        ),
        Denial::HardDenied(_) => FleetError::new(FleetErrorKind::HardDenied, "hard-denied target"),
        Denial::EmergencyStop => FleetError::new(FleetErrorKind::EmergencyStop, "emergency stop"),
        Denial::RemoteAuthorityOperation => {
            FleetError::new(FleetErrorKind::ApprovalRequired, "remote authority denied")
        }
    }
}

struct CdpClient {
    stream: TcpStream,
    next_id: u64,
}

impl CdpClient {
    async fn connect(raw: &str) -> Result<Self> {
        if !is_loopback_websocket_url(raw) {
            return Err(FleetError::new(
                FleetErrorKind::UntrustedEndpoint,
                "managed CDP websocket must remain loopback",
            ));
        }
        let url = Url::parse(raw).map_err(|_| {
            FleetError::new(
                FleetErrorKind::UntrustedEndpoint,
                "CDP websocket URL is invalid",
            )
        })?;
        if url.scheme() == "wss" {
            return Err(FleetError::new(
                FleetErrorKind::UnsupportedCapability,
                "managed CDP execution currently supports local ws endpoints only",
            ));
        }
        let host = url.host_str().expect("validated host");
        let port = url.port_or_known_default().expect("validated port");
        let mut stream = TcpStream::connect((host, port)).await.map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "managed CDP websocket is unavailable")
        })?;
        let path = if url.path().is_empty() {
            "/"
        } else {
            url.path()
        };
        let authority = if host.contains(':') {
            format!("[{host}]:{port}")
        } else {
            format!("{host}:{port}")
        };
        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {authority}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).await.map_err(|_| {
            FleetError::new(FleetErrorKind::Io, "could not open managed CDP websocket")
        })?;
        let mut headers = Vec::new();
        loop {
            if headers.len() > 16 * 1024 {
                return Err(FleetError::new(
                    FleetErrorKind::Oversized,
                    "managed CDP websocket handshake exceeded size limit",
                ));
            }
            let mut byte = [0_u8; 1];
            stream.read_exact(&mut byte).await.map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not read CDP websocket handshake")
            })?;
            headers.push(byte[0]);
            if headers.ends_with(b"\r\n\r\n") {
                break;
            }
        }
        if !headers.starts_with(b"HTTP/1.1 101") && !headers.starts_with(b"HTTP/1.0 101") {
            return Err(FleetError::new(
                FleetErrorKind::Io,
                "managed CDP websocket handshake failed",
            ));
        }
        Ok(Self { stream, next_id: 0 })
    }

    async fn call(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id;
        let request = serde_json::json!({"id": id, "method": method, "params": params});
        self.write_text(&request.to_string()).await?;
        self.read_text().await.and_then(|text| {
            serde_json::from_str(&text)
                .map_err(|_| FleetError::new(FleetErrorKind::Malformed, "CDP response malformed"))
        })
    }

    async fn write_text(&mut self, text: &str) -> Result<()> {
        let payload = text.as_bytes();
        let mut frame = vec![0x81];
        if payload.len() < 126 {
            frame.push(0x80 | payload.len() as u8);
        } else if payload.len() <= u16::MAX as usize {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        } else {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "CDP request exceeded websocket frame limit",
            ));
        }
        let mask = [0x4a, 0x43, 0x4f, 0x44];
        frame.extend_from_slice(&mask);
        frame.extend(
            payload
                .iter()
                .enumerate()
                .map(|(index, byte)| byte ^ mask[index % 4]),
        );
        self.stream
            .write_all(&frame)
            .await
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not write CDP request"))
    }

    async fn read_text(&mut self) -> Result<String> {
        let mut head = [0_u8; 2];
        self.stream
            .read_exact(&mut head)
            .await
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not read CDP response"))?;
        if head[0] & 0x0f != 1 {
            return Err(FleetError::new(
                FleetErrorKind::Malformed,
                "CDP response was not a text frame",
            ));
        }
        let mut len = usize::from(head[1] & 0x7f);
        if len == 126 {
            let mut ext = [0_u8; 2];
            self.stream.read_exact(&mut ext).await.map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not read CDP response length")
            })?;
            len = u16::from_be_bytes(ext) as usize;
        }
        if len > 1024 * 1024 {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "CDP response exceeded size limit",
            ));
        }
        let masked = head[1] & 0x80 != 0;
        let mut mask = [0_u8; 4];
        if masked {
            self.stream.read_exact(&mut mask).await.map_err(|_| {
                FleetError::new(FleetErrorKind::Io, "could not read CDP response mask")
            })?;
        }
        let mut payload = vec![0_u8; len];
        self.stream
            .read_exact(&mut payload)
            .await
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not read CDP response body"))?;
        if masked {
            for (index, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[index % 4];
            }
        }
        String::from_utf8(payload)
            .map_err(|_| FleetError::new(FleetErrorKind::Malformed, "CDP response was not UTF-8"))
    }
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
            Ok(FleetRequest::action_with_payload(
                envelope.id,
                envelope.auth,
                target,
                action,
                deadline,
                envelope.payload,
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
        FleetErrorKind::HardDenied => "hardDenied",
        FleetErrorKind::EmergencyStop => "emergencyStop",
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
