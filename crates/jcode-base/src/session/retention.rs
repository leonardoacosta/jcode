//! Opt-in retention for the on-disk session store (`[sessions.retention]`).
//!
//! `classify` is pure: keep rules (saved, live, unparseable) run first so no
//! later rule can override them. `run_gc` is dry-run unless `apply` is set,
//! and refuses to apply while `enabled = false`. Applied deletions move files
//! into `sessions/.trash/` (purged after [`TRASH_RETENTION_DAYS`]) and re-check
//! `saved` from disk immediately before each move.

use super::{Session, SessionStatus};
use crate::config::SessionRetentionConfig;
use anyhow::{Result, bail};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const TRASH_DIR: &str = ".trash";
pub const TRASH_RETENTION_DAYS: i64 = 7;
const SAMPLE_IDS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    Saved,
    Live,
    Unparseable,
    DebugExpired,
    TinyExpired,
    Recent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Keep(Reason),
    Candidate(Reason),
}

/// Minimal facts needed to classify one session file.
#[derive(Debug, Clone)]
pub struct SessionMeta {
    /// `None` when the file could not be parsed.
    pub parsed: Option<ParsedMeta>,
    pub modified: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ParsedMeta {
    pub saved: bool,
    pub is_debug: bool,
    pub message_count: usize,
    pub live: bool,
}

pub fn classify(
    meta: &SessionMeta,
    now: DateTime<Utc>,
    policy: &SessionRetentionConfig,
) -> Verdict {
    let Some(p) = &meta.parsed else {
        return Verdict::Keep(Reason::Unparseable);
    };
    if p.saved {
        return Verdict::Keep(Reason::Saved);
    }
    if p.live {
        return Verdict::Keep(Reason::Live);
    }
    let age = now - meta.modified;
    if p.is_debug && age > days(policy.debug_max_age_days) {
        return Verdict::Candidate(Reason::DebugExpired);
    }
    if p.message_count <= policy.tiny_max_messages && age > days(policy.tiny_max_age_days) {
        return Verdict::Candidate(Reason::TinyExpired);
    }
    Verdict::Keep(Reason::Recent)
}

fn days(n: u64) -> Duration {
    Duration::days(i64::try_from(n).unwrap_or(i64::MAX / 86_400_000))
}

#[derive(Deserialize)]
struct RawMeta {
    #[serde(default)]
    saved: bool,
    #[serde(default)]
    is_debug: bool,
    #[serde(default)]
    messages: Vec<serde::de::IgnoredAny>,
    #[serde(default)]
    status: SessionStatus,
    #[serde(default)]
    last_pid: Option<u32>,
}

fn read_meta(path: &Path, modified: DateTime<Utc>) -> SessionMeta {
    let parsed = std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<RawMeta>(&bytes).ok())
        .map(|raw| {
            // Journal lines append messages after the snapshot; count them so
            // a short snapshot with a long journal is never "tiny".
            let journal = super::session_journal_path_from_snapshot(path);
            let journal_lines = std::fs::read(&journal)
                .map(|b| b.iter().filter(|c| **c == b'\n').count())
                .unwrap_or(0);
            ParsedMeta {
                saved: raw.saved,
                is_debug: raw.is_debug,
                message_count: raw.messages.len() + journal_lines,
                live: matches!(raw.status, SessionStatus::Active)
                    && raw
                        .last_pid
                        .is_some_and(crate::platform::is_process_running),
            }
        });
    SessionMeta { parsed, modified }
}

#[derive(Debug, Default, Serialize)]
pub struct GcReport {
    pub applied: bool,
    pub scanned: usize,
    pub candidates: usize,
    pub candidate_bytes: u64,
    pub removed: usize,
    pub bytes_freed: u64,
    pub skipped_on_recheck: usize,
    pub by_reason: BTreeMap<Reason, usize>,
    pub sample_ids: Vec<String>,
    pub removed_ids: Vec<String>,
}

/// Scan `sessions_dir` and classify every `<id>.json`. Moves candidates to
/// trash only when `apply` is true; errors if `apply` with retention disabled.
pub fn run_gc_in(
    sessions_dir: &Path,
    now: DateTime<Utc>,
    policy: &SessionRetentionConfig,
    apply: bool,
) -> Result<GcReport> {
    if apply && !policy.enabled {
        bail!(
            "refusing to delete sessions: set `sessions.retention.enabled = true` in config first"
        );
    }
    let mut report = GcReport {
        applied: apply,
        ..Default::default()
    };
    let Ok(entries) = std::fs::read_dir(sessions_dir) else {
        return Ok(report);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let Ok(md) = entry.metadata() else { continue };
        if !md.is_file() {
            continue;
        }
        let modified: DateTime<Utc> = md.modified().map(Into::into).unwrap_or(now);
        report.scanned += 1;
        let verdict = classify(&read_meta(&path, modified), now, policy);
        let (Verdict::Keep(reason) | Verdict::Candidate(reason)) = verdict;
        *report.by_reason.entry(reason).or_default() += 1;
        if !matches!(verdict, Verdict::Candidate(_)) {
            continue;
        }
        let id = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let bytes = md.len();
        report.candidates += 1;
        report.candidate_bytes += bytes;
        if report.sample_ids.len() < SAMPLE_IDS {
            report.sample_ids.push(id.clone());
        }
        if !apply {
            continue;
        }
        // Safety re-check: full load (snapshot + journal), skip on any doubt.
        match Session::load_from_path(&path) {
            Ok(session) if !session.saved => {}
            _ => {
                report.skipped_on_recheck += 1;
                continue;
            }
        }
        if move_to_trash(sessions_dir, &path, now).is_ok() {
            report.removed += 1;
            report.bytes_freed += bytes;
            report.removed_ids.push(id);
        }
    }
    if apply {
        purge_trash(sessions_dir, now);
    }
    Ok(report)
}

/// Trash is bucketed by move date (`.trash/<YYYY-MM-DD>/`) because `rename`
/// preserves mtime, so file timestamps cannot tell when something was trashed.
fn move_to_trash(sessions_dir: &Path, path: &Path, now: DateTime<Utc>) -> std::io::Result<()> {
    let trash = sessions_dir
        .join(TRASH_DIR)
        .join(now.format("%Y-%m-%d").to_string());
    std::fs::create_dir_all(&trash)?;
    let companions: Vec<PathBuf> = vec![
        super::session_journal_path_from_snapshot(path),
        path.with_extension("bak"),
    ];
    std::fs::rename(path, trash.join(path.file_name().unwrap_or_default()))?;
    for extra in companions {
        if extra.exists() {
            let _ = std::fs::rename(&extra, trash.join(extra.file_name().unwrap_or_default()));
        }
    }
    Ok(())
}

fn purge_trash(sessions_dir: &Path, now: DateTime<Utc>) {
    let Ok(entries) = std::fs::read_dir(sessions_dir.join(TRASH_DIR)) else {
        return;
    };
    let cutoff = (now - Duration::days(TRASH_RETENTION_DAYS)).date_naive();
    for entry in entries.flatten() {
        let name = entry.file_name();
        if let Ok(day) = chrono::NaiveDate::parse_from_str(&name.to_string_lossy(), "%Y-%m-%d")
            && day < cutoff
        {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

pub fn sessions_dir() -> Result<PathBuf> {
    Ok(crate::storage::jcode_dir()?.join("sessions"))
}

pub fn run_gc(apply: bool) -> Result<GcReport> {
    let policy = crate::config::config().sessions.retention.clone();
    run_gc_in(&sessions_dir()?, Utc::now(), &policy, apply)
}

/// Startup hook: runs only when both `enabled` and `auto_gc` are true.
pub fn auto_gc_on_startup() {
    let policy = crate::config::config().sessions.retention.clone();
    if !(policy.enabled && policy.auto_gc) {
        return;
    }
    if let Ok(dir) = sessions_dir() {
        let _ = run_gc_in(&dir, Utc::now(), &policy, true);
    }
}

#[cfg(test)]
#[path = "retention_tests.rs"]
mod tests;
