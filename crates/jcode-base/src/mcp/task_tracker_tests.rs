//! Tests for the durable MCP task tracker. Every test name starts with
//! `mcp_task_tracking_` so `cargo test -p jcode-base mcp_task_tracking` runs
//! them. All servers are fakes and all state lives in a temporary directory.

use super::*;
use crate::background::{BackgroundTaskManager, TaskStatusFile, truncate_cells};
use crate::bus::{BackgroundTaskStatus, Bus, BusEvent};
use crate::mcp::tasks::{ResolvedTaskServer, TaskRemote};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::sync::Mutex as StdMutex;
use tempfile::{TempDir, tempdir};
use tokio::time::Instant;

const SERVER: &str = "srv";
const IDENTITY: &str = "fp:endpoint-a";
const REMOTE_ID: &str = "remote-1";

fn base_time() -> DateTime<Utc> {
    DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc)
}

/// Wall clock that follows tokio's paused time, so expiry and the poll cadence
/// move together.
struct TestClock {
    epoch: Instant,
}

#[async_trait]
impl TrackerClock for TestClock {
    fn now(&self) -> DateTime<Utc> {
        base_time() + chrono::Duration::from_std(self.epoch.elapsed()).unwrap()
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

fn ts(secs: i64) -> String {
    (base_time() + chrono::Duration::seconds(secs)).to_rfc3339()
}

fn snap(id: &str, status: RemoteStatus, updated_secs: i64) -> RemoteTask {
    RemoteTask {
        task_id: id.to_string(),
        status,
        status_message: None,
        created_at: ts(0),
        last_updated_at: ts(updated_secs),
        ttl_ms: None,
        poll_interval_ms: None,
        input_requests: None,
        result: None,
        error: None,
    }
}

/// Fake server. Records every call; never has a way to receive `tools/call`.
struct FakeRemote {
    start: Instant,
    get_calls: StdMutex<Vec<(String, Duration)>>,
    cancel_calls: StdMutex<Vec<String>>,
    script: StdMutex<VecDeque<std::result::Result<TaskPoll, String>>>,
    default_poll: StdMutex<std::result::Result<TaskPoll, String>>,
    cancel_result: StdMutex<std::result::Result<(), String>>,
}

impl FakeRemote {
    fn new(default_poll: std::result::Result<TaskPoll, String>) -> Arc<Self> {
        Arc::new(Self {
            start: Instant::now(),
            get_calls: StdMutex::new(Vec::new()),
            cancel_calls: StdMutex::new(Vec::new()),
            script: StdMutex::new(VecDeque::new()),
            default_poll: StdMutex::new(default_poll),
            cancel_result: StdMutex::new(Ok(())),
        })
    }

    fn working(id: &str) -> Arc<Self> {
        Self::new(Ok(TaskPoll::Task(snap(id, RemoteStatus::Working, 0))))
    }

    fn push(&self, poll: std::result::Result<TaskPoll, String>) {
        self.script.lock().unwrap().push_back(poll);
    }

    fn set_default(&self, poll: std::result::Result<TaskPoll, String>) {
        *self.default_poll.lock().unwrap() = poll;
    }

    fn gets(&self) -> usize {
        self.get_calls.lock().unwrap().len()
    }

    fn get_offsets_ms(&self) -> Vec<u128> {
        self.get_calls
            .lock()
            .unwrap()
            .iter()
            .map(|(_, at)| at.as_millis())
            .collect()
    }

    fn cancels(&self) -> usize {
        self.cancel_calls.lock().unwrap().len()
    }
}

#[async_trait]
impl TaskRemote for FakeRemote {
    async fn get_task(&self, task_id: &str) -> Result<TaskPoll> {
        self.get_calls
            .lock()
            .unwrap()
            .push((task_id.to_string(), self.start.elapsed()));
        let next = self.script.lock().unwrap().pop_front();
        match next.unwrap_or_else(|| self.default_poll.lock().unwrap().clone()) {
            Ok(poll) => Ok(poll),
            Err(message) => Err(anyhow!(message)),
        }
    }

    async fn cancel_task(&self, task_id: &str) -> Result<()> {
        self.cancel_calls.lock().unwrap().push(task_id.to_string());
        self.cancel_result
            .lock()
            .unwrap()
            .clone()
            .map_err(|m| anyhow!(m))
    }
}

#[derive(Default)]
struct FakeSource {
    servers: StdMutex<HashMap<String, std::result::Result<ResolvedTaskServer, ResolveError>>>,
    resolves: StdMutex<Vec<String>>,
}

impl FakeSource {
    fn set(&self, server: &str, identity: &str, remote: Arc<FakeRemote>) {
        self.servers.lock().unwrap().insert(
            server.to_string(),
            Ok(ResolvedTaskServer {
                identity: identity.to_string(),
                remote,
            }),
        );
    }

    fn set_error(&self, server: &str, error: ResolveError) {
        self.servers
            .lock()
            .unwrap()
            .insert(server.to_string(), Err(error));
    }
}

#[async_trait]
impl TaskSource for FakeSource {
    async fn resolve(&self, server: &str) -> std::result::Result<ResolvedTaskServer, ResolveError> {
        self.resolves.lock().unwrap().push(server.to_string());
        self.servers
            .lock()
            .unwrap()
            .get(server)
            .cloned()
            .unwrap_or(Err(ResolveError::Unknown))
    }
}

struct Harness {
    _tmp: TempDir,
    epoch: Instant,
    manager: Arc<BackgroundTaskManager>,
    source: Arc<FakeSource>,
    remote: Arc<FakeRemote>,
    tracker: McpTaskTracker,
}

impl Harness {
    fn with_remote(remote: Arc<FakeRemote>) -> Self {
        let tmp = tempdir().unwrap();
        let manager = Arc::new(BackgroundTaskManager::with_output_dir(
            tmp.path().join("tasks"),
        ));
        let source = Arc::new(FakeSource::default());
        source.set(SERVER, IDENTITY, Arc::clone(&remote));
        let epoch = Instant::now();
        let tracker = Self::tracker_for(&source, &manager, epoch);
        Self {
            _tmp: tmp,
            epoch,
            manager,
            source,
            remote,
            tracker,
        }
    }

    fn new() -> Self {
        Self::with_remote(FakeRemote::working(REMOTE_ID))
    }

    /// A tracker over shared persisted state, as a restarted process would
    /// build it. All trackers of one test share the same wall-clock epoch.
    fn tracker_for(
        source: &Arc<FakeSource>,
        manager: &Arc<BackgroundTaskManager>,
        epoch: Instant,
    ) -> McpTaskTracker {
        McpTaskTracker::with_clock(
            Arc::clone(source) as Arc<dyn TaskSource>,
            Arc::clone(manager),
            Arc::new(TestClock { epoch }),
        )
    }

    fn restart(&self) -> McpTaskTracker {
        Self::tracker_for(&self.source, &self.manager, self.epoch)
    }

    async fn register(&self, handle: RemoteTask) -> String {
        self.tracker
            .register(handle, SERVER, IDENTITY, "build", "session-1", "call-1")
            .await
            .unwrap()
    }

    async fn status(&self, task_id: &str) -> TaskStatusFile {
        self.manager.status(task_id).await.unwrap()
    }

    async fn record(&self, task_id: &str) -> McpTaskRecord {
        self.status(task_id).await.mcp_task.unwrap()
    }
}

/// Advance paused time. Timer wake-ups run in order and each wake-up finishes
/// its file I/O before the next, because blocking work inhibits auto-advance.
async fn run_for_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

// ---- registration ---------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_register_persists_running_record_without_owner() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;

    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Running);
    assert_eq!(status.tool_name, "mcp:srv:build");
    assert_eq!(status.session_id, "session-1");
    // No in-process owner: orphan reconciliation must never fail this record.
    assert!(status.owner_pid.is_none() && status.owner_instance.is_none());
    assert!(!status.detached && status.pid.is_none());
    let record = status.mcp_task.unwrap();
    assert_eq!(record.server, SERVER);
    assert_eq!(record.identity, IDENTITY);
    assert_eq!(record.remote_task_id, REMOTE_ID);
    assert_eq!(record.tool, "build");
    assert_eq!(record.tool_call_id, "call-1");
    assert_eq!(record.remote_state, McpRemoteState::Tracking);
    assert_eq!(h.manager.reconcile_orphaned_tasks().await, 0);
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Running);
    assert!(h.tracker.is_polling(&id));
    // Registration alone sends nothing.
    assert_eq!(h.remote.gets(), 0);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_register_fails_closed_when_persistence_fails() {
    let tmp = tempdir().unwrap();
    // The output directory's parent is a regular file, so nothing can be written.
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, "not a directory").unwrap();
    let manager = Arc::new(BackgroundTaskManager::with_output_dir(
        blocker.join("tasks"),
    ));
    let remote = FakeRemote::working(REMOTE_ID);
    let source = Arc::new(FakeSource::default());
    source.set(SERVER, IDENTITY, Arc::clone(&remote));
    let tracker = Harness::tracker_for(&source, &manager, Instant::now());

    let error = tracker
        .register(
            snap(REMOTE_ID, RemoteStatus::Working, 0),
            SERVER,
            IDENTITY,
            "build",
            "session-1",
            "call-1",
        )
        .await
        .expect_err("an unpersisted handle must not be returned");
    assert!(error.to_string().contains("not tracked"), "{error}");
    run_for_ms(600_000).await;
    assert_eq!(remote.gets(), 0, "an untracked task must never be polled");
    assert!(manager.list().await.is_empty());
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_register_of_terminal_handle_finalizes_without_polling() {
    let h = Harness::new();
    let mut done = snap(REMOTE_ID, RemoteStatus::Completed, 1);
    done.result = Some(json!({"content": [{"type": "text", "text": "instant"}]}));
    let id = h.register(done).await;

    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Completed);
    assert!(!h.tracker.is_polling(&id));
    run_for_ms(60_000).await;
    assert_eq!(h.remote.gets(), 0);
    assert_eq!(h.manager.output(&id).await.as_deref(), Some("instant"));
}

// ---- restart and resume ---------------------------------------------------

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_resume_after_restart_polls_without_replaying_the_call() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.tracker.shutdown();
    assert!(!h.tracker.is_polling(&id));

    // New process: fresh tracker over the same persisted directory.
    let mut done = snap(REMOTE_ID, RemoteStatus::Completed, 9);
    done.result = Some(json!({"content": [{"type": "text", "text": "built"}]}));
    h.remote.push(Ok(TaskPoll::Task(done)));
    let restarted = h.restart();
    assert_eq!(restarted.resume_all().await, 1);
    run_for_ms(100).await;

    // Exactly one tasks/get for the saved remote id, no cancel, and the fake
    // has no tools/call entry point to be replayed through.
    let calls = h.remote.get_calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, REMOTE_ID);
    assert_eq!(h.remote.cancels(), 0);
    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Completed);
    assert_eq!(h.record(&id).await.remote_status, RemoteStatus::Completed);
    // Resuming again finds nothing nonterminal.
    assert_eq!(restarted.resume_all().await, 0);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_resume_twice_starts_one_loop() {
    let h = Harness::new();
    h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    assert_eq!(
        h.tracker.resume_all().await,
        0,
        "register already started it"
    );
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_resume_with_unknown_task_marks_not_found_and_stops() {
    let h = Harness::with_remote(FakeRemote::new(Ok(TaskPoll::NotFound(
        "Invalid params: unknown task".into(),
    ))));
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.tracker.shutdown();
    let restarted = h.restart();
    restarted.resume_all().await;
    run_for_ms(3_600_000).await;

    assert_eq!(h.remote.gets(), 1, "NotFound must not be retried");
    let record = h.record(&id).await;
    assert_eq!(record.remote_state, McpRemoteState::NotFound);
    // Last known state and task id are kept; it is not a task failure.
    assert_eq!(record.remote_status, RemoteStatus::Working);
    assert_eq!(record.remote_task_id, REMOTE_ID);
    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Lost);
    assert!(record.state_label().contains("expired/not found"));
    assert_eq!(h.remote.cancels(), 0);
    assert!(!restarted.is_polling(&id));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_identity_mismatch_is_unresolved_and_never_reaches_the_server() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.tracker.shutdown();

    // Same server name now resolves to a different endpoint or credentials.
    let imposter = FakeRemote::working(REMOTE_ID);
    h.source.set(SERVER, "fp:endpoint-b", Arc::clone(&imposter));
    let restarted = h.restart();
    restarted.resume_all().await;
    run_for_ms(600_000).await;

    assert_eq!(
        imposter.gets(),
        0,
        "saved task id must not reach another server"
    );
    assert_eq!(imposter.cancels(), 0);
    assert_eq!(h.remote.gets(), 0);
    let record = h.record(&id).await;
    assert!(matches!(record.remote_state, McpRemoteState::Unresolved(_)));
    assert!(record.state_label().starts_with("unresolved"));
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Running);
    assert!(!restarted.is_polling(&id));

    // Cancel sends nothing either. The task is unresolved, so the user can
    // dismiss it locally; it is closed as Lost, never as cancelled.
    let outcome = restarted.cancel(&id).await.unwrap();
    assert_eq!(outcome, CancelOutcome::AbandonedLocally);
    assert_eq!(imposter.cancels(), 0);
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Lost);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_unknown_server_is_unresolved() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.tracker.shutdown();
    h.source.set_error(SERVER, ResolveError::Unknown);
    let restarted = h.restart();
    restarted.resume_all().await;
    run_for_ms(60_000).await;

    assert!(matches!(
        h.record(&id).await.remote_state,
        McpRemoteState::Unresolved(_)
    ));
    assert_eq!(h.remote.gets(), 0);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_unavailable_server_backs_off_then_recovers() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.tracker.shutdown();
    h.source
        .set_error(SERVER, ResolveError::Unavailable("connecting".into()));
    let restarted = h.restart();
    restarted.resume_all().await;
    run_for_ms(100).await;
    let record = h.record(&id).await;
    assert!(matches!(
        record.remote_state,
        McpRemoteState::RemoteUnavailable(_)
    ));
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Running);
    assert_eq!(h.remote.gets(), 0);

    // Server comes back: the loop is still alive and picks it up.
    h.source.set(SERVER, IDENTITY, Arc::clone(&h.remote));
    h.remote.set_default(Ok(TaskPoll::Task(snap(
        REMOTE_ID,
        RemoteStatus::Working,
        5,
    ))));
    run_for_ms(60_000).await;
    assert!(h.remote.gets() >= 1);
    assert_eq!(h.record(&id).await.remote_state, McpRemoteState::Tracking);
}

// ---- polling --------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_lost_notification_polls_at_the_suggested_interval() {
    let h = Harness::new();
    let mut handle = snap(REMOTE_ID, RemoteStatus::Working, 0);
    handle.poll_interval_ms = Some(2_000);
    h.register(handle).await;

    run_for_ms(1_900).await;
    assert_eq!(h.remote.gets(), 0, "no sooner than the suggested interval");
    run_for_ms(200).await;
    assert_eq!(h.remote.gets(), 1);
    run_for_ms(2_000).await;
    assert_eq!(h.remote.gets(), 2);
    let offsets = h.remote.get_offsets_ms();
    assert!(offsets[1] - offsets[0] >= 2_000);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_poll_interval_defaults_to_5s_and_is_clamped() {
    let mut record = McpTaskRecord {
        server: "s".into(),
        identity: "i".into(),
        remote_task_id: "t".into(),
        tool: "x".into(),
        tool_call_id: "c".into(),
        remote_status: RemoteStatus::Working,
        status_message: None,
        created_at: ts(0),
        last_updated_at: ts(0),
        ttl_ms: None,
        poll_interval_ms: None,
        expires_at: None,
        cancel_requested: false,
        cancel_error: None,
        result: None,
        result_truncated: false,
        error: None,
        input_requests: None,
        remote_state: McpRemoteState::Tracking,
    };
    assert_eq!(poll_delay(&record, 0), Duration::from_secs(5));
    record.poll_interval_ms = Some(10);
    assert_eq!(poll_delay(&record, 0), Duration::from_secs(1));
    record.poll_interval_ms = Some(86_400_000);
    assert_eq!(poll_delay(&record, 0), Duration::from_secs(300));
    record.poll_interval_ms = Some(2_000);
    assert_eq!(poll_delay(&record, 1), Duration::from_secs(4));
    assert_eq!(poll_delay(&record, 3), Duration::from_secs(16));
    assert_eq!(poll_delay(&record, 40), Duration::from_secs(300));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_stale_snapshot_does_not_regress_newer_state() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut newer = snap(REMOTE_ID, RemoteStatus::InputRequired, 50);
    newer.status_message = Some("newer".into());
    h.remote.push(Ok(TaskPoll::Task(newer)));
    // Older snapshot claiming `working`, then an equal-time conflicting one.
    let mut stale = snap(REMOTE_ID, RemoteStatus::Working, 30);
    stale.status_message = Some("stale".into());
    h.remote.push(Ok(TaskPoll::Task(stale)));
    let mut equal = snap(REMOTE_ID, RemoteStatus::Working, 50);
    equal.status_message = Some("equal".into());
    h.remote.push(Ok(TaskPoll::Task(equal)));
    h.remote.set_default(Ok(TaskPoll::Task(snap(
        REMOTE_ID,
        RemoteStatus::Working,
        10,
    ))));

    run_for_ms(21_000).await;
    assert!(h.remote.gets() >= 3);
    let record = h.record(&id).await;
    assert_eq!(record.remote_status, RemoteStatus::InputRequired);
    assert_eq!(record.status_message.as_deref(), Some("newer"));
    assert_eq!(record.last_updated_at, ts(50));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_hint_triggers_an_immediate_poll() {
    let h = Harness::new();
    let mut handle = snap(REMOTE_ID, RemoteStatus::Working, 0);
    handle.poll_interval_ms = Some(300_000);
    let id = h.register(handle).await;
    run_for_ms(1_000).await;
    assert_eq!(h.remote.gets(), 0);

    h.remote.push(Ok(TaskPoll::Task(snap(
        REMOTE_ID,
        RemoteStatus::InputRequired,
        3,
    ))));
    assert!(h.tracker.hint(SERVER, REMOTE_ID));
    run_for_ms(50).await;
    assert_eq!(h.remote.gets(), 1);
    // The state came from tasks/get, not from the hint.
    assert_eq!(
        h.record(&id).await.remote_status,
        RemoteStatus::InputRequired
    );
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_hint_ignores_unknown_and_other_servers_tasks() {
    let h = Harness::new();
    let mut handle = snap(REMOTE_ID, RemoteStatus::Working, 0);
    handle.poll_interval_ms = Some(300_000);
    h.register(handle.clone()).await;

    // A second server that also has a task with the same remote id.
    let other_remote = FakeRemote::working(REMOTE_ID);
    h.source.set("other", "fp:other", Arc::clone(&other_remote));
    h.tracker
        .register(handle, "other", "fp:other", "scan", "session-1", "call-2")
        .await
        .unwrap();

    assert!(!h.tracker.hint("nobody", REMOTE_ID));
    assert!(!h.tracker.hint(SERVER, "unknown-task"));
    run_for_ms(50).await;
    assert_eq!(h.remote.gets() + other_remote.gets(), 0);

    assert!(h.tracker.hint("other", REMOTE_ID));
    run_for_ms(50).await;
    assert_eq!(other_remote.gets(), 1);
    assert_eq!(
        h.remote.gets(),
        0,
        "hint for one server must not poll another"
    );
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_transient_error_keeps_state_backs_off_and_is_not_failure() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    for _ in 0..3 {
        h.remote.push(Err("connection reset".into()));
    }
    run_for_ms(5_100).await;
    let mid = h.record(&id).await;
    assert!(matches!(
        mid.remote_state,
        McpRemoteState::RemoteUnavailable(_)
    ));
    assert_eq!(mid.remote_status, RemoteStatus::Working, "last state kept");
    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Running);
    assert!(status.error.is_none());
    assert!(mid.state_label().contains("remote unavailable"));
    assert_eq!(mid.short_state(), "unavailable");

    run_for_ms(80_000).await;
    let offsets = h.remote.get_offsets_ms();
    // Polls at 5 s, 15 s (+10 s), 35 s (+20 s) fail; the one at 75 s (+40 s)
    // succeeds and clears the unavailable condition.
    assert!(offsets[1] - offsets[0] >= 10_000, "{offsets:?}");
    assert!(offsets[2] - offsets[1] >= 20_000, "{offsets:?}");
    assert_eq!(h.record(&id).await.remote_state, McpRemoteState::Tracking);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_completed_persists_result_and_stops_polling() {
    let h = Harness::new();
    let mut events = Bus::global().subscribe();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut done = snap(REMOTE_ID, RemoteStatus::Completed, 10);
    done.result =
        Some(json!({"content": [{"type": "text", "text": "all good"}], "isError": false}));
    h.remote.push(Ok(TaskPoll::Task(done)));
    run_for_ms(5_100).await;

    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Completed);
    let record = status.mcp_task.unwrap();
    assert_eq!(record.remote_status, RemoteStatus::Completed);
    assert_eq!(
        record.result.as_ref().unwrap()["content"][0]["text"],
        "all good"
    );
    assert_eq!(h.manager.output(&id).await.as_deref(), Some("all good"));
    let gets = h.remote.gets();
    run_for_ms(600_000).await;
    assert_eq!(h.remote.gets(), gets, "no polling after a terminal state");
    assert!(!h.tracker.is_polling(&id));

    // Same publish path as other tasks: one BackgroundTaskCompleted.
    let mut completions = 0;
    while let Ok(event) = events.try_recv() {
        if let BusEvent::BackgroundTaskCompleted(done) = event
            && done.task_id == id
        {
            assert_eq!(done.status, BackgroundTaskStatus::Completed);
            assert!(done.output_preview.contains("all good"));
            completions += 1;
        }
    }
    assert_eq!(completions, 1);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_failed_keeps_the_error_and_is_not_a_success() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut failed = snap(REMOTE_ID, RemoteStatus::Failed, 10);
    failed.error = Some(json!({"code": -32000, "message": "disk full"}));
    h.remote.push(Ok(TaskPoll::Task(failed)));
    run_for_ms(5_100).await;

    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Failed);
    assert_eq!(status.error.as_deref(), Some("disk full"));
    let record = status.mcp_task.unwrap();
    assert_eq!(record.error.as_ref().unwrap()["code"], -32000);
    assert!(record.result.is_none());
    assert!(!h.tracker.is_polling(&id));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_input_required_is_surfaced_bounded_and_never_answered() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut waiting = snap(REMOTE_ID, RemoteStatus::InputRequired, 10);
    waiting.input_requests = Some(json!({"req-1": {"method": "elicitation/create",
        "params": {"message": "x".repeat(50_000)}}}));
    h.remote.set_default(Ok(TaskPoll::Task(waiting)));
    run_for_ms(20_000).await;

    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Running);
    let record = status.mcp_task.unwrap();
    assert_eq!(record.remote_status, RemoteStatus::InputRequired);
    let stored = record.input_requests.as_ref().unwrap().as_str().unwrap();
    assert!(
        stored.len() <= crate::background::MCP_TASK_INPUT_CAP,
        "{}",
        stored.len()
    );
    let lines = record.detail_lines();
    let input_line = lines
        .iter()
        .find(|l| l.starts_with("Input required"))
        .unwrap();
    assert!(input_line.contains("untrusted, not answered"));
    assert!(!input_line.contains('\n'));
    // Only tasks/get was ever sent: nothing was answered or cancelled.
    assert_eq!(h.remote.cancels(), 0);
    assert!(
        h.remote.gets() >= 2,
        "still polling while input is outstanding"
    );
}

// ---- cancellation ---------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_cancel_is_requested_then_confirmed_by_the_server() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.remote.push(Ok(TaskPoll::Task(snap(
        REMOTE_ID,
        RemoteStatus::Cancelled,
        5,
    ))));
    let outcome = h.tracker.cancel(&id).await.unwrap();

    assert_eq!(outcome, CancelOutcome::Confirmed);
    assert_eq!(h.remote.cancels(), 1);
    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Failed);
    assert_eq!(status.error.as_deref(), Some("Cancelled by remote"));
    let record = status.mcp_task.unwrap();
    assert!(record.cancel_requested);
    assert_eq!(record.remote_status, RemoteStatus::Cancelled);
    assert_eq!(record.state_label(), "cancelled (confirmed)");
    run_for_ms(60_000).await;
    assert!(!h.tracker.is_polling(&id));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_cancel_ack_without_terminal_state_stays_requested() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let outcome = h.tracker.cancel(&id).await.unwrap();
    assert_eq!(outcome, CancelOutcome::Requested);
    let record = h.record(&id).await;
    assert!(record.cancel_requested);
    assert_eq!(record.remote_status, RemoteStatus::Working);
    assert_eq!(record.state_label(), "working, cancellation requested");
    assert_eq!(record.short_state(), "cancel-req");
    assert!(h.tracker.is_polling(&id));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_cancel_racing_with_completion_keeps_the_server_outcome() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut done = snap(REMOTE_ID, RemoteStatus::Completed, 5);
    done.result = Some(json!({"content": [{"type": "text", "text": "finished anyway"}]}));
    h.remote.push(Ok(TaskPoll::Task(done)));
    let outcome = h.tracker.cancel(&id).await.unwrap();

    assert_eq!(
        outcome,
        CancelOutcome::ServerFinished(RemoteStatus::Completed)
    );
    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Completed);
    let record = status.mcp_task.unwrap();
    assert_eq!(record.remote_status, RemoteStatus::Completed);
    let label = record.state_label();
    assert!(label.contains("server finished first"), "{label}");
    assert!(!label.contains("cancelled"), "{label}");
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_cancel_racing_with_failure_keeps_the_failure() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut failed = snap(REMOTE_ID, RemoteStatus::Failed, 5);
    failed.error = Some(json!({"code": -1, "message": "boom"}));
    h.remote.push(Ok(TaskPoll::Task(failed)));
    let outcome = h.tracker.cancel(&id).await.unwrap();
    assert_eq!(outcome, CancelOutcome::ServerFinished(RemoteStatus::Failed));
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Failed);
    assert_eq!(h.status(&id).await.error.as_deref(), Some("boom"));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_cancel_request_failure_stays_unconfirmed_and_keeps_polling() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    *h.remote.cancel_result.lock().unwrap() = Err("connection lost".into());
    let outcome = h.tracker.cancel(&id).await.unwrap();

    assert!(matches!(outcome, CancelOutcome::Unacknowledged(_)));
    let record = h.record(&id).await;
    assert!(record.cancel_requested);
    assert!(record.cancel_error.is_some());
    assert_ne!(record.remote_status, RemoteStatus::Cancelled);
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Running);
    assert!(record.state_label().contains("unacknowledged"));
    assert!(h.tracker.is_polling(&id));
    // Reachable again: polling continues and the server decides.
    h.remote.set_default(Ok(TaskPoll::Task(snap(
        REMOTE_ID,
        RemoteStatus::Cancelled,
        9,
    ))));
    run_for_ms(5_100).await;
    assert_eq!(h.record(&id).await.remote_status, RemoteStatus::Cancelled);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_cancel_is_refused_for_terminal_and_untracked_tasks() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.remote.push(Ok(TaskPoll::Task(snap(
        REMOTE_ID,
        RemoteStatus::Completed,
        5,
    ))));
    run_for_ms(5_100).await;
    let error = h.tracker.cancel(&id).await.unwrap_err();
    assert!(error.to_string().contains("already finished"), "{error}");
    assert_eq!(h.remote.cancels(), 0);

    let error = h.tracker.cancel("no-such-task").await.unwrap_err();
    assert!(error.to_string().contains("not found"), "{error}");
    let plain = h
        .manager
        .spawn("bash", "session-1", |_| async {
            std::future::pending::<()>().await;
            Ok(crate::background::TaskResult::completed(Some(0)))
        })
        .await;
    let error = h.tracker.cancel(&plain.task_id).await.unwrap_err();
    assert!(error.to_string().contains("not an MCP task"), "{error}");
    assert_eq!(h.remote.cancels(), 0);
    h.manager.cancel(&plain.task_id).await.unwrap();
}

// ---- expiry ---------------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_expiry_stops_polling_and_is_distinct_from_terminal_states() {
    let h = Harness::new();
    let mut handle = snap(REMOTE_ID, RemoteStatus::Working, 0);
    handle.ttl_ms = Some(30_000);
    let id = h.register(handle).await;
    assert_eq!(h.record(&id).await.expires_at, Some(ts(30)));

    run_for_ms(31_000).await;
    let gets = h.remote.gets();
    assert!((1..=6).contains(&gets), "{gets}");
    let record = h.record(&id).await;
    assert_eq!(record.remote_state, McpRemoteState::Expired);
    assert_eq!(
        record.remote_status,
        RemoteStatus::Working,
        "last status kept"
    );
    assert_eq!(record.remote_task_id, REMOTE_ID);
    assert!(
        record.result.is_none(),
        "an unretrieved result is never claimed"
    );
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Lost);
    assert!(record.state_label().starts_with("expired"));
    run_for_ms(600_000).await;
    assert_eq!(h.remote.gets(), gets);
    assert!(!h.tracker.is_polling(&id));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_resume_of_an_already_expired_task_does_not_poll() {
    let h = Harness::new();
    let mut handle = snap(REMOTE_ID, RemoteStatus::Working, 0);
    handle.ttl_ms = Some(10_000);
    let id = h.register(handle).await;
    h.tracker.shutdown();
    // The daemon is down while the TTL passes.
    run_for_ms(20_000).await;

    let restarted = h.restart();
    restarted.resume_all().await;
    run_for_ms(100).await;
    assert_eq!(h.remote.gets(), 0);
    assert_eq!(h.record(&id).await.remote_state, McpRemoteState::Expired);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_null_ttl_never_expires_locally() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    run_for_ms(3_600_000).await;
    let record = h.record(&id).await;
    assert!(record.expires_at.is_none());
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Running);
    assert!(h.remote.gets() > 100);
}

// ---- persistence ----------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_persistence_round_trip_holds_no_credential_values() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut done = snap(REMOTE_ID, RemoteStatus::Completed, 7);
    done.result = Some(json!({"content": [{"type": "text", "text": "ok"}]}));
    h.remote.push(Ok(TaskPoll::Task(done)));
    run_for_ms(5_100).await;

    let raw = std::fs::read_to_string(h.manager.status_path_for(&id)).unwrap();
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(parsed["mcp_task"]["identity"], IDENTITY);
    fn walk(value: &Value, hits: &mut Vec<String>) {
        const WORDS: [&str; 7] = [
            "token",
            "secret",
            "authorization",
            "password",
            "api_key",
            "bearer",
            "cookie",
        ];
        let flag = |text: &str, hits: &mut Vec<String>| {
            let lower = text.to_ascii_lowercase();
            if WORDS.iter().any(|word| lower.contains(word)) {
                hits.push(text.to_string());
            }
        };
        match value {
            Value::Object(map) => {
                for (key, inner) in map {
                    flag(key, hits);
                    walk(inner, hits);
                }
            }
            Value::Array(items) => items.iter().for_each(|item| walk(item, hits)),
            Value::String(text) => flag(text, hits),
            _ => {}
        }
    }
    let mut hits = Vec::new();
    walk(&parsed, &mut hits);
    assert!(
        hits.is_empty(),
        "credential-like content in status file: {hits:?}"
    );

    // A different manager over the same directory reads the identical record.
    let reread = BackgroundTaskManager::with_output_dir(
        h.manager
            .status_path_for(&id)
            .parent()
            .unwrap()
            .to_path_buf(),
    );
    let listed = reread.list().await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].mcp_task, Some(h.record(&id).await));
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_oversized_result_is_bounded_and_flagged() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut done = snap(REMOTE_ID, RemoteStatus::Completed, 7);
    done.result = Some(json!({"content": [{"type": "text", "text": "y".repeat(300_000)}]}));
    h.remote.push(Ok(TaskPoll::Task(done)));
    run_for_ms(5_100).await;

    let record = h.record(&id).await;
    assert!(record.result_truncated);
    let stored = record.result.unwrap();
    assert!(stored.as_str().unwrap().len() <= crate::background::MCP_TASK_RESULT_CAP);
    let size = std::fs::metadata(h.manager.status_path_for(&id))
        .unwrap()
        .len();
    assert!(size < 200_000, "status file stayed bounded: {size}");
}

#[test]
fn mcp_task_tracking_legacy_status_files_have_no_mcp_task() {
    let status = TaskStatusFile {
        task_id: "t".into(),
        tool_name: "bash".into(),
        display_name: None,
        session_id: "s".into(),
        status: BackgroundTaskStatus::Running,
        exit_code: None,
        error: None,
        started_at: "2026-01-01T00:00:00Z".into(),
        completed_at: None,
        duration_secs: None,
        pid: None,
        pid_start_time: None,
        owner_generation: None,
        owner_pid: None,
        owner_instance: None,
        detached: false,
        notify: true,
        wake: false,
        progress: None,
        event_history: Vec::new(),
        stall_wake_seconds: None,
        mcp_task: None,
    };
    let json = serde_json::to_string(&status).unwrap();
    assert!(!json.contains("mcp_task"));
    let back: TaskStatusFile = serde_json::from_str(&json).unwrap();
    assert!(back.mcp_task.is_none());
}

// ---- UI rows --------------------------------------------------------------

fn sample_record() -> McpTaskRecord {
    McpTaskRecord {
        server: "srv".into(),
        identity: IDENTITY.into(),
        remote_task_id: REMOTE_ID.into(),
        tool: "build".into(),
        tool_call_id: "call-1".into(),
        remote_status: RemoteStatus::Working,
        status_message: Some("compiling".into()),
        created_at: ts(0),
        last_updated_at: ts(1),
        ttl_ms: Some(60_000),
        poll_interval_ms: None,
        expires_at: Some(ts(60)),
        cancel_requested: false,
        cancel_error: None,
        result: None,
        result_truncated: false,
        error: None,
        input_requests: None,
        remote_state: McpRemoteState::Tracking,
    }
}

#[test]
fn mcp_task_tracking_ui_state_labels_cover_every_condition() {
    let mut record = sample_record();
    assert_eq!(record.kind_label(), "mcp:srv:build");
    assert_eq!(record.state_label(), "working");
    record.remote_status = RemoteStatus::InputRequired;
    assert_eq!(record.state_label(), "input_required");
    record.cancel_requested = true;
    assert_eq!(
        record.state_label(),
        "input_required, cancellation requested"
    );
    record.remote_status = RemoteStatus::Cancelled;
    assert_eq!(record.state_label(), "cancelled (confirmed)");
    record.remote_status = RemoteStatus::Failed;
    assert!(record.state_label().contains("server finished first"));
    record.cancel_requested = false;
    record.remote_status = RemoteStatus::Working;
    record.remote_state = McpRemoteState::RemoteUnavailable("timeout".into());
    assert_eq!(record.state_label(), "working, remote unavailable: timeout");
    record.remote_state = McpRemoteState::Unresolved("identity changed".into());
    assert_eq!(record.state_label(), "unresolved: identity changed");
    record.remote_state = McpRemoteState::Expired;
    assert_eq!(record.state_label(), "expired (last working)");
    record.remote_state = McpRemoteState::NotFound;
    assert_eq!(record.state_label(), "expired/not found (last working)");
}

#[test]
fn mcp_task_tracking_ui_detail_lines_show_result_error_input_and_expiry() {
    let mut record = sample_record();
    record.remote_status = RemoteStatus::Completed;
    record.result = Some(json!({"content": [{"type": "text", "text": "line1\nline2\u{1b}[31m"}]}));
    record.result_truncated = true;
    record.error = Some(json!({"code": -1, "message": "bad"}));
    record.input_requests = Some(json!({"a": 1}));
    let lines = record.detail_lines();
    let text = lines.join("\n");
    assert!(text.contains("MCP task: mcp:srv:build"));
    assert!(text.contains("Remote task id: remote-1"));
    assert!(text.contains("Expires: "));
    assert!(text.contains("Result (truncated): "));
    assert!(text.contains("Remote error: "));
    assert!(text.contains("Input required (untrusted, not answered): "));
    assert!(text.contains("Remote message: compiling"));
    // Untrusted text cannot add rows or terminal escapes.
    assert_eq!(lines.len(), text.lines().count());
    assert!(!text.contains('\u{1b}'));
}

#[test]
fn mcp_task_tracking_ui_rows_truncate_by_display_cells_without_wrapping() {
    use jcode_render_core::{ColumnWidth, WidthMeasure};
    let cells = |text: &str| ColumnWidth.measure(text);
    let wide = "日本語のとても長い理由です。".repeat(20);
    for width in [1usize, 2, 3, 10, 25] {
        let cut = truncate_cells(&wide, width);
        assert!(
            cells(&cut) <= width,
            "{width}: {cut} is {} cells",
            cells(&cut)
        );
        assert!(!cut.contains('\n'));
        if width > 1 {
            assert!(cut.ends_with('…'), "{cut}");
        }
    }
    assert_eq!(truncate_cells("short", 10), "short");
    assert_eq!(truncate_cells("anything", 0), "");

    let mut record = sample_record();
    record.remote_state = McpRemoteState::Unresolved(format!("{wide}\nsecond line"));
    let label = record.state_label();
    assert!(!label.contains('\n'));
    assert!(
        cells(&label) <= "unresolved: ".len() + 60,
        "{}",
        cells(&label)
    );
    for line in record.detail_lines() {
        assert!(!line.contains('\n'));
    }
}

// ---- review fixes ----------------------------------------------------------

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_equal_second_terminal_snapshot_is_accepted() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 5)).await;
    // Same one-second timestamp, but the task finished: must not be ignored.
    h.remote.push(Ok(TaskPoll::Task(snap(
        REMOTE_ID,
        RemoteStatus::Completed,
        5,
    ))));
    run_for_ms(5_100).await;
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Completed);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_terminal_state_is_never_replaced_by_a_working_snapshot() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.manager
        .update_mcp_task(&id, |r| {
            let t = snap(REMOTE_ID, RemoteStatus::Completed, 9);
            apply_snapshot(r, &t, base_time())
        })
        .await
        .unwrap();
    let stored = h.record(&id).await;
    assert_eq!(stored.remote_status, RemoteStatus::Completed);
    let mut again = stored.clone();
    assert!(!apply_snapshot(
        &mut again,
        &snap(REMOTE_ID, RemoteStatus::Working, 99),
        base_time()
    ));
    assert_eq!(again.remote_status, RemoteStatus::Completed);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_unresolved_task_can_be_dismissed_without_contacting_a_server() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.source.set_error(SERVER, ResolveError::Unknown);
    run_for_ms(5_100).await;
    assert!(matches!(
        h.record(&id).await.remote_state,
        McpRemoteState::Unresolved(_)
    ));
    let outcome = h.tracker.cancel(&id).await.unwrap();
    assert_eq!(outcome, CancelOutcome::AbandonedLocally);
    assert_eq!(h.remote.cancels(), 0, "nothing was sent");
    assert_eq!(h.status(&id).await.status, BackgroundTaskStatus::Lost);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_cancel_with_identity_mismatch_never_reaches_the_server() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    h.source
        .set(SERVER, "fp:other-endpoint", Arc::clone(&h.remote));
    let error = h.tracker.cancel(&id).await.unwrap_err();
    assert!(error.to_string().contains("no longer matches"), "{error}");
    assert_eq!(h.remote.cancels(), 0);
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_hint_flood_keeps_minimum_poll_spacing() {
    let h = Harness::new();
    h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    run_for_ms(10).await;
    for _ in 0..20 {
        h.tracker.hint(SERVER, REMOTE_ID);
        run_for_ms(20).await;
    }
    // 20 hints in 400 ms with a 1 s floor produce at most one extra poll.
    assert!(h.remote.gets() <= 2, "gets = {}", h.remote.gets());
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_failed_error_text_is_bounded_in_the_status() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut failed = snap(REMOTE_ID, RemoteStatus::Failed, 5);
    failed.error = Some(json!({"code": -32000, "message": "x".repeat(70_000)}));
    h.remote.push(Ok(TaskPoll::Task(failed)));
    run_for_ms(5_100).await;
    let status = h.status(&id).await;
    assert_eq!(status.status, BackgroundTaskStatus::Failed);
    let shown = status.error.unwrap();
    assert!(
        shown.chars().count() <= 300,
        "len {}",
        shown.chars().count()
    );
}

#[tokio::test(start_paused = true)]
async fn mcp_task_tracking_kind_label_drops_backticks_and_bidi_controls() {
    let h = Harness::new();
    let id = h.register(snap(REMOTE_ID, RemoteStatus::Working, 0)).await;
    let mut record = h.record(&id).await;
    record.server = "s`rv\u{202E}".into();
    record.tool = "to`ol".into();
    assert_eq!(record.kind_label(), "mcp:srv:tool");
}
