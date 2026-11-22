//! Background task execution manager
//!
//! Allows tools to run in the background and notify the agent when complete.
//! Uses file-based storage for crash resilience + event channel for real-time notifications.

use crate::bus::{
    BackgroundTaskCompleted, BackgroundTaskProgress, BackgroundTaskProgressEvent,
    BackgroundTaskStatus, Bus, BusEvent,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, RwLock, watch};
use tokio::task::JoinHandle;
use tokio::time::{Duration, Instant as TokioInstant, MissedTickBehavior};

mod model;
#[path = "background/stall_watchdog.rs"]
mod stall_watchdog;

pub use model::{
    BackgroundCleanupResult, BackgroundTaskEventKind, BackgroundTaskEventRecord,
    BackgroundTaskInfo, BackgroundTaskWaitReason, BackgroundTaskWaitResult, MCP_TASK_INPUT_CAP,
    MCP_TASK_RESULT_CAP, McpRemoteState, McpTaskFinish, McpTaskRecord, RunningBackgroundProgress,
    TaskResult, TaskStatusFile, bound_json, cell_width, format_progress_display,
    format_progress_summary, render_progress_bar, truncate_cells,
};
use model::{
    EXIT_MARKER_PREFIX, RunningTask, normalize_delivery, progress_equivalent,
    progress_event_record, progress_wait_reason, push_task_event, task_dir, terminal_event_record,
};

/// Resolves the execution generation a session is currently running under.
/// Registered by the server, which owns durable session control; the base
/// crate cannot depend on it. Tasks created while it is unset or returns
/// `None` are not generation-bound.
pub type GenerationSource = fn(&str) -> Option<u64>;

static GENERATION_SOURCE: std::sync::OnceLock<GenerationSource> = std::sync::OnceLock::new();

/// Register the generation resolver used to stamp new tasks. First call wins.
pub fn set_generation_source(source: GenerationSource) {
    let _ = GENERATION_SOURCE.set(source);
}

fn current_owner_generation(session_id: &str) -> Option<u64> {
    GENERATION_SOURCE
        .get()
        .and_then(|source| source(session_id))
}

/// Whether the process behind a detached task is still the one registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProcessIdentity {
    /// Alive, and its start time matches (or identity cannot be disproved).
    Alive,
    Gone,
    /// The pid is alive but belongs to a different, later process.
    Reused,
}

/// Outcome of [`BackgroundTaskManager::reattach_detached_tasks`].
#[derive(Debug, Clone, Default)]
pub struct ReattachReport {
    /// Still-running tasks whose pid and start time match; the caller resumes
    /// output following for these.
    pub reattached: Vec<TaskStatusFile>,
    /// Tasks whose process ended while no server watched them and whose exit
    /// status was never recorded.
    pub lost: usize,
    /// Tasks finalized from a recorded exit status.
    pub finalized: usize,
}

/// Manages background task execution
pub struct BackgroundTaskManager {
    tasks: Arc<RwLock<HashMap<String, RunningTask>>>,
    /// Serializes status updates, including completion and delivery flag capture,
    /// so stale Running snapshots cannot replace terminal status.
    progress_updates: Arc<Mutex<()>>,
    /// Live stall watchdogs by task id. Each entry is a spawned monitor that
    /// fires a [`BusEvent::BackgroundTaskStalled`] after its task produces no
    /// output bytes and no progress events for the configured window.
    stall_watchdogs: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    /// Status files already seen in a terminal state by [`Self::sweep`]. A
    /// finished task never runs again, so the periodic sweep skips them.
    settled: Arc<Mutex<std::collections::HashSet<PathBuf>>>,
    /// Serializes finalization of detached tasks. The sweep, `list`, `status`,
    /// the output follower and `wait` all reconcile the same status file, and
    /// without this two of them both see `Running`, both write a terminal
    /// status and both publish a completion, so the owner is woken twice.
    /// Never held together with `progress_updates` in the opposite order:
    /// finalization does not take that lock.
    finalize_lock: Arc<Mutex<()>>,
    output_dir: PathBuf,
}

impl BackgroundTaskManager {
    /// Create a manager rooted at a specific output directory.
    ///
    /// Primarily for tests; production code should use [`global`].
    pub fn with_output_dir(output_dir: PathBuf) -> Self {
        if let Err(error) = std::fs::create_dir_all(&output_dir) {
            crate::logging::warn(&format!(
                "Cannot create background task output directory {}: {error}",
                output_dir.display()
            ));
        }
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            progress_updates: Arc::new(Mutex::new(())),
            stall_watchdogs: Arc::new(RwLock::new(HashMap::new())),
            settled: Arc::new(Mutex::new(std::collections::HashSet::new())),
            finalize_lock: Arc::new(Mutex::new(())),
            output_dir,
        }
    }

    /// Create a new background task manager
    pub fn new() -> Self {
        let output_dir = task_dir();
        Self::with_output_dir(output_dir)
    }

    /// Generate a short, unique task ID
    fn generate_task_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        const TASK_ID_ALPHABET: &[u8; 36] = b"abcdefghijklmnopqrstuvwxyz0123456789";

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        // Use last 6 digits of timestamp + 4 random chars
        let rand_part: String = (0..4)
            .map(|_| {
                let idx = (rand::random::<u8>() as usize) % TASK_ID_ALPHABET.len();
                TASK_ID_ALPHABET[idx] as char
            })
            .collect();
        format!(
            "{}{}",
            &timestamp.to_string()[timestamp.to_string().len().saturating_sub(6)..],
            rand_part
        )
    }

    pub fn output_path_for(&self, task_id: &str) -> PathBuf {
        self.output_dir.join(format!("{}.output", task_id))
    }

    pub fn status_path_for(&self, task_id: &str) -> PathBuf {
        self.output_dir.join(format!("{}.status.json", task_id))
    }

    fn publish_task_started_activity(
        task_id: &str,
        tool_name: &str,
        display_name: Option<&str>,
        session_id: &str,
        wake: bool,
    ) {
        // Lifecycle visibility is independent of transcript delivery and agent wake.
        let label = crate::message::background_task_display_label(tool_name, display_name);
        let safe_label = label.replace('`', "'");
        Bus::global().publish(BusEvent::UiActivity(crate::bus::UiActivity::background(
            Some(session_id.to_string()),
            format!(
                "**Background task started** `{}` · `{}`\n\nJcode is running this in the background. Progress, checkpoints, and completion will appear here.{}",
                task_id,
                safe_label,
                if wake {
                    crate::message::BACKGROUND_TASK_STARTED_WAKE_LINE
                } else {
                    ""
                }
            ),
            Some(format!("Background task started · {}", label)),
        )));
    }

    fn status_duration_secs(started_at: &str, completed_at: DateTime<Utc>) -> Option<f64> {
        DateTime::parse_from_rfc3339(started_at)
            .ok()
            .and_then(|started| (completed_at - started.with_timezone(&Utc)).to_std().ok())
            .map(|duration| duration.as_secs_f64())
    }

    fn parse_exit_code_from_output(output: &str) -> Option<i32> {
        output.lines().rev().find_map(|line| {
            let trimmed = line.trim();
            let suffix = trimmed.strip_prefix(EXIT_MARKER_PREFIX)?;
            let suffix = suffix.strip_suffix(" ---")?;
            suffix.trim().parse::<i32>().ok()
        })
    }

    async fn read_status_file(&self, path: &std::path::Path) -> Option<TaskStatusFile> {
        let content = fs::read_to_string(path).await.ok()?;
        serde_json::from_str(&content).ok()
    }

    async fn write_status_file(&self, path: &std::path::Path, status: &TaskStatusFile) {
        if let Ok(json) = serde_json::to_string_pretty(status) {
            let _ = fs::write(path, json).await;
        }
    }

    /// Classify the process behind a detached task. A matching pid alone is
    /// not enough: a later process can reuse it, so the start time captured at
    /// registration must also match when both sides are known. Where either is
    /// unavailable identity cannot be disproved and the process counts as
    /// alive, so uncertain liveness is never reported as gone.
    fn detached_process_identity(status: &TaskStatusFile) -> ProcessIdentity {
        let Some(pid) = status.pid else {
            return ProcessIdentity::Gone;
        };
        // Compare start times before the liveness verdict: an unreaped zombie
        // that took a reused pid is still not our task's process, and reaping
        // it would steal that child's exit status.
        let running = crate::platform::is_process_running(pid);
        match (
            status.pid_start_time,
            crate::platform::process_start_time(pid),
        ) {
            (Some(recorded), Some(current)) if recorded != current => ProcessIdentity::Reused,
            _ if running => ProcessIdentity::Alive,
            _ => ProcessIdentity::Gone,
        }
    }

    async fn finalize_detached_status_if_needed(
        &self,
        status: TaskStatusFile,
        status_path: &std::path::Path,
    ) -> TaskStatusFile {
        if status.status != BackgroundTaskStatus::Running || !status.detached {
            return status;
        }

        let Some(pid) = status.pid else {
            return status;
        };

        let identity = Self::detached_process_identity(&status);
        // A reused pid may name an unrelated child of this process: reaping it
        // would steal that child's exit status.
        let reaped_exit = if identity == ProcessIdentity::Reused {
            None
        } else {
            crate::platform::try_reap_child_process(pid).ok().flatten()
        };
        if reaped_exit.is_none() && identity == ProcessIdentity::Alive {
            return status;
        }

        // The sweep, `list`, `status`, the output follower and `wait` all land
        // here for the same file. Exactly one may finalize and publish; the
        // rest must observe its result instead of announcing a second
        // completion to the owning session.
        let _finalizing = self.finalize_lock.lock().await;
        let mut status = match self.read_status_file(status_path).await {
            Some(fresh) if fresh.status != BackgroundTaskStatus::Running => return fresh,
            Some(fresh) => fresh,
            None => status,
        };

        let output_path = self.output_path_for(&status.task_id);
        let output = fs::read_to_string(&output_path).await.unwrap_or_default();
        let exit_code = reaped_exit.or_else(|| Self::parse_exit_code_from_output(&output));
        let completed_at = Utc::now();
        let duration_secs = Self::status_duration_secs(&status.started_at, completed_at);
        // Only a status the process (or its wrapper) actually recorded decides
        // success. A process that vanished without one is `Lost`: its outcome
        // is unknown, never assumed good.
        let final_status = match exit_code {
            Some(0) => BackgroundTaskStatus::Completed,
            Some(_) => BackgroundTaskStatus::Failed,
            None => BackgroundTaskStatus::Lost,
        };
        let final_error = match (&final_status, exit_code) {
            (BackgroundTaskStatus::Failed, Some(code)) => {
                Some(format!("Command exited with code {}", code))
            }
            (BackgroundTaskStatus::Lost, _) if identity == ProcessIdentity::Reused => Some(
                "Process is gone: its pid was reused by another process. Exit status unknown; output file is kept"
                    .to_string(),
            ),
            (BackgroundTaskStatus::Lost, _) => Some(
                "Process exited (status unknown): no exit status was recorded while no server was watching. Output file is kept"
                    .to_string(),
            ),
            _ => None,
        };

        status.status = final_status.clone();
        status.exit_code = exit_code;
        status.error = final_error.clone();
        status.completed_at = Some(completed_at.to_rfc3339());
        status.duration_secs = duration_secs;
        status.pid = Some(pid);
        push_task_event(
            &mut status,
            terminal_event_record(final_status.clone(), exit_code, final_error.as_deref()),
        );

        self.write_status_file(status_path, &status).await;

        let output_preview = if output.len() > 500 {
            format!("{}...", crate::util::truncate_str(&output, 500))
        } else {
            output
        };
        Bus::global().publish(BusEvent::BackgroundTaskCompleted(BackgroundTaskCompleted {
            task_id: status.task_id.clone(),
            tool_name: status.tool_name.clone(),
            display_name: status.display_name.clone(),
            session_id: status.session_id.clone(),
            status: final_status,
            exit_code,
            output_preview,
            output_file: output_path,
            duration_secs: duration_secs.unwrap_or_default(),
            notify: status.notify,
            wake: status.wake,
        }));

        status
    }

    /// True when a non-detached `Running` status file provably belongs to a
    /// process image that no longer exists, so no future can ever finalize it.
    ///
    /// Rules, deliberately conservative because the task dir is shared by
    /// every jcode process on the machine:
    /// - Terminal, detached, or pid-bearing files are never orphans here
    ///   (detached reconciliation is `finalize_detached_status_if_needed`).
    /// - Files owned by this exact process image are never orphans: the
    ///   initial status file is written before the task lands in the live
    ///   map, so "Running + not in map + my instance" can simply mean the
    ///   task is still bootstrapping.
    /// - Files owned by this PID but a different instance token are orphans:
    ///   an exec-based reload replaced the process image, so the owning
    ///   future is gone even though the PID matches.
    /// - Files owned by another PID are orphans only once that process is
    ///   dead.
    /// - Files without owner metadata (written by older builds) are left
    ///   alone; only the explicit startup sweep in
    ///   [`Self::reconcile_orphaned_tasks`] handles those.
    fn status_is_reconcilable_orphan(status: &TaskStatusFile) -> bool {
        if status.status != BackgroundTaskStatus::Running || status.detached || status.pid.is_some()
        {
            return false;
        }
        let Some(owner_pid) = status.owner_pid else {
            return false;
        };
        if status.owner_instance.as_deref() == Some(model::process_instance_token()) {
            return false;
        }
        if owner_pid == std::process::id() {
            return true;
        }
        !crate::platform::is_process_running(owner_pid)
    }

    /// Finalize an orphaned non-detached `Running` status file as `Failed`.
    ///
    /// The owning process's task future died with the process (crash or
    /// exec-based server reload), so without this the file reads `Running`
    /// forever: `bg list`/`bg status` show a phantom task and `bg wait`
    /// blocks until its timeout.
    async fn finalize_orphaned_status_if_needed(
        &self,
        mut status: TaskStatusFile,
        status_path: &std::path::Path,
    ) -> TaskStatusFile {
        if !Self::status_is_reconcilable_orphan(&status) {
            return status;
        }
        // Belt and braces: never rewrite a task this process is executing.
        if self.is_live_task(&status.task_id) {
            return status;
        }

        let completed_at = Utc::now();
        let duration_secs = Self::status_duration_secs(&status.started_at, completed_at);
        let error =
            "Task orphaned: the owning server process exited (reloaded or crashed) before the task finished"
                .to_string();
        status.status = BackgroundTaskStatus::Failed;
        status.exit_code = None;
        status.error = Some(error.clone());
        status.completed_at = Some(completed_at.to_rfc3339());
        status.duration_secs = duration_secs;
        push_task_event(
            &mut status,
            terminal_event_record(BackgroundTaskStatus::Failed, None, Some(&error)),
        );
        self.write_status_file(status_path, &status).await;

        let output_path = self.output_path_for(&status.task_id);
        let output = fs::read_to_string(&output_path).await.unwrap_or_default();
        let output_preview = if output.len() > 500 {
            format!("{}...", crate::util::truncate_str(&output, 500))
        } else {
            output
        };
        Bus::global().publish(BusEvent::BackgroundTaskCompleted(BackgroundTaskCompleted {
            task_id: status.task_id.clone(),
            tool_name: status.tool_name.clone(),
            display_name: status.display_name.clone(),
            session_id: status.session_id.clone(),
            status: BackgroundTaskStatus::Failed,
            exit_code: None,
            output_preview,
            output_file: output_path,
            duration_secs: duration_secs.unwrap_or_default(),
            notify: status.notify,
            wake: status.wake,
        }));

        status
    }

    /// Startup/reload sweep: mark orphaned non-detached `Running` status
    /// files as `Failed` with a "server reloaded" note.
    ///
    /// Only owner-tagged files are considered, using the liveness rules of
    /// [`Self::status_is_reconcilable_orphan`]. Files without owner metadata
    /// (written by older builds, or by processes that legitimately still run
    /// them) are left untouched: the task dir is shared machine-wide, so
    /// without owner metadata there is no safe way to distinguish a phantom
    /// from another live process's task. Returns how many files were
    /// reconciled.
    pub async fn reconcile_orphaned_tasks(&self) -> usize {
        let mut reconciled = 0;
        let Ok(mut entries) = fs::read_dir(&self.output_dir).await else {
            return reconciled;
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(status) = self.read_status_file(&path).await else {
                continue;
            };
            if !Self::status_is_reconcilable_orphan(&status) {
                continue;
            }
            if self.tasks.read().await.contains_key(&status.task_id) {
                continue;
            }
            self.finalize_orphaned_status_if_needed(status, &path).await;
            reconciled += 1;
        }
        reconciled
    }

    /// Reconcile every persisted detached task against its live process.
    ///
    /// Run once when a server image starts (reload, restart or crash
    /// recovery). Detached tasks are the ones that outlive a server, so each
    /// `Running` file is checked by pid *and* recorded start time:
    /// - alive and matching: reattached. It stays `Running`, keeps its task
    ///   id, owner generation and delivery flags, and is returned so the
    ///   caller can resume output following and stall watching;
    /// - gone or pid reused: finalized once through the same path as every
    ///   other reconciler. A recorded exit status (wrapper marker or reaped
    ///   child) decides success or failure; without one the task is `Lost`
    ///   and its exit status stays unknown. The output file is never touched.
    ///
    /// Idempotent: a second call finds no new work to finalize and returns
    /// the same live set, and finalization publishes at most one completion.
    pub async fn reattach_detached_tasks(&self) -> ReattachReport {
        let mut report = ReattachReport::default();
        let Ok(mut entries) = fs::read_dir(&self.output_dir).await else {
            return report;
        };
        let mut candidates = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if !path.to_string_lossy().ends_with(".status.json") {
                continue;
            }
            let Some(status) = self.read_status_file(&path).await else {
                continue;
            };
            if status.status == BackgroundTaskStatus::Running
                && status.detached
                && status.pid.is_some()
            {
                candidates.push((path, status));
            }
        }
        candidates.sort_by(|a, b| a.1.task_id.cmp(&b.1.task_id));
        for (path, status) in candidates {
            if Self::detached_process_identity(&status) == ProcessIdentity::Alive {
                report.reattached.push(status);
                continue;
            }
            let settled = self.finalize_detached_status_if_needed(status, &path).await;
            match settled.status {
                BackgroundTaskStatus::Lost => report.lost += 1,
                BackgroundTaskStatus::Running => report.reattached.push(settled),
                _ => report.finalized += 1,
            }
        }
        report
    }

    /// Execution generation recorded for a task, read straight from its status
    /// file without reconciling it. `None` for unknown tasks and tasks that are
    /// not generation-bound.
    pub async fn owner_generation_of(&self, task_id: &str) -> Option<u64> {
        self.read_status_file(&self.status_path_for(task_id))
            .await?
            .owner_generation
    }

    pub fn reserve_task_info(&self) -> BackgroundTaskInfo {
        let task_id = Self::generate_task_id();
        let output_file = self.output_path_for(&task_id);
        let status_file = self.status_path_for(&task_id);
        BackgroundTaskInfo {
            task_id,
            output_file,
            status_file,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Detached task registration mirrors persisted status fields and existing call sites"
    )]
    pub async fn register_detached_task(
        &self,
        info: &BackgroundTaskInfo,
        tool_name: &str,
        display_name: Option<String>,
        session_id: &str,
        pid: u32,
        started_at: &str,
        notify: bool,
        wake: bool,
    ) {
        let (notify, wake) = normalize_delivery(notify, wake);
        let status = TaskStatusFile {
            task_id: info.task_id.clone(),
            tool_name: tool_name.to_string(),
            display_name,
            session_id: session_id.to_string(),
            status: BackgroundTaskStatus::Running,
            exit_code: None,
            error: None,
            started_at: started_at.to_string(),
            completed_at: None,
            duration_secs: None,
            pid: Some(pid),
            // Detached processes outlive this server, so no in-process owner:
            // reconciliation must never clobber them.
            owner_pid: None,
            owner_instance: None,
            detached: true,
            notify,
            wake,
            progress: None,
            event_history: Vec::new(),
            pid_start_time: crate::platform::process_start_time(pid),
            owner_generation: current_owner_generation(session_id),
            stall_wake_seconds: None,
            mcp_task: None,
        };
        self.write_status_file(&info.status_file, &status).await;
        Self::publish_task_started_activity(
            &info.task_id,
            tool_name,
            status.display_name.as_deref(),
            session_id,
            wake,
        );
    }

    /// Persist a new MCP task record as a `Running` background task.
    ///
    /// The status file is written before anything else and a write failure is
    /// returned, so the caller never exposes a task handle that was not
    /// persisted. The record has no in-process owner (`owner_pid` and
    /// `owner_instance` are `None`), so orphan reconciliation never fails it:
    /// the remote server, not this process, owns the work. A record whose
    /// remote status is already terminal is finalized immediately.
    pub async fn register_mcp_task(
        &self,
        record: McpTaskRecord,
        session_id: &str,
        notify: bool,
        wake: bool,
    ) -> Result<BackgroundTaskInfo> {
        let (notify, wake) = normalize_delivery(notify, wake);
        let info = self.reserve_task_info();
        let tool_name = record.kind_label();
        let mut status = TaskStatusFile {
            task_id: info.task_id.clone(),
            tool_name: tool_name.clone(),
            display_name: None,
            session_id: session_id.to_string(),
            status: BackgroundTaskStatus::Running,
            exit_code: None,
            error: None,
            started_at: Utc::now().to_rfc3339(),
            completed_at: None,
            duration_secs: None,
            pid: None,
            owner_pid: None,
            owner_instance: None,
            detached: false,
            notify,
            wake,
            progress: None,
            event_history: Vec::new(),
            pid_start_time: None,
            owner_generation: current_owner_generation(session_id),
            stall_wake_seconds: None,
            mcp_task: Some(record),
        };
        fs::create_dir_all(&self.output_dir).await?;
        let json = serde_json::to_string_pretty(&status)?;
        write_atomic(&info.status_file, &json).await?;
        Self::publish_task_started_activity(&info.task_id, &tool_name, None, session_id, wake);
        if let Some(finish) = status.mcp_task.as_ref().and_then(McpTaskRecord::finish) {
            let _guard = self.progress_updates.lock().await;
            let _finalizing = self.finalize_lock.lock().await;
            self.finalize_mcp_status(&mut status, finish).await?;
        }
        Ok(info)
    }

    /// Read-modify-write the MCP part of a task record.
    ///
    /// `update` returns whether it changed the record; unchanged records are
    /// not rewritten. A remote terminal status (or expiry / not-found) also
    /// finalizes the background-task status once and publishes its completion
    /// like any other task. Records that are already terminal, or that carry
    /// no MCP task, are returned untouched. Returns `Ok(None)` for an unknown
    /// task and an error when the new state cannot be persisted.
    pub async fn update_mcp_task<F>(
        &self,
        task_id: &str,
        update: F,
    ) -> Result<Option<TaskStatusFile>>
    where
        F: FnOnce(&mut McpTaskRecord) -> bool,
    {
        let _status_guard = self.progress_updates.lock().await;
        let _finalizing = self.finalize_lock.lock().await;
        let Some(mut status) = self.read_status_file(&self.status_path_for(task_id)).await else {
            return Ok(None);
        };
        let Some(record) = status.mcp_task.as_mut() else {
            return Ok(Some(status));
        };
        if status.status != BackgroundTaskStatus::Running || !update(record) {
            return Ok(Some(status));
        }
        match record.finish() {
            Some(finish) => self.finalize_mcp_status(&mut status, finish).await?,
            None => {
                let json = serde_json::to_string_pretty(&status)?;
                write_atomic(&self.status_path_for(task_id), &json).await?;
            }
        }
        Ok(Some(status))
    }

    /// Move an MCP record to its terminal background status, persist it and
    /// publish the completion. Callers hold `progress_updates` and
    /// `finalize_lock`.
    async fn finalize_mcp_status(
        &self,
        status: &mut TaskStatusFile,
        finish: McpTaskFinish,
    ) -> Result<()> {
        let completed_at = Utc::now();
        let duration_secs = Self::status_duration_secs(&status.started_at, completed_at);
        status.status = finish.status.clone();
        status.exit_code = None;
        status.error = finish.error.clone();
        status.completed_at = Some(completed_at.to_rfc3339());
        status.duration_secs = duration_secs;
        push_task_event(
            status,
            terminal_event_record(finish.status.clone(), None, finish.error.as_deref()),
        );
        let output_path = self.output_path_for(&status.task_id);
        if let Err(error) = fs::write(&output_path, &finish.output).await {
            crate::logging::warn(&format!(
                "Cannot write MCP task output {}: {error}",
                output_path.display()
            ));
        }
        let json = serde_json::to_string_pretty(&*status)?;
        write_atomic(&self.status_path_for(&status.task_id), &json).await?;
        let output_preview = if finish.output.len() > 500 {
            format!("{}...", crate::util::truncate_str(&finish.output, 500))
        } else {
            finish.output
        };
        Bus::global().publish(BusEvent::BackgroundTaskCompleted(BackgroundTaskCompleted {
            task_id: status.task_id.clone(),
            tool_name: status.tool_name.clone(),
            display_name: status.display_name.clone(),
            session_id: status.session_id.clone(),
            status: finish.status,
            exit_code: None,
            output_preview,
            output_file: output_path,
            duration_secs: duration_secs.unwrap_or_default(),
            notify: status.notify,
            wake: status.wake,
        }));
        Ok(())
    }

    /// Spawn a background task
    ///
    /// The `execute_fn` receives the output file path and should write output there.
    /// It returns a TaskResult with exit code and optional error.
    pub async fn spawn<F, Fut>(
        &self,
        tool_name: &str,
        session_id: &str,
        execute_fn: F,
    ) -> BackgroundTaskInfo
    where
        F: FnOnce(PathBuf) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<TaskResult>> + Send,
    {
        self.spawn_with_notify(tool_name, None, session_id, true, false, execute_fn)
            .await
    }

    /// Spawn a background task with explicit notify flag
    pub async fn spawn_with_notify<F, Fut>(
        &self,
        tool_name: &str,
        display_name: Option<String>,
        session_id: &str,
        notify: bool,
        wake: bool,
        execute_fn: F,
    ) -> BackgroundTaskInfo
    where
        F: FnOnce(PathBuf) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<TaskResult>> + Send,
    {
        let runtime_activity = crate::runtime_activity::global()
            .enter_session(session_id)
            .await;
        let (notify, wake) = normalize_delivery(notify, wake);
        let task_id = Self::generate_task_id();
        let output_path = self.output_dir.join(format!("{}.output", task_id));
        let status_path = self.output_dir.join(format!("{}.status.json", task_id));
        let started_at_rfc3339 = chrono::Utc::now().to_rfc3339();

        // Write initial status file
        let initial_status = TaskStatusFile {
            task_id: task_id.clone(),
            tool_name: tool_name.to_string(),
            display_name: display_name.clone(),
            session_id: session_id.to_string(),
            status: BackgroundTaskStatus::Running,
            exit_code: None,
            error: None,
            started_at: started_at_rfc3339.clone(),
            completed_at: None,
            duration_secs: None,
            pid: None,
            owner_pid: Some(std::process::id()),
            owner_instance: Some(model::process_instance_token().to_string()),
            detached: false,
            notify,
            wake,
            progress: None,
            event_history: Vec::new(),
            pid_start_time: None,
            owner_generation: current_owner_generation(session_id),
            stall_wake_seconds: None,
            mcp_task: None,
        };
        let owner_generation = initial_status.owner_generation;
        if let Ok(json) = serde_json::to_string_pretty(&initial_status) {
            let _ = std::fs::write(&status_path, json);
        }
        Self::publish_task_started_activity(
            &task_id,
            tool_name,
            display_name.as_deref(),
            session_id,
            wake,
        );

        let output_path_clone = output_path.clone();
        let status_path_clone = status_path.clone();
        let task_id_clone = task_id.clone();
        let tool_name_owned = tool_name.to_string();
        let display_name_owned = display_name.clone();
        let session_id_owned = session_id.to_string();
        let started_at = Instant::now();
        let started_at_rfc3339_for_task = started_at_rfc3339.clone();
        let (delivery_flags_tx, delivery_flags_rx) = watch::channel((notify, wake));
        let status_updates = Arc::clone(&self.progress_updates);
        let tasks_for_prune = Arc::clone(&self.tasks);
        let (registered_tx, registered_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn the background task
        let handle = tokio::spawn(runtime_activity.scope(async move {
            let result = execute_fn(output_path_clone.clone()).await;

            let duration_secs = started_at.elapsed().as_secs_f64();
            let (status, exit_code, error) = match &result {
                Ok(task_result) => {
                    let status = task_result.status.clone().unwrap_or_else(|| {
                        if task_result.error.is_some() {
                            BackgroundTaskStatus::Failed
                        } else {
                            BackgroundTaskStatus::Completed
                        }
                    });
                    (status, task_result.exit_code, task_result.error.clone())
                }
                Err(e) => (BackgroundTaskStatus::Failed, None, Some(e.to_string())),
            };

            let status_guard = status_updates.lock().await;
            let (notify_flag, wake_flag) = *delivery_flags_rx.borrow();
            let prior_status = tokio::fs::read_to_string(&status_path_clone)
                .await
                .ok()
                .and_then(|content| serde_json::from_str::<TaskStatusFile>(&content).ok());
            let prior_progress = prior_status
                .as_ref()
                .and_then(|status| status.progress.clone());
            let prior_event_history = prior_status
                .map(|status| status.event_history)
                .unwrap_or_default();

            // Update status file
            let mut final_status = TaskStatusFile {
                task_id: task_id_clone.clone(),
                tool_name: tool_name_owned.clone(),
                display_name: display_name_owned.clone(),
                session_id: session_id_owned.clone(),
                status: status.clone(),
                exit_code,
                error: error.clone(),
                started_at: started_at_rfc3339_for_task,
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
                duration_secs: Some(duration_secs),
                pid: None,
                owner_pid: Some(std::process::id()),
                owner_instance: Some(model::process_instance_token().to_string()),
                detached: false,
                notify: notify_flag,
                wake: wake_flag,
                progress: prior_progress,
                event_history: prior_event_history,
                pid_start_time: None,
                owner_generation,
                stall_wake_seconds: None,
                mcp_task: None,
            };
            push_task_event(
                &mut final_status,
                terminal_event_record(status.clone(), exit_code, error.as_deref()),
            );
            if let Ok(json) = serde_json::to_string_pretty(&final_status) {
                let _ = tokio::fs::write(&status_path_clone, json).await;
            }

            // Drop this task from the live map now that its terminal status is
            // persisted. Order matters: pruning only after the status-file
            // write keeps "in the live map while the status file says Running"
            // equivalent to "a task future is actually executing", which the
            // run_plan duplicate-driver guard and self-dev build reconciliation
            // rely on. Awaiting registration first means a task that finishes
            // instantly cannot race the insert below and leave a permanent
            // phantom entry in the map.
            drop(status_guard);
            if let Err(error) = registered_rx.await {
                crate::logging::debug(&format!(
                    "Background task registration ended before completion: {error}"
                ));
            }
            tasks_for_prune.write().await.remove(&task_id_clone);

            // Read output preview for notification
            let output_preview = tokio::fs::read_to_string(&output_path_clone)
                .await
                .map(|s| {
                    if s.len() > 500 {
                        format!("{}...", crate::util::truncate_str(&s, 500))
                    } else {
                        s
                    }
                })
                .unwrap_or_default();

            // Publish completion event to the bus
            Bus::global().publish(BusEvent::BackgroundTaskCompleted(BackgroundTaskCompleted {
                task_id: task_id_clone,
                tool_name: tool_name_owned,
                display_name: display_name_owned,
                session_id: session_id_owned,
                status,
                exit_code,
                output_preview,
                output_file: output_path_clone,
                duration_secs,
                notify: notify_flag,
                wake: wake_flag,
            }));

            result
        }));

        // Track the running task
        let running_task = RunningTask {
            task_id: task_id.clone(),
            tool_name: tool_name.to_string(),
            display_name,
            session_id: session_id.to_string(),
            status_path: status_path.clone(),
            owner_generation,
            started_at,
            started_at_rfc3339,
            delivery_flags: delivery_flags_tx,
            handle,
        };

        self.tasks
            .write()
            .await
            .insert(task_id.clone(), running_task);
        let _ = registered_tx.send(());

        BackgroundTaskInfo {
            task_id,
            output_file: output_path,
            status_file: status_path,
        }
    }

    /// Adopt an already-spawned task as a background task.
    /// Used when the user moves a currently-executing tool to background via Alt+B.
    /// The `handle` is an already-running tokio task; we just register it for tracking
    /// and wire up completion notifications.
    pub async fn adopt(
        &self,
        tool_name: &str,
        session_id: &str,
        handle: JoinHandle<Result<jcode_tool_types::ToolOutput>>,
    ) -> BackgroundTaskInfo {
        self.adopt_with_options(tool_name, None, session_id, true, false, handle)
            .await
    }

    /// Adopt an already-spawned task as a background task, with explicit display
    /// name and delivery flags. Used both for user-initiated handoff (Alt+B) and
    /// for promoting a foreground command that exceeded its timeout but is still
    /// running, so it keeps running and surfaces as a background-task card.
    pub async fn adopt_with_options(
        &self,
        tool_name: &str,
        display_name: Option<String>,
        session_id: &str,
        notify: bool,
        wake: bool,
        handle: JoinHandle<Result<jcode_tool_types::ToolOutput>>,
    ) -> BackgroundTaskInfo {
        let mut handle = model::AbortAdoptedTask(handle);
        // Reserve before the foreground caller can release its turn/tool guard.
        let runtime_activity = crate::runtime_activity::global()
            .enter_session(session_id)
            .await;
        let (notify, wake) = normalize_delivery(notify, wake);
        let task_id = Self::generate_task_id();
        let output_path = self.output_dir.join(format!("{}.output", task_id));
        let status_path = self.output_dir.join(format!("{}.status.json", task_id));

        // Retain an output artifact before publishing the running task or
        // returning its ID. Adopted work buffers its output until completion,
        // when the wrapper below replaces this initially empty file. Create it
        // before spawning that wrapper so a fast completion cannot be truncated.
        let _ = std::fs::write(&output_path, "");

        let initial_status = TaskStatusFile {
            task_id: task_id.clone(),
            tool_name: tool_name.to_string(),
            display_name: display_name.clone(),
            session_id: session_id.to_string(),
            status: BackgroundTaskStatus::Running,
            exit_code: None,
            error: None,
            started_at: chrono::Utc::now().to_rfc3339(),
            completed_at: None,
            duration_secs: None,
            pid: None,
            owner_pid: Some(std::process::id()),
            owner_instance: Some(model::process_instance_token().to_string()),
            detached: false,
            notify,
            wake,
            progress: None,
            event_history: Vec::new(),
            pid_start_time: None,
            owner_generation: current_owner_generation(session_id),
            stall_wake_seconds: None,
            mcp_task: None,
        };
        let owner_generation = initial_status.owner_generation;
        if let Ok(json) = serde_json::to_string_pretty(&initial_status) {
            let _ = std::fs::write(&status_path, json);
        }
        Self::publish_task_started_activity(
            &task_id,
            tool_name,
            display_name.as_deref(),
            session_id,
            wake,
        );

        let output_path_clone = output_path.clone();
        let status_path_clone = status_path.clone();
        let task_id_clone = task_id.clone();
        let tool_name_owned = tool_name.to_string();
        let session_id_owned = session_id.to_string();
        let started_at = Instant::now();
        let started_at_rfc3339 = initial_status.started_at.clone();
        let display_name_owned = initial_status.display_name.clone();
        let (delivery_flags_tx, delivery_flags_rx) = watch::channel((notify, wake));
        let status_updates = Arc::clone(&self.progress_updates);
        let tasks_for_prune = Arc::clone(&self.tasks);
        let (registered_tx, registered_rx) = tokio::sync::oneshot::channel::<()>();
        let wrapper_handle = tokio::spawn(runtime_activity.scope(async move {
            let tool_result = (&mut handle.0).await;
            let duration_secs = started_at.elapsed().as_secs_f64();

            let (status, exit_code, error, output_text) = match tool_result {
                Ok(Ok(output)) => (
                    BackgroundTaskStatus::Completed,
                    Some(0),
                    None,
                    output.output,
                ),
                Ok(Err(e)) => (
                    BackgroundTaskStatus::Failed,
                    None,
                    Some(e.to_string()),
                    e.to_string(),
                ),
                Err(e) => (
                    BackgroundTaskStatus::Failed,
                    None,
                    Some(e.to_string()),
                    format!("Task panicked: {}", e),
                ),
            };

            if let Ok(mut file) = File::create(&output_path_clone).await {
                let _ = file.write_all(output_text.as_bytes()).await;
            }

            let status_guard = status_updates.lock().await;
            let (notify_flag, wake_flag) = *delivery_flags_rx.borrow();
            let prior_status = tokio::fs::read_to_string(&status_path_clone)
                .await
                .ok()
                .and_then(|content| serde_json::from_str::<TaskStatusFile>(&content).ok());
            let prior_progress = prior_status
                .as_ref()
                .and_then(|status| status.progress.clone());
            let prior_event_history = prior_status
                .map(|status| status.event_history)
                .unwrap_or_default();

            let mut final_status = TaskStatusFile {
                task_id: task_id_clone.clone(),
                tool_name: tool_name_owned.clone(),
                display_name: display_name_owned.clone(),
                session_id: session_id_owned.clone(),
                status: status.clone(),
                exit_code,
                error: error.clone(),
                started_at: started_at_rfc3339,
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
                duration_secs: Some(duration_secs),
                pid: None,
                owner_pid: Some(std::process::id()),
                owner_instance: Some(model::process_instance_token().to_string()),
                detached: false,
                notify: notify_flag,
                wake: wake_flag,
                progress: prior_progress,
                event_history: prior_event_history,
                pid_start_time: None,
                owner_generation,
                stall_wake_seconds: None,
                mcp_task: None,
            };
            push_task_event(
                &mut final_status,
                terminal_event_record(status.clone(), exit_code, error.as_deref()),
            );
            if let Ok(json) = serde_json::to_string_pretty(&final_status) {
                let _ = tokio::fs::write(&status_path_clone, json).await;
            }

            // Prune the live-map entry only after the terminal status file is
            // persisted (and after registration below, so instant completions
            // cannot race the insert and leave a phantom entry).
            drop(status_guard);
            if let Err(error) = registered_rx.await {
                crate::logging::debug(&format!(
                    "Background task registration ended before completion: {error}"
                ));
            }
            tasks_for_prune.write().await.remove(&task_id_clone);

            let output_preview = if output_text.len() > 500 {
                format!("{}...", crate::util::truncate_str(&output_text, 500))
            } else {
                output_text
            };

            Bus::global().publish(BusEvent::BackgroundTaskCompleted(BackgroundTaskCompleted {
                task_id: task_id_clone,
                tool_name: tool_name_owned,
                display_name: display_name_owned,
                session_id: session_id_owned,
                status: status.clone(),
                exit_code,
                output_preview,
                output_file: output_path_clone,
                duration_secs,
                notify: notify_flag,
                wake: wake_flag,
            }));

            Ok(TaskResult {
                exit_code,
                error,
                status: Some(status),
            })
        }));

        let running_task = RunningTask {
            task_id: task_id.clone(),
            tool_name: tool_name.to_string(),
            display_name: None,
            session_id: session_id.to_string(),
            status_path: status_path.clone(),
            owner_generation,
            started_at,
            started_at_rfc3339: initial_status.started_at.clone(),
            delivery_flags: delivery_flags_tx,
            handle: wrapper_handle,
        };

        self.tasks
            .write()
            .await
            .insert(task_id.clone(), running_task);
        let _ = registered_tx.send(());

        BackgroundTaskInfo {
            task_id,
            output_file: output_path,
            status_file: status_path,
        }
    }

    /// Periodic maintenance pass: reconcile tasks that may still be running
    /// and return the number of status files on disk.
    ///
    /// `list` opens, parses and reconciles every status file through tokio's
    /// blocking pool, which costs several thread wakeups per file, and the
    /// task directory keeps thousands of finished tasks. A once-a-minute
    /// caller (the runtime memory heartbeat) made that the server's largest
    /// source of idle wakeups. This lists the directory in one blocking call
    /// and only reads files not yet seen in a terminal state, so a quiet
    /// server reads nothing after the first pass.
    ///
    /// Reconciliation is kept because it is what finalizes a detached task,
    /// and publishes its completion, when the task outlived the server that
    /// started it and no one is waiting on it. The count is by file
    /// extension, so an unparseable status file is included.
    pub async fn sweep(&self) -> usize {
        let dir = self.output_dir.clone();
        let paths = tokio::task::spawn_blocking(move || {
            std::fs::read_dir(dir)
                .map(|entries| {
                    entries
                        .flatten()
                        .map(|entry| entry.path())
                        .filter(|path| path.extension().is_some_and(|e| e == "json"))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();

        let pending: Vec<PathBuf> = {
            let mut settled = self.settled.lock().await;
            // Drop entries for files that were cleaned up.
            if settled.len() > paths.len() {
                let present: std::collections::HashSet<&PathBuf> = paths.iter().collect();
                settled.retain(|path| present.contains(path));
            }
            paths
                .iter()
                .filter(|path| !settled.contains(*path))
                .cloned()
                .collect()
        };

        for path in pending {
            let Some(status) = self.read_status_file(&path).await else {
                continue;
            };
            let status = self.finalize_detached_status_if_needed(status, &path).await;
            let status = self.finalize_orphaned_status_if_needed(status, &path).await;
            if status.status != BackgroundTaskStatus::Running {
                self.settled.lock().await.insert(path);
            }
        }

        paths.len()
    }

    /// Whether any task is still `Running`. A task directory that exists but
    /// cannot be read counts as running: unknown is not idle.
    pub async fn has_running_tasks(&self) -> bool {
        match fs::read_dir(&self.output_dir).await {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => return true,
            _ => {}
        }
        self.list()
            .await
            .iter()
            .any(|task| task.status == BackgroundTaskStatus::Running)
    }

    /// List all tasks (both running and completed from disk)
    pub async fn list(&self) -> Vec<TaskStatusFile> {
        let mut results = Vec::new();

        // Read all status files from disk
        if let Ok(mut entries) = fs::read_dir(&self.output_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false)
                    && let Some(status) = self.read_status_file(&path).await
                {
                    let reconciled = self.finalize_detached_status_if_needed(status, &path).await;
                    let reconciled = self
                        .finalize_orphaned_status_if_needed(reconciled, &path)
                        .await;
                    results.push(reconciled);
                }
            }
        }

        // Sort by task_id (which includes timestamp)
        results.sort_by(|a, b| b.task_id.cmp(&a.task_id));
        results
    }

    /// Get status of a specific task
    pub async fn status(&self, task_id: &str) -> Option<TaskStatusFile> {
        let status_path = self.status_path_for(task_id);
        let status = self.read_status_file(&status_path).await?;
        let status = self
            .finalize_detached_status_if_needed(status, &status_path)
            .await;
        Some(
            self.finalize_orphaned_status_if_needed(status, &status_path)
                .await,
        )
    }

    /// Best-effort synchronous check for whether a task is still live in this process.
    pub fn is_live_task(&self, task_id: &str) -> bool {
        let Ok(tasks) = self.tasks.try_read() else {
            return false;
        };
        tasks.contains_key(task_id)
    }

    /// Get full output of a task
    pub async fn output(&self, task_id: &str) -> Option<String> {
        let output_path = self.output_path_for(task_id);
        fs::read_to_string(&output_path).await.ok()
    }

    /// Wait for a task to finish, emit progress, or reach the caller's maximum wait.
    ///
    /// This combines bus-driven wakeups with a light periodic status reconciliation so
    /// detached tasks, missed broadcast messages, or crash/reload edges still return no
    /// later than `max_wait` and can notice completion without active polling by the agent.
    pub async fn wait(
        &self,
        task_id: &str,
        max_wait: Duration,
        return_on_progress: bool,
    ) -> Option<BackgroundTaskWaitResult> {
        let mut bus_rx = Bus::global().subscribe();
        let initial = self.status(task_id).await?;
        if initial.status != BackgroundTaskStatus::Running {
            return Some(BackgroundTaskWaitResult {
                reason: BackgroundTaskWaitReason::AlreadyFinished,
                task: initial,
                progress_event: None,
                event_record: None,
            });
        }
        if max_wait.is_zero() {
            return Some(BackgroundTaskWaitResult {
                reason: BackgroundTaskWaitReason::Timeout,
                task: initial,
                progress_event: None,
                event_record: None,
            });
        }

        let mut last_progress = initial.progress.clone();
        let deadline = TokioInstant::now() + max_wait;
        let timeout = tokio::time::sleep_until(deadline);
        tokio::pin!(timeout);
        let mut poll = tokio::time::interval(Duration::from_secs(1));
        poll.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = &mut timeout => {
                    let task = self.status(task_id).await?;
                    let reason = if task.status == BackgroundTaskStatus::Running {
                        BackgroundTaskWaitReason::Timeout
                    } else {
                        BackgroundTaskWaitReason::Finished
                    };
                    return Some(BackgroundTaskWaitResult {
                        reason,
                        task,
                        progress_event: None,
                        event_record: None,
                    });
                }
                _ = poll.tick() => {
                    let task = self.status(task_id).await?;
                    if task.status != BackgroundTaskStatus::Running {
                        return Some(BackgroundTaskWaitResult {
                            reason: BackgroundTaskWaitReason::Finished,
                            task,
                            progress_event: None,
                            event_record: None,
                        });
                    }
                    if return_on_progress && task.progress != last_progress {
                        let event_record = task.event_history.last().cloned();
                        return Some(BackgroundTaskWaitResult {
                            reason: progress_wait_reason(event_record.as_ref()),
                            progress_event: None,
                            task,
                            event_record,
                        });
                    }
                    last_progress = task.progress.clone();
                }
                event = bus_rx.recv() => {
                    match event {
                        Ok(BusEvent::BackgroundTaskCompleted(event)) if event.task_id == task_id => {
                            let task = self.status(task_id).await?;
                            return Some(BackgroundTaskWaitResult {
                                reason: BackgroundTaskWaitReason::Finished,
                                task,
                                progress_event: None,
                                event_record: None,
                            });
                        }
                        Ok(BusEvent::BackgroundTaskProgress(event)) if event.task_id == task_id => {
                            if return_on_progress {
                                let task = self.status(task_id).await?;
                                let event_record = task.event_history.last().cloned();
                                return Some(BackgroundTaskWaitResult {
                                    reason: progress_wait_reason(event_record.as_ref()),
                                    task,
                                    progress_event: Some(event),
                                    event_record,
                                });
                            }
                        }
                        Ok(BusEvent::BackgroundTaskStalled(event)) if event.task_id == task_id => {
                            // A stall is always worth returning: the caller
                            // should decide whether to keep waiting or act.
                            let task = self.status(task_id).await?;
                            return Some(BackgroundTaskWaitResult {
                                reason: BackgroundTaskWaitReason::Stalled,
                                task,
                                progress_event: None,
                                event_record: None,
                            });
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            let task = self.status(task_id).await?;
                            if task.status != BackgroundTaskStatus::Running {
                                return Some(BackgroundTaskWaitResult {
                                    reason: BackgroundTaskWaitReason::Finished,
                                    task,
                                    progress_event: None,
                                    event_record: None,
                                });
                            }
                            if return_on_progress && task.progress != last_progress {
                                let event_record = task.event_history.last().cloned();
                                return Some(BackgroundTaskWaitResult {
                                    reason: progress_wait_reason(event_record.as_ref()),
                                    progress_event: None,
                                    task,
                                    event_record,
                                });
                            }
                            last_progress = task.progress.clone();
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            let task = self.status(task_id).await?;
                            let reason = if task.status == BackgroundTaskStatus::Running {
                                BackgroundTaskWaitReason::Timeout
                            } else {
                                BackgroundTaskWaitReason::Finished
                            };
                            return Some(BackgroundTaskWaitResult {
                                reason,
                                task,
                                progress_event: None,
                                event_record: None,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Update progress for an existing background task.
    pub async fn update_progress(
        &self,
        task_id: &str,
        progress: BackgroundTaskProgress,
    ) -> Result<Option<TaskStatusFile>> {
        self.update_progress_with_event_kind(task_id, progress, BackgroundTaskEventKind::Progress)
            .await
    }

    /// Record an explicit checkpoint for an existing background task.
    pub async fn update_checkpoint(
        &self,
        task_id: &str,
        progress: BackgroundTaskProgress,
    ) -> Result<Option<TaskStatusFile>> {
        self.update_progress_with_event_kind(task_id, progress, BackgroundTaskEventKind::Checkpoint)
            .await
    }

    async fn update_progress_with_event_kind(
        &self,
        task_id: &str,
        progress: BackgroundTaskProgress,
        event_kind: BackgroundTaskEventKind,
    ) -> Result<Option<TaskStatusFile>> {
        let _progress_update_guard = self.progress_updates.lock().await;
        // Detached finalization takes only `finalize_lock`; holding it for the
        // read-modify-write keeps this writer from restoring `Running` over a
        // terminal status that was written in between (a second completion).
        let _finalizing = self.finalize_lock.lock().await;
        let status_path = self.status_path_for(task_id);
        let Some(mut status) = self.read_status_file(&status_path).await else {
            return Ok(None);
        };
        if status.status != BackgroundTaskStatus::Running {
            return Ok(Some(status));
        }

        let mut progress = progress.normalize();
        if let Some(existing) = status.progress.as_ref() {
            if progress_equivalent(existing, &progress) {
                return Ok(Some(status));
            }

            // A determinate bar represents progress through the whole task. A
            // lower later value would make that representation knowingly
            // inconsistent, regardless of whether it was explicitly reported
            // or inferred from output. Keep the last trustworthy high-water
            // mark instead of allowing the UI to move backwards.
            if let (Some(existing_percent), Some(new_percent)) =
                (existing.percent, progress.percent)
                && new_percent < existing_percent
            {
                return Ok(Some(status));
            }

            let existing_is_more_determinate = existing.percent.is_some()
                || matches!((existing.current, existing.total), (_, Some(total)) if total > 0);
            let new_is_less_determinate = progress.percent.is_none()
                && !matches!((progress.current, progress.total), (_, Some(total)) if total > 0);
            if existing_is_more_determinate && new_is_less_determinate {
                // Preserve the measurable high-water mark while still allowing
                // a new checkpoint/phase message to reach status and clients.
                progress.kind = existing.kind.clone();
                progress.percent = existing.percent;
                progress.current = existing.current;
                progress.total = existing.total;
                if progress.unit.is_none() {
                    progress.unit.clone_from(&existing.unit);
                }
            }
        }

        status.progress = Some(progress.clone());
        push_task_event(
            &mut status,
            progress_event_record(event_kind, progress.clone()),
        );
        self.write_status_file(&status_path, &status).await;

        Bus::global().publish(BusEvent::BackgroundTaskProgress(
            BackgroundTaskProgressEvent {
                task_id: status.task_id.clone(),
                tool_name: status.tool_name.clone(),
                display_name: status.display_name.clone(),
                session_id: status.session_id.clone(),
                progress,
            },
        ));

        Ok(Some(status))
    }

    /// How often a stall watchdog samples output size and progress state.
    const STALL_POLL_INTERVAL: Duration = Duration::from_secs(5);
    /// Minimum accepted stall window, to keep watchdogs from spamming wakes.
    pub const MIN_STALL_WAKE_SECONDS: u64 = 30;

    /// Update delivery behavior for an existing background task.
    ///
    /// This supports retroactively enabling notify/wake after the task was already started.
    pub async fn update_delivery(
        &self,
        task_id: &str,
        notify: bool,
        wake: bool,
    ) -> Result<Option<TaskStatusFile>> {
        let _status_guard = self.progress_updates.lock().await;
        let _finalizing = self.finalize_lock.lock().await;
        let (notify, wake) = normalize_delivery(notify, wake);
        let status_path = self.status_path_for(task_id);
        let Some(mut status) = self.read_status_file(&status_path).await else {
            return Ok(None);
        };
        if status.status != BackgroundTaskStatus::Running {
            return Ok(Some(status));
        }
        status.notify = notify;
        status.wake = wake;
        let event_status = status.status.clone();
        let event_exit_code = status.exit_code;
        let event_progress = status.progress.clone();
        push_task_event(
            &mut status,
            BackgroundTaskEventRecord {
                kind: BackgroundTaskEventKind::DeliveryUpdated,
                timestamp: Utc::now().to_rfc3339(),
                message: Some(format!("notify={}, wake={}", notify, wake)),
                status: Some(event_status),
                exit_code: event_exit_code,
                progress: event_progress,
            },
        );
        self.write_status_file(&status_path, &status).await;

        if let Some(task) = self.tasks.read().await.get(task_id) {
            let _ = task.delivery_flags.send((notify, wake));
        }

        Ok(Some(status))
    }

    /// Cancel a running task
    pub async fn cancel(&self, task_id: &str) -> Result<bool> {
        self.cancel_with_grace(task_id, std::time::Duration::from_millis(400))
            .await
    }

    /// Cancel a running task, allowing detached processes a configurable grace period
    /// between TERM and KILL on Unix.
    pub async fn cancel_with_grace(
        &self,
        task_id: &str,
        _graceful_timeout: std::time::Duration,
    ) -> Result<bool> {
        // Never hold the map or status lock while awaiting the task: completion
        // needs both locks before it can exit.
        let task = self.tasks.write().await.remove(task_id);
        if let Some(mut task) = task {
            task.handle.abort();
            let _ = (&mut task.handle).await;
            let _status_guard = self.progress_updates.lock().await;

            // Update status file
            let (notify_flag, wake_flag) = *task.delivery_flags.borrow();
            let mut final_status = TaskStatusFile {
                task_id: task.task_id,
                tool_name: task.tool_name,
                display_name: task.display_name,
                session_id: task.session_id,
                status: BackgroundTaskStatus::Failed,
                exit_code: None,
                error: Some("Cancelled by user".to_string()),
                started_at: task.started_at_rfc3339,
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
                duration_secs: Some(task.started_at.elapsed().as_secs_f64()),
                pid: None,
                owner_pid: Some(std::process::id()),
                owner_instance: Some(model::process_instance_token().to_string()),
                detached: false,
                notify: notify_flag,
                wake: wake_flag,
                progress: None,
                event_history: Vec::new(),
                pid_start_time: None,
                owner_generation: task.owner_generation,
                stall_wake_seconds: None,
                mcp_task: None,
            };
            let event_status = final_status.status.clone();
            let event_exit_code = final_status.exit_code;
            let event_error = final_status.error.clone();
            push_task_event(
                &mut final_status,
                terminal_event_record(event_status, event_exit_code, event_error.as_deref()),
            );
            if let Ok(json) = serde_json::to_string_pretty(&final_status) {
                let _ = fs::write(&task.status_path, json).await;
            }

            Ok(true)
        } else {
            let _status_guard = self.progress_updates.lock().await;

            let status_path = self.status_path_for(task_id);
            let Some(mut status) = self.read_status_file(&status_path).await else {
                return Ok(false);
            };
            // A dead or pid-reused process is finalized here, so the signals
            // below can never reach an unrelated process that took the pid.
            status = self
                .finalize_detached_status_if_needed(status, &status_path)
                .await;
            if status.status != BackgroundTaskStatus::Running || !status.detached {
                return Ok(false);
            }

            let Some(pid) = status.pid else {
                return Ok(false);
            };

            // Hold the finalization lock from kill to terminal write so a
            // concurrent reconciler cannot publish its own outcome for the
            // process this call is about to kill, then lose to this write.
            let _finalizing = self.finalize_lock.lock().await;
            match self.read_status_file(&status_path).await {
                Some(fresh) if fresh.status == BackgroundTaskStatus::Running => status = fresh,
                _ => return Ok(false),
            }

            #[cfg(unix)]
            {
                let _ = crate::platform::signal_detached_process_group(pid, libc::SIGTERM);
                tokio::time::sleep(_graceful_timeout).await;
                if Self::detached_process_identity(&status) == ProcessIdentity::Alive {
                    let _ = crate::platform::signal_detached_process_group(pid, libc::SIGKILL);
                }
            }
            #[cfg(windows)]
            {
                let _ = crate::platform::signal_detached_process_group(pid, 0);
            }

            let completed_at = Utc::now();
            status.status = BackgroundTaskStatus::Failed;
            status.exit_code = None;
            status.error = Some("Cancelled by user".to_string());
            status.completed_at = Some(completed_at.to_rfc3339());
            status.duration_secs = Self::status_duration_secs(&status.started_at, completed_at);
            let event_status = status.status.clone();
            let event_exit_code = status.exit_code;
            let event_error = status.error.clone();
            push_task_event(
                &mut status,
                terminal_event_record(event_status, event_exit_code, event_error.as_deref()),
            );
            self.write_status_file(&status_path, &status).await;
            Ok(true)
        }
    }

    /// Abort every live in-process task before an exec-based server reload.
    ///
    /// `exec` replaces the process image without running destructors, so
    /// without this the spawned task futures simply vanish: their
    /// `kill_on_drop` children (e.g. cargo builds) are never killed and keep
    /// running orphaned, and their status files stay `Running` until the next
    /// process's reconcile sweep happens to notice. Aborting the handles here
    /// drops the futures (killing children) and persisting a terminal
    /// `Failed` status makes the interruption deterministic and immediately
    /// visible to `bg wait`/`bg status` and self-dev queue reconciliation.
    ///
    /// Returns the number of tasks finalized.
    pub async fn abort_live_tasks_for_reload(&self) -> usize {
        let tasks: Vec<RunningTask> = {
            let mut map = self.tasks.write().await;
            map.drain().map(|(_, task)| task).collect()
        };
        let mut finalized = 0;

        for task in tasks {
            task.handle.abort();
            // Wait (bounded) for the aborted future to actually drop, so
            // kill_on_drop children are killed before the upcoming exec.
            let _ = tokio::time::timeout(Duration::from_secs(2), task.handle).await;

            let (notify_flag, wake_flag) = *task.delivery_flags.borrow();
            let prior_status = self.read_status_file(&task.status_path).await;
            // If the task won the race and finished naturally, keep its real
            // terminal status instead of stamping it as interrupted.
            if prior_status
                .as_ref()
                .is_some_and(|status| status.status != BackgroundTaskStatus::Running)
            {
                continue;
            }
            let error = "Interrupted by server reload: the owning server process was replaced before the task finished".to_string();
            let mut final_status = TaskStatusFile {
                task_id: task.task_id,
                tool_name: task.tool_name,
                display_name: prior_status
                    .as_ref()
                    .and_then(|status| status.display_name.clone())
                    .or(task.display_name),
                session_id: task.session_id,
                status: BackgroundTaskStatus::Failed,
                exit_code: None,
                error: Some(error.clone()),
                started_at: task.started_at_rfc3339,
                completed_at: Some(chrono::Utc::now().to_rfc3339()),
                duration_secs: Some(task.started_at.elapsed().as_secs_f64()),
                pid: None,
                owner_pid: Some(std::process::id()),
                owner_instance: Some(model::process_instance_token().to_string()),
                detached: false,
                notify: notify_flag,
                wake: wake_flag,
                progress: prior_status
                    .as_ref()
                    .and_then(|status| status.progress.clone()),
                event_history: prior_status
                    .map(|status| status.event_history)
                    .unwrap_or_default(),
                pid_start_time: None,
                owner_generation: task.owner_generation,
                stall_wake_seconds: None,
                mcp_task: None,
            };
            push_task_event(
                &mut final_status,
                terminal_event_record(BackgroundTaskStatus::Failed, None, Some(&error)),
            );
            self.write_status_file(&task.status_path, &final_status)
                .await;
            finalized += 1;
        }

        finalized
    }

    /// Best-effort synchronous snapshot of currently running tasks.
    /// This avoids async calls in render paths.
    pub fn running_snapshot(&self) -> (usize, Vec<String>, Option<RunningBackgroundProgress>) {
        let Ok(tasks) = self.tasks.try_read() else {
            return (0, Vec::new(), None);
        };

        let mut rows: Vec<RunningBackgroundProgress> = Vec::new();
        for task in tasks.values() {
            let status = std::fs::read_to_string(&task.status_path)
                .ok()
                .and_then(|content| serde_json::from_str::<TaskStatusFile>(&content).ok());
            let progress = status.as_ref().and_then(|status| status.progress.clone());
            let label = status
                .as_ref()
                .and_then(|status| status.display_name.clone())
                .or_else(|| task.display_name.clone())
                .unwrap_or_else(|| task.tool_name.clone());

            rows.push(RunningBackgroundProgress {
                task_id: task.task_id.clone(),
                tool_name: task.tool_name.clone(),
                label,
                detail: progress.map(|progress| format_progress_display(&progress, 10)),
            });
        }

        rows.sort_by(|a, b| b.task_id.cmp(&a.task_id));
        let latest = rows.iter().find(|row| row.detail.is_some()).cloned();

        (
            tasks.len(),
            rows.iter().map(|row| row.label.clone()).collect(),
            latest,
        )
    }

    /// Best-effort synchronous lookup of detached tasks that are still running
    /// for a specific session.
    ///
    /// This is primarily used during self-dev reload recovery, where the new
    /// process needs to remind the agent that a previous `bash` command was
    /// persisted into the background instead of being interrupted.
    pub fn persisted_detached_running_tasks_for_session(
        &self,
        session_id: &str,
    ) -> Vec<TaskStatusFile> {
        // The task directory accumulates thousands of status files and every
        // History bootstrap asks this question, so a reconnect storm re-read,
        // re-parsed, and re-stat'ed all of them per client. Keep one index of
        // running detached tasks, rebuilt only when the directory changes
        // (its mtime moves when a status file is created, renamed, or removed;
        // status files are replaced atomically) or after a short TTL.
        struct Index {
            dir_mtime: Option<std::time::SystemTime>,
            built_at: Instant,
            running: Vec<TaskStatusFile>,
        }
        static INDEX: std::sync::LazyLock<std::sync::Mutex<Option<Index>>> =
            std::sync::LazyLock::new(Default::default);
        // New tasks create a status file (moving the dir mtime), and tasks that
        // finished are dropped below by the pid liveness filter, so the TTL
        // only bounds exotic in-place edits. Keep it long enough that the index
        // prewarmed at server start covers a whole reconnect storm.
        const TTL: std::time::Duration = std::time::Duration::from_secs(30);

        let dir_mtime = std::fs::metadata(&self.output_dir)
            .and_then(|meta| meta.modified())
            .ok();
        let mut guard = INDEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let fresh = guard.as_ref().is_some_and(|index| {
            index.dir_mtime.is_some()
                && index.dir_mtime == dir_mtime
                && index.built_at.elapsed() < TTL
        });
        if !fresh {
            let mut running = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&self.output_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                        continue;
                    }
                    let Some(status) = std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|content| serde_json::from_str::<TaskStatusFile>(&content).ok())
                    else {
                        continue;
                    };
                    if status.status == BackgroundTaskStatus::Running && status.detached {
                        running.push(status);
                    }
                }
            }
            *guard = Some(Index {
                dir_mtime,
                built_at: Instant::now(),
                running,
            });
        }
        let mut matches: Vec<TaskStatusFile> = guard
            .as_ref()
            .map(|index| {
                index
                    .running
                    .iter()
                    .filter(|status| status.session_id == session_id)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        drop(guard);
        matches.retain(|status| Self::detached_process_identity(status) == ProcessIdentity::Alive);

        matches.sort_by(|a, b| a.task_id.cmp(&b.task_id));
        matches
    }
}

impl Default for BackgroundTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global singleton for background task manager
static BACKGROUND_MANAGER: std::sync::OnceLock<Arc<BackgroundTaskManager>> =
    std::sync::OnceLock::new();

/// Get the global background task manager
pub fn global() -> &'static BackgroundTaskManager {
    global_arc()
}

/// The global manager as a shared handle, for owners that outlive a borrow
/// (for example the MCP task tracker's polling tasks).
pub fn global_arc() -> &'static Arc<BackgroundTaskManager> {
    BACKGROUND_MANAGER.get_or_init(|| Arc::new(BackgroundTaskManager::new()))
}

/// Write `contents` to `path` through a temp file and rename, so a crash
/// mid-write never leaves a torn record. The MCP task record is the only copy
/// of an out-of-process job's remote id.
async fn write_atomic(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = std::path::PathBuf::from(tmp);
    fs::write(&tmp, contents).await?;
    fs::rename(&tmp, path).await
}

#[cfg(test)]
mod tests;

#[path = "background/cleanup.rs"]
mod cleanup;
