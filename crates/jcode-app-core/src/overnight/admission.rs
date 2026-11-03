use super::*;
use std::fs::{File, OpenOptions};

/// Authority/accounting state must never recover an older backup implicitly.
/// Unlike generic storage reads this is read-only and fails on corrupt primary.
pub(super) fn read_manifest_at(path: &Path) -> Result<OvernightManifest> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("Cannot read current overnight manifest {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("Corrupt current overnight manifest {}; automatic backup recovery is forbidden for authority/accounting state", path.display()))
}

/// A held OS lock, not execution authority. Dispatch remains denied until a
/// provider can reserve an authoritative monetary upper bound before sending.
pub struct TaskAttemptClaim {
    _owner_lock: File,
    decision: crate::config::AdmissionDecision,
}

impl TaskAttemptClaim {
    pub fn dispatch_decision(&self) -> &crate::config::AdmissionDecision {
        &self.decision
    }
}

fn lock(path: &Path) -> Result<File> {
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        anyhow::ensure!(
            meta.is_file() && !meta.file_type().is_symlink(),
            "Unsafe overnight lock path"
        );
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path)?;
    file.try_lock().map_err(|_| {
        anyhow::anyhow!("Overnight state/attempt is owned by a live operation; retry later")
    })?;
    Ok(file)
}

/// Serialize every writer on a stable sidecar, because atomic JSON replacement
/// changes the manifest inode. Never refund counters, restore cancellation, or
/// mutate the original authorization through an unrelated stale save.
pub(super) fn save_manifest_at(path: &Path, proposed: &OvernightManifest) -> Result<()> {
    let parent = path.parent().context("Manifest has no parent")?;
    std::fs::create_dir_all(parent)?;
    let _transaction = lock(&path.with_extension("lock"))?;
    let mut merged = proposed.clone();
    match read_manifest_at(path) {
        Ok(current) => {
            anyhow::ensure!(
                current.run_id == proposed.run_id,
                "Manifest run identity mismatch"
            );
            if let Some(original) = &current.task_admission {
                let updated = merged
                    .task_admission
                    .as_mut()
                    .context("Cannot remove persisted task authority")?;
                anyhow::ensure!(
                    original.authorization == updated.authorization
                        && original.scope == updated.scope,
                    "Cannot widen or change original persisted task authorization"
                );
                anyhow::ensure!(
                    current.working_dir == merged.working_dir,
                    "Cannot change authorized task workspace"
                );
                anyhow::ensure!(
                    current.started_at == merged.started_at,
                    "Cannot reset task runtime origin"
                );
                updated.attempts = updated.attempts.max(original.attempts);
                updated.runtime_seconds = updated.runtime_seconds.max(original.runtime_seconds);
                updated.spent_microusd = updated.spent_microusd.max(original.spent_microusd);
                updated.cancelled |= original.cancelled;
                updated.dependencies_ready &= original.dependencies_ready;
                updated.workspace_ready &= original.workspace_ready;
                updated.ownership_verified &= original.ownership_verified;
                updated.setup_required |= original.setup_required;
                updated.deadline_expired |= original.deadline_expired;
                merged.task_attempt = current.task_attempt;
            } else {
                anyhow::ensure!(
                    merged.task_admission.is_none(),
                    "Legacy manual run cannot acquire task authority through save"
                );
            }
            if current.cancel_requested_at.is_some() {
                merged.cancel_requested_at = current.cancel_requested_at;
                if !matches!(
                    merged.status,
                    OvernightRunStatus::Completed | OvernightRunStatus::Failed
                ) {
                    merged.status = current.status.clone();
                }
            }
            if matches!(
                current.status,
                OvernightRunStatus::Completed | OvernightRunStatus::Failed
            ) {
                merged.status = current.status.clone();
                merged.completed_at = current.completed_at;
            }
        }
        Err(error) if !path.try_exists()? => {
            let _ = error;
        }
        Err(error) => return Err(error).context("Cannot replace unreadable overnight manifest"),
    }
    storage::write_json(path, &merged)
}

/// Explicit native admission entry point. Requires a pre-existing user-authorized
/// manifest, never derives authority from mission text or a deadline.
pub fn start_task_attempt(
    path: &Path,
    policy: &crate::config::TasksConfig,
    scope: &str,
) -> Result<TaskAttemptClaim> {
    validate_runtime_manifest(path)?;
    let current = current_policy_ceiling(policy)?;
    claim_task_attempt(path, &current, scope, false)
}

/// Explicit restart only, no scan/activation. An absent legacy snapshot fails
/// closed, original consumed attempts and narrower authority survive restart.
pub fn restart_task_attempt(
    path: &Path,
    policy: &crate::config::TasksConfig,
    scope: &str,
) -> Result<TaskAttemptClaim> {
    validate_runtime_manifest(path)?;
    let current = current_policy_ceiling(policy)?;
    claim_task_attempt(path, &current, scope, true)
}

fn current_policy_ceiling(
    requested: &crate::config::TasksConfig,
) -> Result<crate::config::TasksConfig> {
    let current = crate::config::Config::load_strict()?.tasks;
    Ok(crate::config::TasksConfig {
        unattended: current.unattended && requested.unattended,
        max_concurrent: current.max_concurrent.min(requested.max_concurrent),
        max_attempts: current.max_attempts.min(requested.max_attempts),
        max_runtime_seconds: current
            .max_runtime_seconds
            .min(requested.max_runtime_seconds),
        max_spend_microusd: current.max_spend_microusd.min(requested.max_spend_microusd),
    })
}

fn validate_runtime_manifest(path: &Path) -> Result<()> {
    let root = runs_dir()?
        .canonicalize()
        .context("Overnight runs root must exist")?;
    anyhow::ensure!(
        path.file_name().is_some_and(|name| name == "manifest.json")
            && path.parent().and_then(Path::parent) == Some(root.as_path()),
        "Task manifest is outside the trusted overnight runtime root"
    );
    let directory = std::fs::symlink_metadata(path.parent().context("No run directory")?)?;
    let file = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        directory.is_dir()
            && !directory.file_type().is_symlink()
            && file.is_file()
            && !file.file_type().is_symlink(),
        "Task manifest/run directory must not be symlinks"
    );
    let manifest = read_manifest_at(path)?;
    anyhow::ensure!(
        !manifest.run_id.is_empty()
            && manifest
                .run_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_')),
        "Invalid overnight run identity"
    );
    let expected = root.join(&manifest.run_id).join("manifest.json");
    anyhow::ensure!(
        path == expected,
        "Task manifest is outside the trusted overnight runtime root"
    );
    Ok(())
}

fn claim_task_attempt(
    path: &Path,
    policy: &crate::config::TasksConfig,
    scope: &str,
    restart: bool,
) -> Result<TaskAttemptClaim> {
    policy.validate().map_err(anyhow::Error::msg)?;
    let metadata = std::fs::symlink_metadata(path).context("Task manifest must already exist")?;
    anyhow::ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "Task manifest must be a real file"
    );
    let path = path
        .canonicalize()
        .context("Task manifest must already exist")?;
    let parent = path.parent().context("Manifest has no parent")?;
    // Stable coordinator lock, not a planning/authorization ledger. One admission
    // at a time in this runtime root makes capacity observation + claim atomic.
    let root = parent.parent().context("Manifest has no runtime root")?;
    let _capacity = lock(&root.join(".task-admission.lock"))?;
    let _transaction = lock(&path.with_extension("lock"))?;
    let owner_lock = lock(&path.with_extension("attempt.lock"))?;
    let mut manifest = read_manifest_at(&path)?;
    let snapshot = manifest
        .task_admission
        .as_ref()
        .context("Persisted task authorization is missing")?;
    anyhow::ensure!(
        snapshot.scope == scope,
        "Requested scope differs from original task scope"
    );
    anyhow::ensure!(
        restart == manifest.task_attempt.is_some(),
        "Use restart for an existing attempt; start requires no prior attempt"
    );
    let workspace = Path::new(
        manifest
            .working_dir
            .as_deref()
            .context("Task workspace is unknown")?,
    )
    .canonicalize()?;
    if let Some(previous) = &manifest.task_attempt {
        anyhow::ensure!(
            previous.workspace == workspace,
            "Task workspace identity changed since original admission"
        );
    }
    let mut active = 0u32;
    // Production paths are overnight/runs/<run>/manifest.json. Fixtures can use
    // direct files too; exclude this candidate and never create other manifests.
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        anyhow::ensure!(
            !entry.file_type()?.is_symlink(),
            "Unsafe symlink in overnight runs root"
        );
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let candidate = entry.path().join("manifest.json");
        if let Ok(metadata) = std::fs::symlink_metadata(&candidate) {
            anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "Unsafe symlink task manifest"
            );
        }
        if candidate == path || !candidate.is_file() {
            continue;
        }
        let other = read_manifest_at(&candidate)?;
        if other.task_attempt.is_some() && lock(&candidate.with_extension("attempt.lock")).is_err()
        {
            active = active.saturating_add(1);
        }
    }
    let mut observed = snapshot.clone();
    observed.active_attempts = observed.active_attempts.max(active);
    observed.live_attempt = None; // OS ownership lock, not a PID, establishes no live attempt.
    observed.cancelled |= manifest.cancel_requested_at.is_some()
        || !matches!(manifest.status, OvernightRunStatus::Running);
    observed.workspace_ready &= workspace.is_dir();
    let elapsed = Utc::now()
        .signed_duration_since(manifest.started_at)
        .num_seconds()
        .max(0) as u64;
    observed.runtime_seconds = observed.runtime_seconds.max(elapsed);
    let mut decision = policy.admit(&observed);
    anyhow::ensure!(
        decision.state == crate::config::AdmissionState::Ready,
        "Task admission denied: {}",
        decision.reasons.join("; ")
    );
    let snapshot = manifest
        .task_admission
        .as_mut()
        .expect("validated snapshot");
    snapshot.attempts = snapshot
        .attempts
        .checked_add(1)
        .context("Task attempt counter overflow")?;
    snapshot.runtime_seconds = observed.runtime_seconds;
    snapshot.live_attempt = None;
    // Token usage and public price estimates do not reserve provider billing.
    // Persist a precise denial before touching sessions, providers or tool setup.
    decision.state = crate::config::AdmissionState::Blocked;
    decision.reasons = vec!["budget-unverifiable: provider lacks authoritative pre-send monetary bound; no provider/session effects dispatched".into()];
    manifest.process_id = std::process::id();
    manifest.task_attempt = Some(jcode_overnight_core::OvernightTaskAttempt {
        generation: snapshot.attempts,
        owner: crate::id::new_id("task-attempt"),
        process_id: std::process::id(),
        claimed_at: Utc::now(),
        workspace,
        decision: decision.clone(),
    });
    storage::write_json(&path, &manifest)?;
    Ok(TaskAttemptClaim {
        _owner_lock: owner_lock,
        decision,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};

    fn start_task_attempt(
        path: &Path,
        policy: &crate::config::TasksConfig,
        scope: &str,
    ) -> Result<TaskAttemptClaim> {
        super::claim_task_attempt(path, policy, scope, false)
    }

    fn restart_task_attempt(
        path: &Path,
        policy: &crate::config::TasksConfig,
        scope: &str,
    ) -> Result<TaskAttemptClaim> {
        super::claim_task_attempt(path, policy, scope, true)
    }

    #[test]
    fn task_admission_atomic_public_boundary_rejects_external_path_and_accepts_sandbox_root() {
        let _env = storage::lock_test_env();
        let (temp, path, policy) = fixture();
        let original = std::env::var_os("JCODE_HOME");
        crate::env::set_var("JCODE_HOME", temp.path());
        std::fs::write(
            temp.path().join("config.toml"),
            "[tasks]\nunattended = true\n",
        )
        .unwrap();
        assert!(super::start_task_attempt(&path, &policy, "bounded").is_err());
        let directory = temp.path().join("overnight/runs/atomic");
        std::fs::create_dir_all(&directory).unwrap();
        let trusted = directory.join("manifest.json");
        std::fs::copy(&path, &trusted).unwrap();
        let claim = super::start_task_attempt(&trusted, &policy, "bounded").unwrap();
        assert_eq!(
            claim.dispatch_decision().state,
            crate::config::AdmissionState::Blocked
        );
        assert!(super::restart_task_attempt(&trusted, &policy, "bounded").is_err());
        drop(claim);
        drop(super::restart_task_attempt(&trusted, &policy, "bounded").unwrap());
        std::fs::write(
            temp.path().join("config.toml"),
            "[tasks]\nunattended = false\n",
        )
        .unwrap();
        assert!(super::restart_task_attempt(&trusted, &policy, "bounded").is_err());
        let corrupt = b"{corrupt-latest-authority";
        let backup = std::fs::read(trusted.with_extension("bak")).unwrap();
        std::fs::write(&trusted, corrupt).unwrap();
        assert!(super::super::load_manifest("atomic").is_err());
        assert!(super::super::latest_manifest().is_err());
        assert!(super::super::cancel_latest_run().is_err());
        assert_eq!(std::fs::read(&trusted).unwrap(), corrupt);
        assert_eq!(
            std::fs::read(trusted.with_extension("bak")).unwrap(),
            backup
        );
        if let Some(value) = original {
            crate::env::set_var("JCODE_HOME", value);
        } else {
            crate::env::remove_var("JCODE_HOME");
        }
    }

    fn fixture() -> (tempfile::TempDir, PathBuf, crate::config::TasksConfig) {
        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("runs/atomic");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("manifest.json");
        let mut manifest = super::super::tests::test_manifest(temp.path(), "atomic");
        manifest.working_dir = Some(temp.path().canonicalize().unwrap().to_str().unwrap().into());
        manifest.task_admission = Some(crate::config::TaskAdmissionSnapshot {
            scope: "bounded".into(),
            authorization: Some(crate::config::TaskAuthorization {
                scope: "bounded".into(),
                max_attempts: 3,
                max_runtime_seconds: 300,
                max_spend_microusd: 100,
            }),
            ownership_verified: true,
            dependencies_ready: true,
            workspace_ready: true,
            ..Default::default()
        });
        storage::write_json(&path, &manifest).unwrap();
        (
            temp,
            path,
            crate::config::TasksConfig {
                unattended: true,
                ..Default::default()
            },
        )
    }

    #[test]
    fn task_admission_atomic_start_restart_preserves_authority_and_denies_unknown_spend() {
        let (_temp, path, mut policy) = fixture();
        let first = start_task_attempt(&path, &policy, "bounded").unwrap();
        assert!(
            first
                .dispatch_decision()
                .reasons
                .iter()
                .any(|r| r.contains("budget-unverifiable"))
        );
        assert_eq!(
            first.dispatch_decision().state,
            crate::config::AdmissionState::Blocked
        );
        assert!(restart_task_attempt(&path, &policy, "bounded").is_err());
        let saved: OvernightManifest = storage::read_json(&path).unwrap();
        assert_eq!(saved.task_admission.as_ref().unwrap().attempts, 1);
        assert!(
            saved
                .task_attempt
                .as_ref()
                .unwrap()
                .decision
                .reasons
                .iter()
                .any(|r| r.contains("budget-unverifiable"))
        );
        drop(first);
        policy.max_attempts = 100;
        let second = restart_task_attempt(&path, &policy, "bounded").unwrap();
        let saved: OvernightManifest = storage::read_json(&path).unwrap();
        assert_eq!(saved.task_admission.as_ref().unwrap().attempts, 2);
        assert_eq!(
            saved
                .task_admission
                .as_ref()
                .unwrap()
                .authorization
                .as_ref()
                .unwrap()
                .max_attempts,
            3
        );
        drop(second);
        assert!(restart_task_attempt(&path, &policy, "expanded").is_err());
    }

    #[test]
    fn task_admission_atomic_race_has_one_live_owner() {
        let (_temp, path, policy) = fixture();
        let barrier = Arc::new(Barrier::new(3));
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let (path, policy, barrier) = (path.clone(), policy.clone(), barrier.clone());
                std::thread::spawn(move || {
                    barrier.wait();
                    start_task_attempt(&path, &policy, "bounded")
                })
            })
            .collect();
        barrier.wait();
        let claims: Vec<_> = workers.into_iter().map(|t| t.join().unwrap()).collect();
        assert_eq!(claims.iter().filter(|c| c.is_ok()).count(), 1);
        let saved: OvernightManifest = storage::read_json(&path).unwrap();
        assert_eq!(saved.task_admission.unwrap().attempts, 1);
    }

    #[test]
    fn task_admission_atomic_child_process_probe() {
        let Some(path) = std::env::var_os("JCODE_TEST_TASK_ADMISSION_MANIFEST") else {
            return;
        };
        let policy = crate::config::TasksConfig {
            unattended: true,
            ..Default::default()
        };
        assert!(restart_task_attempt(Path::new(&path), &policy, "bounded").is_err());
    }

    #[test]
    fn task_admission_atomic_live_owner_blocks_another_process_and_attempts_are_finite() {
        let (_temp, path, policy) = fixture();
        let claim = start_task_attempt(&path, &policy, "bounded").unwrap();
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "overnight::admission::tests::task_admission_atomic_child_process_probe",
            ])
            .env("JCODE_TEST_TASK_ADMISSION_MANIFEST", &path)
            .output()
            .unwrap();
        assert!(
            child.status.success(),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
        drop(claim);
        drop(restart_task_attempt(&path, &policy, "bounded").unwrap());
        drop(restart_task_attempt(&path, &policy, "bounded").unwrap());
        assert!(restart_task_attempt(&path, &policy, "bounded").is_err());
        let saved: OvernightManifest = storage::read_json(&path).unwrap();
        assert_eq!(saved.task_admission.unwrap().attempts, 3);
    }

    #[test]
    fn task_admission_atomic_cancel_and_stale_save_cannot_refund_or_widen() {
        let (_temp, path, policy) = fixture();
        let mut stale: OvernightManifest = storage::read_json(&path).unwrap();
        let claim = start_task_attempt(&path, &policy, "bounded").unwrap();
        drop(claim);
        let mut cancelled: OvernightManifest = storage::read_json(&path).unwrap();
        cancelled.cancel_requested_at = Some(Utc::now());
        cancelled.status = OvernightRunStatus::CancelRequested;
        save_manifest_at(&path, &cancelled).unwrap();
        save_manifest_at(&path, &stale).unwrap();
        let saved: OvernightManifest = storage::read_json(&path).unwrap();
        assert!(saved.cancel_requested_at.is_some());
        assert_eq!(saved.task_admission.as_ref().unwrap().attempts, 1);
        assert!(restart_task_attempt(&path, &policy, "bounded").is_err());
        stale
            .task_admission
            .as_mut()
            .unwrap()
            .authorization
            .as_mut()
            .unwrap()
            .max_spend_microusd = 999;
        assert!(save_manifest_at(&path, &stale).is_err());
    }

    #[test]
    fn task_admission_atomic_capacity_and_current_policy_narrowing() {
        let (temp, path, mut policy) = fixture();
        let claim = start_task_attempt(&path, &policy, "bounded").unwrap();
        let sibling = temp.path().join("runs/other/manifest.json");
        std::fs::create_dir_all(sibling.parent().unwrap()).unwrap();
        let mut other = super::super::tests::test_manifest(temp.path(), "other");
        let saved: OvernightManifest = storage::read_json(&path).unwrap();
        other.task_admission = saved.task_admission;
        other.working_dir = saved.working_dir;
        storage::write_json(&sibling, &other).unwrap();
        assert!(start_task_attempt(&sibling, &policy, "bounded").is_err());
        drop(claim);
        policy.max_attempts = 1;
        assert!(restart_task_attempt(&path, &policy, "bounded").is_err());
    }

    #[test]
    fn task_admission_atomic_completion_after_cancel_stays_terminal() {
        let (_temp, path, _policy) = fixture();
        let mut manifest: OvernightManifest = storage::read_json(&path).unwrap();
        manifest.status = OvernightRunStatus::CancelRequested;
        manifest.cancel_requested_at = Some(Utc::now());
        save_manifest_at(&path, &manifest).unwrap();
        manifest.status = OvernightRunStatus::Completed;
        manifest.completed_at = Some(Utc::now());
        save_manifest_at(&path, &manifest).unwrap();
        let saved: OvernightManifest = storage::read_json(&path).unwrap();
        assert!(matches!(saved.status, OvernightRunStatus::Completed));
        assert!(saved.cancel_requested_at.is_some());
    }

    #[test]
    fn task_admission_atomic_corrupt_primary_never_restores_older_backup() {
        let (_temp, path, policy) = fixture();
        let stale: OvernightManifest = storage::read_json(&path).unwrap();
        drop(start_task_attempt(&path, &policy, "bounded").unwrap());
        drop(restart_task_attempt(&path, &policy, "bounded").unwrap());
        let backup = std::fs::read(path.with_extension("bak")).unwrap();
        let older: OvernightManifest = serde_json::from_slice(&backup).unwrap();
        assert!(older.task_admission.unwrap().attempts < 2);
        std::fs::write(&path, b"{corrupt-current-N").unwrap();
        assert!(restart_task_attempt(&path, &policy, "bounded").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{corrupt-current-N");
        let mut cancelled = stale;
        cancelled.cancel_requested_at = Some(Utc::now());
        cancelled.status = OvernightRunStatus::CancelRequested;
        assert!(save_manifest_at(&path, &cancelled).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"{corrupt-current-N");
        assert_eq!(std::fs::read(path.with_extension("bak")).unwrap(), backup);
    }

    #[test]
    fn task_admission_atomic_corrupt_capacity_candidate_is_not_recovered() {
        let (temp, path, policy) = fixture();
        let candidate = temp.path().join("runs/corrupt/manifest.json");
        std::fs::create_dir_all(candidate.parent().unwrap()).unwrap();
        std::fs::write(&candidate, b"{corrupt-candidate").unwrap();
        let original: OvernightManifest = storage::read_json(&path).unwrap();
        storage::write_json(&candidate.with_extension("bak"), &original).unwrap();
        assert!(start_task_attempt(&path, &policy, "bounded").is_err());
        assert_eq!(std::fs::read(candidate).unwrap(), b"{corrupt-candidate");
    }

    #[test]
    fn task_admission_atomic_rejects_dependencies_exhaustion_legacy_and_workspace_drift() {
        for case in [
            "dependencies",
            "attempts",
            "legacy",
            "workspace",
            "scope",
            "disabled",
        ] {
            let (_temp, path, mut policy) = fixture();
            let mut manifest: OvernightManifest = storage::read_json(&path).unwrap();
            match case {
                "dependencies" => {
                    manifest.task_admission.as_mut().unwrap().dependencies_ready = false
                }
                "attempts" => manifest.task_admission.as_mut().unwrap().attempts = 3,
                "legacy" => manifest.task_admission = None,
                "workspace" => manifest.working_dir = Some("/nonexistent-workspace".into()),
                "scope" => manifest.task_admission.as_mut().unwrap().scope = "expanded".into(),
                _ => policy.unattended = false,
            }
            storage::write_json(&path, &manifest).unwrap();
            assert!(
                start_task_attempt(&path, &policy, "bounded").is_err(),
                "{case}"
            );
            let saved: OvernightManifest = storage::read_json(&path).unwrap();
            assert!(saved.task_attempt.is_none());
        }
    }
}
