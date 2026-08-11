use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

/// Directory holding the Mac-local broker sockets.
///
/// Deliberately short: macOS caps `sockaddr_un.sun_path` at 104 bytes, so a
/// longer directory makes the broker fail to bind at runtime.
pub fn default_socket_dir(home: &Path) -> PathBuf {
    home.join(".jcode/mac-fleet")
}

/// Default broker socket the native host connects to.
pub fn default_broker_socket_path(home: &Path) -> PathBuf {
    default_socket_dir(home).join("broker.sock")
}

/// Default native-messaging secret path used to authenticate to the broker.
pub fn default_native_secret_path(home: &Path) -> PathBuf {
    home.join("Library/Application Support/Jcode/MacBrowserFleet/native.secret")
}

/// How the binary was invoked, after inspecting `argv[1..]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invocation {
    /// Run the broker with the remaining arguments.
    Broker,
    /// Run the Mac-local authority CLI.
    Authority,
    /// Serve native messaging over stdio.
    ///
    /// Chrome and Edge launch native hosts with the manifest path as the first
    /// argument and the calling origin as the second, so anything that is not
    /// an explicit subcommand must be treated as a browser-initiated launch
    /// rather than a usage error.
    NativeHost,
}

/// Classify a native-messaging launch without consuming the argument list.
pub fn classify_invocation<I, S>(args: I) -> Invocation
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    match args.into_iter().next() {
        None => Invocation::NativeHost,
        Some(first) => match first.as_ref().to_string_lossy().as_ref() {
            "broker" => Invocation::Broker,
            "authority" => Invocation::Authority,
            "native-host" => Invocation::NativeHost,
            _ => Invocation::NativeHost,
        },
    }
}

#[derive(Default)]
struct NativeHostSession {
    browser_kind: Option<BrowserKind>,
    profile_label: Option<String>,
    session_id: Option<String>,
    next_request: u64,
}

impl NativeHostSession {
    fn source_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "browserKind": self.browser_kind.unwrap_or(BrowserKind::Chrome),
            "profileLabel": self.profile_label.clone().unwrap_or_else(default_profile_label),
        })
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.next_request = self.next_request.saturating_add(1);
        format!(
            "native-{prefix}-{}-{}",
            self.session_id.as_deref().unwrap_or("unknown"),
            self.next_request
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
enum InternalWireAction {
    ExtensionInventorySnapshot,
    ExtensionDisconnect,
    ExtensionActionPoll,
    ExtensionActionResult,
}

#[derive(Clone, Debug, Deserialize)]
struct InternalEnvelope {
    version: u16,
    auth: String,
    id: String,
    action: InternalWireAction,
    #[serde(default)]
    payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionSnapshotPayload {
    snapshot: ExtensionSnapshot,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionSourcePayload {
    browser_kind: BrowserKind,
    #[serde(default = "default_profile_label")]
    profile_label: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionActionResultPayload {
    request_id: String,
    ok: bool,
    #[serde(default)]
    result: serde_json::Value,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionSnapshot {
    browser_kind: BrowserKind,
    #[serde(default = "default_profile_label")]
    profile_label: String,
    generation: u64,
    windows: Vec<ExtensionWindow>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionWindow {
    window_ref: String,
    native_window_id: u64,
    #[serde(default)]
    tabs: Vec<ExtensionTab>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionTab {
    tab_ref: String,
    #[serde(default)]
    window_ref: Option<String>,
    #[serde(default)]
    native_window_id: Option<u64>,
    native_tab_id: u64,
    #[serde(default)]
    url: Option<String>,
}

struct PendingExtensionAction {
    source_key: String,
    expires_at: Instant,
    message: serde_json::Value,
}

fn default_profile_label() -> String {
    "Default".to_string()
}

fn ordinary_browser_id(browser: BrowserKind) -> &'static str {
    match browser {
        BrowserKind::Chrome => "ordinary-chrome",
        BrowserKind::Edge => "ordinary-edge",
    }
}

fn extension_inventory_profile(profile_label: &str) -> String {
    format!("ordinary:{profile_label}")
}

fn extension_source_key(browser: BrowserKind, profile_label: &str) -> String {
    format!(
        "{:?}:{}",
        browser,
        extension_inventory_profile(profile_label)
    )
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

async fn forward_internal_request(
    config: &NativeHostBridgeConfig,
    id: String,
    action: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value> {
    let mut broker_payload = serde_json::to_vec(&serde_json::json!({
        "version": 1,
        "auth": config.secret,
        "id": id,
        "action": action,
        "payload": payload,
    }))
    .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode native broker request"))?;
    if broker_payload.len() > config.max_payload_bytes {
        return Err(FleetError::new(
            FleetErrorKind::Oversized,
            "native broker request exceeded size limit",
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
    let value: serde_json::Value = serde_json::from_slice(&response)
        .map_err(|_| FleetError::new(FleetErrorKind::Malformed, "broker response was malformed"))?;
    if value.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
        Ok(value
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    } else {
        Err(FleetError::new(
            FleetErrorKind::Io,
            "fleet broker rejected native request",
        ))
    }
}

fn native_error_payload(error: FleetError) -> Result<Vec<u8>> {
    serde_json::to_vec(&serde_json::json!({
        "ok": false,
        "error": {"kind": error_kind_name(error.kind()), "message": error.to_string()}
    }))
    .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode native error"))
}

async fn handle_extension_payload(
    config: &NativeHostBridgeConfig,
    session: &mut NativeHostSession,
    payload: &[u8],
) -> Result<Option<Vec<u8>>> {
    let message: serde_json::Value = serde_json::from_slice(payload)
        .map_err(|_| FleetError::new(FleetErrorKind::Malformed, "native request was malformed"))?;
    let Some(message_type) = message.get("type").and_then(serde_json::Value::as_str) else {
        return forward_native_payload(config, payload).await.map(Some);
    };

    match message_type {
        "hello" => {
            let protocol_version = message
                .get("protocolVersion")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| {
                    FleetError::new(FleetErrorKind::Malformed, "hello version is required")
                })?;
            if protocol_version != 1 {
                return Err(FleetError::new(
                    FleetErrorKind::UnsupportedVersion,
                    "unsupported native host protocol version",
                ));
            }
            let browser_kind: BrowserKind =
                serde_json::from_value(message.get("browserKind").cloned().ok_or_else(|| {
                    FleetError::new(FleetErrorKind::Malformed, "browser kind is required")
                })?)
                .map_err(|_| {
                    FleetError::new(FleetErrorKind::Malformed, "browser kind is invalid")
                })?;
            let session_id = message
                .get("sessionId")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| {
                    FleetError::new(FleetErrorKind::Malformed, "session id is required")
                })?
                .to_string();
            session.browser_kind = Some(browser_kind);
            session.profile_label = message
                .get("profileLabel")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            session.session_id = Some(session_id.clone());
            serde_json::to_vec(&serde_json::json!({
                "type": "hello_ack",
                "protocolVersion": 1,
                "sessionId": session_id,
            }))
            .map(Some)
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode hello ack"))
        }
        "inventory_snapshot" => {
            let snapshot = message.get("snapshot").cloned().ok_or_else(|| {
                FleetError::new(FleetErrorKind::Malformed, "inventory snapshot is required")
            })?;
            if let Ok(parsed) = serde_json::from_value::<ExtensionSnapshot>(snapshot.clone()) {
                session.browser_kind = Some(parsed.browser_kind);
                session.profile_label = Some(parsed.profile_label);
            }
            let id = session.next_id("snapshot");
            let _ = forward_internal_request(
                config,
                id,
                "extensionInventorySnapshot",
                serde_json::json!({"snapshot": snapshot}),
            )
            .await?;
            Ok(None)
        }
        "inventory_delta" => {
            let request_id = session.next_id("inventory-refresh");
            serde_json::to_vec(&serde_json::json!({
                "type": "inventory_request",
                "requestId": request_id,
            }))
            .map(Some)
            .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode inventory request"))
        }
        "action_poll" => {
            let id = message
                .get("requestId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| session.next_id("poll"));
            let result = forward_internal_request(
                config,
                id,
                "extensionActionPoll",
                session.source_payload(),
            )
            .await?;
            serde_json::to_vec(&result)
                .map(Some)
                .map_err(|_| FleetError::new(FleetErrorKind::Io, "could not encode poll response"))
        }
        "action_response" => {
            let request_id = message
                .get("requestId")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let _ = forward_internal_request(
                config,
                session.next_id("action-result"),
                "extensionActionResult",
                serde_json::json!({
                    "requestId": request_id,
                    "ok": message.get("ok").and_then(serde_json::Value::as_bool).unwrap_or(false),
                    "result": message.get("result").cloned().unwrap_or_else(|| serde_json::json!({})),
                    "error": message.get("error").cloned(),
                }),
            )
            .await?;
            Ok(None)
        }
        _ => Err(FleetError::new(
            FleetErrorKind::Malformed,
            "native message type is not supported",
        )),
    }
}

async fn disconnect_extension_session(
    config: &NativeHostBridgeConfig,
    session: &mut NativeHostSession,
) {
    let Some(browser_kind) = session.browser_kind else {
        return;
    };
    let profile_label = session
        .profile_label
        .clone()
        .unwrap_or_else(default_profile_label);
    let _ = forward_internal_request(
        config,
        session.next_id("disconnect"),
        "extensionDisconnect",
        serde_json::json!({"browserKind": browser_kind, "profileLabel": profile_label}),
    )
    .await;
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
    let mut session = NativeHostSession::default();
    while let Some(payload) = read_native_message(reader, config.max_payload_bytes).await? {
        let response = match handle_extension_payload(&config, &mut session, &payload).await {
            Ok(response) => response,
            Err(error) => Some(native_error_payload(error)?),
        };
        if let Some(response) = response {
            write_native_message(writer, &response, config.max_payload_bytes).await?;
        }
    }
    disconnect_extension_session(&config, &mut session).await;
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
    pub native_secret: Option<String>,
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
    extension_source_key: Option<String>,
    native_window_id: Option<u64>,
    native_tab_id: Option<u64>,
    extension_generation: Option<u64>,
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
            extension_source_key: None,
            native_window_id: None,
            native_tab_id: None,
            extension_generation: None,
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
            extension_source_key: None,
            native_window_id: None,
            native_tab_id: None,
            extension_generation: None,
            origin: origin_for_url(&url),
            context: context_for_url(&url),
        }
    }

    fn extension(
        reference: TargetRef,
        source_key: String,
        native_window_id: u64,
        native_tab_id: u64,
        extension_generation: u64,
        url: String,
    ) -> Self {
        Self {
            reference,
            websocket_url: None,
            extension_source_key: Some(source_key),
            native_window_id: Some(native_window_id),
            native_tab_id: Some(native_tab_id),
            extension_generation: Some(extension_generation),
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
    pending_extension_actions: VecDeque<PendingExtensionAction>,
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
            pending_extension_actions: VecDeque::new(),
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

    fn apply_extension_snapshot(
        &mut self,
        snapshot: ExtensionSnapshot,
    ) -> Result<serde_json::Value> {
        let source_key = extension_source_key(snapshot.browser_kind, &snapshot.profile_label);
        let browser_id = ordinary_browser_id(snapshot.browser_kind).to_string();
        let mut targets = Vec::new();
        for window in snapshot.windows {
            for tab in window.tabs {
                let window_ref = tab.window_ref.unwrap_or_else(|| window.window_ref.clone());
                let native_window_id = tab.native_window_id.unwrap_or(window.native_window_id);
                let url = tab.url.unwrap_or_else(|| "about:blank".to_string());
                targets.push(ManagedTarget::extension(
                    TargetRef {
                        browser_id: browser_id.clone(),
                        window_id: window_ref,
                        tab_id: tab.tab_ref,
                        generation: 0,
                    },
                    source_key.clone(),
                    native_window_id,
                    tab.native_tab_id,
                    snapshot.generation,
                    url,
                ));
            }
        }
        self.apply_inventory(InventoryUpdate::managed(
            snapshot.browser_kind,
            extension_inventory_profile(&snapshot.profile_label),
            targets,
        ))?;
        Ok(serde_json::json!({
            "generation": self.generation,
            "connectedTargets": self.targets.len(),
        }))
    }

    fn disconnect_extension_source(&mut self, source: ExtensionSourcePayload) -> serde_json::Value {
        let source_key = extension_source_key(source.browser_kind, &source.profile_label);
        let before = self.targets.len();
        self.targets
            .retain(|_, target| target.extension_source_key.as_deref() != Some(&source_key));
        self.pending_extension_actions
            .retain(|action| action.source_key != source_key);
        let removed = before.saturating_sub(self.targets.len());
        if removed > 0 {
            self.generation = self.generation.saturating_add(1);
            for target in self.targets.values_mut() {
                target.reference.generation = self.generation;
            }
        }
        serde_json::json!({"removedTargets": removed, "generation": self.generation})
    }

    fn handle_internal_bytes(&mut self, bytes: &[u8]) -> Result<serde_json::Value> {
        if bytes.len() > self.config.max_payload_bytes {
            return Err(FleetError::new(
                FleetErrorKind::Oversized,
                "internal request exceeded size limit",
            ));
        }
        let request: InternalEnvelope = serde_json::from_slice(bytes).map_err(|_| {
            FleetError::new(FleetErrorKind::Malformed, "internal request was malformed")
        })?;
        if request.version != 1 {
            return Err(FleetError::new(
                FleetErrorKind::UnsupportedVersion,
                "unsupported internal protocol version",
            ));
        }
        let Some(native_secret) = self.config.native_secret.as_deref() else {
            return Err(FleetError::new(
                FleetErrorKind::Unauthenticated,
                "internal authentication failed",
            ));
        };
        if request.auth != native_secret {
            return Err(FleetError::new(
                FleetErrorKind::Unauthenticated,
                "internal authentication failed",
            ));
        }
        if request.id.trim().is_empty() {
            return Err(FleetError::new(
                FleetErrorKind::Malformed,
                "internal request id is required",
            ));
        }
        match request.action {
            InternalWireAction::ExtensionInventorySnapshot => {
                let payload: ExtensionSnapshotPayload = serde_json::from_value(request.payload)
                    .map_err(|_| {
                        FleetError::new(
                            FleetErrorKind::Malformed,
                            "extension inventory snapshot was malformed",
                        )
                    })?;
                self.apply_extension_snapshot(payload.snapshot)
            }
            InternalWireAction::ExtensionDisconnect => {
                let payload: ExtensionSourcePayload = serde_json::from_value(request.payload)
                    .map_err(|_| {
                        FleetError::new(FleetErrorKind::Malformed, "extension source was malformed")
                    })?;
                Ok(self.disconnect_extension_source(payload))
            }
            InternalWireAction::ExtensionActionPoll => {
                let payload: ExtensionSourcePayload = serde_json::from_value(request.payload)
                    .map_err(|_| {
                        FleetError::new(FleetErrorKind::Malformed, "extension source was malformed")
                    })?;
                Ok(self.poll_extension_action(payload))
            }
            InternalWireAction::ExtensionActionResult => {
                let payload: ExtensionActionResultPayload = serde_json::from_value(request.payload)
                    .map_err(|_| {
                        FleetError::new(
                            FleetErrorKind::Malformed,
                            "extension action result was malformed",
                        )
                    })?;
                Ok(serde_json::json!({
                    "requestId": payload.request_id,
                    "ok": payload.ok,
                    "received": true,
                    "hasResult": !payload.result.is_null(),
                    "hasError": payload.error.is_some(),
                }))
            }
        }
    }

    fn prune_expired_extension_actions(&mut self) {
        let now = Instant::now();
        self.pending_extension_actions
            .retain(|action| action.expires_at > now);
    }

    fn queue_extension_action(
        &mut self,
        request_id: &str,
        target: ManagedTarget,
        action: Action,
        deadline: Duration,
        payload: &serde_json::Value,
    ) -> Result<FleetResponse> {
        self.prune_expired_extension_actions();
        if self.pending_extension_actions.len() >= self.config.max_in_flight {
            return Err(FleetError::new(
                FleetErrorKind::DeadlineExceeded,
                "too many extension actions are pending",
            ));
        }
        let source_key = target.extension_source_key.clone().ok_or_else(|| {
            FleetError::new(
                FleetErrorKind::UnsupportedCapability,
                "target is not connected through an ordinary extension",
            )
        })?;
        let native_window_id = target.native_window_id.ok_or_else(|| {
            FleetError::new(
                FleetErrorKind::UnsupportedCapability,
                "target has no native window id",
            )
        })?;
        let native_tab_id = target.native_tab_id.ok_or_else(|| {
            FleetError::new(
                FleetErrorKind::UnsupportedCapability,
                "target has no native tab id",
            )
        })?;
        let extension_action = match action {
            Action::Navigate => "navigate",
            _ => {
                return Err(FleetError::new(
                    FleetErrorKind::UnsupportedCapability,
                    "action is not supported by ordinary extension tabs",
                ));
            }
        };
        let mut extension_payload = serde_json::Map::new();
        if let Some(url) = payload.get("url").cloned() {
            extension_payload.insert("url".to_string(), url);
        }
        let message = serde_json::json!({
            "type": "action_request",
            "requestId": request_id,
            "generation": target.extension_generation.unwrap_or(target.reference.generation),
            "action": extension_action,
            "target": {"windowId": native_window_id, "tabId": native_tab_id},
            "payload": extension_payload,
        });
        self.pending_extension_actions
            .push_back(PendingExtensionAction {
                source_key,
                expires_at: Instant::now() + deadline,
                message,
            });
        Ok(FleetResponse::Accepted)
    }

    fn poll_extension_action(&mut self, source: ExtensionSourcePayload) -> serde_json::Value {
        self.prune_expired_extension_actions();
        let source_key = extension_source_key(source.browser_kind, &source.profile_label);
        let Some(index) = self
            .pending_extension_actions
            .iter()
            .position(|action| action.source_key == source_key)
        else {
            return serde_json::json!({"type": "action_idle"});
        };
        self.pending_extension_actions
            .remove(index)
            .map(|action| action.message)
            .unwrap_or_else(|| serde_json::json!({"type": "action_idle"}))
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
                deadline,
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
                    Decision::Allow => {
                        if state.websocket_url.is_some() {
                            self.execute_cdp_action(state, *action, payload).await
                        } else {
                            self.queue_extension_action(
                                request.id(),
                                state,
                                *action,
                                *deadline,
                                payload,
                            )
                        }
                    }
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

        let is_internal_request = serde_json::from_slice::<serde_json::Value>(&line)
            .ok()
            .and_then(|value| {
                value
                    .get("action")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
            })
            .is_some_and(|action| action.starts_with("extension"));
        let result: Result<serde_json::Value> = if is_internal_request {
            self.handle_internal_bytes(&line)
        } else {
            let codec = ProtocolCodec::new(
                vec![1],
                self.config.max_payload_bytes,
                self.config.secret.clone(),
            );
            match codec.decode_request(&line).and_then(envelope_to_request) {
                Ok(request) => self.handle(request).await.and_then(|response| {
                    serde_json::to_value(response).map_err(|_| {
                        FleetError::new(FleetErrorKind::Io, "could not encode fleet response")
                    })
                }),
                Err(error) => Err(error),
            }
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
