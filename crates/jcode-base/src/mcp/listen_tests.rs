use super::*;
use crate::mcp::notifications::{
    ListenEnd, ListenTransport, McpBackend, McpNotificationConfig, NotificationClass,
    NotificationEngine, SharedNotificationStatus,
};
use crate::mcp::protocol::{JsonRpcResponse, ServerCapabilities};
use crate::mcp::{McpClient, McpServerConfig};
use futures::stream;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::{AtomicUsize, Ordering};

const SUB: &str = "io.modelcontextprotocol/subscriptionId";

fn ack(id: u64, filter: Value) -> Value {
    json!({"jsonrpc":"2.0","method":"notifications/subscriptions/acknowledged",
           "params":{"_meta":{SUB:id},"notifications":filter}})
}

fn complete(id: u64) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":{"resultType":"complete"}})
}

// ---- pure helpers ---------------------------------------------------------

#[test]
fn modern_meta_is_added_to_params_and_keeps_existing_fields() {
    let with = with_modern_meta(Some(json!({"cursor":"c"}))).unwrap();
    assert_eq!(with["cursor"], "c");
    assert_eq!(with["_meta"][META_VERSION], MODERN_VERSION);
    assert_eq!(
        with["_meta"]["io.modelcontextprotocol/clientInfo"]["name"],
        "jcode"
    );
    assert_eq!(
        with_modern_meta(None).unwrap()["_meta"][META_VERSION],
        MODERN_VERSION
    );
    // A non-object cannot carry metadata; it is not mangled.
    assert_eq!(with_modern_meta(Some(json!([1]))), Some(json!([1])));
}

#[test]
fn modern_meta_merges_with_an_existing_meta_object() {
    let with = with_modern_meta(Some(json!({"_meta":{"progressToken":"p"}}))).unwrap();
    assert_eq!(with["_meta"]["progressToken"], "p");
    assert_eq!(with["_meta"][META_VERSION], MODERN_VERSION);
}

#[test]
fn only_protocol_level_modern_errors_pin_the_era() {
    assert!(is_modern_error_code(-32022));
    assert!(is_modern_error_code(-32020));
    // Legacy servers answer unknown pre-initialize requests with these.
    for code in [-32601, -32602, -32600, -32603, -32000] {
        assert!(!is_modern_error_code(code), "{code}");
    }
}

#[test]
fn discover_result_is_modern_only_when_it_lists_the_revision() {
    assert!(discover_is_modern(
        &json!({"supportedVersions":["2026-07-28"]})
    ));
    assert!(discover_is_modern(
        &json!({"supportedVersions":["2025-11-25","2026-07-28"]})
    ));
    assert!(!discover_is_modern(
        &json!({"supportedVersions":["2025-11-25"]})
    ));
    assert!(!discover_is_modern(&json!({"capabilities":{}})));
    assert!(!discover_is_modern(&json!("x")));
}

// ---- stdio route -----------------------------------------------------------

#[test]
fn route_takes_only_the_active_listen_streams_messages() {
    let route = ListenRoute::default();
    // No stream open: nothing is consumed.
    assert!(!route.offer(&ack(7, json!({}))));
    let mut rx = route.install(7);
    // Ordinary responses to other ids and unrelated notifications pass through.
    assert!(!route.offer(&json!({"jsonrpc":"2.0","id":3,"result":{}})));
    assert!(!route.offer(&json!({"jsonrpc":"2.0","method":"notifications/progress","params":{}})));
    // Another subscription id is not ours.
    assert!(!route.offer(&ack(8, json!({}))));
    // Ours: ack, notification, then the final result closes the route.
    assert!(route.offer(&ack(7, json!({"toolsListChanged":true}))));
    assert!(route.offer(
        &json!({"jsonrpc":"2.0","method":"notifications/tools/list_changed",
                                "params":{"_meta":{SUB:7}}})
    ));
    assert!(route.offer(&complete(7)));
    assert!(
        !route.offer(&complete(7)),
        "route is closed after completion"
    );
    let mut got = 0;
    while rx.try_recv().is_ok() {
        got += 1;
    }
    assert_eq!(got, 3);
}

#[test]
fn route_turns_server_cancellation_into_completion_and_closes_on_overflow() {
    let route = ListenRoute::default();
    let mut rx = route.install(4);
    assert!(
        route.offer(&json!({"jsonrpc":"2.0","method":"notifications/cancelled",
                                "params":{"requestId":4}}))
    );
    assert_eq!(rx.try_recv().unwrap()["result"]["resultType"], "complete");

    // The spec's own cancel example uses a string request id.
    let route = ListenRoute::default();
    let mut rx = route.install(9);
    assert!(
        route.offer(&json!({"jsonrpc":"2.0","method":"notifications/cancelled",
                                "params":{"requestId":"9"}}))
    );
    assert_eq!(rx.try_recv().unwrap()["result"]["resultType"], "complete");
    // A different id, in either form, is not ours.
    let route = ListenRoute::default();
    let _rx = route.install(9);
    assert!(
        !route.offer(&json!({"jsonrpc":"2.0","method":"notifications/cancelled",
                                 "params":{"requestId":"10"}}))
    );

    // Overflow ends the stream (abrupt) instead of blocking the shared reader.
    let route = ListenRoute::default();
    let mut rx = route.install(5);
    for _ in 0..(STDIO_LISTEN_BUFFER + 10) {
        route.offer(&ack(5, json!({})));
    }
    let mut seen = 0;
    while rx.try_recv().is_ok() {
        seen += 1;
    }
    assert_eq!(seen, STDIO_LISTEN_BUFFER);
    assert!(rx.try_recv().is_err());
    route.close();
}

// ---- supervisor policy -----------------------------------------------------

#[derive(Clone, Default)]
struct Backend;

#[async_trait::async_trait]
impl McpBackend for Backend {
    async fn request(&self, _m: &str, _p: Option<Value>) -> Result<JsonRpcResponse> {
        Ok(JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id: Some(1),
            result: Some(json!({"tools":[],"prompts":[],"resources":[]})),
            error: None,
        })
    }
    async fn refresh_tools(&self) -> Result<()> {
        Ok(())
    }
}

/// Each `open_listen` pops the next scripted stream. `Hold` keeps it open
/// (after its items) until the test cancels.
enum Script {
    Items(Vec<Result<Value>>),
    Hold(Vec<Result<Value>>),
}

struct Scripted {
    scripts: StdMutex<std::collections::VecDeque<Script>>,
    opens: AtomicUsize,
    open_times: StdMutex<Vec<tokio::time::Instant>>,
}

impl Scripted {
    fn new(scripts: Vec<Script>) -> Self {
        Self {
            scripts: StdMutex::new(scripts.into()),
            opens: AtomicUsize::new(0),
            open_times: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl ListenTransport for Scripted {
    async fn open_listen(
        &self,
        id: u64,
        _request: Value,
    ) -> Result<futures::stream::BoxStream<'static, Result<Value>>> {
        self.opens.fetch_add(1, Ordering::SeqCst);
        self.open_times
            .lock()
            .unwrap()
            .push(tokio::time::Instant::now());
        let script = self.scripts.lock().unwrap().pop_front();
        // Items address the id the engine chose, so tests write `0` and we patch it.
        let patch = move |items: Vec<Result<Value>>| -> Vec<Result<Value>> {
            items
                .into_iter()
                .map(|item| {
                    item.map(|mut v| {
                        if v.get("id") == Some(&json!(0)) {
                            v["id"] = json!(id);
                        }
                        if v.pointer("/params/_meta").is_some() {
                            v["params"]["_meta"][SUB] = json!(id);
                        }
                        v
                    })
                })
                .collect()
        };
        match script {
            Some(Script::Items(items)) => Ok(stream::iter(patch(items)).boxed()),
            Some(Script::Hold(items)) => {
                Ok(stream::iter(patch(items)).chain(stream::pending()).boxed())
            }
            None => anyhow::bail!("no more scripted streams"),
        }
    }
    async fn cancel_listen(&self, _id: u64) {}
}

fn engine() -> NotificationEngine<Backend> {
    let cfg = McpNotificationConfig {
        refresh: vec![NotificationClass::ToolsListChanged],
        ..Default::default()
    };
    NotificationEngine::new(
        "srv",
        Backend,
        cfg,
        MODERN_VERSION,
        &ServerCapabilities::default(),
        true,
        SharedNotificationStatus::default(),
    )
}

fn policy() -> ListenPolicy {
    ListenPolicy {
        initial_backoff: Duration::from_secs(1),
        max_backoff: Duration::from_secs(8),
        stable_after: Duration::from_secs(30),
    }
}

async fn run(
    transport: &Scripted,
    engine: &mut NotificationEngine<Backend>,
    cancel: &Notify,
) -> Stopped {
    let ids = AtomicUsize::new(10);
    supervise(
        engine,
        transport,
        || ids.fetch_add(1, Ordering::SeqCst) as u64,
        || false,
        cancel,
        &policy(),
    )
    .await
}

#[tokio::test(start_paused = true)]
async fn empty_ack_then_complete_stops_without_a_reconnect_loop() {
    // Firecrawl's real behavior: acknowledge an empty filter, then complete.
    let t = Scripted::new(vec![Script::Items(vec![
        Ok(ack(0, json!({}))),
        Ok(complete(0)),
    ])]);
    let mut e = engine();
    let stopped = run(&t, &mut e, &Notify::new()).await;
    assert_eq!(stopped, Stopped::NothingToListenTo);
    assert_eq!(t.opens.load(Ordering::SeqCst), 1, "must not reopen");
}

#[tokio::test(start_paused = true)]
async fn nothing_to_listen_to_is_recorded_on_the_status() {
    let t = Scripted::new(vec![Script::Items(vec![
        Ok(ack(0, json!({}))),
        Ok(complete(0)),
    ])]);
    let mut e = engine();
    run(&t, &mut e, &Notify::new()).await;
    // The reason is visible to `mcp list`, not only in a log line.
    assert!(
        e.status_snapshot()
            .inactive
            .iter()
            .any(|i| i.what == "subscriptions/listen" && i.reason.contains("no notification"))
    );
}

#[tokio::test(start_paused = true)]
async fn abrupt_close_reconnects_with_exponential_backoff_then_listens() {
    let t = Scripted::new(vec![
        Script::Items(vec![Ok(ack(0, json!({"toolsListChanged":true})))]), // ends abruptly
        Script::Items(vec![Err(anyhow::anyhow!("reset"))]),
        Script::Hold(vec![Ok(ack(0, json!({"toolsListChanged":true})))]),
    ]);
    let cancel = Arc::new(Notify::new());
    let mut e = engine();
    let c2 = Arc::clone(&cancel);
    let stopper = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;
        c2.notify_one();
    });
    let stopped = run(&t, &mut e, &cancel).await;
    stopper.abort();
    assert_eq!(stopped, Stopped::Done);
    assert_eq!(t.opens.load(Ordering::SeqCst), 3);
    let times = t.open_times.lock().unwrap().clone();
    assert_eq!(times[1] - times[0], Duration::from_secs(1));
    assert_eq!(
        times[2] - times[1],
        Duration::from_secs(2),
        "backoff doubles"
    );
}

#[tokio::test(start_paused = true)]
async fn honored_filter_then_complete_reconnects_after_the_initial_backoff() {
    // The server honored the filter, so a completion is a rotation or restart,
    // not "nothing to listen to": reconnect after the initial backoff.
    let ok = || Ok(ack(0, json!({"toolsListChanged":true})));
    let t = Scripted::new(vec![
        Script::Items(vec![ok(), Ok(complete(0))]),
        Script::Hold(vec![ok()]),
    ]);
    let cancel = Arc::new(Notify::new());
    let c2 = Arc::clone(&cancel);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;
        c2.notify_one();
    });
    let mut e = engine();
    let stopped = run(&t, &mut e, &cancel).await;
    assert_eq!(stopped, Stopped::Done);
    assert_eq!(t.opens.load(Ordering::SeqCst), 2);
    let times = t.open_times.lock().unwrap().clone();
    assert_eq!(times[1] - times[0], Duration::from_secs(1));
}

#[tokio::test(start_paused = true)]
async fn transport_failures_never_end_the_loop_and_backoff_is_capped() {
    // 40 consecutive failures (far more than the old 8-run cap) and then a
    // healthy stream: notifications must come back on their own.
    let mut scripts: Vec<Script> = (0..40)
        .map(|_| Script::Items(vec![Err(anyhow::anyhow!("reset"))]))
        .collect();
    scripts.push(Script::Hold(vec![Ok(ack(
        0,
        json!({"toolsListChanged":true}),
    ))]));
    let t = Scripted::new(scripts);
    let cancel = Arc::new(Notify::new());
    let c2 = Arc::clone(&cancel);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        c2.notify_one();
    });
    let mut e = engine();
    let stopped = run(&t, &mut e, &cancel).await;
    assert_eq!(stopped, Stopped::Done);
    assert_eq!(
        t.opens.load(Ordering::SeqCst),
        41,
        "kept retrying until it worked"
    );
    let times = t.open_times.lock().unwrap().clone();
    let gaps: Vec<u64> = times.windows(2).map(|w| (w[1] - w[0]).as_secs()).collect();
    assert_eq!(
        &gaps[..5],
        &[1, 2, 4, 8, policy().max_backoff.as_secs().min(16)]
    );
    assert!(
        gaps.iter().all(|g| *g <= policy().max_backoff.as_secs()),
        "{gaps:?}"
    );
    assert_eq!(
        *gaps.last().unwrap(),
        policy().max_backoff.as_secs(),
        "capped"
    );
}

#[tokio::test(start_paused = true)]
async fn completion_before_any_ack_is_retried_not_treated_as_nothing_to_listen_to() {
    // A draining or restarting server can complete the stream at any time.
    let t = Scripted::new(vec![
        Script::Items(vec![Ok(complete(0))]),
        Script::Hold(vec![Ok(ack(0, json!({"toolsListChanged":true})))]),
    ]);
    let cancel = Arc::new(Notify::new());
    let c2 = Arc::clone(&cancel);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;
        c2.notify_one();
    });
    let mut e = engine();
    assert_eq!(run(&t, &mut e, &cancel).await, Stopped::Done);
    assert_eq!(t.opens.load(Ordering::SeqCst), 2);
    assert!(
        e.status_snapshot().inactive.is_empty(),
        "nothing was ruled out"
    );
}

#[tokio::test(start_paused = true)]
async fn a_jsonrpc_error_answer_is_terminal_and_keeps_its_code() {
    let t = Scripted::new(vec![Script::Items(vec![Ok(json!({
        "jsonrpc":"2.0","id":0,"error":{"code":-32601,"message":"no listen here"}}))])]);
    let mut e = engine();
    let stopped = run(&t, &mut e, &Notify::new()).await;
    assert_eq!(stopped, Stopped::Rejected);
    assert_eq!(t.opens.load(Ordering::SeqCst), 1, "no tight retry loop");
    let status = e.status_snapshot();
    let err = status.errors.get("listen").unwrap();
    assert!(
        err.contains("-32601") && err.contains("no listen here"),
        "{err}"
    );
    assert!(
        status
            .inactive
            .iter()
            .any(|i| i.what == "subscriptions/listen")
    );
}

#[tokio::test(start_paused = true)]
async fn a_transport_that_rejects_open_is_terminal_but_plain_errors_retry() {
    struct Rejecting(AtomicUsize);
    #[async_trait::async_trait]
    impl ListenTransport for Rejecting {
        async fn open_listen(
            &self,
            _id: u64,
            _r: Value,
        ) -> Result<futures::stream::BoxStream<'static, Result<Value>>> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(crate::mcp::notifications::ListenRejected("HTTP 404".into()).into())
        }
        async fn cancel_listen(&self, _id: u64) {}
    }
    let t = Rejecting(AtomicUsize::new(0));
    let mut e = engine();
    let ids = AtomicUsize::new(1);
    let stopped = supervise(
        &mut e,
        &t,
        || ids.fetch_add(1, Ordering::SeqCst) as u64,
        || false,
        &Notify::new(),
        &policy(),
    )
    .await;
    assert_eq!(stopped, Stopped::Rejected);
    assert_eq!(t.0.load(Ordering::SeqCst), 1);
    assert!(e.status_snapshot().errors["listen"].contains("HTTP 404"));
}

#[tokio::test(start_paused = true)]
async fn open_failure_backs_off_and_cancel_wins_during_backoff() {
    let t = Scripted::new(vec![]); // every open fails
    let cancel = Arc::new(Notify::new());
    let c2 = Arc::clone(&cancel);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        c2.notify_one();
    });
    let mut e = engine();
    let stopped = run(&t, &mut e, &cancel).await;
    assert_eq!(stopped, Stopped::Done);
    assert_eq!(t.opens.load(Ordering::SeqCst), 2);
}

#[tokio::test(start_paused = true)]
async fn closed_connection_stops_without_opening() {
    let t = Scripted::new(vec![Script::Hold(vec![])]);
    let mut e = engine();
    let stopped = supervise(&mut e, &t, || 1, || true, &Notify::new(), &policy()).await;
    assert_eq!(stopped, Stopped::Done);
    assert_eq!(t.opens.load(Ordering::SeqCst), 0);
}

// ---- end to end over real stdio child processes ------------------------------

/// A modern server: no `initialize`. Answers `server/discover`, lists tools
/// (changing after `$D/go`) and holds a `subscriptions/listen` stream.
const MODERN_STDIO: &str = r#"
D="$1"
LOG="$D/log"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$LOG"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"server/discover"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"complete","supportedVersions":["2026-07-28"],"capabilities":{"tools":{"listChanged":true}},"_meta":{"io.modelcontextprotocol/serverInfo":{"name":"modern-fake","version":"1"}}}}\n' "$id"
      ;;
    *'"initialize"'*)
      printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32601,"message":"modern only"}}\n' "$id"
      ;;
    *'"subscriptions/listen"'*)
      printf '{"jsonrpc":"2.0","method":"notifications/subscriptions/acknowledged","params":{"_meta":{"io.modelcontextprotocol/subscriptionId":%s},"notifications":{"toolsListChanged":true}}}\n' "$id"
      (
        while [ ! -f "$D/go" ]; do sleep 0.05; done
        printf '{"jsonrpc":"2.0","method":"notifications/tools/list_changed","params":{"_meta":{"io.modelcontextprotocol/subscriptionId":%s}}}\n' "$id"
      ) &
      ;;
    *'"tools/list"'*)
      if [ -f "$D/go" ]; then
        printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[{"name":"two","inputSchema":{"type":"object"}}]}}\n' "$id"
      else
        printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[{"name":"one","inputSchema":{"type":"object"}}]}}\n' "$id"
      fi
      ;;
  esac
done
"#;

/// Legacy: answers `server/discover` with a generic error (as many do) and
/// only then speaks `initialize`.
const LEGACY_STDIO: &str = r#"
D="$1"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$D/log"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"server/discover"'*)
      printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32602,"message":"not initialized"}}\n' "$id" ;;
    *'"initialize"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":true}},"serverInfo":{"name":"legacy-fake","version":"0"}}}\n' "$id" ;;
    *'"tools/list"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[{"name":"one","inputSchema":{"type":"object"}}]}}\n' "$id" ;;
  esac
done
"#;

/// Firecrawl's observed behavior: modern, acknowledges nothing, completes at once.
const EMPTY_ACK_STDIO: &str = r#"
D="$1"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$D/log"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"server/discover"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"complete","supportedVersions":["2026-07-28"],"capabilities":{"tools":{}}}}\n' "$id" ;;
    *'"subscriptions/listen"'*)
      printf '{"jsonrpc":"2.0","method":"notifications/subscriptions/acknowledged","params":{"_meta":{"io.modelcontextprotocol/subscriptionId":%s},"notifications":{}}}\n' "$id"
      printf '{"jsonrpc":"2.0","id":%s,"result":{"resultType":"complete"}}\n' "$id" ;;
    *'"tools/list"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[]}}\n' "$id" ;;
  esac
done
"#;

fn stdio_config(script: &str, dir: &std::path::Path, opted_in: bool) -> McpServerConfig {
    serde_json::from_value(json!({
        "command": "/bin/sh",
        "args": ["-c", script, "fake", dir],
        "shared": false,
        "timeout_secs": 10,
        "notifications": if opted_in {
            json!({"refresh": ["tools_list_changed"]})
        } else {
            Value::Null
        },
    }))
    .unwrap()
}

fn log_lines(dir: &std::path::Path) -> Vec<Value> {
    std::fs::read_to_string(dir.join("log"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn methods(dir: &std::path::Path) -> Vec<String> {
    log_lines(dir)
        .iter()
        .filter_map(|v| v["method"].as_str().map(str::to_string))
        .collect()
}

async fn eventually(what: &str, check: impl Fn() -> bool) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !check() {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("never reached: {what}"));
}

#[tokio::test]
async fn modern_stdio_server_skips_initialize_carries_meta_and_listens() {
    let dir = tempfile::tempdir().unwrap();
    let client = McpClient::connect("m".into(), &stdio_config(MODERN_STDIO, dir.path(), true))
        .await
        .expect("modern connect");
    let handle = client.handle();
    assert_eq!(handle.server_info().unwrap().name, "modern-fake");
    assert!(handle.capabilities().tools.unwrap().list_changed);
    assert_eq!(handle.tools()[0].name, "one");

    let sent = methods(dir.path());
    assert!(
        !sent.iter().any(|m| m == "initialize"),
        "no handshake: {sent:?}"
    );
    assert!(!sent.iter().any(|m| m == "notifications/initialized"));
    for line in log_lines(dir.path()) {
        assert_eq!(
            line["params"]["_meta"][META_VERSION], MODERN_VERSION,
            "every modern request carries _meta: {line}"
        );
    }

    eventually("listen opened", || {
        methods(dir.path())
            .iter()
            .any(|m| m == "subscriptions/listen")
    })
    .await;
    let listen = log_lines(dir.path())
        .into_iter()
        .find(|v| v["method"] == "subscriptions/listen")
        .unwrap();
    assert_eq!(
        listen["params"]["notifications"],
        json!({"toolsListChanged": true})
    );

    // A hint arrives on the shared stdout, tagged with the subscription id.
    std::fs::write(dir.path().join("go"), "").unwrap();
    eventually("catalog refreshed from the list_changed hint", || {
        handle.tools().first().is_some_and(|t| t.name == "two")
    })
    .await;
}

#[tokio::test]
async fn legacy_stdio_server_falls_back_on_any_discover_error() {
    let dir = tempfile::tempdir().unwrap();
    let client = McpClient::connect("l".into(), &stdio_config(LEGACY_STDIO, dir.path(), true))
        .await
        .expect("legacy connect");
    assert_eq!(client.handle().server_info().unwrap().name, "legacy-fake");
    let sent = methods(dir.path());
    let discover = sent.iter().position(|m| m == "server/discover").unwrap();
    let init = sent.iter().position(|m| m == "initialize").unwrap();
    assert!(discover < init, "probe first, then initialize: {sent:?}");
    assert!(!sent.iter().any(|m| m == "subscriptions/listen"));
    // After fallback, legacy requests carry no modern metadata.
    let tools_list = log_lines(dir.path())
        .into_iter()
        .find(|v| v["method"] == "tools/list")
        .unwrap();
    assert!(tools_list["params"]["_meta"].is_null());
}

#[tokio::test]
async fn servers_that_did_not_opt_in_are_never_probed() {
    let dir = tempfile::tempdir().unwrap();
    let _client = McpClient::connect("l".into(), &stdio_config(LEGACY_STDIO, dir.path(), false))
        .await
        .expect("connect");
    assert!(!methods(dir.path()).iter().any(|m| m == "server/discover"));
}

#[tokio::test]
async fn empty_acknowledgement_listens_exactly_once_and_does_not_loop() {
    let dir = tempfile::tempdir().unwrap();
    let client = McpClient::connect("e".into(), &stdio_config(EMPTY_ACK_STDIO, dir.path(), true))
        .await
        .expect("connect");
    eventually("inactive reason recorded", || {
        client
            .handle()
            .notification_status()
            .inactive
            .iter()
            .any(|i| i.what == "subscriptions/listen")
    })
    .await;
    // Give a reconnect loop time to show itself. Backoff would reopen at 1s.
    tokio::time::sleep(Duration::from_millis(2500)).await;
    let opens = methods(dir.path())
        .iter()
        .filter(|m| *m == "subscriptions/listen")
        .count();
    assert_eq!(
        opens, 1,
        "an empty ack plus complete is not a reason to reconnect"
    );
}

#[tokio::test]
async fn shutdown_sends_cancelled_for_the_open_listen_stream() {
    let dir = tempfile::tempdir().unwrap();
    let mut client = McpClient::connect("m".into(), &stdio_config(MODERN_STDIO, dir.path(), true))
        .await
        .unwrap();
    eventually("listen opened", || {
        methods(dir.path())
            .iter()
            .any(|m| m == "subscriptions/listen")
    })
    .await;
    client.shutdown().await;
    let sent = log_lines(dir.path());
    let listen_id = sent
        .iter()
        .find(|v| v["method"] == "subscriptions/listen")
        .unwrap()["id"]
        .clone();
    assert!(
        sent.iter()
            .any(|v| v["method"] == "notifications/cancelled"
                && v["params"]["requestId"] == listen_id),
        "stdio cancels by notification: {sent:?}"
    );
}

/// A legacy server that names 2026-07-28 in `initialize` (without ever
/// answering `server/discover`) must not switch the client to modern mode.
const FORCING_STDIO: &str = r#"
D="$1"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$D/log"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"server/discover"'*)
      printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32601,"message":"unknown"}}\n' "$id" ;;
    *'"initialize"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2026-07-28","capabilities":{"tools":{"listChanged":true}},"serverInfo":{"name":"forcing","version":"0"}}}\n' "$id" ;;
    *'"tools/list"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[]}}\n' "$id" ;;
  esac
done
"#;

/// Modern but refuses our version: must not be sent `initialize`.
const REFUSING_STDIO: &str = r#"
D="$1"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$D/log"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"server/discover"'*)
      printf '{"jsonrpc":"2.0","id":%s,"error":{"code":-32022,"message":"unsupported protocol version","data":{"supported":["2027-01-01"]}}}\n' "$id" ;;
  esac
done
"#;

/// Never answers anything: a stdio server slower than the probe.
const SILENT_THEN_LEGACY_STDIO: &str = r#"
D="$1"
n=0
while IFS= read -r line; do
  printf '%s\n' "$line" >> "$D/log"
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"server/discover"'*) ;; # swallowed: legacy servers may never answer
    *'"initialize"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":{"name":"slow-legacy","version":"0"}}}\n' "$id" ;;
    *'"tools/list"'*)
      printf '{"jsonrpc":"2.0","id":%s,"result":{"tools":[]}}\n' "$id" ;;
  esac
done
"#;

#[tokio::test]
async fn a_legacy_server_naming_the_modern_version_in_initialize_stays_legacy() {
    let dir = tempfile::tempdir().unwrap();
    let client = McpClient::connect("f".into(), &stdio_config(FORCING_STDIO, dir.path(), true))
        .await
        .expect("connect");
    assert!(!client.handle().is_modern());
    client.refresh_tools().await.unwrap();
    let sent = log_lines(dir.path());
    for line in sent.iter().filter(|v| v["method"] == "tools/list") {
        assert!(
            line["params"]["_meta"].is_null(),
            "no metadata injected: {line}"
        );
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(
        !methods(dir.path())
            .iter()
            .any(|m| m == "subscriptions/listen")
    );
}

#[tokio::test]
async fn a_recognized_modern_error_is_not_followed_by_initialize() {
    let dir = tempfile::tempdir().unwrap();
    let err = McpClient::connect("r".into(), &stdio_config(REFUSING_STDIO, dir.path(), true))
        .await
        .err()
        .expect("must fail");
    let text = format!("{err:#}");
    assert!(text.contains("-32022"), "{text}");
    assert!(!methods(dir.path()).iter().any(|m| m == "initialize"));
}

#[tokio::test]
async fn a_silent_server_is_treated_as_legacy_after_the_probe_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = stdio_config(SILENT_THEN_LEGACY_STDIO, dir.path(), true);
    // The probe is capped by the request timeout, which keeps this test fast.
    config.timeout_secs = Some(1);
    let client = McpClient::connect("s".into(), &config)
        .await
        .expect("falls back to initialize");
    assert_eq!(client.handle().server_info().unwrap().name, "slow-legacy");
    assert!(!client.handle().is_modern());
    let sent = methods(dir.path());
    let discover = sent.iter().position(|m| m == "server/discover").unwrap();
    let init = sent.iter().position(|m| m == "initialize").unwrap();
    assert!(discover < init);
}

#[tokio::test]
async fn an_abruptly_ended_stdio_stream_cancels_its_subscription_on_the_server() {
    // Overflow makes the route drop the stream. The engine must then tell the
    // server to stop the old subscription before a new one is opened, or the
    // server keeps streaming to an id nobody reads.
    let (tx, mut rx) = mpsc::channel::<String>(64);
    let transport = StdioListen {
        writer_tx: tx,
        route: ListenRoute::default(),
    };
    let mut e = engine();
    let route = transport.route.clone();
    let flood = tokio::spawn(async move {
        // Wait until the listen request is on the wire (route installed).
        while !route.is_open() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        for _ in 0..(STDIO_LISTEN_BUFFER + 5) {
            route.offer(&ack(77, json!({})));
        }
    });
    let end = tokio::time::timeout(
        Duration::from_secs(5),
        e.run_listen(&transport, 77, &Notify::new()),
    )
    .await
    .expect("stream must end once the route overflows");
    flood.await.unwrap();
    assert_eq!(end, ListenEnd::Abrupt);
    let mut lines = Vec::new();
    while let Ok(line) = rx.try_recv() {
        lines.push(serde_json::from_str::<Value>(line.trim()).unwrap());
    }
    assert_eq!(lines[0]["method"], "subscriptions/listen");
    assert!(
        lines
            .iter()
            .any(|v| v["method"] == "notifications/cancelled" && v["params"]["requestId"] == 77),
        "old subscription must be cancelled: {lines:?}"
    );
}

// ---- end to end over HTTP ------------------------------------------------------

mod http {
    use super::*;
    use std::collections::HashMap;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;

    #[derive(Clone, Debug)]
    struct Seen {
        method: String,
        headers: HashMap<String, String>,
        body: Value,
    }

    struct Server {
        url: String,
        seen: Arc<StdMutex<Vec<Seen>>>,
        listen_closed: Arc<AtomicUsize>,
        task: tokio::task::JoinHandle<()>,
    }

    impl Drop for Server {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    /// How the fixture treats `server/discover` and `subscriptions/listen`.
    #[derive(Clone, Copy, PartialEq)]
    enum Mode {
        Modern,
        /// HTTP 400 with an empty body: a legacy server (spec: fall back).
        Legacy,
        /// HTTP 400 with a recognized modern JSON-RPC error body.
        ModernRefusing,
        /// Modern, but `subscriptions/listen` answers HTTP 404.
        ListenRefused,
    }

    async fn start(modern: bool) -> Server {
        start_mode(if modern { Mode::Modern } else { Mode::Legacy }).await
    }

    async fn start_mode(mode: Mode) -> Server {
        let modern = mode;
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let seen: Arc<StdMutex<Vec<Seen>>> = Default::default();
        let listen_closed = Arc::new(AtomicUsize::new(0));
        let (seen2, closed2) = (Arc::clone(&seen), Arc::clone(&listen_closed));
        let task = tokio::spawn(async move {
            loop {
                let Ok((socket, _)) = listener.accept().await else {
                    return;
                };
                let (seen, closed) = (Arc::clone(&seen2), Arc::clone(&closed2));
                tokio::spawn(async move {
                    let _ = serve(socket, modern, seen, closed).await;
                });
            }
        });
        Server {
            url,
            seen,
            listen_closed,
            task,
        }
    }

    async fn serve(
        socket: tokio::net::TcpStream,
        modern: Mode,
        seen: Arc<StdMutex<Vec<Seen>>>,
        closed: Arc<AtomicUsize>,
    ) -> anyhow::Result<()> {
        let mut reader = BufReader::new(socket);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let mut headers = HashMap::new();
        loop {
            line.clear();
            reader.read_line(&mut line).await?;
            if line == "\r\n" || line.is_empty() {
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.to_ascii_lowercase(), v.trim().to_string());
            }
        }
        let size: usize = headers
            .get("content-length")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let mut bytes = vec![0; size];
        reader.read_exact(&mut bytes).await?;
        let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        let method = body["method"].as_str().unwrap_or("").to_string();
        seen.lock().unwrap().push(Seen {
            method: method.clone(),
            headers: headers.clone(),
            body: body.clone(),
        });
        let mut socket = reader.into_inner();
        let id = body["id"].clone();
        let json_reply = |value: Value| {
            let text = value.to_string();
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                text.len()
            )
        };
        match method.as_str() {
            "server/discover" if modern == Mode::Legacy => {
                let reply =
                    "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                socket.write_all(reply.as_bytes()).await?;
            }
            "server/discover" if modern == Mode::ModernRefusing => {
                let text = json!({"jsonrpc":"2.0","id":null,"error":{"code":-32022,
                    "message":"unsupported protocol version","data":{"supported":["2027-01-01"]}}})
                .to_string();
                let reply = format!(
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                    text.len()
                );
                socket.write_all(reply.as_bytes()).await?;
            }
            "subscriptions/listen" if modern == Mode::ListenRefused => {
                let reply =
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                socket.write_all(reply.as_bytes()).await?;
            }
            "tools/call" => {
                let reply = json_reply(json!({"jsonrpc":"2.0","id":id,"result":{
                    "content":[{"type":"text","text":"ok"}]}}));
                socket.write_all(reply.as_bytes()).await?;
            }
            "server/discover" => {
                let reply = json_reply(json!({"jsonrpc":"2.0","id":id,"result":{
                    "resultType":"complete","supportedVersions":["2026-07-28"],
                    "capabilities":{"tools":{"listChanged":true}}}}));
                socket.write_all(reply.as_bytes()).await?;
            }
            "initialize" => {
                let reply = json_reply(json!({"jsonrpc":"2.0","id":id,"result":{
                    "protocolVersion":"2025-11-25","capabilities":{"tools":{"listChanged":true}},
                    "serverInfo":{"name":"legacy-http","version":"0"}}}));
                socket.write_all(reply.as_bytes()).await?;
            }
            "notifications/initialized" => {
                socket
                    .write_all(
                        b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .await?;
            }
            "tools/list" => {
                let reply = json_reply(json!({"jsonrpc":"2.0","id":id,"result":{"tools":[
                    {"name":"one","inputSchema":{"type":"object"}},
                    {"name":"sql","inputSchema":{"type":"object","properties":{
                        "region":{"type":"string","x-mcp-header":"Region"},
                        "query":{"type":"string"}}}},
                    {"name":"broken","inputSchema":{"type":"object","properties":{
                        "a":{"type":"number","x-mcp-header":"Bad"}}}}]}}));
                socket.write_all(reply.as_bytes()).await?;
            }
            "subscriptions/listen" => {
                socket
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n")
                    .await?;
                let chunk = |data: String| format!("{:x}\r\n{data}\r\n", data.len());
                // Keepalive comment first: must be ignored.
                socket
                    .write_all(chunk(": keepalive\n\n".into()).as_bytes())
                    .await?;
                let ack = json!({"jsonrpc":"2.0","method":"notifications/subscriptions/acknowledged",
                    "params":{"_meta":{SUB:id},"notifications":{"toolsListChanged":true}}});
                socket
                    .write_all(chunk(format!("data: {ack}\n\n")).as_bytes())
                    .await?;
                let hint = json!({"jsonrpc":"2.0","method":"notifications/tools/list_changed",
                    "params":{"_meta":{SUB:id}}});
                socket
                    .write_all(chunk(format!("data: {hint}\n\n")).as_bytes())
                    .await?;
                // Hold the stream open until the client closes it.
                let mut byte = [0u8; 1];
                let _ = socket.read(&mut byte).await;
                closed.fetch_add(1, Ordering::SeqCst);
            }
            _ => {}
        }
        Ok(())
    }

    fn config(url: &str) -> McpServerConfig {
        serde_json::from_value(json!({
            "type": "http", "url": url, "timeout_secs": 5,
            "notifications": {"refresh": ["tools_list_changed"]},
        }))
        .unwrap()
    }

    #[tokio::test]
    async fn modern_http_sends_headers_meta_and_listens_until_shutdown() {
        let server = start(true).await;
        let mut client = McpClient::connect("h".into(), &config(&server.url))
            .await
            .expect("modern http connect");
        eventually("listen request", || {
            server
                .seen
                .lock()
                .unwrap()
                .iter()
                .any(|s| s.method == "subscriptions/listen")
        })
        .await;
        let seen = server.seen.lock().unwrap().clone();
        assert!(!seen.iter().any(|s| s.method == "initialize"));
        for s in &seen {
            assert_eq!(
                s.headers.get("mcp-protocol-version").map(String::as_str),
                Some(MODERN_VERSION),
                "{}",
                s.method
            );
            assert_eq!(s.headers.get("mcp-method"), Some(&s.method), "{}", s.method);
            assert!(!s.headers.contains_key("mcp-session-id"), "sessionless");
            assert_eq!(s.body["params"]["_meta"][META_VERSION], MODERN_VERSION);
        }
        // The hint (after a keepalive comment) caused a second tools/list.
        eventually("hint refresh", || {
            server
                .seen
                .lock()
                .unwrap()
                .iter()
                .filter(|s| s.method == "tools/list")
                .count()
                >= 2
        })
        .await;
        client.shutdown().await;
        eventually("server saw the stream closed", || {
            server.listen_closed.load(Ordering::SeqCst) == 1
        })
        .await;
    }

    #[tokio::test]
    async fn http_400_without_a_modern_error_falls_back_to_initialize() {
        let server = start(false).await;
        let client = McpClient::connect("h".into(), &config(&server.url))
            .await
            .expect("legacy http connect");
        assert_eq!(client.handle().server_info().unwrap().name, "legacy-http");
        let seen = server.seen.lock().unwrap().clone();
        let discover = seen
            .iter()
            .position(|s| s.method == "server/discover")
            .unwrap();
        let init = seen.iter().position(|s| s.method == "initialize").unwrap();
        assert!(discover < init);
        // After fallback the legacy handshake is not tagged as modern.
        assert_ne!(
            seen[init]
                .headers
                .get("mcp-protocol-version")
                .map(String::as_str),
            Some(MODERN_VERSION)
        );
        assert!(!seen.iter().any(|s| s.method == "subscriptions/listen"));
    }

    #[tokio::test]
    async fn tools_call_mirrors_x_mcp_header_values_and_invalid_tools_are_excluded() {
        let server = start(true).await;
        let client = McpClient::connect("h".into(), &config(&server.url))
            .await
            .expect("connect");
        let names: Vec<String> = client
            .handle()
            .tools()
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert!(names.contains(&"sql".to_string()));
        assert!(
            !names.contains(&"broken".to_string()),
            "a tool with an invalid annotation must be excluded: {names:?}"
        );
        client
            .call_tool("sql", json!({"region": "Hello, 世界", "query": "select 1"}))
            .await
            .expect("call");
        let call = server
            .seen
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.method == "tools/call")
            .cloned()
            .unwrap();
        assert_eq!(call.headers["mcp-method"], "tools/call");
        assert_eq!(call.headers["mcp-name"], "sql");
        assert_eq!(
            call.headers["mcp-param-region"], "=?base64?SGVsbG8sIOS4lueVjA==?=",
            "{:?}",
            call.headers
        );
        assert!(
            !call.headers.contains_key("mcp-param-query"),
            "unannotated stays out"
        );
        // An omitted annotated argument omits its header.
        client
            .call_tool("sql", json!({"query": "x"}))
            .await
            .unwrap();
        let second = server
            .seen
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.method == "tools/call")
            .nth(1)
            .cloned()
            .unwrap();
        assert!(!second.headers.contains_key("mcp-param-region"));
    }

    #[tokio::test]
    async fn a_modern_error_body_on_http_400_is_not_followed_by_initialize_or_sse() {
        let server = start_mode(Mode::ModernRefusing).await;
        let err = McpClient::connect("h".into(), &config(&server.url))
            .await
            .err()
            .expect("must fail");
        let text = format!("{err:#}");
        assert!(text.contains("-32022"), "{text}");
        let seen = server.seen.lock().unwrap().clone();
        assert!(
            !seen.iter().any(|s| s.method == "initialize"),
            "{:?}",
            seen.len()
        );
    }

    #[tokio::test]
    async fn http_404_on_listen_is_a_refusal_recorded_once_not_retried() {
        let server = start_mode(Mode::ListenRefused).await;
        let client = McpClient::connect("h".into(), &config(&server.url))
            .await
            .expect("connect");
        eventually("refusal recorded", || {
            client
                .handle()
                .notification_status()
                .inactive
                .iter()
                .any(|i| i.what == "subscriptions/listen")
        })
        .await;
        let status = client.handle().notification_status();
        assert!(
            status.errors["listen"].contains("404"),
            "{:?}",
            status.errors
        );
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let opens = server
            .seen
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.method == "subscriptions/listen")
            .count();
        assert_eq!(opens, 1, "a 404 cannot be fixed by retrying");
    }
}
