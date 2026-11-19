//! Durable session authority, separate from mutable session/swarm projections.

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlState {
    Runnable,
    StopRequested,
    Draining,
    Stopped,
    Unresolved,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ControlRecord {
    pub schema_version: u32,
    pub session_id: String,
    pub owner_session_id: Option<String>,
    pub revision: u64,
    pub last_allocated_generation: u64,
    pub admitted_generation: Option<u64>,
    pub revoked_through: u64,
    pub state: ControlState,
    pub latest_stop_request: Option<String>,
    #[serde(default)]
    pub unresolved_categories: HashSet<String>,
}

impl ControlRecord {
    fn new(id: &str, owner: Option<&str>) -> Self {
        Self {
            schema_version: 1,
            session_id: id.into(),
            owner_session_id: owner.map(str::to_owned),
            revision: 0,
            last_allocated_generation: 0,
            admitted_generation: None,
            revoked_through: 0,
            state: ControlState::Runnable,
            latest_stop_request: None,
            unresolved_categories: HashSet::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GenerationToken {
    pub session_id: String,
    pub generation: u64,
    root: PathBuf,
}

impl GenerationToken {
    pub(crate) fn reserve(&self) -> Result<WorkGuard> {
        ControlStore::new(self.root.clone()).reserve(self)
    }

    pub(crate) fn mark_external_effect(&self, category: &str) -> Result<()> {
        ControlStore::new(self.root.clone()).mark_external_effect(self, category)
    }
}

#[derive(Default)]
struct ControlEntry {
    fail_closed: bool,
    outstanding: HashMap<u64, usize>,
    changed: Arc<tokio::sync::Notify>,
}

// Entries retain persistence-failure barriers for this daemon's lifetime.
static ENTRIES: LazyLock<Mutex<HashMap<PathBuf, Arc<Mutex<ControlEntry>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) struct WorkGuard {
    entry: Arc<Mutex<ControlEntry>>,
    pub token: GenerationToken,
}

impl Drop for WorkGuard {
    fn drop(&mut self) {
        let mut entry = self.entry.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(count) = entry.outstanding.get_mut(&self.token.generation) {
            *count -= 1;
            if *count == 0 {
                entry.outstanding.remove(&self.token.generation);
            }
        }
        entry.changed.notify_waiters();
    }
}

#[derive(Clone)]
pub(crate) struct ControlStore {
    root: PathBuf,
}

impl ControlStore {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn path(&self, id: &str) -> Result<PathBuf> {
        ensure!(
            !id.is_empty()
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
            "Invalid session control identity"
        );
        Ok(self.root.join(format!("{id}.json")))
    }

    fn entry(&self, id: &str) -> Result<Arc<Mutex<ControlEntry>>> {
        let path = self.path(id)?;
        let mut entries = ENTRIES
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control registry poisoned"))?;
        Ok(Arc::clone(entries.entry(path).or_default()))
    }

    pub(crate) fn inspect(&self, id: &str) -> Result<Option<ControlRecord>> {
        let bytes = match std::fs::read(self.path(id)?) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e).context("Cannot read session control"),
        };
        let record: ControlRecord =
            serde_json::from_slice(&bytes).context("Corrupt session control, bytes preserved")?;
        ensure!(
            record.schema_version == 1,
            "Unsupported session control version, bytes preserved"
        );
        ensure!(record.session_id == id, "Session control identity mismatch");
        ensure!(
            record.revoked_through <= record.last_allocated_generation,
            "Invalid revoked generation"
        );
        if let Some(generation) = record.admitted_generation {
            ensure!(
                generation > record.revoked_through
                    && generation <= record.last_allocated_generation,
                "Invalid admitted generation"
            );
        }
        Ok(Some(record))
    }

    fn required(&self, id: &str) -> Result<ControlRecord> {
        self.inspect(id)?
            .with_context(|| format!("Unknown session '{id}'"))
    }

    pub(crate) fn register(&self, id: &str, owner: Option<&str>) -> Result<ControlRecord> {
        let entry = self.entry(id)?;
        let _guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        if let Some(record) = self.inspect(id)? {
            ensure!(
                record.owner_session_id.as_deref() == owner,
                "Session control owner is immutable"
            );
            return Ok(record);
        }
        let mut ancestor = owner.map(str::to_owned);
        let mut seen = HashSet::from([id.to_owned()]);
        while let Some(parent) = ancestor {
            self.path(&parent)?;
            ensure!(seen.insert(parent.clone()), "Session owner cycle");
            ancestor = self.inspect(&parent)?.and_then(|r| r.owner_session_id);
        }
        let record = ControlRecord::new(id, owner);
        crate::storage::write_json(&self.path(id)?, &record)?;
        Ok(record)
    }

    fn save_next(&self, record: &mut ControlRecord) -> Result<()> {
        record.revision = record
            .revision
            .checked_add(1)
            .context("Session control revision exhausted")?;
        crate::storage::write_json(&self.path(&record.session_id)?, record)
    }

    #[cfg(test)]
    fn replace(&self, expected_revision: u64, record: ControlRecord) -> Result<()> {
        let entry = self.entry(&record.session_id)?;
        let _guard = entry.lock().unwrap();
        ensure!(
            self.required(&record.session_id)?.revision == expected_revision,
            "Stale session control revision"
        );
        crate::storage::write_json(&self.path(&record.session_id)?, &record)
    }

    fn allocate(&self, record: &mut ControlRecord) -> Result<GenerationToken> {
        record.last_allocated_generation = record
            .last_allocated_generation
            .checked_add(1)
            .context("Execution generation exhausted")?;
        record.admitted_generation = Some(record.last_allocated_generation);
        self.save_next(record)?;
        Ok(GenerationToken {
            session_id: record.session_id.clone(),
            generation: record.last_allocated_generation,
            root: self.root.clone(),
        })
    }

    pub(crate) fn admit(&self, id: &str) -> Result<GenerationToken> {
        let entry = self.entry(id)?;
        let guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        ensure!(
            guard.outstanding.is_empty(),
            "Previous session generation remains active"
        );
        let mut record = self.required(id)?;
        ensure!(
            record.state == ControlState::Runnable,
            "Session '{id}' admission revoked: {:?}",
            record.state
        );
        self.allocate(&mut record)
    }

    pub(crate) fn runnable(&self, id: &str) -> Result<()> {
        let entry = self.entry(id)?;
        let guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        if let Some(record) = self.inspect(id)? {
            ensure!(
                record.state == ControlState::Runnable,
                "Session '{id}' admission revoked: {:?}",
                record.state
            );
        }
        Ok(())
    }

    pub(crate) fn reserve(&self, token: &GenerationToken) -> Result<WorkGuard> {
        ensure!(
            token.root == self.root,
            "Execution control namespace mismatch"
        );
        let entry = self.entry(&token.session_id)?;
        {
            let mut guard = entry
                .lock()
                .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
            ensure!(!guard.fail_closed, "Session control persistence unresolved");
            let record = self.required(&token.session_id)?;
            ensure!(
                record.state == ControlState::Runnable
                    && record.admitted_generation == Some(token.generation)
                    && token.generation > record.revoked_through,
                "Execution generation revoked"
            );
            *guard.outstanding.entry(token.generation).or_default() += 1;
        }
        Ok(WorkGuard {
            entry,
            token: token.clone(),
        })
    }

    pub(crate) fn authorized(&self, requester: &str, target: &str, force: bool) -> Result<()> {
        let target_record = self.required(target)?;
        if force || requester == target {
            return Ok(());
        }
        let mut ancestor = target_record.owner_session_id;
        let mut seen = HashSet::from([target.to_owned()]);
        while let Some(parent) = ancestor {
            ensure!(seen.insert(parent.clone()), "Session owner cycle");
            if parent == requester {
                return Ok(());
            }
            ancestor = self
                .inspect(&parent)?
                .and_then(|record| record.owner_session_id);
        }
        bail!(
            "Refusing to stop session '{target}' because it is not owned by this requester. Pass force=true explicitly for non-owned control"
        )
    }

    pub(crate) fn request_stop(
        &self,
        requester: &str,
        target: &str,
        force: bool,
        request: &str,
    ) -> Result<ControlRecord> {
        let entry = self.entry(target)?;
        let mut guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        // Do not introduce a stop barrier on an unauthorized target.
        if let Err(error) = self.required(target) {
            guard.fail_closed = true;
            return Err(error);
        }
        self.authorized(requester, target, force)?;
        guard.fail_closed = true;
        let mut record = self.required(target)?;
        if record.state != ControlState::Runnable {
            guard.fail_closed = false;
            return Ok(record);
        }
        record.state = ControlState::StopRequested;
        record.revoked_through = record.last_allocated_generation;
        record.admitted_generation = None;
        record.latest_stop_request = Some(request.into());
        self.save_next(&mut record)?;
        guard.fail_closed = false;
        Ok(record)
    }

    pub(crate) fn project<T>(
        &self,
        id: &str,
        status: &str,
        update: impl FnOnce() -> T,
    ) -> Result<T> {
        let entry = self.entry(id)?;
        let guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        if let Some(record) = self.inspect(id)? {
            ensure!(
                record.state == ControlState::Runnable
                    || (record.state == ControlState::Stopped && status == "stopped"),
                "Revoked session projection rejected"
            );
        }
        Ok(update())
    }

    pub(crate) fn work_observation(&self, id: &str) -> Result<(usize, Arc<tokio::sync::Notify>)> {
        let entry = self.entry(id)?;
        let guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        Ok((guard.outstanding.values().sum(), Arc::clone(&guard.changed)))
    }

    pub(crate) fn mark_external_effect(
        &self,
        token: &GenerationToken,
        category: &str,
    ) -> Result<()> {
        ensure!(
            token.root == self.root,
            "Execution control namespace mismatch"
        );
        let entry = self.entry(&token.session_id)?;
        let mut guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        let mut record = self.required(&token.session_id)?;
        ensure!(
            record.state == ControlState::Runnable
                && record.admitted_generation == Some(token.generation),
            "Execution generation revoked"
        );
        if record.unresolved_categories.insert(category.into())
            && let Err(error) = self.save_next(&mut record)
        {
            guard.fail_closed = true;
            return Err(error);
        }
        Ok(())
    }

    fn stop_state(&self, id: &str, state: ControlState) -> Result<ControlRecord> {
        let entry = self.entry(id)?;
        let mut guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        let mut record = self.required(id)?;
        ensure!(
            record.state != ControlState::Runnable,
            "Session stop was not requested"
        );
        if record.state != state && record.state != ControlState::Stopped {
            record.state = state;
            if let Err(error) = self.save_next(&mut record) {
                guard.fail_closed = true;
                return Err(error);
            }
        }
        Ok(record)
    }

    pub(crate) async fn drain(
        &self,
        id: &str,
        deadline: std::time::Duration,
    ) -> Result<ControlRecord> {
        let record = self.stop_state(id, ControlState::Draining)?;
        if record.state == ControlState::Stopped {
            return Ok(record);
        }
        super::swarm::cancel_active_turns(id);
        let expires = tokio::time::Instant::now() + deadline;
        loop {
            let (_, changed) = self.work_observation(id)?;
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let (outstanding, _) = self.work_observation(id)?;
            let record = self.required(id)?;
            if !record.unresolved_categories.is_empty() {
                return self.stop_state(id, ControlState::Unresolved);
            }
            if outstanding == 0 {
                return self.settle(id, record.revision);
            }
            if tokio::time::timeout_at(expires, notified).await.is_err() {
                return self.stop_state(id, ControlState::Unresolved);
            }
        }
    }

    // Only the settlement owner may call this after accounting for blocking IO,
    // owned processes and unknown external work. A dropped turn alone is insufficient.
    pub(crate) fn settle(&self, id: &str, expected_revision: u64) -> Result<ControlRecord> {
        let entry = self.entry(id)?;
        let guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        ensure!(
            guard.outstanding.is_empty(),
            "Session work remains outstanding"
        );
        let mut record = self.required(id)?;
        ensure!(
            record.unresolved_categories.is_empty(),
            "Unknown external effects remain unresolved"
        );
        ensure!(
            record.revision == expected_revision,
            "Stale session control revision"
        );
        ensure!(
            matches!(
                record.state,
                ControlState::StopRequested | ControlState::Draining | ControlState::Unresolved
            ),
            "Session stop was not requested"
        );
        record.state = ControlState::Stopped;
        self.save_next(&mut record)?;
        Ok(record)
    }

    #[cfg(test)]
    pub(crate) fn resume(&self, id: &str, expected_revision: u64) -> Result<GenerationToken> {
        let entry = self.entry(id)?;
        let guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(
            !guard.fail_closed && guard.outstanding.is_empty(),
            "Predecessor work remains unresolved"
        );
        let mut record = self.required(id)?;
        ensure!(
            record.revision == expected_revision && record.state == ControlState::Stopped,
            "Resume predecessor conflict"
        );
        record.state = ControlState::Runnable;
        self.allocate(&mut record)
    }
}

pub(crate) fn import_identity(id: &str, owner_hint: Option<&str>) -> Result<ControlRecord> {
    let store = store();
    if let Some(record) = store.inspect(id)? {
        return Ok(record);
    }
    let owner = match crate::session::Session::load_startup_stub(id) {
        Ok(session) => session.parent_id,
        Err(_) => owner_hint.map(str::to_owned),
    };
    if let Some(parent) = owner.as_deref()
        && store.inspect(parent)?.is_none()
    {
        let parent_owner = crate::session::Session::load_startup_stub(parent)
            .ok()
            .and_then(|s| s.parent_id);
        store.register(parent, parent_owner.as_deref())?;
    }
    store.register(id, owner.as_deref())
}

pub(crate) fn store() -> ControlStore {
    ControlStore::new(crate::storage::durable_state_dir().join("session-control"))
}

tokio::task_local! { static EXECUTION: GenerationToken; }

pub(crate) fn current_token() -> Option<GenerationToken> {
    EXECUTION.try_with(Clone::clone).ok()
}

pub(crate) async fn scope<F: Future>(token: Option<GenerationToken>, future: F) -> F::Output {
    let future = Box::pin(future);
    match token {
        Some(token) => EXECUTION.scope(token, future).await,
        None => future.await,
    }
}

pub(crate) async fn run<F: Future>(id: &str, owner: Option<&str>, future: F) -> Result<F::Output> {
    let future = Box::pin(future);
    let store = store();
    store.register(id, owner)?;
    let token = store.admit(id)?;
    let _work = store.reserve(&token)?;
    Ok(scope(Some(token), future).await)
}

pub(crate) fn reserve_current(id: &str) -> Result<Option<WorkGuard>> {
    let store = store();
    if let Some(token) = current_token() {
        ensure!(
            token.session_id == id,
            "Execution session identity mismatch"
        );
        return token.reserve().map(Some);
    }
    // Legacy/direct tool invocations cannot borrow an admitted generation.
    store.runnable(id)?;
    ensure!(
        store.inspect(id)?.is_none(),
        "Missing execution generation for controlled session"
    );
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoked_projection_cannot_publish_ready_or_release_claims() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        store.admit("worker").unwrap();
        let stopped = store
            .request_stop("worker", "worker", false, "stop")
            .unwrap();
        let mut projection = "running";
        assert!(
            store
                .project("worker", "ready", || projection = "ready")
                .is_err()
        );
        assert!(
            store
                .project("worker", "stopped", || projection = "stopped")
                .is_err()
        );
        assert_eq!(projection, "running");
        store.settle("worker", stopped.revision).unwrap();
        store
            .project("worker", "stopped", || projection = "stopped")
            .unwrap();
        assert_eq!(projection, "stopped");
        assert!(
            store
                .project("worker", "completed", || projection = "completed")
                .is_err()
        );
        assert_eq!(projection, "stopped");
    }

    #[tokio::test]
    async fn stop_signals_active_turn_and_waits_for_reserved_work() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("stop_signal_worker", None).unwrap();
        let token = store.admit("stop_signal_worker").unwrap();
        let work = token.reserve().unwrap();
        let signal = jcode_agent_runtime::InterruptSignal::new();
        let active =
            crate::turn_cancel_registry::register_active_turn("stop_signal_worker", signal.clone());
        let stopped = store
            .request_stop("stop_signal_worker", "stop_signal_worker", false, "stop")
            .unwrap();
        let waiting = store.clone();
        let task = tokio::spawn(async move {
            waiting
                .drain("stop_signal_worker", std::time::Duration::from_secs(2))
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(1), signal.notified())
            .await
            .unwrap();
        assert!(
            !task.is_finished(),
            "stop must not settle while real work exists"
        );
        drop(active);
        drop(work);
        let settled = task.await.unwrap().unwrap();
        assert_eq!(settled.state, ControlState::Stopped);
        assert!(settled.revision > stopped.revision);
        assert_eq!(
            store
                .drain("stop_signal_worker", std::time::Duration::ZERO)
                .await
                .unwrap(),
            settled
        );
    }

    #[tokio::test]
    async fn settlement_deadline_and_unknown_effect_preserve_revocation() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("deadline_worker", None).unwrap();
        let token = store.admit("deadline_worker").unwrap();
        let work = token.reserve().unwrap();
        store
            .request_stop("deadline_worker", "deadline_worker", false, "stop")
            .unwrap();
        let unresolved = store
            .drain("deadline_worker", std::time::Duration::ZERO)
            .await
            .unwrap();
        assert_eq!(unresolved.state, ControlState::Unresolved);
        assert!(store.admit("deadline_worker").is_err());
        drop(work);
        assert_eq!(
            store
                .drain("deadline_worker", std::time::Duration::from_secs(1))
                .await
                .unwrap()
                .state,
            ControlState::Stopped
        );

        store.register("external_worker", None).unwrap();
        let external = store.admit("external_worker").unwrap();
        store
            .mark_external_effect(&external, "shell_children")
            .unwrap();
        store
            .request_stop("external_worker", "external_worker", false, "stop")
            .unwrap();
        let restarted = ControlStore::new(root.path().to_owned());
        let unknown = restarted
            .drain("external_worker", std::time::Duration::from_secs(1))
            .await
            .unwrap();
        assert_eq!(unknown.state, ControlState::Unresolved);
        assert!(unknown.unresolved_categories.contains("shell_children"));
        assert!(
            restarted
                .settle("external_worker", unknown.revision)
                .is_err()
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
        ) -> Result<crate::provider::EventStream> {
            bail!("native admission test must not call provider")
        }
        fn name(&self) -> &str {
            "unused"
        }
        fn fork(&self) -> Arc<dyn crate::provider::Provider> {
            Arc::new(Self)
        }
    }

    #[test]
    fn native_write_without_token_or_after_revocation_is_denied() {
        let _lock = crate::storage::lock_test_env();
        let root = tempfile::tempdir().unwrap();
        struct Restore(Option<std::ffi::OsString>);
        impl Drop for Restore {
            fn drop(&mut self) {
                match self.0.take() {
                    Some(value) => crate::env::set_var("JCODE_RUNTIME_DIR", value),
                    None => crate::env::remove_var("JCODE_RUNTIME_DIR"),
                }
            }
        }
        let _restore = Restore(std::env::var_os("JCODE_RUNTIME_DIR"));
        crate::env::set_var("JCODE_RUNTIME_DIR", root.path());
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let store = store();
            store.register("worker", None).unwrap();
            let token = store.admit("worker").unwrap();
            let registry = crate::tool::Registry::new(Arc::new(UnusedProvider)).await;
            let marker = root.path().join("marker.txt");
            let input = serde_json::json!({"file_path":marker,"content":"before-stop"});
            let ctx = crate::tool::ToolContext {
                session_id: "worker".into(),
                message_id: "message".into(),
                tool_call_id: "write".into(),
                working_dir: Some(root.path().to_owned()),
                stdin_request_tx: None,
                graceful_shutdown_signal: None,
                execution_mode: crate::tool::ToolExecutionMode::AgentTurn,
            };
            assert!(
                registry
                    .execute("write", input.clone(), ctx.clone())
                    .await
                    .is_err()
            );
            assert!(!marker.exists());
            scope(
                Some(token.clone()),
                registry.execute("write", input.clone(), ctx.clone()),
            )
            .await
            .unwrap();
            assert_eq!(std::fs::read_to_string(&marker).unwrap(), "before-stop");
            store
                .request_stop("worker", "worker", false, "stop")
                .unwrap();
            let after = serde_json::json!({"file_path":marker,"content":"revoked"});
            assert!(
                scope(Some(token), registry.execute("write", after, ctx))
                    .await
                    .is_err()
            );
            assert_eq!(std::fs::read_to_string(&marker).unwrap(), "before-stop");
            assert!(run("worker", None, async {}).await.is_err());
        });
    }

    #[tokio::test]
    async fn explicit_detached_token_scope_does_not_borrow_new_generation() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let old = store.admit("worker").unwrap();
        let captured = scope(Some(old.clone()), async { current_token().unwrap() }).await;
        let stop = store
            .request_stop("worker", "worker", false, "stop")
            .unwrap();
        let settled = store.settle("worker", stop.revision).unwrap();
        let fresh = store.resume("worker", settled.revision).unwrap();
        tokio::spawn(scope(Some(captured), async move {
            assert_eq!(current_token().unwrap(), old);
            assert!(store.reserve(&current_token().unwrap()).is_err());
            assert!(store.reserve(&fresh).is_ok());
        }))
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn moved_blocking_reservation_notifies_only_after_real_work_drops() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let token = store.admit("worker").unwrap();
        let reservation = store.reserve(&token).unwrap();
        let (count, changed) = store.work_observation("worker").unwrap();
        assert_eq!(count, 1);
        let notified = changed.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        let (release, wait) = std::sync::mpsc::channel();
        let task = tokio::task::spawn_blocking(move || {
            wait.recv().unwrap();
            drop(reservation);
        });
        let stop = store
            .request_stop("worker", "worker", false, "stop")
            .unwrap();
        assert!(store.settle("worker", stop.revision).is_err());
        assert_eq!(store.work_observation("worker").unwrap().0, 1);
        release.send(()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), notified)
            .await
            .unwrap();
        task.await.unwrap();
        assert_eq!(store.work_observation("worker").unwrap().0, 0);
        assert!(store.settle("worker", stop.revision).is_ok());
    }

    #[test]
    fn simultaneous_stop_and_admission_serialize_at_record_lock() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let runner = store.clone();
        let gate = Arc::clone(&barrier);
        let thread = std::thread::spawn(move || {
            gate.wait();
            runner.admit("worker")
        });
        barrier.wait();
        let stop = store
            .request_stop("worker", "worker", false, "stop")
            .unwrap();
        if let Ok(token) = thread.join().unwrap() {
            assert!(token.generation <= stop.revoked_through);
            assert!(store.reserve(&token).is_err());
        }
        assert!(store.admit("worker").is_err());
    }

    #[test]
    fn stop_survives_restart_and_identity_registration() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("parent", None).unwrap();
        store.register("worker", Some("parent")).unwrap();
        let token = store.admit("worker").unwrap();
        let stopped = store
            .request_stop("parent", "worker", false, "stop-1")
            .unwrap();
        assert_eq!(stopped.state, ControlState::StopRequested);
        assert_eq!(stopped.revoked_through, token.generation);
        let restarted = ControlStore::new(root.path().to_owned());
        assert!(restarted.admit("worker").is_err());
        restarted.register("worker", Some("parent")).unwrap();
        assert!(restarted.admit("worker").is_err());
        assert_eq!(restarted.inspect("worker").unwrap().unwrap(), stopped);
        assert_eq!(
            restarted
                .request_stop("parent", "worker", false, "stop-1")
                .unwrap(),
            stopped
        );
    }

    #[test]
    fn retained_owner_is_immutable_and_transitive() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("root", None).unwrap();
        store.register("parent", Some("root")).unwrap();
        store.register("worker", Some("parent")).unwrap();
        assert!(store.register("root", Some("other")).is_err());
        assert!(store.register("worker", None).is_err());
        assert!(
            store
                .request_stop("stranger", "worker", false, "bad")
                .is_err()
        );
        assert!(store.request_stop("root", "worker", false, "good").is_ok());
        assert!(
            store
                .request_stop("stranger", "worker", true, "override")
                .is_ok()
        );
    }

    #[test]
    fn corrupt_unsupported_and_stale_records_fail_closed_without_overwrite() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        let initial = store.register("worker", None).unwrap();
        let mut stale = initial.clone();
        store.admit("worker").unwrap();
        stale.revision += 1;
        assert!(store.replace(initial.revision, stale).is_err());
        let path = store.path("worker").unwrap();
        for bytes in [b"not-json".to_vec(), {
            let mut unsupported = initial.clone();
            unsupported.schema_version = 999;
            serde_json::to_vec(&unsupported).unwrap()
        }] {
            std::fs::write(&path, &bytes).unwrap();
            assert!(store.inspect("worker").is_err());
            assert!(store.register("worker", None).is_err());
            assert!(store.admit("worker").is_err());
            assert_eq!(std::fs::read(&path).unwrap(), bytes);
        }
    }

    #[test]
    fn failed_stop_persistence_retains_local_barrier() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let token = store.admit("worker").unwrap();
        let path = store.path("worker").unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(
            store
                .request_stop("worker", "worker", false, "stop-1")
                .is_err()
        );
        std::fs::remove_dir(&path).unwrap();
        // Even if a stale snapshot later appears, failed durable stop is not runnable.
        crate::storage::write_json(&path, &ControlRecord::new("worker", None)).unwrap();
        assert!(store.admit("worker").is_err());
        assert!(store.reserve(&token).is_err());
    }

    #[test]
    fn stopped_generation_cannot_borrow_resumed_authority_or_settle_live_work() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let old = store.admit("worker").unwrap();
        let work = store.reserve(&old).unwrap();
        let stop = store
            .request_stop("worker", "worker", false, "stop-1")
            .unwrap();
        assert!(store.reserve(&old).is_err());
        assert!(store.settle("worker", stop.revision).is_err());
        drop(work);
        let settled = store.settle("worker", stop.revision).unwrap();
        assert_eq!(settled.state, ControlState::Stopped);
        let fresh = store.resume("worker", settled.revision).unwrap();
        assert!(fresh.generation > old.generation);
        assert!(store.reserve(&old).is_err());
        assert!(store.reserve(&fresh).is_ok());
        assert!(store.resume("worker", settled.revision).is_err());
    }

    #[test]
    fn traversal_and_owner_cycles_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        assert!(store.register("../escape", None).is_err());
        assert!(store.register("self", Some("self")).is_err());
        store.register("first", Some("second")).unwrap();
        assert!(store.register("second", Some("first")).is_err());
    }
}
