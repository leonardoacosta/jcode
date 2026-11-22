//! Native agent-state reporting for the herdr terminal multiplexer.
//!
//! When jcode runs inside a herdr pane (`HERDR_ENV=1`), it reports its state
//! (`idle` / `working` / `blocked`), the command that resumes the current
//! session, and releases the pane when the user quits. See [`derive_state`]
//! for what each state means and <https://herdr.dev/docs/add-herdr-support/>.
//!
//! Reports go through `"$HERDR_BIN_PATH" pane report-agent ...` on a single
//! background thread. Only the latest desired report is kept, every command
//! has a short timeout, and a failed report is retried with bounded backoff
//! until herdr accepts it or a newer report replaces it. A session change
//! clears the previous identity before reporting its replacement. Failures
//! never block the TUI. Outside herdr every entry point is a no-op.

use std::io::Read;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Stable integration id. Must not start with `herdr:`.
const SOURCE: &str = "jcode";
/// Agent name shown in herdr's sidebar and `herdr agent list`.
const AGENT: &str = "jcode";
/// Upper bound on a single herdr CLI call.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
/// herdr's resume argv limits.
const MAX_RESUME_ARGS: usize = 64;
const MAX_RESUME_BYTES: usize = 8 * 1024;
/// Longest `--message` sent with a `blocked` report.
const MAX_MESSAGE_BYTES: usize = 200;
/// Most herdr error output kept for the log.
const MAX_STDERR_BYTES: usize = 512;
/// Wait after the first failed report; doubles per failure up to the cap.
const RETRY_BASE: Duration = Duration::from_secs(1);
const RETRY_CAP: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Working,
    Blocked,
}

impl AgentState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Blocked => "blocked",
        }
    }
}

/// What the pane is doing, gathered by the TUI for [`derive_state`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StateInputs<'a> {
    /// A turn is being processed.
    pub processing: bool,
    /// Reason of the first incomplete todo that needs the user to log in.
    pub user_auth_reason: Option<&'a str>,
    /// A swarm worker this session spawned (directly or transitively) runs.
    pub owned_worker_running: bool,
    /// A background task of this session that wakes it on completion runs.
    pub wake_task_running: bool,
}

/// `blocked` while the user must act, else `working` while this session or
/// work it waits on runs, else `idle`. Ordinary dependency waits and blocked
/// workers do not block the pane: the user cannot unblock them.
pub fn derive_state(inputs: StateInputs<'_>) -> (AgentState, Option<String>) {
    if let Some(reason) = inputs.user_auth_reason.and_then(sanitize_message) {
        return (AgentState::Blocked, Some(reason));
    }
    if inputs.processing || inputs.owned_worker_running || inputs.wake_task_running {
        return (AgentState::Working, None);
    }
    (AgentState::Idle, None)
}

/// One line without control characters, at most [`MAX_MESSAGE_BYTES`].
fn sanitize_message(text: &str) -> Option<String> {
    let line = text
        .split(char::is_control)
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ");
    if line.is_empty() {
        return None;
    }
    let mut end = line.len().min(MAX_MESSAGE_BYTES);
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    Some(line[..end].trim_end().to_string())
}

/// A state report plus, when known, the session that `resume_argv` reopens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateReport {
    pub state: AgentState,
    pub message: Option<String>,
    pub session_id: Option<String>,
    pub resume_argv: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy)]
struct RetryPolicy {
    base: Duration,
    cap: Duration,
}

const RETRY: RetryPolicy = RetryPolicy {
    base: RETRY_BASE,
    cap: RETRY_CAP,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct HerdrEnv {
    bin: String,
    pane_id: String,
}

impl HerdrEnv {
    fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Option<Self> {
        let non_empty = |key: &str| get(key).filter(|value| !value.trim().is_empty());
        if non_empty("HERDR_ENV").as_deref() != Some("1") {
            return None;
        }
        // The socket must exist for the CLI to reach herdr; require it as the
        // docs do, even though we talk to herdr through its binary.
        non_empty("HERDR_SOCKET_PATH")?;
        Some(Self {
            bin: non_empty("HERDR_BIN_PATH")?,
            pane_id: non_empty("HERDR_PANE_ID")?,
        })
    }
}

enum Job {
    Report(StateReport),
}

#[derive(Default)]
struct Slot {
    pending: Option<Job>,
    released: bool,
}

struct Reporter {
    env: HerdrEnv,
    slot: Arc<(Mutex<Slot>, Condvar)>,
    /// Latest report requested by [`sync`]. The worker keeps retrying it
    /// until herdr accepts it, so an equal request needs no new job.
    desired: Mutex<Option<StateReport>>,
}

static REPORTER: OnceLock<Option<Reporter>> = OnceLock::new();
static LAST_SEQ: Mutex<u64> = Mutex::new(0);
/// Set once herdr rejects a report carrying a resume command. herdr older
/// than 0.9.2 fails the whole report with a usage error on the `--` resume
/// separator, so after that we keep reporting state without it.
static RESUME_UNSUPPORTED: AtomicBool = AtomicBool::new(false);

fn reporter() -> Option<&'static Reporter> {
    REPORTER
        .get_or_init(|| {
            let env = HerdrEnv::from_lookup(|key| std::env::var(key).ok())?;
            crate::logging::info(&format!(
                "herdr: reporting agent state for pane {}",
                env.pane_id
            ));
            let slot = Arc::new((Mutex::new(Slot::default()), Condvar::new()));
            let worker_slot = Arc::clone(&slot);
            let worker_env = env.clone();
            let spawned = std::thread::Builder::new()
                .name("herdr-reporter".into())
                .spawn(move || worker_loop(worker_env, worker_slot, RETRY));
            if let Err(error) = spawned {
                crate::logging::warn(&format!("herdr: failed to start reporter: {error}"));
                return None;
            }
            Some(Reporter {
                env,
                slot,
                desired: Mutex::new(None),
            })
        })
        .as_ref()
}

/// True when this process runs inside a herdr pane and reports to it.
pub fn active() -> bool {
    reporter().is_some()
}

/// Report the current state and session. Cheap enough for every UI tick:
/// outside herdr it returns immediately, and an unchanged request is dropped
/// without allocating (the worker already retries it until it is accepted).
/// A newer report replaces one that has not been delivered.
pub fn sync(state: AgentState, message: Option<&str>, session_id: Option<&str>) {
    let Some(reporter) = reporter() else {
        return;
    };
    request(
        &reporter.desired,
        &reporter.slot,
        state,
        message,
        session_id,
    );
}

/// Record a desired report and hand it to the worker unless it equals the
/// last one requested.
fn request(
    desired: &Mutex<Option<StateReport>>,
    slot: &(Mutex<Slot>, Condvar),
    state: AgentState,
    message: Option<&str>,
    session_id: Option<&str>,
) {
    let report = {
        let mut last = lock(desired);
        if last.as_ref().is_some_and(|last| {
            last.state == state
                && last.message.as_deref() == message
                && last.session_id.as_deref() == session_id
        }) {
            return;
        }
        let report = StateReport {
            state,
            message: message.map(str::to_owned),
            session_id: session_id.map(str::to_owned),
            resume_argv: session_id
                .and_then(|id| resume_argv_for_session(id, |key| std::env::var(key).ok())),
        };
        *last = Some(report.clone());
        report
    };
    let (slot, wake) = slot;
    let mut slot = lock(slot);
    if slot.released {
        return;
    }
    slot.pending = Some(Job::Report(report));
    wake.notify_one();
}

/// Release the pane because the user is quitting jcode. Blocks for at most
/// [`COMMAND_TIMEOUT`]. Do not call this when jcode is about to exec itself
/// (reload/update) since the same agent keeps running in the pane.
pub fn release() {
    let Some(reporter) = reporter() else {
        return;
    };
    let seq = {
        let mut slot = lock(&reporter.slot.0);
        if slot.released {
            return;
        }
        slot.released = true;
        slot.pending = None;
        reporter.slot.1.notify_one();
        next_seq()
    };
    run_herdr(&reporter.env.bin, &release_args(&reporter.env.pane_id, seq));
}

fn worker_loop(env: HerdrEnv, slot: Arc<(Mutex<Slot>, Condvar)>, retry: RetryPolicy) {
    let (lock_slot, wake) = &*slot;
    let mut attempted_id: Option<String> = None;
    // Undelivered report and when to try it again.
    let mut current: Option<(StateReport, Instant)> = None;
    let mut backoff = retry.base;
    loop {
        let report = {
            let mut guard = lock(lock_slot);
            loop {
                if guard.released {
                    return;
                }
                if let Some(Job::Report(report)) = guard.pending.take() {
                    backoff = retry.base;
                    break report;
                }
                match &current {
                    None => guard = wake.wait(guard).unwrap_or_else(|e| e.into_inner()),
                    Some((report, due)) => {
                        let now = Instant::now();
                        if now >= *due {
                            break report.clone();
                        }
                        guard = wake
                            .wait_timeout(guard, *due - now)
                            .unwrap_or_else(|e| e.into_inner())
                            .0;
                    }
                }
            }
        };
        if deliver(&env, &slot, &mut attempted_id, &report) {
            current = None;
            backoff = retry.base;
        } else {
            current = Some((report, Instant::now() + backoff));
            backoff = backoff.saturating_mul(2).min(retry.cap);
        }
    }
}

/// Send one desired report, clearing a previous session identity first.
/// Returns true once herdr accepted every step.
fn deliver(
    env: &HerdrEnv,
    slot: &Arc<(Mutex<Slot>, Condvar)>,
    attempted_id: &mut Option<String>,
    report: &StateReport,
) -> bool {
    if attempted_id.is_some() && *attempted_id != report.session_id {
        let clear = StateReport {
            session_id: None,
            resume_argv: None,
            ..report.clone()
        };
        if !send_report(env, slot, &clear) {
            return false;
        }
        *attempted_id = None;
        if report.session_id.is_none() {
            return true;
        }
    }
    if lock(&slot.0).released {
        return false;
    }
    if report.session_id.is_some() {
        attempted_id.clone_from(&report.session_id);
    }
    let delivered = send_report(env, slot, report);
    if delivered && report.session_id.is_none() {
        *attempted_id = None;
    }
    delivered
}

fn send_report(env: &HerdrEnv, slot: &Arc<(Mutex<Slot>, Condvar)>, report: &StateReport) -> bool {
    let mut report = report.clone();
    if RESUME_UNSUPPORTED.load(Ordering::Relaxed) {
        report.resume_argv = None;
    }
    let sequence = || {
        let guard = lock(&slot.0);
        if guard.released {
            None
        } else {
            Some(next_seq())
        }
    };
    let Some(seq) = sequence() else {
        return false;
    };
    let status = run_herdr(&env.bin, &report_args(&env.pane_id, &report, seq));
    // Only a usage error on a report that carried a resume command means
    // "this herdr predates resume". The worker retries timeouts and other
    // failures, and they must not disable resume.
    if status.is_some_and(|status| status.success()) {
        return true;
    }
    if !(is_usage_error(status) && resume_command(&report).is_some()) {
        return false;
    }
    report.resume_argv = None;
    let Some(seq) = sequence() else {
        return false;
    };
    let retry = run_herdr(&env.bin, &report_args(&env.pane_id, &report, seq));
    if retry.is_some_and(|status| status.success()) {
        RESUME_UNSUPPORTED.store(true, Ordering::Relaxed);
        crate::logging::info(
            "herdr: this herdr rejects resume commands (needs 0.9.2+), reporting state only",
        );
        return true;
    }
    false
}

/// Exit code clap-based CLIs such as herdr use for argument/usage errors.
const USAGE_ERROR_EXIT_CODE: i32 = 2;

fn is_usage_error(status: Option<ExitStatus>) -> bool {
    status.and_then(|status| status.code()) == Some(USAGE_ERROR_EXIT_CODE)
}

/// The resume command [`report_args`] sends after `--`, if any. herdr only
/// accepts one alongside a session id, and only when it passes its validation.
fn resume_command(report: &StateReport) -> Option<&Vec<String>> {
    report.session_id.as_ref()?;
    report
        .resume_argv
        .as_ref()
        .filter(|argv| valid_resume_argv(argv))
}

/// Strictly increasing across reports, sessions, and exec reloads: wall-clock
/// microseconds, bumped past the previous value if the clock stalls.
fn next_seq() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0);
    let mut last = lock(&LAST_SEQ);
    *last = now.max(last.saturating_add(1));
    *last
}

fn report_args(pane_id: &str, report: &StateReport, seq: u64) -> Vec<String> {
    let mut args: Vec<String> = [
        "pane",
        "report-agent",
        pane_id,
        "--source",
        SOURCE,
        "--agent",
        AGENT,
        "--state",
        report.state.as_str(),
        "--seq",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    args.push(seq.to_string());
    if let Some(message) = report.message.as_deref() {
        // One `--message=` word, so a reason such as `--` stays a value.
        args.push(format!("--message={message}"));
    }
    if let Some(session_id) = report.session_id.as_deref() {
        args.push("--agent-session-id".into());
        args.push(session_id.to_string());
        if let Some(argv) = resume_command(report) {
            args.push("--".into());
            args.extend(argv.iter().cloned());
        }
    }
    args
}

fn release_args(pane_id: &str, seq: u64) -> Vec<String> {
    vec![
        "pane".into(),
        "release-agent".into(),
        pane_id.into(),
        "--source".into(),
        SOURCE.into(),
        "--agent".into(),
        AGENT.into(),
        "--seq".into(),
        seq.to_string(),
    ]
}

/// herdr rejects resume commands whose first word is a path, that contain an
/// apostrophe or control character, or that exceed 64 args / 8 KiB.
pub fn valid_resume_argv(argv: &[String]) -> bool {
    let Some(program) = argv.first() else {
        return false;
    };
    if program.is_empty() || program.contains('/') || program.contains('\\') {
        return false;
    }
    if argv.len() > MAX_RESUME_ARGS {
        return false;
    }
    let total: usize = argv.iter().map(|arg| arg.len()).sum::<usize>() + argv.len();
    if total > MAX_RESUME_BYTES {
        return false;
    }
    argv.iter()
        .all(|arg| !arg.contains('\'') && !arg.chars().any(char::is_control))
}

/// Command that reopens `session_id` from the pane's working directory,
/// keeping SSH attach targets and self-dev mode so the resumed client
/// behaves like this one. The model and provider live with the session.
fn resume_argv_for_session(
    session_id: &str,
    get: impl Fn(&str) -> Option<String>,
) -> Option<Vec<String>> {
    let mut argv = vec!["jcode".to_string()];
    if let Some(host) = get("JCODE_SSH_REMOTE").filter(|h| !h.trim().is_empty()) {
        argv.extend(["--ssh".to_string(), host]);
        for (flag, variable) in [
            ("--ssh-binary", "JCODE_SSH_BINARY"),
            ("--ssh-server-socket", "JCODE_SSH_SERVER_SOCKET"),
            ("--remote-working-dir", "JCODE_SSH_WORKING_DIR"),
        ] {
            if let Some(value) = get(variable) {
                argv.extend([flag.to_string(), value]);
            }
        }
    }
    argv.extend(["--resume".to_string(), session_id.to_string()]);
    if get(jcode_selfdev_types::CLIENT_SELFDEV_ENV).is_some() {
        argv.push("self-dev".to_string());
    }
    valid_resume_argv(&argv).then_some(argv)
}

/// Result of one herdr CLI call.
struct HerdrRun {
    /// Exit status, `None` when it could not start or timed out.
    status: Option<ExitStatus>,
    timed_out: bool,
    /// First [`MAX_STDERR_BYTES`] of its error output.
    stderr: Vec<u8>,
}

/// Run one herdr CLI call with a timeout and log a failure. Returns its exit
/// status, or `None` when it could not be started or timed out.
fn run_herdr(bin: &str, args: &[String]) -> Option<ExitStatus> {
    let run = run_herdr_captured(bin, args, COMMAND_TIMEOUT);
    if !run.status.is_some_and(|status| status.success()) {
        crate::logging::warn(&failure_log_line(args, &run));
    }
    run.status
}

fn run_herdr_captured(bin: &str, args: &[String], timeout: Duration) -> HerdrRun {
    let child = Command::new(bin)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .env("JCODE_HOOKS_DISABLED", "1")
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(error) => {
            return HerdrRun {
                status: None,
                timed_out: false,
                stderr: format!("failed to run {bin}: {error}").into_bytes(),
            };
        }
    };
    // Drain stderr on its own thread so a chatty herdr can never fill the
    // pipe and stall; keep only the head. A full drain (not an early close)
    // keeps herdr's real exit code, which the resume fallback depends on.
    let stderr = child.stderr.take().map(|mut pipe| {
        let (tx, rx) = std::sync::mpsc::channel();
        let spawned = std::thread::Builder::new()
            .name("herdr-stderr".into())
            .spawn(move || {
                let mut kept = Vec::new();
                let mut buf = [0u8; 1024];
                while let Ok(n) = pipe.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let room = MAX_STDERR_BYTES - kept.len();
                    kept.extend_from_slice(&buf[..n.min(room)]);
                }
                let _ = tx.send(kept);
            });
        spawned.ok().map(|_| rx)
    });
    let deadline = Instant::now() + timeout;
    let (status, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (Some(status), false),
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break (None, true);
            }
        }
    };
    let stderr = stderr
        .flatten()
        .and_then(|rx| rx.recv_timeout(Duration::from_millis(200)).ok())
        .unwrap_or_default();
    HerdrRun {
        status,
        timed_out,
        stderr,
    }
}

/// Names the command, state and sequence but no argument values, which can
/// carry environment-derived resume details such as SSH targets.
fn failure_log_line(args: &[String], run: &HerdrRun) -> String {
    let after = |flag: &str| {
        args.iter()
            .position(|arg| arg == flag)
            .and_then(|index| args.get(index + 1))
            .map(String::as_str)
            .unwrap_or("-")
    };
    let outcome = match (run.status, run.timed_out) {
        (Some(status), _) => status.to_string(),
        (None, true) => format!("timed out after {}s", COMMAND_TIMEOUT.as_secs()),
        (None, false) => "not started".to_string(),
    };
    format!(
        "herdr: {} for pane {} state={} seq={} failed: {}; stderr: {}",
        args.get(1).map(String::as_str).unwrap_or("-"),
        args.get(2).map(String::as_str).unwrap_or("-"),
        after("--state"),
        after("--seq"),
        outcome,
        String::from_utf8_lossy(&run.stderr).trim()
    )
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> Option<HerdrEnv> {
        let map: std::collections::HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        HerdrEnv::from_lookup(|key| map.get(key).cloned())
    }

    #[test]
    fn activates_only_inside_herdr_with_all_vars() {
        let full = [
            ("HERDR_ENV", "1"),
            ("HERDR_PANE_ID", "p1"),
            ("HERDR_BIN_PATH", "/usr/bin/herdr"),
            ("HERDR_SOCKET_PATH", "/tmp/herdr.sock"),
        ];
        assert_eq!(
            env(&full),
            Some(HerdrEnv {
                bin: "/usr/bin/herdr".into(),
                pane_id: "p1".into()
            })
        );
        assert_eq!(env(&[]), None);
        let mut not_one = full;
        not_one[0] = ("HERDR_ENV", "0");
        assert_eq!(env(&not_one), None);
        for skip in 1..full.len() {
            let partial: Vec<_> = full
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, kv)| *kv)
                .collect();
            assert_eq!(env(&partial), None, "missing {}", full[skip].0);
        }
    }

    #[test]
    fn report_args_include_resume_command_after_separator() {
        let report = StateReport {
            state: AgentState::Idle,
            message: None,
            session_id: Some("session_fox_123".into()),
            resume_argv: resume_argv_for_session("session_fox_123", |_| None),
        };
        assert_eq!(
            report_args("p1", &report, 42),
            [
                "pane",
                "report-agent",
                "p1",
                "--source",
                "jcode",
                "--agent",
                "jcode",
                "--state",
                "idle",
                "--seq",
                "42",
                "--agent-session-id",
                "session_fox_123",
                "--",
                "jcode",
                "--resume",
                "session_fox_123"
            ]
        );
    }

    #[test]
    fn report_args_without_session_omit_resume() {
        let report = StateReport {
            state: AgentState::Working,
            message: None,
            session_id: None,
            resume_argv: None,
        };
        let args = report_args("p1", &report, 7);
        assert_eq!(&args[7..], ["--state", "working", "--seq", "7"]);
    }

    #[test]
    fn release_args_match_herdr_cli() {
        assert_eq!(
            release_args("p1", 9),
            [
                "pane",
                "release-agent",
                "p1",
                "--source",
                "jcode",
                "--agent",
                "jcode",
                "--seq",
                "9"
            ]
        );
    }

    #[test]
    fn resume_argv_validation_follows_herdr_rules() {
        let ok =
            |v: &[&str]| valid_resume_argv(&v.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        assert!(ok(&["jcode", "--resume", "session_a"]));
        assert!(!ok(&[]));
        assert!(!ok(&["/usr/bin/jcode"]));
        assert!(!ok(&["jcode", "it's"]));
        assert!(!ok(&["jcode", "a\nb"]));
        assert!(!ok(&vec!["x"; 65]));
        assert!(!ok(&["jcode", &"x".repeat(9000)]));
        assert_eq!(resume_argv_for_session("bad'id", |_| None), None);
    }

    #[test]
    fn resume_argv_keeps_ssh_target_and_selfdev() {
        let vars: std::collections::HashMap<&str, &str> = [
            ("JCODE_SSH_REMOTE", "devbox"),
            ("JCODE_SSH_WORKING_DIR", "/srv/repo"),
            (jcode_selfdev_types::CLIENT_SELFDEV_ENV, "1"),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            resume_argv_for_session("session_a", |k| vars.get(k).map(|v| v.to_string())).unwrap(),
            [
                "jcode",
                "--ssh",
                "devbox",
                "--remote-working-dir",
                "/srv/repo",
                "--resume",
                "session_a",
                "self-dev"
            ]
        );
    }

    #[test]
    fn seq_is_strictly_increasing() {
        let a = next_seq();
        let b = next_seq();
        let c = next_seq();
        assert!(a < b && b < c);
    }

    /// Fake herdr for `send_report` tests. Each call is logged; any call
    /// carrying the `--` resume separator exits with `resume_exit_code`.
    /// Holds the test lock and resets [`RESUME_UNSUPPORTED`] on drop, so a
    /// failing assertion cannot leak the global flag into other tests.
    #[cfg(unix)]
    struct FakeHerdr {
        dir: std::path::PathBuf,
        env: HerdrEnv,
        _serial: std::sync::MutexGuard<'static, ()>,
    }

    #[cfg(unix)]
    impl FakeHerdr {
        fn new(resume_exit_code: i32) -> Self {
            Self::with_script(&format!(
                "case \" $* \" in *\" --agent-session-id \"*) ;; *) marker=\"$(dirname \"$0\")/clear-failed\"; if [ ! -e \"$marker\" ]; then touch \"$marker\"; exit 1; fi ;; esac\nfor a in \"$@\"; do [ \"$a\" = -- ] && exit {resume_exit_code}; done\nexit 0\n"
            ))
        }

        /// Fake whose body runs after the call is logged. `$n` is the
        /// 1-based number of this call.
        fn with_script(body: &str) -> Self {
            use std::os::unix::fs::PermissionsExt;
            static SERIAL: Mutex<()> = Mutex::new(());
            let serial = lock(&SERIAL);
            RESUME_UNSUPPORTED.store(false, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(format!(
                "jcode-herdr-fake-{}-{}",
                std::process::id(),
                next_seq()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let bin = dir.join("herdr");
            // The script logs next to itself via `$0` rather than embedding
            // the temp path, so any `TMPDIR` (spaces, quotes) works.
            std::fs::write(
                &bin,
                format!(
                    "#!/bin/sh\nlog=\"$(dirname \"$0\")/calls.log\"\necho \"$*\" >> \"$log\"\nn=$(wc -l < \"$log\")\n{body}"
                ),
            )
            .unwrap();
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
            let env = HerdrEnv {
                bin: bin.display().to_string(),
                pane_id: "p1".into(),
            };
            Self {
                dir,
                env,
                _serial: serial,
            }
        }

        fn calls(&self) -> Vec<String> {
            std::fs::read_to_string(self.dir.join("calls.log"))
                .unwrap_or_default()
                .lines()
                .map(str::to_owned)
                .collect()
        }
    }

    #[cfg(unix)]
    impl Drop for FakeHerdr {
        fn drop(&mut self) {
            RESUME_UNSUPPORTED.store(false, Ordering::Relaxed);
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[cfg(unix)]
    fn idle_report_with_resume() -> StateReport {
        StateReport {
            state: AgentState::Idle,
            message: None,
            session_id: Some("session_fox_1".into()),
            resume_argv: resume_argv_for_session("session_fox_1", |_| None),
        }
    }

    #[cfg(unix)]
    #[test]
    fn report_falls_back_to_state_only_when_herdr_rejects_resume() {
        // herdr < 0.9.2 treats the `--` resume separator as a usage error.
        let herdr = FakeHerdr::new(USAGE_ERROR_EXIT_CODE);
        let slot = Arc::new((Mutex::new(Slot::default()), Condvar::new()));
        send_report(&herdr.env, &slot, &idle_report_with_resume());
        assert!(RESUME_UNSUPPORTED.load(Ordering::Relaxed));
        send_report(&herdr.env, &slot, &idle_report_with_resume());

        let calls = herdr.calls();
        assert_eq!(calls.len(), 3, "{calls:?}");
        assert!(calls[0].contains(" -- jcode --resume session_fox_1"));
        assert!(!calls[1].contains(" -- "));
        assert!(calls[1].contains("--agent-session-id session_fox_1"));
        // Remembered: later reports skip the doomed first attempt.
        assert!(!calls[2].contains(" -- "));
    }

    #[cfg(unix)]
    #[test]
    fn report_keeps_resume_after_non_usage_failure() {
        // A transient failure (not a usage error) must not disable resume.
        let herdr = FakeHerdr::new(1);
        let slot = Arc::new((Mutex::new(Slot::default()), Condvar::new()));
        send_report(&herdr.env, &slot, &idle_report_with_resume());
        assert!(!RESUME_UNSUPPORTED.load(Ordering::Relaxed));

        let calls = herdr.calls();
        assert_eq!(calls.len(), 1, "no state-only retry expected: {calls:?}");
        assert!(calls[0].contains(" -- jcode --resume session_fox_1"));
    }

    #[cfg(unix)]
    const FAST_RETRY: RetryPolicy = RetryPolicy {
        base: Duration::from_millis(20),
        cap: Duration::from_millis(80),
    };

    /// Worker thread over a fake herdr, driven like [`sync`] drives it.
    #[cfg(unix)]
    struct Harness {
        slot: Arc<(Mutex<Slot>, Condvar)>,
        desired: Mutex<Option<StateReport>>,
        worker: Option<std::thread::JoinHandle<()>>,
    }

    #[cfg(unix)]
    impl Harness {
        fn start(herdr: &FakeHerdr, retry: RetryPolicy) -> Self {
            let slot = Arc::new((Mutex::new(Slot::default()), Condvar::new()));
            let worker_slot = Arc::clone(&slot);
            let env = herdr.env.clone();
            let worker = std::thread::spawn(move || worker_loop(env, worker_slot, retry));
            Self {
                slot,
                desired: Mutex::new(None),
                worker: Some(worker),
            }
        }

        /// Same path as [`sync`], including its duplicate filter.
        fn sync(&self, state: AgentState, session_id: Option<&str>) {
            request(&self.desired, &self.slot, state, None, session_id);
        }

        fn submit(&self, state: AgentState, session_id: Option<&str>) {
            lock(&self.slot.0).pending = Some(Job::Report(StateReport {
                state,
                message: None,
                session_id: session_id.map(str::to_owned),
                resume_argv: None,
            }));
            self.slot.1.notify_one();
        }

        fn release(mut self) {
            lock(&self.slot.0).released = true;
            self.slot.1.notify_one();
            self.worker.take().unwrap().join().unwrap();
        }
    }

    #[cfg(unix)]
    fn wait_for_calls(herdr: &FakeHerdr, n: usize) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(3);
        while herdr.calls().len() < n && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        herdr.calls()
    }

    #[cfg(unix)]
    #[test]
    fn identical_sync_during_backoff_adds_no_attempt() {
        // Every call fails, so the worker sits in backoff. Repeating the same
        // request must not reset the backoff or start extra attempts.
        let herdr = FakeHerdr::with_script("exit 1\n");
        let slow = RetryPolicy {
            base: Duration::from_millis(400),
            cap: Duration::from_secs(5),
        };
        let worker = Harness::start(&herdr, slow);
        worker.sync(AgentState::Idle, None);
        assert_eq!(wait_for_calls(&herdr, 1).len(), 1);
        for _ in 0..20 {
            worker.sync(AgentState::Idle, None);
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(herdr.calls().len(), 1, "{:?}", herdr.calls());
        worker.release();
    }

    #[cfg(unix)]
    #[test]
    fn worker_clears_old_identity_before_reporting_replacement() {
        // The first identity clear fails; the worker must retry the clear
        // and never report session_b while session_a may still be held.
        let herdr = FakeHerdr::new(0);
        let worker = Harness::start(&herdr, FAST_RETRY);
        worker.submit(AgentState::Working, Some("session_a"));
        assert_eq!(wait_for_calls(&herdr, 1).len(), 1);
        worker.submit(AgentState::Working, Some("session_b"));
        let calls = wait_for_calls(&herdr, 4);
        assert_eq!(calls.len(), 4, "{calls:?}");
        assert!(!calls[1].contains("--agent-session-id"), "{calls:?}");
        assert!(!calls[2].contains("--agent-session-id"), "{calls:?}");
        assert!(calls[3].contains("--agent-session-id session_b"));
        worker.release();
    }

    #[cfg(unix)]
    #[test]
    fn worker_retries_failed_report_until_accepted_then_stops() {
        let herdr = FakeHerdr::with_script("[ \"$n\" -le 3 ] && exit 1\nexit 0\n");
        let worker = Harness::start(&herdr, FAST_RETRY);
        worker.submit(AgentState::Idle, None);
        let calls = wait_for_calls(&herdr, 4);
        assert_eq!(calls.len(), 4, "{calls:?}");
        assert!(calls.iter().all(|call| call.contains("--state idle")));
        // Each attempt carries a fresh, increasing sequence number.
        let seqs: Vec<u64> = calls
            .iter()
            .map(|call| {
                let words: Vec<&str> = call.split(' ').collect();
                let at = words.iter().position(|w| *w == "--seq").unwrap();
                words[at + 1].parse().unwrap()
            })
            .collect();
        assert!(seqs.windows(2).all(|pair| pair[0] < pair[1]), "{seqs:?}");
        std::thread::sleep(FAST_RETRY.cap * 3);
        assert_eq!(herdr.calls().len(), 4, "no reports after success");
        worker.release();
    }

    #[cfg(unix)]
    #[test]
    fn worker_backoff_doubles_up_to_cap() {
        let herdr = FakeHerdr::with_script("exit 1\n");
        let retry = RetryPolicy {
            base: Duration::from_millis(40),
            cap: Duration::from_millis(80),
        };
        let worker = Harness::start(&herdr, retry);
        worker.submit(AgentState::Idle, None);
        // Attempts at about 0, 40, 120, 200 ms: well under one per 40 ms.
        std::thread::sleep(Duration::from_millis(230));
        let calls = herdr.calls().len();
        assert!((3..=5).contains(&calls), "{calls} attempts");
        worker.release();
    }

    #[cfg(unix)]
    #[test]
    fn newer_state_replaces_pending_retry() {
        // Only `working` reports fail, so the retry is pending when the
        // state changes to `idle`.
        let herdr = FakeHerdr::with_script(
            "case \" $* \" in *\" --state working \"*) exit 1 ;; esac\nexit 0\n",
        );
        let worker = Harness::start(
            &herdr,
            RetryPolicy {
                base: Duration::from_millis(200),
                cap: Duration::from_millis(200),
            },
        );
        worker.submit(AgentState::Working, None);
        assert_eq!(wait_for_calls(&herdr, 1).len(), 1);
        worker.submit(AgentState::Idle, None);
        let calls = wait_for_calls(&herdr, 2);
        assert!(calls[1].contains("--state idle"), "{calls:?}");
        std::thread::sleep(Duration::from_millis(500));
        let calls = herdr.calls();
        assert_eq!(
            calls.len(),
            2,
            "stale working must not be retried: {calls:?}"
        );
        worker.release();
    }

    #[cfg(unix)]
    #[test]
    fn release_cancels_pending_retry() {
        let herdr = FakeHerdr::with_script("exit 1\n");
        let worker = Harness::start(
            &herdr,
            RetryPolicy {
                base: Duration::from_millis(100),
                cap: Duration::from_millis(100),
            },
        );
        worker.submit(AgentState::Working, None);
        assert_eq!(wait_for_calls(&herdr, 1).len(), 1);
        worker.release();
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(herdr.calls().len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn timeout_before_usage_error_still_falls_back_to_state_only() {
        // First call hangs past the timeout; the retry then hits the old-herdr
        // usage error on `--` and must fall back to a state-only report.
        let herdr = FakeHerdr::with_script(
            "[ \"$n\" -eq 1 ] && exec sleep 5\nfor a in \"$@\"; do [ \"$a\" = -- ] && exit 2; done\nexit 0\n",
        );
        let worker = Harness::start(&herdr, FAST_RETRY);
        lock(&worker.slot.0).pending = Some(Job::Report(idle_report_with_resume()));
        worker.slot.1.notify_one();
        let calls = wait_for_calls(&herdr, 3);
        assert_eq!(calls.len(), 3, "{calls:?}");
        assert!(calls[1].contains(" -- jcode --resume session_fox_1"));
        assert!(!calls[2].contains(" -- "));
        assert!(calls[2].contains("--agent-session-id session_fox_1"));
        // The call is logged before herdr exits; the flag follows its exit.
        let deadline = Instant::now() + Duration::from_secs(3);
        while !RESUME_UNSUPPORTED.load(Ordering::Relaxed) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(RESUME_UNSUPPORTED.load(Ordering::Relaxed));
        worker.release();
    }

    #[cfg(unix)]
    #[test]
    fn failure_log_names_state_seq_status_and_stderr() {
        let herdr =
            FakeHerdr::with_script("echo '{\"error\":\"resume_not_accepted\"}' >&2\nexit 1\n");
        let report = StateReport {
            state: AgentState::Blocked,
            message: Some("Log in".into()),
            session_id: None,
            resume_argv: None,
        };
        let args = report_args("p1", &report, 77);
        let run = run_herdr_captured(&herdr.env.bin, &args, COMMAND_TIMEOUT);
        let line = failure_log_line(&args, &run);
        assert!(line.contains("report-agent for pane p1"), "{line}");
        assert!(line.contains("state=blocked seq=77"), "{line}");
        assert!(line.contains("exit status: 1"), "{line}");
        assert!(line.contains("resume_not_accepted"), "{line}");
    }

    #[cfg(unix)]
    #[test]
    fn stderr_capture_is_bounded_and_timeout_still_applies() {
        let herdr = FakeHerdr::with_script("head -c 4096 /dev/zero | tr '\\0' x >&2\nexit 3\n");
        let run = run_herdr_captured(&herdr.env.bin, &[], COMMAND_TIMEOUT);
        assert_eq!(run.status.and_then(|s| s.code()), Some(3));
        assert_eq!(run.stderr.len(), MAX_STDERR_BYTES);
        // FakeHerdr holds a non-reentrant test lock: release it first.
        drop(herdr);

        let slow = FakeHerdr::with_script("exec sleep 5\n");
        let started = Instant::now();
        let run = run_herdr_captured(&slow.env.bin, &[], Duration::from_millis(100));
        assert!(run.timed_out && run.status.is_none());
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(failure_log_line(&[], &run).contains("timed out"));
    }

    #[test]
    fn report_args_put_message_before_resume_separator() {
        let report = StateReport {
            state: AgentState::Blocked,
            message: Some("Type the keyring password".into()),
            session_id: Some("session_fox_123".into()),
            resume_argv: resume_argv_for_session("session_fox_123", |_| None),
        };
        let args = report_args("p1", &report, 5);
        assert_eq!(
            &args[7..],
            [
                "--state",
                "blocked",
                "--seq",
                "5",
                "--message=Type the keyring password",
                "--agent-session-id",
                "session_fox_123",
                "--",
                "jcode",
                "--resume",
                "session_fox_123"
            ]
        );
    }

    #[test]
    fn message_is_one_line_and_truncated_at_char_boundary() {
        assert_eq!(
            sanitize_message("  a\n\tb\u{7}c  ").as_deref(),
            Some("a b c")
        );
        assert_eq!(sanitize_message(" \n "), None);
        let long = "é".repeat(150); // 300 bytes
        let cut = sanitize_message(&long).unwrap();
        assert!(cut.len() <= MAX_MESSAGE_BYTES);
        assert_eq!(cut.len(), MAX_MESSAGE_BYTES);
        assert!(cut.chars().all(|c| c == 'é'));
        let odd = format!("a{}", "é".repeat(150));
        assert_eq!(sanitize_message(&odd).unwrap().len(), MAX_MESSAGE_BYTES - 1);
    }

    #[test]
    fn derive_state_precedence() {
        let base = StateInputs::default();
        let reason = Some("Unlock Login Keyring");
        let cases = [
            (base, AgentState::Idle),
            (
                StateInputs {
                    processing: true,
                    ..base
                },
                AgentState::Working,
            ),
            (
                StateInputs {
                    owned_worker_running: true,
                    ..base
                },
                AgentState::Working,
            ),
            (
                StateInputs {
                    wake_task_running: true,
                    ..base
                },
                AgentState::Working,
            ),
            (
                StateInputs {
                    user_auth_reason: reason,
                    ..base
                },
                AgentState::Blocked,
            ),
            (
                StateInputs {
                    user_auth_reason: reason,
                    processing: true,
                    owned_worker_running: true,
                    wake_task_running: true,
                },
                AgentState::Blocked,
            ),
            // A blank reason is not a user blocker.
            (
                StateInputs {
                    user_auth_reason: Some(" \n"),
                    ..base
                },
                AgentState::Idle,
            ),
        ];
        for (inputs, expected) in cases {
            assert_eq!(derive_state(inputs).0, expected, "{inputs:?}");
        }
        assert_eq!(
            derive_state(StateInputs {
                user_auth_reason: reason,
                ..base
            })
            .1
            .as_deref(),
            reason
        );
        assert_eq!(
            derive_state(StateInputs {
                processing: true,
                ..base
            })
            .1,
            None
        );
    }
}
