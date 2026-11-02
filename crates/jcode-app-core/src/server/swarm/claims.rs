//! Persistent, transactional file claims shared by swarm server processes.

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
    Granted,
    Warn { holder: String },
    Blocked { holder: String },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum FileClaim {
    // Keep committed ownership compatible with the original on-disk map.
    Committed(String),
    Reserved {
        holder: String,
        reservations: HashSet<String>,
    },
}

impl FileClaim {
    fn holder(&self) -> &str {
        match self {
            Self::Committed(holder) | Self::Reserved { holder, .. } => holder,
        }
    }
}

#[derive(Debug, Default)]
pub(in crate::server) struct FileClaimStore {
    claims: BTreeMap<String, FileClaim>,
    persist_path: Option<PathBuf>,
}

fn canonical_key(path: &Path) -> anyhow::Result<String> {
    fn resolve(path: &Path, depth: usize) -> anyhow::Result<PathBuf> {
        anyhow::ensure!(
            depth < 40,
            "Too many symbolic links in claim path {}",
            path.display()
        );
        let absolute = std::path::absolute(path)?;
        let mut key = PathBuf::new();
        for component in absolute.components() {
            if component == std::path::Component::ParentDir {
                key.pop();
            } else {
                key.push(component);
                if matches!(component, std::path::Component::Normal(_)) {
                    match std::fs::symlink_metadata(&key) {
                        Ok(metadata) if metadata.file_type().is_symlink() => {
                            let target = std::fs::read_link(&key)?;
                            let target = if target.is_absolute() {
                                target
                            } else {
                                key.parent().unwrap().join(target)
                            };
                            key = resolve(&target, depth + 1)?;
                        }
                        Ok(_) => {
                            if let Ok(canonical) = std::fs::canonicalize(&key) {
                                key = canonical;
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => return Err(error.into()),
                    }
                }
            }
        }
        Ok(key)
    }
    Ok(resolve(path, 0)?.to_string_lossy().into_owned())
}

/// Successful mutations retain reservations. Dropping a failed/cancelled tool
/// rolls back only its newly reserved paths.
#[derive(Default)]
pub(crate) struct FileClaims {
    session_id: String,
    reservation_id: String,
    persist_path: Option<PathBuf>,
    paths: BTreeMap<PathBuf, String>,
    pending: HashSet<String>,
    warnings: Vec<String>,
}

impl FileClaims {
    async fn mutate(
        &mut self,
        path: &Path,
        mutation: impl FnOnce(&Path) -> std::io::Result<()> + Send + 'static,
    ) -> std::io::Result<()> {
        let path = path.to_path_buf();
        let mut operation = FileClaims::default();
        operation.session_id = self.session_id.clone();
        operation.reservation_id = self.reservation_id.clone();
        operation.persist_path = self.persist_path.clone();
        if let Some(key) = self.paths.get(&path) {
            operation.paths.insert(path.clone(), key.clone());
            if self.pending.remove(key) {
                operation.pending.insert(key.clone());
            }
        }
        // Tokio filesystem work continues after its awaiting future is dropped.
        // Keep the reservation with that work until its real outcome is known.
        let (result, mut operation) = tokio::task::spawn_blocking(move || {
            let result = mutation(&path)
                .and_then(|()| operation.commit(&path).map_err(std::io::Error::other));
            (result, operation)
        })
        .await
        .map_err(std::io::Error::other)?;
        self.pending.extend(operation.pending.drain());
        result
    }

    pub(crate) async fn write(&mut self, path: &Path, contents: &[u8]) -> std::io::Result<()> {
        let contents = contents.to_vec();
        self.mutate(path, move |path| std::fs::write(path, contents))
            .await
    }

    pub(crate) async fn remove_file(&mut self, path: &Path) -> std::io::Result<()> {
        self.mutate(path, |path| std::fs::remove_file(path)).await
    }

    pub(crate) fn commit(&mut self, path: &Path) -> anyhow::Result<()> {
        if let Some(key) = self.paths.get(path) {
            if self.pending.remove(key) {
                // A completed filesystem write must never roll back, even if
                // persisting its completion fails. Its token remains protective.
                FileClaimStore::load(self.persist_path.clone()).finish(
                    &self.session_id,
                    &HashSet::from([key.clone()]),
                    &self.reservation_id,
                    true,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn warnings(&self) -> &[String] {
        &self.warnings
    }
}

impl Drop for FileClaims {
    fn drop(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let mut store = FileClaimStore::load(self.persist_path.clone());
        if let Err(error) =
            store.finish(&self.session_id, &self.pending, &self.reservation_id, false)
        {
            crate::logging::warn(&format!("Failed to roll back file claims: {error}"));
        }
    }
}

impl FileClaimStore {
    pub(in crate::server) fn load(persist_path: Option<PathBuf>) -> Self {
        // Every operation refreshes under the process lock. Never trust this
        // initial snapshot for acquisition, rollback, release or status.
        Self {
            claims: BTreeMap::new(),
            persist_path,
        }
    }

    fn lock_and_reload(&mut self) -> anyhow::Result<Option<std::fs::File>> {
        let Some(path) = &self.persist_path else {
            return Ok(None);
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))?;
        lock.lock()?;
        self.claims = match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(error.into()),
        };
        Ok(Some(lock))
    }

    fn persist(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.persist_path {
            let tmp = path.with_extension("json.tmp");
            std::fs::write(&tmp, serde_json::to_vec(&self.claims)?)?;
            std::fs::rename(tmp, path)?;
        }
        Ok(())
    }

    fn outcome(
        &self,
        session_id: &str,
        key: &str,
        mode: FileClaimMode,
        exempt: bool,
    ) -> ClaimOutcome {
        if mode != FileClaimMode::Off {
            if let Some(holder) = self
                .claims
                .get(key)
                .map(FileClaim::holder)
                .filter(|holder| *holder != session_id)
            {
                return if mode == FileClaimMode::Block && !exempt {
                    ClaimOutcome::Blocked {
                        holder: holder.into(),
                    }
                } else {
                    ClaimOutcome::Warn {
                        holder: holder.into(),
                    }
                };
            }
        }
        ClaimOutcome::Granted
    }

    fn reserve(
        &mut self,
        session_id: &str,
        paths: impl IntoIterator<Item = PathBuf>,
        mode: FileClaimMode,
        exempt: bool,
    ) -> anyhow::Result<FileClaims> {
        let mut reservation = FileClaims {
            session_id: session_id.into(),
            reservation_id: crate::id::new_id("file_claim"),
            persist_path: self.persist_path.clone(),
            paths: BTreeMap::new(),
            pending: HashSet::new(),
            warnings: Vec::new(),
        };
        if mode == FileClaimMode::Off {
            return Ok(reservation);
        }
        let _lock = self.lock_and_reload()?;
        // Preflight the complete batch before recording any reservation.
        for path in paths {
            let key = canonical_key(&path)?;
            match self.outcome(session_id, &key, mode, exempt) {
                ClaimOutcome::Blocked { holder } => anyhow::bail!("Refused: {} is claimed by swarm worker {holder} (swarm.file_claims = \"block\").", path.display()),
                ClaimOutcome::Warn { holder } => reservation.warnings.push(format!("Warning: {} is claimed by swarm worker {holder}; coordinate before further edits.", path.display())),
                ClaimOutcome::Granted => {},
            }
            reservation.paths.insert(path, key);
        }
        for key in reservation.paths.values() {
            let claim = self
                .claims
                .entry(key.clone())
                .or_insert_with(|| FileClaim::Reserved {
                    holder: session_id.into(),
                    reservations: HashSet::new(),
                });
            if let FileClaim::Reserved {
                holder,
                reservations,
            } = claim
            {
                if holder == session_id {
                    reservations.insert(reservation.reservation_id.clone());
                    reservation.pending.insert(key.clone());
                }
            }
        }
        if !reservation.pending.is_empty() {
            self.persist()?;
        }
        Ok(reservation)
    }

    fn finish(
        &mut self,
        session_id: &str,
        keys: &HashSet<String>,
        reservation_id: &str,
        success: bool,
    ) -> anyhow::Result<()> {
        let _lock = self.lock_and_reload()?;
        let mut changed = false;
        for key in keys {
            if let Some(FileClaim::Reserved {
                holder,
                reservations,
            }) = self.claims.get_mut(key)
            {
                if holder == session_id && reservations.remove(reservation_id) {
                    changed = true;
                    if success {
                        self.claims
                            .insert(key.clone(), FileClaim::Committed(session_id.into()));
                    } else if reservations.is_empty() {
                        self.claims.remove(key);
                    }
                }
            }
        }
        if changed {
            self.persist()?;
        }
        Ok(())
    }

    fn release(
        &mut self,
        session_id: &str,
        keys: Option<&HashSet<String>>,
    ) -> anyhow::Result<Vec<String>> {
        let _lock = self.lock_and_reload()?;
        let released: Vec<_> = self
            .claims
            .iter()
            .filter(|(path, holder)| {
                holder.holder() == session_id && keys.is_none_or(|keys| keys.contains(*path))
            })
            .map(|(path, _)| path.clone())
            .collect();
        for path in &released {
            self.claims.remove(path);
        }
        if !released.is_empty() {
            self.persist()?;
        }
        Ok(released)
    }

    pub(in crate::server) fn release_session(
        &mut self,
        session_id: &str,
    ) -> anyhow::Result<Vec<String>> {
        self.release(session_id, None)
    }

    pub(in crate::server) fn active(&mut self) -> anyhow::Result<Vec<(String, String)>> {
        let _lock = self.lock_and_reload()?;
        Ok(self
            .claims
            .iter()
            .map(|(path, holder)| (path.clone(), holder.holder().to_owned()))
            .collect())
    }

    #[cfg(test)]
    fn claim(
        &mut self,
        session_id: &str,
        path: &Path,
        mode: FileClaimMode,
        exempt: bool,
    ) -> ClaimOutcome {
        let result = self.reserve(session_id, [path.to_path_buf()], mode, exempt);
        let outcome = self.outcome(session_id, &canonical_key(path).unwrap(), mode, exempt);
        match result {
            Ok(mut reservation) => {
                self.finish(
                    session_id,
                    &reservation.pending,
                    &reservation.reservation_id,
                    true,
                )
                .unwrap();
                reservation.pending.clear();
            }
            Err(error) => assert!(matches!(outcome, ClaimOutcome::Blocked { .. }), "{error}"),
        }
        outcome
    }
}

pub(in crate::server) fn file_claims() -> &'static StdMutex<FileClaimStore> {
    static STORE: OnceLock<StdMutex<FileClaimStore>> = OnceLock::new();
    STORE.get_or_init(|| {
        StdMutex::new(FileClaimStore::load(
            crate::storage::jcode_dir()
                .ok()
                .map(|dir| dir.join("swarm-file-claims.json")),
        ))
    })
}

pub(in crate::server) fn release_file_claims_of(session_id: &str) {
    if let Ok(mut store) = file_claims().lock() {
        if let Err(error) = store.release_session(session_id) {
            crate::logging::warn(&format!("Failed to release file claims: {error}"));
        }
    }
}

pub(in crate::server) fn unregister_claim_worker(session_id: &str) {
    if let Ok(mut workers) = claim_workers().lock() {
        workers.remove(session_id);
    }
    release_file_claims_of(session_id);
}

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

pub(crate) fn claim_files(
    session_id: &str,
    paths: impl IntoIterator<Item = PathBuf>,
) -> anyhow::Result<FileClaims> {
    let workers = claim_workers()
        .lock()
        .map_err(|_| anyhow::anyhow!("File claim worker lock poisoned"))?;
    if !workers.contains(session_id) {
        return Ok(FileClaims::default());
    }
    let mut store = file_claims()
        .lock()
        .map_err(|_| anyhow::anyhow!("File claim store lock poisoned"))?;
    let mode = configured_claim_mode();
    anyhow::ensure!(
        mode == FileClaimMode::Off || store.persist_path.is_some(),
        "Cannot locate persistent swarm file claim store"
    );
    store.reserve(session_id, paths, mode, false)
}

#[cfg(test)]
fn check_file_claim(session_id: &str, path: &Path) -> anyhow::Result<Option<String>> {
    let mut reservation = claim_files(session_id, [path.to_path_buf()])?;
    reservation.commit(path)?;
    Ok(reservation.warnings.first().cloned())
}

pub(in crate::server) fn file_claims_held_by(session_id: &str) -> Vec<String> {
    file_claims()
        .lock()
        .ok()
        .and_then(|mut store| store.active().ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, holder)| holder == session_id)
        .map(|(path, _)| path)
        .collect()
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

    #[tokio::test]
    async fn cancelled_caller_keeps_claim_with_inflight_mutation() {
        let dir = tempfile::tempdir().unwrap();
        let persist = dir.path().join("claims.json");
        let file = dir.path().join("file");
        let mut store = FileClaimStore::load(Some(persist.clone()));
        let mut reservation = store
            .reserve("worker", [file.clone()], FileClaimMode::Block, false)
            .unwrap();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();
        let path = file.clone();
        let task = tokio::spawn(async move {
            reservation
                .mutate(&path, move |path| {
                    let _ = started_tx.send(());
                    finish_rx
                        .recv_timeout(std::time::Duration::from_secs(10))
                        .unwrap();
                    let result = std::fs::write(path, "finished");
                    let _ = done_tx.send(());
                    result
                })
                .await
        });
        started_rx.await.unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        let mut observer = FileClaimStore::load(Some(persist));
        assert!(
            observer
                .reserve("contender", [file.clone()], FileClaimMode::Block, false)
                .is_err()
        );
        finish_tx.send(()).unwrap();
        done_rx.await.unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "finished");
        assert_eq!(
            observer.active().unwrap(),
            vec![(canonical_key(&file).unwrap(), "worker".into())]
        );
    }

    #[cfg(unix)]
    #[test]
    fn dangling_relative_chains_resolve_and_cycles_fail() {
        let dir = tempfile::tempdir().unwrap();
        let alias = dir.path().join("alias");
        std::os::unix::fs::symlink("target", &alias).unwrap();
        let chain = dir.path().join("chain");
        std::os::unix::fs::symlink("alias", &chain).unwrap();
        assert_eq!(
            canonical_key(&chain).unwrap(),
            canonical_key(&dir.path().join("target")).unwrap()
        );
        let cycle = dir.path().join("cycle");
        std::os::unix::fs::symlink("cycle", &cycle).unwrap();
        assert!(
            canonical_key(&cycle)
                .unwrap_err()
                .to_string()
                .contains("Too many symbolic links")
        );
    }

    #[tokio::test]
    async fn overlapping_reservations_preserve_success_and_inflight_owners() {
        let dir = tempfile::tempdir().unwrap();
        let persist = dir.path().join("claims.json");
        let path = dir.path().join("file");
        let mut first = FileClaimStore::load(Some(persist.clone()));
        let mut second = FileClaimStore::load(Some(persist));
        let older = first
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        let mut newer = second
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        newer.write(&path, b"success").await.unwrap();
        drop(older);
        assert!(
            first
                .reserve("b", [path.clone()], FileClaimMode::Block, false)
                .is_err(),
            "older rollback erased newer success"
        );
        drop(newer);
        drop(
            first
                .reserve("a", [path.clone()], FileClaimMode::Block, false)
                .unwrap(),
        );
        assert!(
            second
                .reserve("b", [path.clone()], FileClaimMode::Block, false)
                .is_err(),
            "later rollback erased prior success"
        );
        first.release_session("a").unwrap();
        let older = first
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        let newer = second
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        drop(newer);
        assert!(
            second
                .reserve("b", [path.clone()], FileClaimMode::Block, false)
                .is_err(),
            "newer rollback erased in-flight reservation"
        );
        drop(older);
        assert!(first.active().unwrap().is_empty());
        let older = first
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        let newer = second
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        drop(older);
        assert!(
            first
                .reserve("b", [path.clone()], FileClaimMode::Block, false)
                .is_err(),
            "older rollback erased newer in-flight reservation"
        );
        drop(newer);
        assert!(second.active().unwrap().is_empty());
        let stale = first
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        first.release_session("a").unwrap();
        let current = second
            .reserve("a", [path.clone()], FileClaimMode::Block, false)
            .unwrap();
        drop(stale);
        assert!(
            first
                .reserve("b", [path.clone()], FileClaimMode::Block, false)
                .is_err(),
            "stale reservation erased a new generation"
        );
        drop(current);
        assert!(first.active().unwrap().is_empty());
        assert!(
            second
                .reserve("b", [path], FileClaimMode::Block, false)
                .is_ok()
        );
    }

    #[test]
    fn stale_stores_and_rollback_preserve_committed_owners() {
        let dir = tempfile::tempdir().unwrap();
        let persist = dir.path().join("claims.json");
        let mut first = FileClaimStore::load(Some(persist.clone()));
        let mut stale = FileClaimStore::load(Some(persist));
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        first.claim("first", &a, FileClaimMode::Block, false);
        stale.claim("second", &b, FileClaimMode::Block, false);
        assert_eq!(first.active().unwrap().len(), 2);
        {
            let _cancelled = first
                .reserve(
                    "first",
                    [a.clone(), dir.path().join("cancelled")],
                    FileClaimMode::Block,
                    false,
                )
                .unwrap();
        }
        assert_eq!(stale.active().unwrap().len(), 2);
        first.release_session("first").unwrap();
        assert_eq!(
            stale.active().unwrap(),
            vec![(canonical_key(&b).unwrap(), "second".into())]
        );
    }

    #[test]
    fn corrupt_store_refuses_acquisition() {
        let dir = tempfile::tempdir().unwrap();
        let persist = dir.path().join("claims.json");
        std::fs::write(&persist, "not json").unwrap();
        let mut store = FileClaimStore::load(Some(persist.clone()));
        assert!(
            store
                .reserve(
                    "worker",
                    [dir.path().join("file")],
                    FileClaimMode::Block,
                    false
                )
                .is_err()
        );
        assert_eq!(std::fs::read_to_string(persist).unwrap(), "not json");
    }

    #[tokio::test]
    async fn file_claim_processes_share_one_owner() {
        use crate::tool::{Registry, ToolContext, ToolExecutionMode};
        use std::time::{Duration, Instant};
        if let Ok(worker) = std::env::var("JCODE_CLAIM_PROCESS_TEST") {
            let home = crate::storage::jcode_dir().unwrap();
            register_claim_worker(&worker);
            let registry = Registry::new(Arc::new(UnusedProvider)).await;
            assert!(file_claims_held_by(&worker).is_empty());
            std::fs::write(home.join(format!("ready-{worker}")), "").unwrap();
            let started = Instant::now();
            while !home.join("start").exists() {
                assert!(
                    started.elapsed() < Duration::from_secs(15),
                    "process barrier timed out"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            let ctx = ToolContext {
                session_id: worker.clone(),
                message_id: "m".into(),
                tool_call_id: "t".into(),
                working_dir: Some(home.clone()),
                stdin_request_tx: None,
                graceful_shutdown_signal: None,
                execution_mode: ToolExecutionMode::Direct,
            };
            let result = registry
                .execute(
                    "write",
                    serde_json::json!({"file_path":"shared", "content":worker}),
                    ctx,
                )
                .await;
            std::fs::write(
                home.join(format!("result-{worker}")),
                if result.is_ok() { "won" } else { "blocked" },
            )
            .unwrap();
            return;
        }
        let home = tempfile::tempdir().unwrap();
        std::fs::write(
            home.path().join("config.toml"),
            "[swarm]\nfile_claims = 'block'\n",
        )
        .unwrap();
        let mut children = Vec::new();
        for worker in ["process-a", "process-b"] {
            let log = std::fs::File::create(home.path().join(format!("log-{worker}"))).unwrap();
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "server::swarm::claims::claim_tests::file_claim_processes_share_one_owner",
                    "--nocapture",
                ])
                .env("JCODE_HOME", home.path())
                .env("JCODE_CLAIM_PROCESS_TEST", worker)
                .stdout(log.try_clone().unwrap())
                .stderr(log)
                .spawn()
                .unwrap();
            children.push((worker, child));
        }
        let started = Instant::now();
        while !children
            .iter()
            .all(|(worker, _)| home.path().join(format!("ready-{worker}")).exists())
            && started.elapsed() < Duration::from_secs(15)
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        std::fs::write(home.path().join("start"), "").unwrap();
        for (worker, child) in &mut children {
            assert!(
                child.wait().unwrap().success(),
                "{}",
                std::fs::read_to_string(home.path().join(format!("log-{worker}"))).unwrap()
            );
        }
        let winners: Vec<_> = children
            .iter()
            .filter(|(worker, _)| {
                std::fs::read_to_string(home.path().join(format!("result-{worker}"))).unwrap()
                    == "won"
            })
            .collect();
        assert_eq!(
            winners.len(),
            1,
            "two processes acquired the same block-mode claim"
        );
        let owner = winners[0].0;
        assert_eq!(
            std::fs::read_to_string(home.path().join("shared")).unwrap(),
            owner
        );
        let claims: BTreeMap<String, String> = serde_json::from_slice(
            &std::fs::read(home.path().join("swarm-file-claims.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            claims[&canonical_key(&home.path().join("shared")).unwrap()],
            owner
        );
    }

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
        if mode == "block" {
            let path = dir.path().join("overlap");
            let older = claim_files("holder", [path.clone()]).unwrap();
            registry
                .execute(
                    "write",
                    json!({"file_path":"overlap", "content":"newer success"}),
                    ctx("holder"),
                )
                .await
                .unwrap();
            drop(older);
            assert!(
                registry
                    .execute(
                        "write",
                        json!({"file_path":"overlap", "content":"bypass"}),
                        ctx("contender")
                    )
                    .await
                    .is_err(),
                "older rollback permitted a conflicting real tool write"
            );
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "newer success");
            let later = claim_files("holder", [path]).unwrap();
            drop(later);
            assert!(
                registry
                    .execute(
                        "write",
                        json!({"file_path":"overlap", "content":"bypass"}),
                        ctx("contender")
                    )
                    .await
                    .is_err(),
                "failed later reservation erased prior successful write"
            );
            release_file_claims_of("holder");
        }
        std::fs::create_dir(dir.path().join("directory")).unwrap();
        std::fs::write(dir.path().join("mismatch"), "actual\n").unwrap();
        for (name, input) in [
            ("write", json!({"file_path":"directory", "content":"new"})),
            (
                "edit",
                json!({"file_path":"mismatch", "old_string":"absent", "new_string":"new"}),
            ),
        ] {
            assert!(
                registry
                    .execute(name, input, ctx("contender"))
                    .await
                    .is_err()
            );
            assert!(
                file_claims_held_by("contender").is_empty(),
                "failed {name} reserved a file"
            );
        }
        for (name, input) in [
            (
                "edit",
                json!({"file_path":"missing", "old_string":"old", "new_string":"new"}),
            ),
            (
                "apply_patch",
                json!({"patch_text":"*** Begin Patch\n*** Update File: missing\n@@\n-old\n+new\n*** End Patch"}),
            ),
            (
                "apply_patch",
                json!({"patch_text":"--- a/missing\n+++ b/missing\n@@ -1 +1 @@\n-old\n+new\n"}),
            ),
        ] {
            let _ = registry.execute(name, input, ctx("contender")).await;
            assert!(!dir.path().join("missing").exists());
            assert!(
                file_claims_held_by("contender").is_empty(),
                "failed {name} reserved a file"
            );
        }
        register_claim_worker("partial");
        registry.execute("apply_patch", json!({"patch_text":"*** Begin Patch\n*** Add File: partial-success\n+written\n*** Update File: partial-failure\n@@\n-old\n+new\n*** End Patch"}), ctx("partial")).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.path().join("partial-success")).unwrap(),
            "written\n"
        );
        assert_eq!(
            file_claims_held_by("partial").len(),
            usize::from(mode != "off")
        );
        if mode != "off" {
            assert_eq!(
                file_claims_held_by("partial"),
                vec![canonical_key(&dir.path().join("partial-success")).unwrap()]
            );
        }
        unregister_claim_worker("partial");
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
                assert!(file_claims_held_by("holder").contains(&canonical_key(&path).unwrap()));
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
                assert!(
                    file_claims_held_by("contender").is_empty(),
                    "blocked batch reserved a file"
                );
                assert_eq!(
                    std::fs::read_to_string(dir.path().join("write")).unwrap(),
                    "new\n"
                );
            }
        }
        if mode != "off" {
            let mut persisted = FileClaimStore::load(Some(
                crate::storage::jcode_dir()
                    .unwrap()
                    .join("swarm-file-claims.json"),
            ));
            assert_eq!(
                persisted
                    .active()
                    .unwrap()
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
            assert!(formatted.contains(&canonical_key(&dir.path().join("write")).unwrap()));
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
            crate::server::swarm_persistence::record_worker_spawn_spec(
                "holder",
                &crate::server::swarm_persistence::WorkerSpawnSpec::default(),
            );
            let recovered = crate::server::swarm_persistence::load_runtime_state();
            assert_eq!(recovered.members["holder"].status, "orphaned");
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
        #[cfg(unix)]
        if mode == "block" {
            register_claim_worker("holder");
            let target = dir.path().join("dangling-target");
            let alias = dir.path().join("dangling-alias");
            std::os::unix::fs::symlink(&target, &alias).unwrap();
            registry
                .execute(
                    "write",
                    json!({"file_path":alias, "content":"holder"}),
                    ctx("holder"),
                )
                .await
                .unwrap();
            let error = registry
                .execute(
                    "write",
                    json!({"file_path":target, "content":"contender"}),
                    ctx("contender"),
                )
                .await
                .unwrap_err();
            assert!(error.to_string().contains("holder"));
            assert_eq!(std::fs::read_to_string(&target).unwrap(), "holder");
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
        assert_eq!(reloaded.active().unwrap().len(), 1);
        assert_eq!(reloaded.active().unwrap()[0].1, "a");

        // Stop/orphan release frees the path for B.
        assert_eq!(reloaded.release_session("a").unwrap().len(), 1);
        assert_eq!(
            reloaded.claim("b", &file, FileClaimMode::Block, false),
            ClaimOutcome::Granted
        );
        assert_eq!(
            FileClaimStore::load(Some(persist)).active().unwrap()[0].1,
            "b"
        );
    }

    #[test]
    fn mode_parse_defaults_to_warn() {
        assert_eq!(FileClaimMode::parse("block"), FileClaimMode::Block);
        assert_eq!(FileClaimMode::parse("OFF"), FileClaimMode::Off);
        assert_eq!(FileClaimMode::parse(""), FileClaimMode::Warn);
    }
}
