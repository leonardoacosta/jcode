//! Selection of the reload executable with downgrade protection.

use super::*;
use anyhow::{Context, Result, ensure};
use std::time::Duration;
use tokio::io::AsyncReadExt;

const CANDIDATE_PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_PROBE_BYTES: u64 = 64 * 1024;

/// Ask a candidate daemon binary which session-control reader version it
/// implements (`version --json` field `control_reader_version`). `Ok(None)`
/// means a valid report without the field: a build that predates durable stop.
/// Any failure to run or parse the probe is an error. The probe is pointed at a
/// nonexistent socket so it never talks to a live daemon.
pub(crate) async fn probe_control_reader_version(candidate: &Path) -> Result<Option<u32>> {
    let mut command = std::process::Command::new(candidate);
    command
        .args(["--no-update", "version", "--json"])
        .env("JCODE_NON_INTERACTIVE", "1")
        .env("JCODE_SOCKET", "/nonexistent/jcode-candidate-probe.sock")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut command = tokio::process::Command::from(command);
    command.kill_on_drop(true);
    let mut child = command
        .spawn()
        .context("could not probe candidate binary")?;
    let mut stdout = child
        .stdout
        .take()
        .context("candidate stdout unavailable")?
        .take(MAX_PROBE_BYTES + 1);
    let bytes = tokio::time::timeout(CANDIDATE_PROBE_TIMEOUT, async {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).await?;
        ensure!(
            bytes.len() as u64 <= MAX_PROBE_BYTES,
            "candidate version report too large"
        );
        ensure!(
            child.wait().await?.success(),
            "candidate version probe failed"
        );
        Ok::<_, anyhow::Error>(bytes)
    })
    .await
    .map_err(|_| anyhow::anyhow!("candidate version probe timed out"))??;
    let report: serde_json::Value =
        serde_json::from_slice(&bytes).context("candidate did not print a version report")?;
    Ok(report
        .get("control_reader_version")
        .and_then(|v| v.as_u64())
        .map(|v| u32::try_from(v).unwrap_or(u32::MAX)))
}

/// Refuse a candidate daemon binary that cannot honor the persisted stop
/// control: its reader version must be at least every record's
/// `min_reader_version`. Fails closed: an unreadable record directory, or a
/// candidate that cannot be probed or reports no version, is a refusal. With no
/// control records there is nothing to protect.
pub(crate) async fn ensure_candidate_control_floor(candidate: &Path) -> Result<()> {
    ensure_candidate_floor_in(&crate::server::session_control::store(), candidate).await
}

async fn ensure_candidate_floor_in(
    store: &crate::server::session_control::ControlStore,
    candidate: &Path,
) -> Result<()> {
    let floor = store.required_reader_floor()?;
    if floor.is_none() {
        return Ok(());
    }
    let version = probe_control_reader_version(candidate).await?;
    crate::server::session_control::ensure_reader_floor(floor, version)
}

pub(crate) fn server_update_candidate(is_selfdev_session: bool) -> Option<(PathBuf, &'static str)> {
    build::shared_server_update_candidate(is_selfdev_session)
}

/// Resolve the binary the reload should actually exec into, with a hard
/// no-downgrade guard.
///
/// `server_update_candidate` can legitimately return an *older* binary (e.g. a
/// `shared-server` channel that an update never advanced, or a leftover self-dev
/// promotion synced from another machine). A forced reload bypasses
/// `server_has_newer_binary`, so without this guard it would silently exec into
/// that older binary and downgrade every connected client.
///
/// We never block a same-or-newer candidate (so self-dev builds, which are
/// freshly written and therefore newer by mtime, still apply). When the
/// candidate is *strictly older* than the running executable we refuse it and
/// re-exec into the current executable instead: same code, fresh process and
/// socket handoff, but no downgrade. Any mtime uncertainty is treated as "do
/// not downgrade".
///
/// Crucially, the candidate is the *newest* reload candidate across BOTH
/// self-dev flavors, not just the one matching `is_selfdev_session`. This keeps
/// the reload target consistent with `server_has_newer_binary`, which also scans
/// both flavors. Without this, a self-dev/canary daemon whose `shared-server`
/// channel is pinned to an *old* self-dev build would advertise
/// `server_has_update = true` (the normal-flavor probe self-heals to the freshly
/// installed release) yet reload into that same old pinned build -> the server
/// reports an update it can never apply, so the client upgrades while the server
/// stays stale and the auto-reload loops until it is suppressed. Selecting the
/// newest candidate across flavors still preserves a deliberately-pinned self-dev
/// build whenever that build is the freshest one on disk (the case the pin is
/// meant to protect).
pub(crate) fn reload_exec_target(is_selfdev_session: bool) -> Option<(PathBuf, &'static str)> {
    // A managed Jcode Cloud daemon runs from an immutable release directory and
    // its runtime is selected by the root-owned updater through one launcher
    // link. That selection is authoritative (a pin may deliberately downgrade),
    // so reload follows it instead of the self-dev/stable channels in ~/.jcode.
    if let Some(target) = managed_cloud_reload_target(running_server_binary().as_deref()) {
        crate::logging::info(&format!(
            "{MANAGED_CLOUD_LIVE_RELOAD_MARKER}: exec target {:?}",
            target.0
        ));
        return Some(target);
    }
    let candidate = newest_reload_candidate(is_selfdev_session)?;
    // On Linux a self-dev rebuild rewrites the running binary in place (a dirty
    // build reuses the same `versions/<hash>` path), which unlinks the running
    // inode. `current_exe()` then resolves `/proc/self/exe` to a path with a
    // trailing " (deleted)" marker that is NOT a real file. If we keep that
    // marker we (a) fail the "same binary" fast-path below, (b) read no mtime so
    // the freshly-built candidate looks like a downgrade, and (c) fall back to
    // re-execing the bogus " (deleted)" path, which does not exist -> the server
    // exits without a replacement and strands every connected client. Strip the
    // marker so we compare against (and can re-exec) the real on-disk path.
    let current_exe = running_server_binary();

    // Identity/mtime comparisons must look through release wrapper scripts to
    // the payload that actually runs (see `build::resolve_binary_payload`):
    // the running exe is the `.bin` payload while channel candidates are tiny
    // wrapper scripts, and comparing wrapper-vs-payload mtimes turned every
    // release install into a phantom "downgrade"/"update". The exec target
    // stays the original candidate path (the wrapper), which is what sets up
    // `LD_LIBRARY_PATH` correctly.
    let candidate_canonical = build::resolve_binary_payload(&candidate.0);
    let current_canonical = current_exe
        .as_ref()
        .map(|p| build::resolve_binary_payload(p));

    let current_mtime = current_canonical.as_deref().and_then(binary_mtime);
    let candidate_mtime = binary_mtime(candidate_canonical.as_path());

    match guarded_reload_target(
        candidate.clone(),
        candidate_canonical.as_path(),
        current_exe.as_deref(),
        current_canonical.as_deref(),
        current_mtime,
        candidate_mtime,
    ) {
        ReloadTargetDecision::UseCandidate(target) => Some(target),
        ReloadTargetDecision::DowngradeBlockedUseCurrent(target) => {
            // Never strand clients by re-execing a binary that is gone from disk.
            // If the running exe was unlinked (e.g. an in-place rebuild) but the
            // candidate still exists, prefer the candidate over refusing to
            // reload. The candidate may be older, but a live downgrade beats a
            // dead server with no replacement.
            if !target.0.exists() && candidate_canonical.exists() {
                crate::logging::warn(&format!(
                    "reload downgrade guard: current binary {:?} is missing on disk; falling back to candidate {:?} to avoid stranding clients",
                    target.0, candidate.0,
                ));
                return Some(candidate);
            }
            crate::logging::warn(&format!(
                "reload downgrade guard: refusing to exec into older candidate; re-execing current binary {:?} instead",
                target.0,
            ));
            Some(target)
        }
        ReloadTargetDecision::DowngradeUnverifiable(target) => {
            crate::logging::warn(&format!(
                "reload downgrade guard: older candidate {:?} detected but current exe is unavailable; proceeding with candidate",
                target.0,
            ));
            Some(target)
        }
    }
}

#[cfg(test)]
mod control_floor_tests {
    use super::*;
    use crate::server::session_control::ControlStore;
    use std::os::unix::fs::PermissionsExt;

    fn fake_candidate(dir: &Path, stdout: &str, exit: i32) -> PathBuf {
        let path = dir.join("candidate");
        std::fs::write(
            &path,
            format!("#!/bin/sh\nprintf '%s' '{stdout}'\nexit {exit}\n"),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn store_with_record() -> (tempfile::TempDir, ControlStore) {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().join("session-control"));
        store.register("worker", None).unwrap();
        (root, store)
    }

    #[tokio::test]
    async fn no_records_means_nothing_to_protect_and_no_probe() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().join("session-control"));
        ensure_candidate_floor_in(&store, Path::new("/nonexistent/candidate"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn candidate_at_or_above_floor_is_accepted() {
        let (root, store) = store_with_record();
        let floor = jcode_swarm_core::control::CONTROL_MIN_READER_VERSION;
        for version in [floor, floor + 1] {
            let bin = fake_candidate(
                root.path(),
                &format!(r#"{{"git_hash":"x","control_reader_version":{version}}}"#),
                0,
            );
            ensure_candidate_floor_in(&store, &bin).await.unwrap();
        }
    }

    #[tokio::test]
    async fn candidate_below_floor_or_without_version_is_refused() {
        let (root, store) = store_with_record();
        let below = fake_candidate(
            root.path(),
            r#"{"git_hash":"x","control_reader_version":1}"#,
            0,
        );
        let error = ensure_candidate_floor_in(&store, &below).await.unwrap_err();
        assert!(
            format!("{error:#}").contains("below persisted"),
            "{error:#}"
        );
        let legacy = fake_candidate(root.path(), r#"{"git_hash":"x"}"#, 0);
        let error = ensure_candidate_floor_in(&store, &legacy)
            .await
            .unwrap_err();
        assert!(
            format!("{error:#}").contains("predates durable stop"),
            "{error:#}"
        );
    }

    #[tokio::test]
    async fn unprobeable_candidate_is_refused_fail_closed() {
        let (root, store) = store_with_record();
        for bin in [
            fake_candidate(root.path(), "not json", 0),
            fake_candidate(root.path(), r#"{"control_reader_version":9}"#, 3),
            root.path().join("missing"),
        ] {
            assert!(
                ensure_candidate_floor_in(&store, &bin).await.is_err(),
                "{bin:?}"
            );
        }
    }

    #[tokio::test]
    async fn higher_record_floor_refuses_a_candidate_at_this_builds_version() {
        let (root, store) = store_with_record();
        let path = store.path("worker").unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value["min_reader_version"] = serde_json::json!(7);
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let bin = fake_candidate(
            root.path(),
            r#"{"git_hash":"x","control_reader_version":2}"#,
            0,
        );
        assert!(ensure_candidate_floor_in(&store, &bin).await.is_err());
        let ok = fake_candidate(
            root.path(),
            r#"{"git_hash":"x","control_reader_version":7}"#,
            0,
        );
        ensure_candidate_floor_in(&store, &ok).await.unwrap();
    }

    #[tokio::test]
    async fn legacy_and_corrupt_records_set_the_floor_without_being_rewritten() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().join("session-control"));
        std::fs::create_dir_all(root.path().join("session-control")).unwrap();
        let legacy = root.path().join("session-control/legacy.json");
        std::fs::write(&legacy, r#"{"schema_version":1,"session_id":"legacy"}"#).unwrap();
        assert_eq!(store.required_reader_floor().unwrap(), Some(1));
        let corrupt = root.path().join("session-control/bad.json");
        std::fs::write(&corrupt, b"{ not json").unwrap();
        assert_eq!(
            store.required_reader_floor().unwrap(),
            Some(jcode_swarm_core::control::CONTROL_MIN_READER_VERSION)
        );
        assert_eq!(std::fs::read(&corrupt).unwrap(), b"{ not json");
        assert_eq!(
            std::fs::read(&legacy).unwrap(),
            br#"{"schema_version":1,"session_id":"legacy"}"#
        );
    }
}
