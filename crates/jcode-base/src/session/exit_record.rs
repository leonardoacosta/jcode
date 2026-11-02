//! Per-PID server exit records: `~/.jcode/exit_records/<pid>.json`.
//!
//! The server writes one whenever it can run code at exit (clean shutdown,
//! caught signal, panic). Crash detection reads it to name the cause instead
//! of a generic "exited unexpectedly".

use crate::storage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const STALE_SECS: i64 = 30 * 24 * 3600;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "cause", rename_all = "snake_case")]
pub enum ExitCause {
    Clean,
    Signal {
        name: String,
    },
    Panic {
        message: String,
        location: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExitRecord {
    pub pid: u32,
    #[serde(flatten)]
    pub cause: ExitCause,
    pub timestamp: DateTime<Utc>,
}

fn dir() -> Option<PathBuf> {
    storage::jcode_dir().ok().map(|d| d.join("exit_records"))
}

fn write_in(dir: &Path, record: &ExitRecord) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_vec(record).map_err(std::io::Error::other)?;
    std::fs::write(dir.join(format!("{}.json", record.pid)), json)
}

fn read_in(dir: &Path, pid: u32) -> Option<ExitRecord> {
    let bytes = std::fs::read(dir.join(format!("{pid}.json"))).ok()?;
    serde_json::from_slice(&bytes)
        .ok()
        .filter(|r: &ExitRecord| r.pid == pid)
}

fn cleanup_in(dir: &Path, now: DateTime<Utc>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let stale = std::fs::read(entry.path())
            .ok()
            .and_then(|b| serde_json::from_slice::<ExitRecord>(&b).ok())
            .map(|r| (now - r.timestamp).num_seconds() > STALE_SECS)
            .unwrap_or(true);
        if stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Write an exit record for the current process. Best effort.
pub fn write_exit_record(cause: ExitCause) {
    if let Some(dir) = dir() {
        let record = ExitRecord {
            pid: std::process::id(),
            cause,
            timestamp: Utc::now(),
        };
        let _ = write_in(&dir, &record);
        cleanup_in(&dir, Utc::now());
    }
}

pub fn read_exit_record(pid: u32) -> Option<ExitRecord> {
    read_in(&dir()?, pid)
}

/// Install a panic hook that records the panic before chaining to the
/// previous hook.
pub fn install_panic_exit_record_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic>".to_string());
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()));
        write_exit_record(ExitCause::Panic { message, location });
        prev(info);
    }));
}

fn describe_record(record: &ExitRecord) -> String {
    match &record.cause {
        ExitCause::Clean => format!("Process {} shut down cleanly", record.pid),
        ExitCause::Signal { name } => format!("Process {} terminated by {name}", record.pid),
        ExitCause::Panic { message, location } => format!(
            "Process {} panicked: {message}{}",
            record.pid,
            location
                .as_deref()
                .map(|l| format!(" at {l}"))
                .unwrap_or_default()
        ),
    }
}

/// Map systemd `Result`/`ExecMainStatus` to a cause. `None` when systemd
/// reports success (no information).
fn describe_systemd(pid: u32, unit: &str, result: &str, status: &str) -> Option<String> {
    let cause = match result {
        "" | "success" => return None,
        "oom-kill" => "oom".to_string(),
        "signal" | "core-dump" => format!("{result} (status {status})"),
        other => format!("{other} (status {status})"),
    };
    Some(format!(
        "Process {pid} exited unexpectedly: {cause} (per {unit})"
    ))
}

fn query_systemd(pid: u32) -> Option<String> {
    let home = std::env::var_os("HOME")?;
    let unit =
        std::fs::read_to_string(Path::new(&home).join(".jcode/require-supervised-server")).ok()?;
    let unit = unit.trim();
    if unit.is_empty() {
        return None;
    }
    let out = std::process::Command::new("systemctl")
        .args([
            "--user",
            "show",
            unit,
            "-p",
            "Result",
            "-p",
            "ExecMainStatus",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let field = |k: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(&format!("{k}=")))
            .unwrap_or("")
            .to_string()
    };
    describe_systemd(pid, unit, &field("Result"), &field("ExecMainStatus"))
}

/// Classified reason for a dead PID: exit record, then systemd, else unknown.
/// Memoized per PID: one crash event can cover hundreds of sessions.
pub fn classify_exit(pid: u32) -> String {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<u32, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(hit) = cache.lock().ok().and_then(|c| c.get(&pid).cloned()) {
        return hit;
    }
    let reason = classify_with(read_exit_record(pid), || query_systemd(pid), pid);
    if let Ok(mut c) = cache.lock() {
        c.insert(pid, reason.clone());
    }
    reason
}

fn classify_with(
    record: Option<ExitRecord>,
    systemd: impl FnOnce() -> Option<String>,
    pid: u32,
) -> String {
    record
        .map(|r| describe_record(&r))
        .or_else(systemd)
        .unwrap_or_else(|| format!("Process {pid} exited unexpectedly: unknown cause"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(pid: u32, cause: ExitCause) -> ExitRecord {
        ExitRecord {
            pid,
            cause,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn write_read_roundtrip_and_stale_cleanup() {
        let tmp = tempfile::tempdir().unwrap();
        let r = rec(
            42,
            ExitCause::Signal {
                name: "SIGTERM".into(),
            },
        );
        write_in(tmp.path(), &r).unwrap();
        assert_eq!(read_in(tmp.path(), 42), Some(r));
        let mut old = rec(7, ExitCause::Clean);
        old.timestamp = Utc::now() - chrono::Duration::days(60);
        write_in(tmp.path(), &old).unwrap();
        std::fs::write(tmp.path().join("9.json"), b"garbage").unwrap();
        cleanup_in(tmp.path(), Utc::now());
        assert!(read_in(tmp.path(), 42).is_some());
        assert!(!tmp.path().join("7.json").exists());
        assert!(!tmp.path().join("9.json").exists());
    }

    #[test]
    fn classifies_signal_panic_oom_and_missing() {
        let sig = classify_with(
            Some(rec(
                1,
                ExitCause::Signal {
                    name: "SIGTERM".into(),
                },
            )),
            || None,
            1,
        );
        assert!(sig.contains("SIGTERM"), "{sig}");
        let panic = classify_with(
            Some(rec(
                1,
                ExitCause::Panic {
                    message: "boom".into(),
                    location: Some("a.rs:3".into()),
                },
            )),
            || None,
            1,
        );
        assert!(panic.contains("panicked: boom at a.rs:3"), "{panic}");
        let oom = classify_with(
            None,
            || describe_systemd(1, "jcode.service", "oom-kill", "9"),
            1,
        );
        assert!(oom.contains(": oom"), "{oom}");
        assert!(describe_systemd(1, "u", "success", "0").is_none());
        let unknown = classify_with(None, || None, 1);
        assert!(unknown.contains("unknown cause") && !unknown.contains("clean"));
    }

    #[test]
    fn corrupt_record_reads_as_missing() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("5.json"), b"{not json").unwrap();
        assert_eq!(read_in(tmp.path(), 5), None);
    }
}
