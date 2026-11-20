use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const TEMP_SERVER_ENV: &str = "JCODE_TEMP_SERVER";
const SERVER_SCOPE_ENV: &str = "JCODE_SERVER_SCOPE";
const OWNER_PID_ENV: &str = "JCODE_SERVER_OWNER_PID";
const TEMP_IDLE_SECS_ENV: &str = "JCODE_TEMP_SERVER_IDLE_SECS";
const DEFAULT_TEMP_IDLE_SECS: u64 = 30 * 60;
const TEMP_SERVER_EXIT_CODE: i32 = super::EXIT_IDLE_TIMEOUT;

/// How long shutdown waits for an in-flight session save.
const SAVE_DRAIN_BOUND: Duration = Duration::from_secs(5);

/// Wait (bounded) for an in-flight session save before the process exits.
/// Exiting between a checkpoint's snapshot write and journal delete used to
/// duplicate history on the next load (#1632). Returns whether the drain
/// finished; an incomplete drain is logged as incomplete and never reported as
/// complete.
pub(crate) async fn drain_session_saves_before_exit() -> bool {
    crate::logging::info("Server stopping: no new work admitted, draining session saves");
    begin_shutdown();
    let drained =
        tokio::task::spawn_blocking(|| crate::session::drain_saves_for_shutdown(SAVE_DRAIN_BOUND))
            .await
            .unwrap_or(false);
    if !drained {
        crate::logging::warn(&format!(
            "Shutdown: a session save was still running after {}s; exiting with the save incomplete (recovery keeps its journal)",
            SAVE_DRAIN_BOUND.as_secs()
        ));
    }
    drained
}

/// Whether new work is admitted. One process-wide instance gates the real
/// server; tests use their own so they never touch it.
#[derive(Default)]
pub(crate) struct Admission(std::sync::atomic::AtomicBool);

impl Admission {
    const fn new() -> Self {
        Self(std::sync::atomic::AtomicBool::new(false))
    }

    pub(crate) fn stopping(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn set_stopping(&self, stopping: bool) {
        self.0.store(stopping, std::sync::atomic::Ordering::SeqCst);
    }
}

static ADMISSION: Admission = Admission::new();

/// Whether this process is stopping or reloading. New work is refused from the
/// moment this is set; persisted work stays for the next process to recover.
pub(crate) fn shutting_down() -> bool {
    ADMISSION.stopping()
}

/// Stop admitting new work. Idempotent.
pub(crate) fn begin_shutdown() {
    ADMISSION.set_stopping(true);
}

/// How a server scope handles idleness. The shared daemon never idle-exits;
/// scheduled and ambient work run inside it and `Restart=always` turns every
/// idle exit into churn. Only a temporary server retires itself.
pub(crate) enum IdleExit {
    Never,
    After(TemporaryServerPolicy),
}

impl IdleExit {
    pub(crate) fn for_scope(policy: Option<TemporaryServerPolicy>) -> Self {
        policy.map_or(Self::Never, Self::After)
    }
}

/// Whether the recorded owner of a temporary server is gone. A server never
/// treats itself as its own dead owner.
pub(crate) fn owner_exited(policy: &TemporaryServerPolicy, alive: impl Fn(u32) -> bool) -> bool {
    policy
        .owner_pid
        .is_some_and(|owner| owner != std::process::id() && !alive(owner))
}

/// Everything that must keep a server alive, gathered so the decision is one
/// pure function. Durable state is read, never copied.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorkPresence {
    pub(crate) clients: usize,
    pub(crate) live_headless_worker: bool,
    /// Pending or in-flight scheduled items, or unconsumed directives.
    pub(crate) ambient_pending: bool,
    /// Background tasks still `Running`.
    pub(crate) background_running: bool,
}

impl WorkPresence {
    pub(crate) fn idle(&self) -> bool {
        super::idle_monitor_should_start(self.clients, self.live_headless_worker)
            && !self.ambient_pending
            && !self.background_running
    }
}

async fn gather_work_presence(
    client_count: &RwLock<usize>,
    sessions: &super::SessionAgents,
    swarm_state: &super::SwarmState,
) -> WorkPresence {
    let ambient_pending = tokio::task::spawn_blocking(crate::ambient::has_pending_work)
        .await
        // An unreadable answer is not "no work".
        .unwrap_or(true);
    let background_running = crate::background::global().has_running_tasks().await;
    WorkPresence {
        clients: *client_count.read().await,
        live_headless_worker: super::has_live_headless_worker(sessions, swarm_state).await,
        ambient_pending,
        background_running,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TemporaryServerPolicy {
    pub(crate) owner_pid: Option<u32>,
    pub(crate) idle_timeout_secs: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TemporaryServerMetadata {
    schema_version: u32,
    scope: String,
    pid: u32,
    ppid: Option<u32>,
    owner_pid: Option<u32>,
    started_at: String,
    socket_path: String,
    debug_socket_path: String,
    idle_timeout_secs: u64,
    argv: Vec<String>,
}

pub fn configure_temporary_server(owner_pid: Option<u32>, idle_timeout_secs: Option<u64>) {
    crate::env::set_var(TEMP_SERVER_ENV, "1");
    crate::env::set_var(SERVER_SCOPE_ENV, "temporary");
    if let Some(owner_pid) = owner_pid {
        crate::env::set_var(OWNER_PID_ENV, owner_pid.to_string());
    }
    if let Some(idle_timeout_secs) = idle_timeout_secs {
        crate::env::set_var(TEMP_IDLE_SECS_ENV, idle_timeout_secs.to_string());
    }
}

pub(crate) fn temporary_server_policy_from_env() -> Option<TemporaryServerPolicy> {
    if !temporary_server_enabled() {
        return None;
    }
    let owner_pid = std::env::var(OWNER_PID_ENV)
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|pid| *pid > 0);
    let idle_timeout_secs = std::env::var(TEMP_IDLE_SECS_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_TEMP_IDLE_SECS);

    Some(TemporaryServerPolicy {
        owner_pid,
        idle_timeout_secs,
    })
}

pub(crate) fn temporary_server_enabled() -> bool {
    env_truthy(TEMP_SERVER_ENV)
        || std::env::var(SERVER_SCOPE_ENV)
            .ok()
            .map(|value| value.eq_ignore_ascii_case("temporary"))
            .unwrap_or(false)
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

pub(crate) fn metadata_path(socket_path: &Path) -> PathBuf {
    let filename = socket_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("jcode.sock");
    socket_path.with_file_name(format!("{filename}.server.json"))
}

pub(crate) fn write_temporary_metadata(
    socket_path: &Path,
    debug_socket_path: &Path,
    policy: &TemporaryServerPolicy,
) -> Option<PathBuf> {
    let path = metadata_path(socket_path);
    let metadata = TemporaryServerMetadata {
        schema_version: 1,
        scope: "temporary".to_string(),
        pid: std::process::id(),
        ppid: parent_pid(),
        owner_pid: policy.owner_pid,
        started_at: chrono::Utc::now().to_rfc3339(),
        socket_path: socket_path.display().to_string(),
        debug_socket_path: debug_socket_path.display().to_string(),
        idle_timeout_secs: policy.idle_timeout_secs,
        argv: std::env::args().collect(),
    };

    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        crate::logging::warn(&format!(
            "Failed to create temporary server metadata directory {}: {}",
            parent.display(),
            error
        ));
        return None;
    }

    match serde_json::to_vec_pretty(&metadata)
        .ok()
        .and_then(|bytes| std::fs::write(&path, bytes).ok().map(|_| ()))
    {
        Some(()) => Some(path),
        None => {
            crate::logging::warn(&format!(
                "Failed to write temporary server metadata {}",
                path.display()
            ));
            None
        }
    }
}

pub(crate) fn cleanup_temporary_metadata(socket_path: &Path) {
    let _ = std::fs::remove_file(metadata_path(socket_path));
}

pub(crate) fn spawn_temporary_lifecycle_monitor(
    client_count: Arc<RwLock<usize>>,
    sessions: super::SessionAgents,
    swarm_state: super::SwarmState,
    socket_path: PathBuf,
    debug_socket_path: PathBuf,
    server_name: String,
    policy: TemporaryServerPolicy,
) {
    tokio::spawn(async move {
        let mut idle_since: Option<Instant> = None;
        let mut check_interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            check_interval.tick().await;

            if owner_exited(&policy, process_alive) {
                crate::logging::info(&format!(
                    "Temporary server owner pid {:?} is gone. Shutting down.",
                    policy.owner_pid
                ));
                shutdown_temporary_server(&server_name, &socket_path, &debug_socket_path).await;
            }

            let presence = gather_work_presence(&client_count, &sessions, &swarm_state).await;
            if presence.idle() {
                if idle_since.is_none() {
                    idle_since = Some(Instant::now());
                    crate::logging::info(&format!(
                        "Temporary server has no clients. It will exit after {} seconds idle.",
                        policy.idle_timeout_secs
                    ));
                }

                if let Some(since) = idle_since
                    && since.elapsed().as_secs() >= policy.idle_timeout_secs
                {
                    crate::logging::info(&format!(
                        "Temporary server idle for {} seconds. Shutting down.",
                        since.elapsed().as_secs()
                    ));
                    shutdown_temporary_server(&server_name, &socket_path, &debug_socket_path).await;
                }
            } else {
                if idle_since.is_some() {
                    crate::logging::info(
                        "Temporary server client connected. Idle timer cancelled.",
                    );
                }
                idle_since = None;
            }
        }
    });
}

async fn shutdown_temporary_server(
    server_name: &str,
    socket_path: &Path,
    debug_socket_path: &Path,
) -> ! {
    // Same orderly path as SIGTERM: refuse new work and drain saves first.
    // Pending scheduled/background work is persisted and recovered by the
    // next server, so it is never marked complete here.
    drain_session_saves_before_exit().await;
    let _ = crate::registry::unregister_server(server_name).await;
    crate::transport::remove_socket(socket_path);
    crate::transport::remove_socket(debug_socket_path);
    cleanup_temporary_metadata(socket_path);
    // `std::process::exit` skips `DaemonLockGuard::drop`, so clear the lock
    // this process holds. Per-run sockets would otherwise leave one stale lock
    // file behind per run.
    #[cfg(unix)]
    super::socket::cleanup_held_daemon_lock();
    // Same for the build-hash sidecar the registry publisher writes next to
    // the socket: an ephemeral server must leave nothing behind.
    let _ = std::fs::remove_file(format!("{}.hash", socket_path.display()));
    std::process::exit(TEMP_SERVER_EXIT_CODE);
}

#[cfg(unix)]
fn parent_pid() -> Option<u32> {
    let ppid = unsafe { libc::getppid() };
    (ppid > 0).then_some(ppid as u32)
}

#[cfg(not(unix))]
fn parent_pid() -> Option<u32> {
    None
}

#[cfg(unix)]
pub(crate) fn process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }

    let rc = unsafe { libc::kill(pid as libc::pid_t, 0) };
    if rc == 0 {
        return true;
    }

    matches!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::EPERM)
    )
}

/// Owner liveness on non-Unix hosts.
///
/// This used to return `true` unconditionally, which was harmless only while
/// nothing depended on it. Now that a headless `run` owns a temporary server
/// through `--owner-pid`, an always-alive answer means the owner's exit is
/// never noticed on Windows and the server lingers until the idle timeout
/// (30 minutes by default, and the timer does not even start while workers are
/// alive). Delegate to the real check the registry already uses.
///
/// Note the asymmetry with the Unix arm above: that one treats `EPERM` as
/// alive, whereas `is_process_running` reports a process it cannot open as
/// gone. For an owner we spawned ourselves, under the same user, the query
/// right is available; and erring toward "gone" only retires a temporary
/// server early, which is the safe direction for one that is supposed to be
/// ephemeral.
#[cfg(not(unix))]
pub(crate) fn process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }

    crate::platform::is_process_running(pid)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EnvGuard {
        _lock: std::sync::MutexGuard<'static, ()>,
        entries: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    impl EnvGuard {
        fn capture(names: &[&'static str]) -> Self {
            // Shared process-wide lock: env vars are global, so a private
            // mutex here would race every other env-mutating test (issue #593).
            let _lock = crate::storage::lock_test_env();
            let entries = names
                .iter()
                .map(|name| (*name, std::env::var_os(name)))
                .collect();
            Self { _lock, entries }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, value) in &self.entries {
                if let Some(value) = value {
                    crate::env::set_var(name, value);
                } else {
                    crate::env::remove_var(name);
                }
            }
        }
    }

    #[test]
    fn temporary_policy_requires_explicit_marker() {
        let _guard = EnvGuard::capture(&[TEMP_SERVER_ENV, SERVER_SCOPE_ENV, OWNER_PID_ENV]);
        crate::env::remove_var(TEMP_SERVER_ENV);
        crate::env::remove_var(SERVER_SCOPE_ENV);
        crate::env::set_var(OWNER_PID_ENV, "123");

        assert_eq!(temporary_server_policy_from_env(), None);
    }

    #[test]
    fn temporary_policy_reads_owner_and_timeout() {
        let _guard = EnvGuard::capture(&[
            TEMP_SERVER_ENV,
            SERVER_SCOPE_ENV,
            OWNER_PID_ENV,
            TEMP_IDLE_SECS_ENV,
        ]);
        crate::env::set_var(SERVER_SCOPE_ENV, "temporary");
        crate::env::set_var(OWNER_PID_ENV, "123");
        crate::env::set_var(TEMP_IDLE_SECS_ENV, "42");

        assert_eq!(
            temporary_server_policy_from_env(),
            Some(TemporaryServerPolicy {
                owner_pid: Some(123),
                idle_timeout_secs: 42,
            })
        );
    }

    #[test]
    fn temporary_metadata_path_is_socket_scoped() {
        assert_eq!(
            metadata_path(Path::new("/tmp/example/jcode.sock")),
            PathBuf::from("/tmp/example/jcode.sock.server.json")
        );
    }

    #[cfg(unix)]
    #[test]
    fn current_process_is_alive() {
        assert!(process_alive(std::process::id()));
        assert!(!process_alive(0));
    }

    /// The non-Unix `process_alive` arm delegates to `is_process_running`, and
    /// that arm only compiles in CI. Exercise the helper itself here so a wrong
    /// path or signature cannot reach Windows unnoticed.
    ///
    /// Pid 0 is deliberately not asserted against the helper: on Unix
    /// `kill(0, 0)` signals the caller's whole process group and reports
    /// "alive", which is why `process_alive` screens pid 0 out before calling
    /// it rather than trusting it.
    #[test]
    fn platform_process_running_detects_this_process() {
        assert!(crate::platform::is_process_running(std::process::id()));
    }
}
