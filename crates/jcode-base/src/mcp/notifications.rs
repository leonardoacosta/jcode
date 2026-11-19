//! MCP server notification handling.
//!
//! # Contract
//!
//! Every notification is an **invalidation hint**, never an instruction. The
//! engine re-reads authoritative state (catalogs, resources) from the same
//! server and keeps the last known state when a re-read fails. Payloads are
//! never forwarded to an agent: a wake carries only a server name and a
//! [`NotificationClass`].
//!
//! # Version split (never mixed)
//!
//! * **2025-11-25 (legacy):** `resources/subscribe` only when the server
//!   advertises `resources.subscribe`; `listChanged` flags per feature.
//! * **2026-07-28 (modern):** one `subscriptions/listen` stream with explicit
//!   filters, correlated by `io.modelcontextprotocol/subscriptionId`.
//!   `resources/subscribe` is never sent. Only a [`ListenTransport`] that can
//!   hold a long-lived response stream enables it. The built-in stdio and
//!   Streamable HTTP transports do not implement it yet, so they report it
//!   inactive; the path is exercised with transport fakes.
//! * Any other version enables nothing.
//!
//! # Dispatch seam
//!
//! Transports call [`InboundSlot::offer`] for every server-originated message
//! that is not a response (`RemoteState::dispatch` and the stdio reader). The
//! slot never blocks and never grows: when the pump falls behind, messages are
//! counted as dropped and the engine reconciles from authoritative state.
//! [`NotificationEngine::ingest`] is the single place that classifies a
//! message. A later change that adds a method family (MCP Tasks, board
//! events) extends [`NotificationClass`] and [`parse`], and inherits the
//! allowlist, coalescing and wake plumbing.
//!
//! # Bounds
//!
//! Each server owns one [`Dispatcher`]: pending hints are keyed (duplicates
//! coalesce), the queue is capped, and an event-rate cap applies. Excess
//! hints are dropped deterministically (newest first) and flag one full
//! reconcile. Servers never share a queue, so one noisy server cannot starve
//! another, and refresh requests use ordinary concurrent MCP requests.

use super::protocol::{JsonRpcResponse, ServerCapabilities};
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{Notify, mpsc};

/// Protocol revision that uses `resources/subscribe` and `listChanged`.
pub const LEGACY_VERSION: &str = "2025-11-25";
/// Protocol revision that uses `subscriptions/listen`.
pub const MODERN_VERSION: &str = "2026-07-28";

const SUBSCRIPTION_ID_KEY: &str = "io.modelcontextprotocol/subscriptionId";
const ACK_METHOD: &str = "notifications/subscriptions/acknowledged";
const MAX_URI_BYTES: usize = 2048;
const MAX_LIST_PAGES: usize = 64;
const MAX_LIST_ITEMS: usize = 5_000;
const MAX_RESOURCE_TEXT_BYTES: usize = 256 * 1024;
const MAX_ERROR_CHARS: usize = 300;
/// Capacity of the transport-to-engine channel.
const INBOUND_CAPACITY: usize = 256;
/// Default per-server pending-hint cap.
pub const DEFAULT_MAX_PENDING: usize = 64;
/// Default per-server events accepted per rolling minute.
pub const DEFAULT_MAX_PER_MINUTE: u32 = 120;
/// Default minimum gap between agent wakes for one server.
pub const DEFAULT_WAKE_COOLDOWN_SECS: u64 = 60;
/// Longest server name carried into a model-visible wake or queue id.
pub const MAX_WAKE_NAME_CHARS: usize = 64;

// ---------------------------------------------------------------------------
// Classes and configuration
// ---------------------------------------------------------------------------

/// A notification kind jcode understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationClass {
    ToolsListChanged,
    PromptsListChanged,
    ResourcesListChanged,
    ResourceUpdated,
}

impl NotificationClass {
    pub const fn method(self) -> &'static str {
        match self {
            Self::ToolsListChanged => "notifications/tools/list_changed",
            Self::PromptsListChanged => "notifications/prompts/list_changed",
            Self::ResourcesListChanged => "notifications/resources/list_changed",
            Self::ResourceUpdated => "notifications/resources/updated",
        }
    }

    pub fn from_method(method: &str) -> Option<Self> {
        [
            Self::ToolsListChanged,
            Self::PromptsListChanged,
            Self::ResourcesListChanged,
            Self::ResourceUpdated,
        ]
        .into_iter()
        .find(|class| class.method() == method)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ToolsListChanged => "tools_list_changed",
            Self::PromptsListChanged => "prompts_list_changed",
            Self::ResourcesListChanged => "resources_list_changed",
            Self::ResourceUpdated => "resource_updated",
        }
    }

    /// Field name in the 2026-07-28 `subscriptions/listen` filter.
    const fn filter_key(self) -> &'static str {
        match self {
            Self::ToolsListChanged => "toolsListChanged",
            Self::PromptsListChanged => "promptsListChanged",
            Self::ResourcesListChanged => "resourcesListChanged",
            Self::ResourceUpdated => "resourceSubscriptions",
        }
    }
}

/// Per-server notification policy. Absent or empty means "opted out": no
/// subscription is opened and no notification is acted on.
///
/// `refresh` and `wake` are independent allowlists. A class in `wake` but not
/// in `refresh` wakes without re-reading, and the reverse refreshes silently.
/// `resource_updated` applies only to URIs listed in `resources`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpNotificationConfig {
    /// Classes allowed to refresh catalogs/resources.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refresh: Vec<NotificationClass>,
    /// Classes allowed to request an agent wake.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wake: Vec<NotificationClass>,
    /// Resource URIs that may be subscribed to, re-read and woken on.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<String>,
    /// Session that receives wakes. Absent routes wakes to the ambient queue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wake_session: Option<String>,
    /// Pending-hint cap for this server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_pending: Option<usize>,
    /// Events accepted per rolling minute for this server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_per_minute: Option<u32>,
    /// Minimum seconds between agent wakes for this server (default 60).
    /// Wakes that arrive sooner are deferred, never dropped silently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wake_cooldown_secs: Option<u64>,
}

impl McpNotificationConfig {
    pub fn refreshes(&self, class: NotificationClass) -> bool {
        self.refresh.contains(&class)
    }

    pub fn wakes(&self, class: NotificationClass) -> bool {
        self.wake.contains(&class)
    }

    pub fn wants(&self, class: NotificationClass) -> bool {
        self.refreshes(class) || self.wakes(class)
    }

    /// Whether anything is opted in.
    pub fn is_active(&self) -> bool {
        !self.refresh.is_empty() || !self.wake.is_empty()
    }

    fn resource_allowed(&self, uri: &str) -> bool {
        self.resources.iter().any(|allowed| allowed == uri)
    }

    fn wake_cooldown(&self) -> Duration {
        Duration::from_secs(
            self.wake_cooldown_secs
                .unwrap_or(DEFAULT_WAKE_COOLDOWN_SECS),
        )
    }

    fn limits(&self) -> Limits {
        Limits {
            max_pending: self.max_pending.unwrap_or(DEFAULT_MAX_PENDING).max(1),
            max_per_minute: self.max_per_minute.unwrap_or(DEFAULT_MAX_PER_MINUTE).max(1),
        }
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Why a server message was not accepted. Never carries message content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    /// Method is not a notification jcode handles.
    Unknown,
    /// Shape or parameters are invalid.
    Malformed,
    /// Class, version or stream state was not negotiated or acknowledged.
    NotNegotiated,
    /// Subscription ID does not match the open `subscriptions/listen`.
    WrongSubscription,
    /// Resource URI is not one of the configured, subscribed URIs.
    UnconfiguredResource,
}

/// A validated invalidation hint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationEvent {
    pub class: NotificationClass,
    /// Set only for [`NotificationClass::ResourceUpdated`].
    pub uri: Option<String>,
}

impl NotificationEvent {
    /// Coalescing key: one pending hint per catalog or resource.
    pub fn key(&self) -> String {
        match &self.uri {
            Some(uri) => format!("{}:{uri}", self.class.as_str()),
            None => self.class.as_str().to_string(),
        }
    }
}

fn valid_uri(uri: &str) -> bool {
    !uri.is_empty() && uri.len() <= MAX_URI_BYTES && !uri.chars().any(char::is_control)
}

/// Parse a JSON-RPC notification into an event plus its subscription ID.
/// Only the method, the resource URI and the subscription ID are read.
pub fn parse(msg: &Value) -> std::result::Result<(NotificationEvent, Option<Value>), Rejection> {
    let obj = msg.as_object().ok_or(Rejection::Malformed)?;
    if obj.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || obj.contains_key("id")
        || obj.contains_key("result")
        || obj.contains_key("error")
    {
        return Err(Rejection::Malformed);
    }
    let method = obj
        .get("method")
        .and_then(Value::as_str)
        .ok_or(Rejection::Malformed)?;
    let class = NotificationClass::from_method(method).ok_or(Rejection::Unknown)?;
    let params = match obj.get("params") {
        None | Some(Value::Null) => None,
        Some(Value::Object(map)) => Some(map),
        Some(_) => return Err(Rejection::Malformed),
    };
    let subscription = params
        .and_then(|p| p.get("_meta"))
        .and_then(|meta| meta.get(SUBSCRIPTION_ID_KEY))
        .cloned();
    let uri = if class == NotificationClass::ResourceUpdated {
        let uri = params
            .and_then(|p| p.get("uri"))
            .and_then(Value::as_str)
            .filter(|uri| valid_uri(uri))
            .ok_or(Rejection::Malformed)?;
        Some(uri.to_string())
    } else {
        None
    };
    Ok((NotificationEvent { class, uri }, subscription))
}

// ---------------------------------------------------------------------------
// Negotiation
// ---------------------------------------------------------------------------

/// Which notification contract a negotiated protocol version uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Era {
    /// 2025-11-25: `resources/subscribe` plus `listChanged` capabilities.
    Legacy,
    /// 2026-07-28: `subscriptions/listen`.
    Modern,
}

impl Era {
    pub fn from_version(version: &str) -> Option<Self> {
        match version {
            LEGACY_VERSION => Some(Self::Legacy),
            MODERN_VERSION => Some(Self::Modern),
            _ => None,
        }
    }
}

/// A requested notification that stays off, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inactive {
    pub what: String,
    pub reason: &'static str,
}

/// What to turn on for one connection. Pure function of version,
/// capabilities, config and transport ability.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NotificationPlan {
    pub era: Option<Era>,
    /// Legacy only: URIs to send `resources/subscribe` for.
    pub subscribe_uris: Vec<String>,
    /// Legacy only: list-changed classes the server advertised and we want.
    pub list_changed: BTreeSet<NotificationClass>,
    /// Modern only: filter for `subscriptions/listen`.
    pub listen_filter: Option<Value>,
    pub inactive: Vec<Inactive>,
}

const LIST_CLASSES: [NotificationClass; 3] = [
    NotificationClass::ToolsListChanged,
    NotificationClass::PromptsListChanged,
    NotificationClass::ResourcesListChanged,
];

fn list_changed_advertised(caps: &ServerCapabilities, class: NotificationClass) -> bool {
    match class {
        NotificationClass::ToolsListChanged => caps.tools.as_ref().is_some_and(|c| c.list_changed),
        NotificationClass::PromptsListChanged => {
            caps.prompts.as_ref().is_some_and(|c| c.list_changed)
        }
        NotificationClass::ResourcesListChanged => {
            caps.resources.as_ref().is_some_and(|c| c.list_changed)
        }
        NotificationClass::ResourceUpdated => false,
    }
}

impl NotificationPlan {
    pub fn negotiate(
        version: &str,
        caps: &ServerCapabilities,
        cfg: &McpNotificationConfig,
        transport_supports_listen: bool,
    ) -> Self {
        let mut plan = Self {
            era: Era::from_version(version),
            ..Self::default()
        };
        let wants_resources =
            cfg.wants(NotificationClass::ResourceUpdated) && !cfg.resources.is_empty();
        match plan.era {
            None => {
                if cfg.is_active() {
                    plan.inactive.push(Inactive {
                        what: "all".into(),
                        reason: "protocol version has no supported notification contract",
                    });
                }
            }
            Some(Era::Legacy) => {
                for class in LIST_CLASSES.into_iter().filter(|c| cfg.wants(*c)) {
                    if list_changed_advertised(caps, class) {
                        plan.list_changed.insert(class);
                    } else {
                        plan.inactive.push(Inactive {
                            what: class.as_str().into(),
                            reason: "server did not advertise listChanged",
                        });
                    }
                }
                if wants_resources {
                    if caps.resources.as_ref().is_some_and(|r| r.subscribe) {
                        plan.subscribe_uris = cfg
                            .resources
                            .iter()
                            .filter(|u| valid_uri(u))
                            .cloned()
                            .collect();
                    } else {
                        plan.inactive.push(Inactive {
                            what: NotificationClass::ResourceUpdated.as_str().into(),
                            reason: "server did not advertise resources.subscribe",
                        });
                    }
                }
            }
            Some(Era::Modern) => {
                let mut filter = serde_json::Map::new();
                for class in LIST_CLASSES.into_iter().filter(|c| cfg.wants(*c)) {
                    filter.insert(class.filter_key().into(), Value::Bool(true));
                }
                if wants_resources {
                    filter.insert(
                        NotificationClass::ResourceUpdated.filter_key().into(),
                        json!(
                            cfg.resources
                                .iter()
                                .filter(|u| valid_uri(u))
                                .collect::<Vec<_>>()
                        ),
                    );
                }
                if filter.is_empty() {
                    // nothing configured for this contract
                } else if transport_supports_listen {
                    plan.listen_filter = Some(Value::Object(filter));
                } else {
                    plan.inactive.push(Inactive {
                        what: "subscriptions/listen".into(),
                        reason: "transport cannot hold a subscriptions/listen stream",
                    });
                }
            }
        }
        plan
    }
}

/// Live acceptance state for one connection.
#[derive(Debug, Clone)]
pub struct NotificationSession {
    era: Option<Era>,
    active_classes: BTreeSet<NotificationClass>,
    active_uris: HashSet<String>,
    listen_id: Option<u64>,
    requested: Option<Value>,
    acked: bool,
}

impl NotificationSession {
    pub fn new(plan: &NotificationPlan) -> Self {
        Self {
            era: plan.era,
            active_classes: if plan.era == Some(Era::Legacy) {
                plan.list_changed.clone()
            } else {
                BTreeSet::new()
            },
            active_uris: HashSet::new(),
            listen_id: None,
            requested: plan.listen_filter.clone(),
            acked: false,
        }
    }

    /// Legacy: a `resources/subscribe` succeeded, so updates for `uri` count.
    pub fn mark_subscribed(&mut self, uri: &str) {
        if self.era == Some(Era::Legacy) {
            self.active_uris.insert(uri.to_string());
            self.active_classes
                .insert(NotificationClass::ResourceUpdated);
        }
    }

    /// Modern: a `subscriptions/listen` request with this JSON-RPC ID was sent.
    /// Nothing is accepted until its acknowledgment arrives.
    pub fn begin_listen(&mut self, id: u64) {
        self.listen_id = Some(id);
        self.acked = false;
        self.active_classes.clear();
        self.active_uris.clear();
    }

    pub fn is_active(&self, class: NotificationClass) -> bool {
        self.active_classes.contains(&class)
    }

    /// Whether updates for `uri` are currently subscribed.
    pub fn uri_active(&self, uri: &str) -> bool {
        self.active_uris.contains(uri)
    }

    fn subscription_matches(&self, value: Option<&Value>) -> bool {
        match (self.listen_id, value) {
            (Some(id), Some(value)) => value.as_u64() == Some(id),
            _ => false,
        }
    }

    /// Apply `notifications/subscriptions/acknowledged`. The acknowledged
    /// subset of the requested filter becomes active; omitted entries are
    /// returned as inactive. Duplicate or foreign acks are rejected.
    pub fn handle_ack(&mut self, msg: &Value) -> std::result::Result<Vec<Inactive>, Rejection> {
        if self.era != Some(Era::Modern) || self.acked || self.listen_id.is_none() {
            return Err(Rejection::NotNegotiated);
        }
        let params = msg
            .get("params")
            .and_then(Value::as_object)
            .ok_or(Rejection::Malformed)?;
        let id = params
            .get("_meta")
            .and_then(|meta| meta.get(SUBSCRIPTION_ID_KEY));
        if !self.subscription_matches(id) {
            return Err(Rejection::WrongSubscription);
        }
        let ack = params
            .get("notifications")
            .and_then(Value::as_object)
            .ok_or(Rejection::Malformed)?;
        let requested = self
            .requested
            .as_ref()
            .and_then(Value::as_object)
            .ok_or(Rejection::NotNegotiated)?;
        let mut omitted = Vec::new();
        for class in LIST_CLASSES {
            let key = class.filter_key();
            if requested.get(key) != Some(&Value::Bool(true)) {
                continue;
            }
            if ack.get(key) == Some(&Value::Bool(true)) {
                self.active_classes.insert(class);
            } else {
                omitted.push(Inactive {
                    what: class.as_str().into(),
                    reason: "server did not acknowledge this filter",
                });
            }
        }
        if let Some(requested_uris) = requested
            .get(NotificationClass::ResourceUpdated.filter_key())
            .and_then(Value::as_array)
        {
            let acked: HashSet<&str> = ack
                .get(NotificationClass::ResourceUpdated.filter_key())
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            for uri in requested_uris.iter().filter_map(Value::as_str) {
                if acked.contains(uri) {
                    self.active_uris.insert(uri.to_string());
                } else {
                    omitted.push(Inactive {
                        what: format!("resource_updated:{uri}"),
                        reason: "server did not acknowledge this filter",
                    });
                }
            }
            if !self.active_uris.is_empty() {
                self.active_classes
                    .insert(NotificationClass::ResourceUpdated);
            }
        }
        self.acked = true;
        Ok(omitted)
    }

    /// Validate a notification against what was negotiated and acknowledged.
    pub fn accept(&self, msg: &Value) -> std::result::Result<NotificationEvent, Rejection> {
        let (event, subscription) = parse(msg)?;
        match self.era {
            None => return Err(Rejection::NotNegotiated),
            Some(Era::Modern) => {
                if !self.acked {
                    return Err(Rejection::NotNegotiated);
                }
                if !self.subscription_matches(subscription.as_ref()) {
                    return Err(Rejection::WrongSubscription);
                }
            }
            Some(Era::Legacy) => {}
        }
        if !self.active_classes.contains(&event.class) {
            return Err(Rejection::NotNegotiated);
        }
        if let Some(uri) = &event.uri
            && !self.active_uris.contains(uri)
        {
            return Err(Rejection::UnconfiguredResource);
        }
        Ok(event)
    }
}

// ---------------------------------------------------------------------------
// Bounded per-server dispatcher
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_pending: usize,
    pub max_per_minute: u32,
}

/// What happened to an offered hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offer {
    Queued,
    /// An equal hint was already pending.
    Coalesced,
    /// Pending queue full: hint dropped, full reconcile flagged.
    Overflow,
    /// Rolling-minute cap reached: hint dropped, full reconcile flagged.
    RateLimited,
}

/// Keyed, bounded queue of invalidation hints for one server.
///
/// Policy, in order: (1) rate cap on every offered event, (2) coalesce by key,
/// (3) pending cap. A dropped hint sets `needs_reconcile`, so loss is repaired
/// by reading authoritative state rather than by replay.
#[derive(Debug)]
pub struct Dispatcher {
    limits: Limits,
    pending: VecDeque<NotificationEvent>,
    keys: HashSet<String>,
    recent: VecDeque<Instant>,
    needs_reconcile: bool,
}

impl Dispatcher {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            pending: VecDeque::new(),
            keys: HashSet::new(),
            recent: VecDeque::new(),
            needs_reconcile: false,
        }
    }

    pub fn offer(&mut self, event: NotificationEvent, now: Instant) -> Offer {
        while self
            .recent
            .front()
            .is_some_and(|t| now.saturating_duration_since(*t) >= Duration::from_secs(60))
        {
            self.recent.pop_front();
        }
        if self.recent.len() >= self.limits.max_per_minute as usize {
            self.needs_reconcile = true;
            return Offer::RateLimited;
        }
        self.recent.push_back(now);
        let key = event.key();
        if self.keys.contains(&key) {
            return Offer::Coalesced;
        }
        if self.pending.len() >= self.limits.max_pending {
            self.needs_reconcile = true;
            return Offer::Overflow;
        }
        self.keys.insert(key);
        self.pending.push_back(event);
        Offer::Queued
    }

    pub fn pop(&mut self) -> Option<NotificationEvent> {
        let event = self.pending.pop_front()?;
        self.keys.remove(&event.key());
        Some(event)
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Returns and clears the "hints were dropped" flag.
    pub fn take_needs_reconcile(&mut self) -> bool {
        std::mem::take(&mut self.needs_reconcile)
    }
}

// ---------------------------------------------------------------------------
// Transport seams
// ---------------------------------------------------------------------------

/// Count of inbound messages the slot had to discard, plus a wake-up so the
/// pump applies a final dropped burst even when no further message arrives.
#[derive(Default)]
pub(super) struct InboundDrops {
    count: AtomicUsize,
    notify: Notify,
}

impl InboundDrops {
    pub(super) fn record(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
        self.notify.notify_one();
    }

    pub(super) fn take(&self) -> usize {
        self.count.swap(0, Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(super) fn get(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

/// Lock-free hand-off from a transport reader to the engine.
#[derive(Clone)]
pub(super) struct InboundSink {
    tx: mpsc::Sender<Value>,
    dropped: Arc<InboundDrops>,
}

/// Late-bound inbound sink. Transports hold a clone from creation; the client
/// installs the sink after the handshake, only for opted-in servers.
#[derive(Clone, Default)]
pub(super) struct InboundSlot(Arc<RwLock<Option<InboundSink>>>);

impl InboundSlot {
    /// Offer a server-originated message. Never blocks. Without an installed
    /// sink (not opted in) the message is discarded.
    pub(super) fn offer(&self, msg: Value) {
        let guard = self.0.read().unwrap_or_else(|p| p.into_inner());
        if let Some(sink) = guard.as_ref()
            && sink.tx.try_send(msg).is_err()
        {
            sink.dropped.record();
        }
    }

    pub(super) fn install(&self) -> (mpsc::Receiver<Value>, Arc<InboundDrops>) {
        let (tx, rx) = mpsc::channel(INBOUND_CAPACITY);
        let dropped = Arc::new(InboundDrops::default());
        *self.0.write().unwrap_or_else(|p| p.into_inner()) = Some(InboundSink {
            tx,
            dropped: Arc::clone(&dropped),
        });
        (rx, dropped)
    }
}

/// How the engine talks to the server for refreshes and subscriptions.
#[async_trait]
pub trait McpBackend: Send + Sync {
    /// Ordinary MCP request; JSON-RPC errors surface as `Err`.
    async fn request(&self, method: &str, params: Option<Value>) -> Result<JsonRpcResponse>;
    /// Re-list tools into the owner's tool catalog.
    async fn refresh_tools(&self) -> Result<()>;
}

/// A transport able to keep a `subscriptions/listen` response stream open.
#[async_trait]
pub trait ListenTransport: Send + Sync {
    /// Open the stream by sending `request`; yields every inbound JSON value
    /// (acknowledgment, notifications, final response). An `Err` item ends the
    /// stream as an abrupt closure. Dropping the stream cancels it over HTTP.
    async fn open_listen(
        &self,
        id: u64,
        request: Value,
    ) -> Result<BoxStream<'static, Result<Value>>>;
    /// Cancel the subscription (`notifications/cancelled` on stdio).
    async fn cancel_listen(&self, id: u64);
}

/// How a `subscriptions/listen` run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenEnd {
    /// Nothing to listen for, or the transport cannot do it.
    Unsupported,
    /// Client cancelled.
    Cancelled,
    /// Server sent a completion result. No reconnect needed.
    Graceful,
    /// Transport dropped without a completion result. Caller should reconnect
    /// and call [`NotificationEngine::reconcile`].
    Abrupt,
    Failed(String),
}

// ---------------------------------------------------------------------------
// Wake routing
// ---------------------------------------------------------------------------

/// An agent-wake request. Identifiers only: never payload or resource content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeRequest {
    pub server: String,
    pub class: NotificationClass,
    /// Configured session target. `None` means the ambient queue.
    pub session: Option<String>,
}

/// What a [`WakeHandler`] did with a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WakeOutcome {
    Queued,
    /// An identical wake was already pending.
    Coalesced,
    /// Not delivered; the reason is reported, never silent.
    Dropped(&'static str),
}

/// Routes wake requests into the host's existing session/ambient machinery.
pub trait WakeHandler: Send + Sync {
    fn wake(&self, request: WakeRequest) -> WakeOutcome;
}

/// Bound a server name for model-visible wake text and queue ids: Unicode
/// alphanumerics plus `._-@/`, everything else (control characters, quotes,
/// newlines, spaces) becomes `_`, at most [`MAX_WAKE_NAME_CHARS`] characters.
pub fn sanitize_server_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .take(MAX_WAKE_NAME_CHARS)
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '@' | '/') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "unnamed".to_string()
    } else {
        cleaned
    }
}

/// Process-wide stream of "this server's tool catalog was re-read" events.
/// Hosts that mirror MCP tools elsewhere (the model-facing tool registry)
/// subscribe and re-register that server's tools. Carries only the server
/// name; subscribers re-read the catalog from the manager.
static CATALOG_CHANGES: std::sync::OnceLock<tokio::sync::broadcast::Sender<String>> =
    std::sync::OnceLock::new();

pub fn subscribe_tool_catalog_changes() -> tokio::sync::broadcast::Receiver<String> {
    CATALOG_CHANGES
        .get_or_init(|| tokio::sync::broadcast::channel(64).0)
        .subscribe()
}

fn publish_tool_catalog_change(server: &str) {
    if let Some(tx) = CATALOG_CHANGES.get() {
        let _ = tx.send(server.to_string());
    }
}

static WAKE_HANDLER: RwLock<Option<Arc<dyn WakeHandler>>> = RwLock::new(None);

/// Install (or clear) the process-wide wake handler. Without one, permitted
/// wakes fail closed: they are dropped, counted and reported.
pub fn set_wake_handler(handler: Option<Arc<dyn WakeHandler>>) {
    *WAKE_HANDLER.write().unwrap_or_else(|p| p.into_inner()) = handler;
}

fn global_wake_handler() -> Option<Arc<dyn WakeHandler>> {
    WAKE_HANDLER
        .read()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
}

// ---------------------------------------------------------------------------
// Observable status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationCounters {
    pub accepted: u64,
    pub coalesced: u64,
    pub dropped_overflow: u64,
    pub dropped_rate: u64,
    pub dropped_inbound: u64,
    pub rejected: u64,
    pub ignored: u64,
    pub refreshes: u64,
    pub refresh_failures: u64,
    /// Wakes handed to the handler and queued or coalesced by it.
    pub wakes: u64,
    /// Wakes held back by the per-server cooldown (delivered later).
    pub wakes_deferred: u64,
    /// Permitted wakes that were not delivered (no handler, ambient off, ...).
    pub wakes_dropped: u64,
}

/// Last read of a resource. Text is bounded; blobs are measured, not stored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceSnapshot {
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub text_truncated: bool,
    pub blob_bytes: Option<usize>,
}

/// Shared, observable result of notification handling for one server.
/// Values change only when a refresh succeeds; failures land in `errors`.
#[derive(Debug, Clone, Default)]
pub struct NotificationStatus {
    pub era: Option<&'static str>,
    pub inactive: Vec<Inactive>,
    pub prompts: Option<Vec<Value>>,
    pub resources: Option<Vec<Value>>,
    pub resource_contents: HashMap<String, ResourceSnapshot>,
    /// Latest refresh failure per target key.
    pub errors: HashMap<String, String>,
    pub counters: NotificationCounters,
}

pub type SharedNotificationStatus = Arc<Mutex<NotificationStatus>>;

fn truncate_error(error: &anyhow::Error) -> String {
    let text = format!("{error:#}");
    text.chars().take(MAX_ERROR_CHARS).collect()
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Outcome of [`NotificationEngine::ingest`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ingest {
    Offered(Offer),
    /// Subscription acknowledgment applied.
    Acknowledged,
    /// Valid but on neither allowlist.
    Ignored,
    Rejected(Rejection),
}

pub struct NotificationEngine<B: McpBackend> {
    server: String,
    backend: B,
    cfg: McpNotificationConfig,
    plan: NotificationPlan,
    session: NotificationSession,
    dispatcher: Dispatcher,
    status: SharedNotificationStatus,
    wake: Option<Arc<dyn WakeHandler>>,
    /// Minimum gap between wakes for this server.
    wake_cooldown: Duration,
    /// Classes waiting for the cooldown. A set, so repeats coalesce.
    pending_wakes: BTreeSet<NotificationClass>,
    last_wake: Option<Instant>,
}

impl<B: McpBackend> NotificationEngine<B> {
    pub fn new(
        server: impl Into<String>,
        backend: B,
        cfg: McpNotificationConfig,
        version: &str,
        caps: &ServerCapabilities,
        transport_supports_listen: bool,
        status: SharedNotificationStatus,
    ) -> Self {
        let plan = NotificationPlan::negotiate(version, caps, &cfg, transport_supports_listen);
        let session = NotificationSession::new(&plan);
        {
            let mut status = status.lock().unwrap_or_else(|p| p.into_inner());
            status.era = match plan.era {
                Some(Era::Legacy) => Some(LEGACY_VERSION),
                Some(Era::Modern) => Some(MODERN_VERSION),
                None => None,
            };
            status.inactive = plan.inactive.clone();
        }
        Self {
            server: server.into(),
            backend,
            dispatcher: Dispatcher::new(cfg.limits()),
            wake_cooldown: cfg.wake_cooldown(),
            pending_wakes: BTreeSet::new(),
            last_wake: None,
            cfg,
            plan,
            session,
            status,
            wake: None,
        }
    }

    /// Route wakes to `handler` instead of the process-wide one.
    pub fn with_wake_handler(mut self, handler: Arc<dyn WakeHandler>) -> Self {
        self.wake = Some(handler);
        self
    }

    /// Override the per-server wake cooldown (tests, hosts with other policy).
    pub fn with_wake_cooldown(mut self, cooldown: Duration) -> Self {
        self.wake_cooldown = cooldown;
        self
    }

    pub fn plan(&self) -> &NotificationPlan {
        &self.plan
    }

    fn with_status<T>(&self, f: impl FnOnce(&mut NotificationStatus) -> T) -> T {
        f(&mut self.status.lock().unwrap_or_else(|p| p.into_inner()))
    }

    /// Restore supported subscriptions for this connection, then reconcile.
    /// Legacy sends `resources/subscribe` per configured URI, only when the
    /// plan allows it. Modern subscriptions open through [`Self::run_listen`].
    pub async fn establish(&mut self) {
        for uri in self.plan.subscribe_uris.clone() {
            match self
                .backend
                .request("resources/subscribe", Some(json!({ "uri": uri })))
                .await
            {
                Ok(_) => self.session.mark_subscribed(&uri),
                Err(error) => self.with_status(|s| {
                    s.errors
                        .insert(format!("subscribe:{uri}"), truncate_error(&error));
                    s.inactive.push(Inactive {
                        what: format!("resource_updated:{uri}"),
                        reason: "resources/subscribe failed",
                    });
                }),
            }
        }
        self.reconcile().await;
    }

    /// Re-read every opted-in catalog and configured resource with ordinary
    /// requests. Never wakes an agent: reconnect and overflow must not create
    /// duplicate work.
    pub async fn reconcile(&mut self) {
        for class in LIST_CLASSES.into_iter().filter(|c| self.cfg.refreshes(*c)) {
            self.refresh(&NotificationEvent { class, uri: None }).await;
        }
        if self.cfg.refreshes(NotificationClass::ResourceUpdated) {
            // Only subscribed URIs are re-read: without a subscription there
            // is no invalidation signal to reconcile against.
            for uri in self.cfg.resources.clone() {
                if !self.session.uri_active(&uri) {
                    continue;
                }
                self.refresh(&NotificationEvent {
                    class: NotificationClass::ResourceUpdated,
                    uri: Some(uri),
                })
                .await;
            }
        }
    }

    /// Classify one server-originated message and queue its hint.
    pub fn ingest(&mut self, msg: Value, now: Instant) -> Ingest {
        if msg.get("method").and_then(Value::as_str) == Some(ACK_METHOD) {
            return match self.session.handle_ack(&msg) {
                Ok(omitted) => {
                    self.with_status(|s| s.inactive.extend(omitted));
                    Ingest::Acknowledged
                }
                Err(reason) => self.reject(reason, &msg),
            };
        }
        let event = match self.session.accept(&msg) {
            Ok(event) => event,
            Err(reason) => return self.reject(reason, &msg),
        };
        let (refresh, wake) = self.policy(&event);
        if !refresh && !wake {
            self.with_status(|s| s.counters.ignored += 1);
            return Ingest::Ignored;
        }
        let offer = self.dispatcher.offer(event, now);
        self.with_status(|s| match offer {
            Offer::Queued => s.counters.accepted += 1,
            Offer::Coalesced => s.counters.coalesced += 1,
            Offer::Overflow => s.counters.dropped_overflow += 1,
            Offer::RateLimited => s.counters.dropped_rate += 1,
        });
        Ingest::Offered(offer)
    }

    fn reject(&self, reason: Rejection, msg: &Value) -> Ingest {
        // Method name only, bounded: payloads and resource contents are never logged.
        let method: String = msg
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .chars()
            .filter(|c| !c.is_control())
            .take(64)
            .collect();
        crate::logging::debug(&format!(
            "MCP [{}]: rejected notification {method}: {reason:?}",
            self.server
        ));
        self.with_status(|s| s.counters.rejected += 1);
        Ingest::Rejected(reason)
    }

    /// Independent refresh and wake permissions for a validated event.
    fn policy(&self, event: &NotificationEvent) -> (bool, bool) {
        let resource_ok = event
            .uri
            .as_deref()
            .is_none_or(|uri| self.cfg.resource_allowed(uri));
        (
            resource_ok && self.cfg.refreshes(event.class),
            resource_ok && self.cfg.wakes(event.class),
        )
    }

    /// Process every pending hint: refresh where allowed, queue wakes where
    /// allowed, then reconcile if hints were dropped. Wakes are released by
    /// [`Self::flush_wakes`] under the per-server cooldown.
    pub async fn drain(&mut self) {
        self.drain_at(Instant::now()).await;
    }

    /// [`Self::drain`] with an explicit clock, so the cooldown is testable.
    pub async fn drain_at(&mut self, now: Instant) {
        while let Some(event) = self.dispatcher.pop() {
            let (refresh, wake) = self.policy(&event);
            if refresh {
                self.refresh(&event).await;
            }
            if wake {
                self.pending_wakes.insert(event.class);
            }
        }
        if self.dispatcher.take_needs_reconcile() {
            self.reconcile().await;
        }
        self.flush_wakes(now);
    }

    /// Time until a held-back wake may be released, if any is waiting.
    fn wake_delay(&self, now: Instant) -> Option<Duration> {
        if self.pending_wakes.is_empty() {
            return None;
        }
        let ready_at = self.last_wake? + self.wake_cooldown;
        Some(ready_at.saturating_duration_since(now))
    }

    /// Release wakes while the per-server cooldown allows: one per window
    /// (a zero cooldown releases all). Held-back wakes stay pending, bounded by
    /// the number of classes, and go out on a later call once the window passes.
    fn flush_wakes(&mut self, now: Instant) {
        while let Some(class) = self.pending_wakes.iter().next().copied() {
            if self
                .last_wake
                .is_some_and(|last| now.saturating_duration_since(last) < self.wake_cooldown)
            {
                self.with_status(|s| s.counters.wakes_deferred += 1);
                return;
            }
            self.pending_wakes.remove(&class);
            self.last_wake = Some(now);
            self.emit_wake(class);
        }
    }

    fn emit_wake(&self, class: NotificationClass) {
        let request = WakeRequest {
            server: sanitize_server_name(&self.server),
            class,
            session: self.cfg.wake_session.clone(),
        };
        let key = format!("wake:{}", class.as_str());
        let outcome = match self.wake.clone().or_else(global_wake_handler) {
            Some(handler) => handler.wake(request),
            None => WakeOutcome::Dropped("no wake handler installed"),
        };
        match outcome {
            WakeOutcome::Queued | WakeOutcome::Coalesced => self.with_status(|s| {
                s.counters.wakes += 1;
                s.errors.remove(&key);
            }),
            WakeOutcome::Dropped(reason) => {
                crate::logging::warn(&format!(
                    "MCP [{}]: permitted {} wake dropped: {reason}",
                    sanitize_server_name(&self.server),
                    class.as_str()
                ));
                self.with_status(|s| {
                    s.counters.wakes_dropped += 1;
                    s.errors.insert(key, reason.to_string());
                });
            }
        }
    }

    async fn refresh(&self, event: &NotificationEvent) {
        let key = event.key();
        let result: Result<()> = match event.class {
            NotificationClass::ToolsListChanged => {
                let result = self.backend.refresh_tools().await;
                if result.is_ok() {
                    publish_tool_catalog_change(&self.server);
                }
                result
            }
            NotificationClass::PromptsListChanged => {
                match list_paged(&self.backend, "prompts/list", "prompts").await {
                    Ok(items) => {
                        self.with_status(|s| s.prompts = Some(items));
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            NotificationClass::ResourcesListChanged => {
                match list_paged(&self.backend, "resources/list", "resources").await {
                    Ok(items) => {
                        self.with_status(|s| s.resources = Some(items));
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
            NotificationClass::ResourceUpdated => {
                let uri = event.uri.clone().unwrap_or_default();
                match read_resource(&self.backend, &uri).await {
                    Ok(snapshot) => {
                        self.with_status(|s| {
                            s.resource_contents.insert(uri, snapshot);
                        });
                        Ok(())
                    }
                    Err(error) => Err(error),
                }
            }
        };
        self.with_status(|s| match result {
            Ok(()) => {
                s.counters.refreshes += 1;
                s.errors.remove(&key);
            }
            Err(error) => {
                s.counters.refresh_failures += 1;
                s.errors.insert(key, truncate_error(&error));
            }
        });
    }

    /// Open `subscriptions/listen` (modern era) and pump it until it ends.
    pub async fn run_listen(
        &mut self,
        transport: &dyn ListenTransport,
        id: u64,
        cancel: &Notify,
    ) -> ListenEnd {
        let Some(filter) = self.plan.listen_filter.clone() else {
            return ListenEnd::Unsupported;
        };
        self.session.begin_listen(id);
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "subscriptions/listen",
            "params": {
                "_meta": {
                    "io.modelcontextprotocol/protocolVersion": MODERN_VERSION,
                    "io.modelcontextprotocol/clientInfo": {
                        "name": "jcode",
                        "version": jcode_build_meta::pkg_version(),
                    },
                    "io.modelcontextprotocol/clientCapabilities": {},
                },
                "notifications": filter,
            }
        });
        let mut stream = match transport.open_listen(id, request).await {
            Ok(stream) => stream,
            Err(error) => return ListenEnd::Failed(truncate_error(&error)),
        };
        loop {
            tokio::select! {
                biased;
                _ = cancel.notified() => {
                    drop(stream);
                    transport.cancel_listen(id).await;
                    return ListenEnd::Cancelled;
                }
                _ = wake_sleep(self.wake_delay(Instant::now())) => {
                    self.flush_wakes(Instant::now());
                }
                item = stream.next() => match item {
                    None | Some(Err(_)) => return ListenEnd::Abrupt,
                    Some(Ok(value)) => {
                        if value.get("method").and_then(Value::as_str) == Some(ACK_METHOD) {
                            // Subscribed now: re-read to close any gap before the ack.
                            if self.ingest(value, Instant::now()) == Ingest::Acknowledged {
                                self.reconcile().await;
                            }
                            continue;
                        }
                        if value.get("id") == Some(&Value::from(id)) {
                            return if value.get("result").is_some() {
                                ListenEnd::Graceful
                            } else {
                                ListenEnd::Failed("subscriptions/listen failed".into())
                            };
                        }
                        self.ingest(value, Instant::now());
                        self.drain().await;
                    }
                },
            }
        }
    }

    /// Feed the engine from a transport slot until the channel closes. Also
    /// wakes on a dropped-message signal (so a final dropped burst still
    /// reconciles) and when a deferred wake becomes due.
    pub(super) async fn run_pump(
        mut self,
        mut rx: mpsc::Receiver<Value>,
        dropped: Arc<InboundDrops>,
    ) {
        self.establish().await;
        loop {
            let delay = self.wake_delay(Instant::now());
            tokio::select! {
                msg = rx.recv() => {
                    let Some(msg) = msg else { break };
                    self.ingest(msg, Instant::now());
                    while let Ok(more) = rx.try_recv() {
                        self.ingest(more, Instant::now());
                    }
                }
                _ = dropped.notify.notified() => {}
                _ = wake_sleep(delay) => {}
            }
            let lost = dropped.take();
            if lost > 0 {
                self.with_status(|s| s.counters.dropped_inbound += lost as u64);
                self.dispatcher.needs_reconcile = true;
            }
            self.drain().await;
        }
    }
}

/// Sleep for `delay`, or never when there is nothing to wait for.
async fn wake_sleep(delay: Option<Duration>) {
    match delay {
        Some(delay) => tokio::time::sleep(delay).await,
        None => std::future::pending::<()>().await,
    }
}

// ---------------------------------------------------------------------------
// Authoritative reads
// ---------------------------------------------------------------------------

/// Read a paginated list (`prompts/list`, `resources/list`) with page, item
/// and cursor-cycle limits.
async fn list_paged<B: McpBackend + ?Sized>(
    backend: &B,
    method: &str,
    field: &str,
) -> Result<Vec<Value>> {
    let mut params = None;
    let mut seen = HashSet::new();
    let mut items = Vec::new();
    for _ in 0..MAX_LIST_PAGES {
        let response = backend.request(method, params).await?;
        let result = response
            .result
            .with_context(|| format!("MCP {method} response missing result"))?;
        let page = result
            .get(field)
            .and_then(Value::as_array)
            .with_context(|| format!("MCP {method} result missing {field}"))?;
        if items.len().saturating_add(page.len()) > MAX_LIST_ITEMS {
            anyhow::bail!("MCP {method} exceeded the {MAX_LIST_ITEMS}-item limit");
        }
        items.extend(page.iter().cloned());
        match result.get("nextCursor").and_then(Value::as_str) {
            Some(cursor) => {
                if !seen.insert(cursor.to_string()) {
                    anyhow::bail!("MCP {method} cursor cycle detected");
                }
                params = Some(json!({ "cursor": cursor }));
            }
            None => return Ok(items),
        }
    }
    anyhow::bail!("MCP {method} exceeded the {MAX_LIST_PAGES}-page limit")
}

async fn read_resource<B: McpBackend + ?Sized>(backend: &B, uri: &str) -> Result<ResourceSnapshot> {
    let response = backend
        .request("resources/read", Some(json!({ "uri": uri })))
        .await?;
    let result = response
        .result
        .context("MCP resources/read response missing result")?;
    let content = result
        .get("contents")
        .and_then(Value::as_array)
        .context("MCP resources/read result missing contents")?
        .iter()
        .find(|c| c.get("uri").and_then(Value::as_str) == Some(uri))
        .context("MCP resources/read result did not contain the requested URI")?;
    let mut snapshot = ResourceSnapshot {
        mime_type: content
            .get("mimeType")
            .and_then(Value::as_str)
            .map(str::to_string),
        ..ResourceSnapshot::default()
    };
    if let Some(text) = content.get("text").and_then(Value::as_str) {
        if text.len() > MAX_RESOURCE_TEXT_BYTES {
            let mut end = MAX_RESOURCE_TEXT_BYTES;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            snapshot.text = Some(text[..end].to_string());
            snapshot.text_truncated = true;
        } else {
            snapshot.text = Some(text.to_string());
        }
    }
    if let Some(blob) = content.get("blob").and_then(Value::as_str) {
        snapshot.blob_bytes = Some(blob.len());
    }
    Ok(snapshot)
}

#[cfg(test)]
#[path = "notifications_tests.rs"]
mod notifications_tests;
