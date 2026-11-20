//! MCP 2026-07-28 support: era detection helpers, the stdio
//! `subscriptions/listen` route and the supervisor that keeps one listen
//! stream alive. jcode only listens: it never changes a server.
//!
//! Policy for a stream that ends (spec 2026-07-28, transports):
//! * An immediate graceful `complete` with nothing honored means the server
//!   has nothing to listen to (Firecrawl does this). Record it and stop. A
//!   reconnect loop would hammer the server for no gain.
//! * Abrupt close, transport errors and completions that arrive before any
//!   acknowledgment reconnect with exponential backoff capped at one minute.
//!   A stream that stayed open long enough resets the backoff. The loop never
//!   gives up on transport trouble, so a short outage cannot silently turn
//!   notifications off.
//! * A deterministic refusal (JSON-RPC error answer, HTTP 4xx) stops the loop;
//!   the server's code and message are in the status `listen` error.

use super::notifications::{
    ListenEnd, ListenTransport, MODERN_VERSION, McpBackend, NotificationEngine,
};
use anyhow::Result;
use futures::StreamExt;
use futures::stream::BoxStream;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::{Notify, mpsc};

pub(super) const META_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
const META_CLIENT_INFO: &str = "io.modelcontextprotocol/clientInfo";
const META_CLIENT_CAPS: &str = "io.modelcontextprotocol/clientCapabilities";
pub(super) const META_SERVER_INFO: &str = "io.modelcontextprotocol/serverInfo";
const META_SUBSCRIPTION_ID: &str = "io.modelcontextprotocol/subscriptionId";

/// Buffered messages for one stdio listen stream. Overflow ends the stream
/// as abrupt, which reconnects and reconciles, rather than blocking the shared
/// stdout reader that also carries ordinary responses.
const STDIO_LISTEN_BUFFER: usize = 256;

/// The `_meta` object every 2026-07-28 request carries. `tasks` declares the
/// Tasks extension in the per-request client capabilities.
pub(super) fn request_meta(tasks: bool) -> Value {
    let caps = if tasks {
        json!({"extensions": {super::tasks::TASKS_EXTENSION: {}}})
    } else {
        json!({})
    };
    json!({
        META_VERSION: MODERN_VERSION,
        META_CLIENT_INFO: {
            "name": "jcode",
            "version": jcode_build_meta::pkg_version(),
        },
        META_CLIENT_CAPS: caps,
    })
}

/// Add the modern `_meta` to request params, keeping any other `_meta` keys
/// (for example a progress token). Non-object params are left alone.
pub(super) fn with_modern_meta(params: Option<Value>, tasks: bool) -> Option<Value> {
    let ours = request_meta(tasks);
    match params {
        None => Some(json!({ "_meta": ours })),
        Some(Value::Object(mut map)) => {
            let mut meta = match map.remove("_meta") {
                Some(Value::Object(existing)) => existing,
                _ => serde_json::Map::new(),
            };
            if let Value::Object(ours) = ours {
                meta.extend(ours);
            }
            map.insert("_meta".to_string(), Value::Object(meta));
            Some(Value::Object(map))
        }
        other => other,
    }
}

/// Error codes a modern server uses for protocol-level refusals. Seeing one
/// to `server/discover` proves the server is modern, so the client must not
/// fall back to `initialize`: -32022 UnsupportedProtocolVersion and -32020
/// HeaderMismatch. -32601 is deliberately absent: legacy servers answer
/// unknown pre-initialize requests with it too.
pub(super) fn is_modern_error_code(code: i64) -> bool {
    matches!(code, -32022 | -32020)
}

/// True when a `server/discover` result lists the modern revision.
pub(super) fn discover_is_modern(result: &Value) -> bool {
    result
        .get("supportedVersions")
        .and_then(Value::as_array)
        .is_some_and(|versions| versions.iter().any(|v| v.as_str() == Some(MODERN_VERSION)))
}

// ---------------------------------------------------------------------------
// stdio route
// ---------------------------------------------------------------------------

/// Sends the messages that belong to the active stdio `subscriptions/listen`
/// request to its stream. Stdout is shared, so the reader asks the route first.
#[derive(Clone, Default)]
pub(super) struct ListenRoute(Arc<Mutex<ActiveListen>>);

/// The listen request id and the channel feeding its stream, while one is open.
type ActiveListen = Option<(u64, mpsc::Sender<Value>)>;

impl ListenRoute {
    fn lock(&self) -> std::sync::MutexGuard<'_, ActiveListen> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn install(&self, id: u64) -> mpsc::Receiver<Value> {
        let (tx, rx) = mpsc::channel(STDIO_LISTEN_BUFFER);
        *self.lock() = Some((id, tx));
        rx
    }

    fn clear(&self, id: u64) {
        let mut guard = self.lock();
        if guard.as_ref().is_some_and(|(active, _)| *active == id) {
            *guard = None;
        }
    }

    /// Cheap check so the stdout reader skips routing work when no stream is open.
    pub(super) fn is_open(&self) -> bool {
        self.lock().is_some()
    }

    /// Reader exit: end any open stream.
    pub(super) fn close(&self) {
        *self.lock() = None;
    }

    /// Offer one stdout message. Returns true when the route consumed it.
    pub(super) fn offer(&self, value: &Value) -> bool {
        let mut guard = self.lock();
        let Some((id, tx)) = guard.as_ref() else {
            return false;
        };
        let id = *id;
        let is_response = value.get("result").is_some() || value.get("error").is_some();
        let forward = if is_response {
            if value.get("id").and_then(Value::as_u64) != Some(id) {
                return false;
            }
            Some(value.clone())
        } else if value.get("method").and_then(Value::as_str) == Some("notifications/cancelled")
            && request_id_matches(value.pointer("/params/requestId"), id)
        {
            // Server-side teardown by cancellation equals a completion.
            Some(json!({"jsonrpc": "2.0", "id": id, "result": {"resultType": "complete"}}))
        } else if value
            .pointer("/params/_meta")
            .and_then(|meta| meta.get(META_SUBSCRIPTION_ID))
            .and_then(Value::as_u64)
            == Some(id)
        {
            Some(value.clone())
        } else {
            return false;
        };
        let ends = is_response || forward.as_ref().is_some_and(|v| v.get("result").is_some());
        if let Some(message) = forward
            && tx.try_send(message).is_err()
        {
            // Full: end the stream abruptly so the supervisor reconnects and
            // reconciles instead of silently losing events.
            *guard = None;
            return true;
        }
        if ends {
            *guard = None;
        }
        true
    }
}

/// A `requestId` may be a number or, as in the spec's own cancel example, its
/// string form.
fn request_id_matches(value: Option<&Value>, id: u64) -> bool {
    match value {
        Some(Value::Number(n)) => n.as_u64() == Some(id),
        Some(Value::String(s)) => s.parse::<u64>().ok() == Some(id),
        _ => false,
    }
}

/// `subscriptions/listen` over a stdio child's shared stdin/stdout.
pub(super) struct StdioListen {
    pub(super) writer_tx: mpsc::Sender<String>,
    pub(super) route: ListenRoute,
}

#[async_trait::async_trait]
impl ListenTransport for StdioListen {
    async fn open_listen(
        &self,
        id: u64,
        request: Value,
    ) -> Result<BoxStream<'static, Result<Value>>> {
        // Route first: the acknowledgment can follow the write immediately.
        let rx = self.route.install(id);
        let line = format!("{request}\n");
        if self.writer_tx.send(line).await.is_err() {
            self.route.clear(id);
            anyhow::bail!("MCP server connection is closed");
        }
        Ok(futures::stream::unfold(rx, |mut rx| async move {
            rx.recv().await.map(|value| (Ok(value), rx))
        })
        .boxed())
    }

    async fn cancel_listen(&self, id: u64) {
        self.route.clear(id);
        let message = json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancelled",
            "params": {"requestId": id, "reason": "client stopped listening"},
        });
        let _ = self.writer_tx.send(format!("{message}\n")).await;
    }
}

// ---------------------------------------------------------------------------
// supervisor
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(super) struct ListenPolicy {
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    /// A stream open at least this long counts as healthy and resets the backoff.
    pub stable_after: Duration,
}

impl Default for ListenPolicy {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            stable_after: Duration::from_secs(30),
        }
    }
}

/// Why the supervisor returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Stopped {
    /// Cancelled, closed or nothing to listen for.
    Done,
    /// The server acknowledged but honors no notification type.
    NothingToListenTo,
    /// The server refused `subscriptions/listen` outright (JSON-RPC error or
    /// HTTP 4xx). Retrying cannot help.
    Rejected,
}

/// Keep one `subscriptions/listen` stream open until cancelled.
///
/// Transport trouble never ends the loop: a short outage (network blip,
/// laptop suspend, server restart) must not silently turn notifications off
/// for the rest of the connection. Backoff is capped at `max_backoff`, so a
/// permanently broken server costs one attempt a minute.
pub(super) async fn supervise<B: McpBackend>(
    engine: &mut NotificationEngine<B>,
    transport: &dyn ListenTransport,
    next_id: impl Fn() -> u64,
    closed: impl Fn() -> bool,
    cancel: &Notify,
    policy: &ListenPolicy,
) -> Stopped {
    let mut backoff = policy.initial_backoff;
    loop {
        if closed() {
            return Stopped::Done;
        }
        let started = tokio::time::Instant::now();
        let end = engine.run_listen(transport, next_id(), cancel).await;
        match end {
            ListenEnd::Unsupported | ListenEnd::Cancelled => return Stopped::Done,
            // Only an acknowledged-and-empty stream means "nothing to listen
            // to". A completion before any acknowledgment is a server that is
            // draining or restarting, so retry like any other end.
            ListenEnd::Graceful if engine.listen_acked() && !engine.listen_honored() => {
                engine.mark_inactive(
                    "subscriptions/listen",
                    "server honors no notification types",
                );
                return Stopped::NothingToListenTo;
            }
            ListenEnd::Rejected(_) => {
                engine.mark_inactive(
                    "subscriptions/listen",
                    "server refused subscriptions/listen",
                );
                return Stopped::Rejected;
            }
            ListenEnd::Graceful | ListenEnd::Abrupt | ListenEnd::Failed(_) => {}
        }
        if started.elapsed() >= policy.stable_after {
            backoff = policy.initial_backoff;
        }
        tokio::select! {
            _ = cancel.notified() => return Stopped::Done,
            _ = tokio::time::sleep(backoff) => {}
        }
        backoff = (backoff * 2).min(policy.max_backoff);
    }
}

#[cfg(test)]
#[path = "listen_tests.rs"]
mod listen_tests;
