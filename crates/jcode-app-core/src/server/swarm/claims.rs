//! Per-file edit claims among swarm workers (`add-swarm-file-claims`).
//!
//! Canonical path -> holder session id. Persisted beside swarm state so
//! claims survive a server reload.

use super::*;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::server) enum FileClaimMode {
    Off,
    Warn,
    Block,
}

impl FileClaimMode {
    pub(in crate::server) fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "off" => Self::Off,
            "block" => Self::Block,
            _ => Self::Warn,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::server) enum ClaimOutcome {
    /// Path claimed (or already held) by the caller, or claims are off.
    Granted,
    /// Edit may proceed; holder named for the warning.
    Warn { holder: String },
    /// Edit refused; holder named for the error.
    Blocked { holder: String },
}

#[derive(Debug, Default)]
pub(in crate::server) struct FileClaimStore {
    claims: BTreeMap<String, String>,
    persist_path: Option<PathBuf>,
}

fn canonical_key(path: &Path) -> String {
    let absolute = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let mut key = PathBuf::new();
    // Resolve existing symlinks and normalize missing descendants too, so a
    // claim made before mkdir/write keeps its identity after creation.
    for component in absolute.components() {
        if component == std::path::Component::ParentDir {
            key.pop();
        } else {
            key.push(component);
            if let Ok(canonical) = std::fs::canonicalize(&key) {
                key = canonical;
            }
        }
    }
    key.to_string_lossy().into_owned()
}

impl FileClaimStore {
    pub(in crate::server) fn load(persist_path: Option<PathBuf>) -> Self {
        let claims = persist_path
            .as_ref()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self {
            claims,
            persist_path,
        }
    }

    fn persist(&self) {
        let Some(path) = &self.persist_path else {
            return;
        };
        if let Ok(json) = serde_json::to_vec(&self.claims) {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(&tmp, path);
            }
        }
    }

    /// Record or check a claim. `exempt_from_block` is for the coordinator:
    /// still warned, never refused.
    pub(in crate::server) fn claim(
        &mut self,
        session_id: &str,
        path: &Path,
        mode: FileClaimMode,
        exempt_from_block: bool,
    ) -> ClaimOutcome {
        if mode == FileClaimMode::Off {
            return ClaimOutcome::Granted;
        }
        let key = canonical_key(path);
        match self.claims.get(&key) {
            Some(holder) if holder != session_id => {
                let holder = holder.clone();
                if mode == FileClaimMode::Block && !exempt_from_block {
                    ClaimOutcome::Blocked { holder }
                } else {
                    ClaimOutcome::Warn { holder }
                }
            }
            Some(_) => ClaimOutcome::Granted,
            None => {
                self.claims.insert(key, session_id.to_string());
                self.persist();
                ClaimOutcome::Granted
            }
        }
    }

    /// Drop every claim held by `session_id`. Returns the released paths.
    pub(in crate::server) fn release_session(&mut self, session_id: &str) -> Vec<String> {
        let released: Vec<String> = self
            .claims
            .iter()
            .filter(|(_, holder)| holder.as_str() == session_id)
            .map(|(path, _)| path.clone())
            .collect();
        if !released.is_empty() {
            for path in &released {
                self.claims.remove(path);
            }
            self.persist();
        }
        released
    }

    /// Active claims as (path, holder), sorted by path.
    pub(in crate::server) fn active(&self) -> Vec<(String, String)> {
        self.claims
            .iter()
            .map(|(p, h)| (p.clone(), h.clone()))
            .collect()
    }
}

/// Process-wide store persisted at `<jcode_dir>/swarm-file-claims.json`.
pub(in crate::server) fn file_claims() -> &'static StdMutex<FileClaimStore> {
    static STORE: OnceLock<StdMutex<FileClaimStore>> = OnceLock::new();
    STORE.get_or_init(|| {
        let path = crate::storage::jcode_dir()
            .ok()
            .map(|dir| dir.join("swarm-file-claims.json"));
        StdMutex::new(FileClaimStore::load(path))
    })
}

/// Release all claims held by a worker (finish, stop, removal/orphan).
pub(in crate::server) fn release_file_claims_of(session_id: &str) {
    if let Ok(mut store) = file_claims().lock() {
        store.release_session(session_id);
    }
}

pub(in crate::server) fn unregister_claim_worker(session_id: &str) {
    if let Ok(mut workers) = claim_workers().lock() {
        workers.remove(session_id);
    }
    release_file_claims_of(session_id);
}

/// Spawned swarm workers that participate in file claims.
fn claim_workers() -> &'static StdMutex<HashSet<String>> {
    static WORKERS: OnceLock<StdMutex<HashSet<String>>> = OnceLock::new();
    WORKERS.get_or_init(|| StdMutex::new(HashSet::new()))
}

pub(in crate::server) fn register_claim_worker(session_id: &str) {
    if let Ok(mut workers) = claim_workers().lock() {
        workers.insert(session_id.to_string());
    }
}

fn configured_claim_mode() -> FileClaimMode {
    crate::config::config()
        .swarm
        .file_claims
        .as_deref()
        .map(FileClaimMode::parse)
        .unwrap_or(FileClaimMode::Warn)
}

/// Claim `path` for a swarm worker before an edit tool writes it.
/// Returns `Ok(Some(warning))` in warn mode on conflict, `Err` in block mode.
/// Non-worker sessions are never claimed or checked.
pub(crate) fn check_file_claim(session_id: &str, path: &Path) -> anyhow::Result<Option<String>> {
    let Ok(workers) = claim_workers().lock() else {
        return Ok(None);
    };
    if !workers.contains(session_id) {
        return Ok(None);
    }
    let outcome = match file_claims().lock() {
        Ok(mut store) => store.claim(session_id, path, configured_claim_mode(), false),
        Err(_) => return Ok(None),
    };
    match outcome {
        ClaimOutcome::Granted => Ok(None),
        ClaimOutcome::Warn { holder } => Ok(Some(format!(
            "Warning: {} is claimed by swarm worker {holder}; coordinate before further edits.",
            path.display()
        ))),
        ClaimOutcome::Blocked { holder } => Err(anyhow::anyhow!(
            "Refused: {} is claimed by swarm worker {holder} (swarm.file_claims = \"block\").",
            path.display()
        )),
    }
}

/// Paths currently claimed by `session_id`.
pub(in crate::server) fn file_claims_held_by(session_id: &str) -> Vec<String> {
    file_claims()
        .lock()
        .map(|s| {
            s.active()
                .into_iter()
                .filter(|(_, h)| h == session_id)
                .map(|(p, _)| p)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod claim_tests {
    use super::*;

    #[test]
    fn new_nested_path_keeps_its_claim_after_creation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("new/sub/../sub/file.rs");
        let mut store = FileClaimStore::default();
        store.claim("a", &path, FileClaimMode::Block, false);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "old").unwrap();
        let alias = dir.path().join("new/sub/file.rs");
        assert_eq!(
            store.claim("b", &alias, FileClaimMode::Block, false),
            ClaimOutcome::Blocked { holder: "a".into() }
        );
    }

    struct UnusedProvider;

    #[async_trait::async_trait]
    impl crate::provider::Provider for UnusedProvider {
        async fn complete(
            &self,
            _: &[crate::message::Message],
            _: &[crate::message::ToolDefinition],
            _: &str,
            _: Option<&str>,
        ) -> anyhow::Result<crate::provider::EventStream> {
            anyhow::bail!("file claim tests must not call a provider")
        }

        fn name(&self) -> &str {
            "unused"
        }
        fn fork(&self) -> Arc<dyn crate::provider::Provider> {
            Arc::new(Self)
        }
    }

    #[tokio::test]
    async fn file_claim_tools_enforce_modes() {
        // Each mode gets its own process, config cache and persisted claim store.
        let Ok(mode) = std::env::var("JCODE_FILE_CLAIM_TEST_MODE") else {
            for mode in ["default", "off", "warn", "block"] {
                let home = tempfile::tempdir().unwrap();
                let config = if mode == "default" {
                    String::new()
                } else {
                    format!("[swarm]\nfile_claims = {mode:?}\n")
                };
                std::fs::write(home.path().join("config.toml"), config).unwrap();
                let result = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "server::swarm::claims::claim_tests::file_claim_tools_enforce_modes",
                        "--nocapture",
                    ])
                    .env("JCODE_HOME", home.path())
                    .env("JCODE_FILE_CLAIM_TEST_MODE", mode)
                    .output()
                    .unwrap();
                assert!(
                    result.status.success(),
                    "{mode}: {}{}",
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                );
            }
            return;
        };
        use crate::tool::{Registry, ToolContext, ToolExecutionMode};
        use serde_json::json;
        let dir = tempfile::tempdir().unwrap();
        let registry = Registry::new(Arc::new(UnusedProvider)).await;
        let ctx = |session: &str| ToolContext {
            session_id: session.into(),
            message_id: "m".into(),
            tool_call_id: "t".into(),
            working_dir: Some(dir.path().to_path_buf()),
            stdin_request_tx: None,
            graceful_shutdown_signal: None,
            execution_mode: ToolExecutionMode::Direct,
        };
        register_claim_worker("holder");
        register_claim_worker("contender");
        for (name, file, input) in [
            (
                "write",
                "write",
                json!({"file_path":"write", "content":"new\n"}),
            ),
            (
                "edit",
                "edit",
                json!({"file_path":"edit", "old_string":"old", "new_string":"new"}),
            ),
            (
                "apply_patch",
                "apply",
                json!({"patch_text":"*** Begin Patch\n*** Update File: apply\n@@\n-old\n+new\n*** End Patch"}),
            ),
            (
                "apply_patch",
                "unified",
                json!({"patch_text":"--- a/unified\n+++ b/unified\n@@ -1 +1 @@\n-old\n+new\n"}),
            ),
        ] {
            let path = dir.path().join(file);
            std::fs::write(&path, "old\n").unwrap();
            registry
                .execute(name, input.clone(), ctx("holder"))
                .await
                .unwrap();
            if mode != "off" {
                assert!(file_claims_held_by("holder").contains(&canonical_key(&path)));
            }
            std::fs::write(&path, "old\n").unwrap();
            let result = registry
                .execute(name, input.clone(), ctx("contender"))
                .await;
            if mode == "block" {
                assert!(result.unwrap_err().to_string().contains("holder"));
                assert_eq!(std::fs::read_to_string(&path).unwrap(), "old\n");
            } else {
                let output = result.unwrap();
                assert_eq!(
                    output.output.contains("holder"),
                    mode != "off",
                    "{}",
                    output.output
                );
                assert_eq!(std::fs::read_to_string(&path).unwrap(), "new\n");
            }
            // Root/coordinator sessions bypass block enforcement and do not claim.
            std::fs::write(&path, "old\n").unwrap();
            registry
                .execute(name, input, ctx("coordinator"))
                .await
                .unwrap();
            assert!(file_claims_held_by("coordinator").is_empty());
        }
        if mode == "block" {
            std::fs::write(dir.path().join("source"), "old\n").unwrap();
            for patch in [
                "*** Begin Patch\n*** Add File: unclaimed\n+fresh\n*** Update File: source\n*** Move to: write\n@@\n-old\n+changed\n*** End Patch",
                "*** Begin Patch\n*** Add File: unclaimed\n+fresh\n*** Delete File: write\n*** End Patch",
                "--- /dev/null\n+++ b/unclaimed\n@@ -0,0 +1 @@\n+fresh\n--- a/write\n+++ b/write\n@@ -1 +1 @@\n-new\n+changed\n",
            ] {
                let error = registry
                    .execute("apply_patch", json!({"patch_text":patch}), ctx("contender"))
                    .await
                    .unwrap_err();
                assert!(error.to_string().contains("holder"), "{error}");
                assert!(!dir.path().join("unclaimed").exists());
                assert_eq!(
                    std::fs::read_to_string(dir.path().join("write")).unwrap(),
                    "new\n"
                );
            }
        }
        if mode != "off" {
            let persisted = FileClaimStore::load(Some(
                crate::storage::jcode_dir()
                    .unwrap()
                    .join("swarm-file-claims.json"),
            ));
            assert_eq!(
                persisted
                    .active()
                    .iter()
                    .filter(|(_, h)| h == "holder")
                    .count(),
                4
            );
            let (member, _rx) = super::super::tests::swarm_member("holder", "agent", true);
            let members = Arc::new(tokio::sync::RwLock::new(HashMap::from([(
                "holder".into(),
                member,
            )])));
            let swarms = Arc::new(tokio::sync::RwLock::new(HashMap::from([(
                "swarm-1".into(),
                HashSet::from(["holder".into()]),
            )])));
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            crate::server::comm_sync::handle_comm_status(
                1,
                "holder".into(),
                "holder".into(),
                &Arc::default(),
                &members,
                &Arc::default(),
                &crate::server::FileTouchService::new(),
                &tx,
            )
            .await;
            let crate::protocol::ServerEvent::CommStatusResponse { snapshot, .. } =
                rx.recv().await.unwrap()
            else {
                panic!("missing status snapshot")
            };
            assert_eq!(snapshot.file_claims.len(), 4);
            let formatted = crate::protocol::format_comm_status_snapshot(&snapshot);
            assert!(formatted.contains("Claims:"));
            assert!(formatted.contains(&canonical_key(&dir.path().join("write"))));
            for status in ["completed", "stopped", "failed", "ready"] {
                super::super::update_member_status_with_report(
                    "holder",
                    status,
                    None,
                    Some("finished".into()),
                    &members,
                    &swarms,
                    None,
                    None,
                    None,
                )
                .await;
                assert!(file_claims_held_by("holder").is_empty(), "{status}");
                // A finished worker can receive more work and claim again.
                check_file_claim("holder", &dir.path().join("next")).unwrap();
                assert_eq!(file_claims_held_by("holder").len(), 1);
            }
            let mut persisted_member = members.read().await["holder"].clone();
            persisted_member.status = "running".into();
            persisted_member.report_back_to_session_id = Some("coordinator".into());
            crate::server::swarm_persistence::persist_swarm_state(
                "swarm-1",
                None,
                Some("coordinator"),
                &[persisted_member],
            );
            // Simulate process-local worker tracking being lost on reload.
            claim_workers().lock().unwrap().remove("holder");
            let recovered = crate::server::swarm_persistence::load_runtime_state();
            assert_eq!(recovered.members["holder"].status, "crashed");
            assert!(
                file_claims_held_by("holder").is_empty(),
                "orphan claims survive recovery"
            );
            check_file_claim("holder", &dir.path().join("resumed")).unwrap();
            assert_eq!(file_claims_held_by("holder").len(), 1);
            super::super::remove_session_from_swarm(
                "holder",
                "swarm-1",
                &members,
                &swarms,
                &Arc::default(),
                &Arc::default(),
            )
            .await;
            assert!(file_claims_held_by("holder").is_empty());
            check_file_claim("holder", &dir.path().join("orphan")).unwrap();
            assert!(
                file_claims_held_by("holder").is_empty(),
                "removed workers must unregister"
            );
            assert!(
                check_file_claim("contender", &dir.path().join("write"))
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn non_worker_sessions_are_never_claimed() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x.rs");
        assert_eq!(
            check_file_claim("not-a-worker-session", &file).unwrap(),
            None
        );
        assert!(file_claims_held_by("not-a-worker-session").is_empty());
    }

    #[test]
    fn claim_warn_block_release_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let persist = dir.path().join("claims.json");
        let file = dir.path().join("lib.rs");
        let mut store = FileClaimStore::load(Some(persist.clone()));

        assert_eq!(
            store.claim("a", &file, FileClaimMode::Warn, false),
            ClaimOutcome::Granted
        );
        assert_eq!(
            store.claim("a", &file, FileClaimMode::Block, false),
            ClaimOutcome::Granted
        );
        assert_eq!(
            store.claim("b", &file, FileClaimMode::Warn, false),
            ClaimOutcome::Warn { holder: "a".into() }
        );
        assert_eq!(
            store.claim("b", &file, FileClaimMode::Block, false),
            ClaimOutcome::Blocked { holder: "a".into() }
        );
        // Coordinator exempt from blocking but still warned.
        assert_eq!(
            store.claim("coord", &file, FileClaimMode::Block, true),
            ClaimOutcome::Warn { holder: "a".into() }
        );
        assert_eq!(
            store.claim("b", &file, FileClaimMode::Off, false),
            ClaimOutcome::Granted
        );

        // Survives reload.
        let mut reloaded = FileClaimStore::load(Some(persist.clone()));
        assert_eq!(reloaded.active().len(), 1);
        assert_eq!(reloaded.active()[0].1, "a");

        // Stop/orphan release frees the path for B.
        assert_eq!(reloaded.release_session("a").len(), 1);
        assert_eq!(
            reloaded.claim("b", &file, FileClaimMode::Block, false),
            ClaimOutcome::Granted
        );
        assert_eq!(FileClaimStore::load(Some(persist)).active()[0].1, "b");
    }

    #[test]
    fn mode_parse_defaults_to_warn() {
        assert_eq!(FileClaimMode::parse("block"), FileClaimMode::Block);
        assert_eq!(FileClaimMode::parse("OFF"), FileClaimMode::Off);
        assert_eq!(FileClaimMode::parse(""), FileClaimMode::Warn);
    }
}
