//! Append-only local audit of board status changes.
//!
//! One JSON object per line in a per-user file under the Jcode data
//! directory. The file is created owner-only (0600 on unix) and opened with
//! `O_APPEND`. This module offers no API to truncate, rotate, rewrite or
//! delete it. Records hold identifiers and state ids only: no credentials,
//! no issue titles or other content.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const AUDIT_FILE_NAME: &str = "board-audit.jsonl";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmationRecord {
    /// RFC 3339 time of the explicit user confirmation.
    pub confirmed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentRecord {
    pub op_id: String,
    pub ts: String,
    pub world: String,
    pub backend: String,
    pub team: String,
    pub project: Option<String>,
    pub issue_id: String,
    pub expected_prior_state: String,
    pub requested_state: String,
    pub confirmation: ConfirmationRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditResult {
    Applied,
    /// Live state no longer matched the expected prior state; nothing written.
    Conflict,
    Failed,
    /// Write sent, result unknown. Not retried automatically.
    Uncertain,
    Reconciled {
        applied: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub op_id: String,
    pub ts: String,
    pub result: AuditResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum AuditEvent {
    Intent(IntentRecord),
    Outcome(OutcomeRecord),
}

pub struct AuditLog {
    path: PathBuf,
    file: Mutex<File>,
}

impl AuditLog {
    /// `<jcode data dir>/board-audit.jsonl`.
    pub fn default_path() -> io::Result<PathBuf> {
        crate::storage::jcode_dir()
            .map(|dir| dir.join(AUDIT_FILE_NAME))
            .map_err(|error| io::Error::other(error.to_string()))
    }

    pub fn open_default() -> io::Result<Self> {
        Self::open(&Self::default_path()?)
    }

    /// Open for append, creating the file owner-only. Existing content is
    /// never truncated and permissions of an existing file are not widened.
    pub fn open(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let mut options = OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            file: Mutex::new(file),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one line and sync it to disk. An error means the record may
    /// not be durable: callers must not dispatch the change it describes.
    pub fn append(&self, event: &AuditEvent) -> io::Result<()> {
        let mut line = serde_json::to_vec(event).map_err(io::Error::other)?;
        line.push(b'\n');
        let mut file = self
            .file
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // One write call on an O_APPEND descriptor keeps a line contiguous.
        file.write_all(&line)?;
        file.flush()?;
        file.sync_data()
    }
}

#[cfg(test)]
#[path = "audit_tests.rs"]
mod tests;
