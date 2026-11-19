//! Durable session authority, separate from mutable session/swarm projections.

use anyhow::{Context, Result, bail, ensure};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

pub(crate) use jcode_swarm_core::control::{ControlRecord, ControlState, OwnedProcess};

/// Category kept in `unresolved_categories` while owned shell children are alive.
pub(crate) const SHELL_CHILDREN: &str = "shell_children";

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

    /// Take ownership of a shell child before spawning it. Call
    /// [`OwnedChild::set_pid`] right after the spawn. `shell_children` clears
    /// only once that process is proven gone.
    pub(crate) fn own_shell_child(&self) -> Result<OwnedChild> {
        ControlStore::new(self.root.clone()).own_shell_child(self)
    }
}

/// How long a dropped [`OwnedChild`] keeps polling for its process to exit.
const CHILD_POLL_WINDOW: std::time::Duration = std::time::Duration::from_secs(600);

/// One spawned shell child owned by a generation. Dropping it never claims
/// settlement by itself: the record keeps the pid until the process is gone.
pub(crate) struct OwnedChild {
    token: GenerationToken,
    pid: Option<u32>,
}

impl OwnedChild {
    pub(crate) fn set_pid(&mut self, pid: u32) {
        self.pid = Some(pid);
        ControlStore::new(self.token.root.clone()).record_child(&self.token, pid);
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        let store = ControlStore::new(self.token.root.clone());
        let id = self.token.session_id.clone();
        let Some(pid) = self.pid else {
            // Never spawned: nothing to prove.
            store.release_pending_child(&id);
            return;
        };
        let _ = store.reconcile_children(&id);
        if store.child_listed(&id, pid)
            && let Ok(handle) = tokio::runtime::Handle::try_current()
        {
            // Bounded: a child that outlives the poll window stays listed and
            // `shell_children` stays unresolved; every later drain or stop
            // reconciles it again, so giving up here never claims settlement.
            handle.spawn(async move {
                let give_up = tokio::time::Instant::now() + CHILD_POLL_WINDOW;
                while tokio::time::Instant::now() < give_up {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    // Record IO and the /proc scan must not run on the async worker.
                    let probe = store.clone();
                    let probe_id = id.clone();
                    let listed = tokio::task::spawn_blocking(move || {
                        let _ = probe.reconcile_children(&probe_id);
                        probe.child_listed(&probe_id, pid)
                    })
                    .await
                    .unwrap_or(false);
                    if !listed {
                        break;
                    }
                }
            });
        }
    }
}

/// True when the child and every member of its process group have exited
/// (reaped, zombie, or pid reused). Children run as session leaders, so
/// `pgid == pid`; a backgrounded grandchild keeps the effect unresolved.
fn process_gone(process: &OwnedProcess) -> bool {
    #[cfg(target_os = "linux")]
    {
        let leader_gone = match std::fs::read_to_string(format!("/proc/{}/stat", process.pid)) {
            Err(_) => true,
            Ok(stat) => match stat.rfind(')') {
                None => false,
                Some(close) => {
                    let fields: Vec<&str> = stat[close + 1..].split_whitespace().collect();
                    let start = fields.get(19).and_then(|v| v.parse::<u64>().ok());
                    (process.start_time.is_some() && start != process.start_time)
                        || matches!(fields.first(), Some(&"Z") | Some(&"X"))
                }
            },
        };
        leader_gone && !group_has_live_member(process.pid)
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        // killpg(0) succeeds while any member exists, zombies included.
        unsafe { libc::kill(-(process.pid as i32), 0) != 0 }
        &&std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    }
    #[cfg(not(unix))]
    {
        let _ = process;
        false
    }
}

#[cfg(target_os = "linux")]
fn group_has_live_member(pgid: u32) -> bool {
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return true;
    };
    dir.flatten().any(|entry| {
        let name = entry.file_name();
        if !name.to_string_lossy().bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
        let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else {
            return false;
        };
        let Some(close) = stat.rfind(')') else {
            return false;
        };
        let mut fields = stat[close + 1..].split_whitespace();
        let state = fields.next();
        let group = fields.nth(1).and_then(|v| v.parse::<u32>().ok());
        group == Some(pgid) && !matches!(state, Some("Z") | Some("X"))
    })
}

/// Start time (clock ticks since boot) used to recognise a reused pid.
fn process_start_time(pid: u32) -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let close = stat.rfind(')')?;
        stat[close + 1..].split_whitespace().nth(19)?.parse().ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

#[derive(Default)]
struct ControlEntry {
    fail_closed: bool,
    /// Owned children whose spawn has not yet reported a pid.
    pending_children: usize,
    outstanding: HashMap<u64, usize>,
    changed: Arc<tokio::sync::Notify>,
    /// Generation allocated by an explicit resume that no turn has claimed yet.
    /// The next turn adopts it instead of allocating another, so waits bound at
    /// resume time name the generation the run will really carry.
    pre_admitted: Option<u64>,
    /// Generation of the most recent run begun in this process. A turn-end
    /// report belongs to it, whatever has been admitted since.
    last_run: Option<u64>,
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

    pub(super) fn path(&self, id: &str) -> Result<PathBuf> {
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
        record.validate(id).map_err(anyhow::Error::msg)?;
        Ok(Some(record))
    }

    /// Control files this build refuses to serve (corrupt, newer schema, or a
    /// reader floor above this build). Bytes are never touched. Each entry is a
    /// `(session id, reason)` pair; the sessions fail closed on every admission.
    pub(crate) fn incompatible_records(&self) -> Vec<(String, String)> {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return Vec::new();
    /// Records whose owner chain reaches `root` (excluding `root`). Unreadable
    /// records are skipped: they cannot prove ownership, and `incompatible_records`
    /// already surfaces them.
    pub(crate) fn owned_records(&self, root: &str) -> Vec<ControlRecord> {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let all: HashMap<String, ControlRecord> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let id = name.strip_suffix(".json")?.to_string();
                Some((id.clone(), self.inspect(&id).ok().flatten()?))
            })
            .collect();
        let reaches_root = |record: &ControlRecord| {
            let mut owner = record.owner_session_id.as_deref();
            for _ in 0..16 {
                match owner {
                    Some(id) if id == root => return true,
                    Some(id) => owner = all.get(id).and_then(|r| r.owner_session_id.as_deref()),
                    None => return false,
                }
            }
            false
        };
        let mut owned: Vec<ControlRecord> =
            all.values().filter(|r| reaches_root(r)).cloned().collect();
        owned.sort_by(|a, b| a.session_id.cmp(&b.session_id));
        owned
    }

        };
        let mut found: Vec<(String, String)> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let id = name.strip_suffix(".json")?.to_string();
                self.inspect(&id).err().map(|e| (id, format!("{e:#}")))
            })
            .collect();
        found.sort();
        found
    }

    /// Highest `min_reader_version` over every persisted control record, or
    /// `None` when there are none. A record whose floor cannot be read counts as
    /// this build's own floor; an unreadable directory is an error (fail closed).
    pub(crate) fn required_reader_floor(&self) -> Result<Option<u32>> {
        let entries = match std::fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e).context("Cannot scan session control"),
        };
        let own = jcode_swarm_core::control::CONTROL_MIN_READER_VERSION;
        let mut floor = None;
        for entry in entries {
            let path = entry.context("Cannot scan session control")?.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let record_floor = std::fs::read(&path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                .map_or(own, |value| match value.get("min_reader_version") {
                    None => 1,
                    Some(v) => v
                        .as_u64()
                        .map_or(own, |v| u32::try_from(v).unwrap_or(u32::MAX)),
                });
            floor = floor.max(Some(record_floor));
        }
        Ok(floor)
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
        // A write by this build carries semantics older readers do not enforce,
        // so it raises the reader floor. Reads never rewrite a record.
        record.schema_version = jcode_swarm_core::control::CONTROL_SCHEMA_VERSION;
        record.min_reader_version = jcode_swarm_core::control::CONTROL_MIN_READER_VERSION;
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
        let mut guard = entry
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
        let adopted = guard.pre_admitted.take().filter(|generation| {
            record.admitted_generation == Some(*generation)
                && record.last_allocated_generation == *generation
        });
        let token = match adopted {
            Some(generation) => GenerationToken {
                session_id: record.session_id.clone(),
                generation,
                root: self.root.clone(),
            },
            None => self.allocate(&mut record)?,
        };
        guard.last_run = Some(token.generation);
        Ok(token)
    }

    /// True while a stop has been requested but not settled, or the record cannot
    /// be trusted. Automatic reassignment must not hand this session's work to a
    /// new owner, because the predecessor's native operations may still be running.
    pub(crate) fn predecessor_unsettled(&self, id: &str) -> bool {
        let fail_closed = self
            .entry(id)
            .map(|entry| entry.lock().map(|guard| guard.fail_closed).unwrap_or(true))
            .unwrap_or(true);
        fail_closed
            || match self.inspect(id) {
                Ok(None) => false,
                Ok(Some(record)) => {
                    !matches!(record.state, ControlState::Runnable | ControlState::Stopped)
                }
                Err(_) => true,
            }
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
        // No barrier before authorization: an unknown or unowned target must
        // not be locked out by a requester that may not stop it. A record that
        // cannot be read cannot prove ownership, so only a requester that needs
        // none (self or force) may install the barrier for it.
        match self.inspect(target) {
            Err(error) => {
                if force || requester == target {
                    guard.fail_closed = true;
                }
                return Err(error);
            }
            Ok(None) => bail!("Unknown session '{target}'"),
            Ok(Some(_)) => {}
        }
        self.authorized(requester, target, force)?;
        guard.fail_closed = true;
        let mut record = self.required(target)?;
        if record.state != ControlState::Runnable {
            guard.fail_closed = false;
            return Ok(record);
        }
        guard.pre_admitted = None;
        record.state = ControlState::StopRequested;
        record.revoked_through = record.last_allocated_generation;
        record.admitted_generation = None;
        record.latest_stop_request = Some(request.into());
        self.save_next(&mut record)?;
        guard.fail_closed = false;
        Ok(record)
    }

    #[cfg(test)]
    pub(crate) fn project<T>(
        &self,
        id: &str,
        status: &str,
        update: impl FnOnce() -> T,
    ) -> Result<T> {
        self.project_report(id, status, false, update)
    }

    /// Like [`project`], and when `report` is true also stamps the generation
    /// that produced the report. The stamp is written under the same entry lock
    /// and only after the revocation and stale-generation checks passed, so a
    /// revoked or older generation can never attribute a report to newer work.
    pub(crate) fn project_report<T>(
        &self,
        id: &str,
        status: &str,
        report: bool,
        update: impl FnOnce() -> T,
    ) -> Result<T> {
        let entry = self.entry(id)?;
        let guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        if let Some(mut record) = self.inspect(id)? {
            ensure!(
                record.state == ControlState::Runnable
                    || (record.state == ControlState::Stopped && status == "stopped"),
                "Revoked session projection rejected"
            );
            let token = current_token().filter(|token| token.session_id == id);
            // A report made from inside an older generation's scope must not
            // act on the generation admitted after a resume.
            if let Some(token) = &token {
                ensure!(
                    record.state != ControlState::Runnable
                        || record.admitted_generation == Some(token.generation),
                    "Stale generation projection rejected"
                );
            }
            if report {
                // The generation is the report's own: the running scope, else
                // the one the reporter named on the wire, else the last run of
                // this session (a turn-end report). Ambient admitted state is
                // never used, so a late report cannot be attributed to newer work.
                let generation = match &token {
                    Some(token) => Some(token.generation),
                    None => REPORTED_GENERATION
                        .try_with(|generation| *generation)
                        .unwrap_or(guard.last_run),
                };
                if let Some(generation) = generation.filter(|generation| *generation > 0) {
                    ensure!(
                        if record.state == ControlState::Runnable {
                            record.admitted_generation == Some(generation)
                        } else {
                            generation <= record.revoked_through
                        },
                        "Stale generation projection rejected"
                    );
                    if record
                        .report_generation
                        .is_none_or(|reported| reported < generation)
                    {
                        record.report_generation = Some(generation);
                        self.save_next(&mut record)?;
                    }
                }
            }
        }
        Ok(update())
    }

    /// Generation a completion wait binds to when it begins. A member
    /// with outstanding work is awaited at its current generation; otherwise
    /// the next turn will be the following generation. `running` is published
    /// before the generation is allocated, so status alone cannot decide this.
    /// `None` for sessions without a control record, which stay unbound.
    pub(crate) fn awaited_generation(&self, id: &str) -> Option<u64> {
        let record = self.inspect(id).ok().flatten()?;
        let (busy, pre_admitted) = self
            .entry(id)
            .ok()
            .map(|entry| {
                entry
                    .lock()
                    .map(|guard| (!guard.outstanding.is_empty(), guard.pre_admitted))
                    .unwrap_or((true, None))
            })
            .unwrap_or((true, None));
        // A resume already allocated the generation the next turn will adopt.
        if let Some(generation) = pre_admitted
            && record.admitted_generation == Some(generation)
        {
            return Some(generation);
        }
        Some(record.last_allocated_generation + u64::from(!busy))
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

    fn own_shell_child(&self, token: &GenerationToken) -> Result<OwnedChild> {
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
        if record.unresolved_categories.insert(SHELL_CHILDREN.into())
            && let Err(error) = self.save_next(&mut record)
        {
            guard.fail_closed = true;
            return Err(error);
        }
        guard.pending_children += 1;
        Ok(OwnedChild {
            token: token.clone(),
            pid: None,
        })
    }

    /// Persist the spawned pid. Allowed after revocation: the child exists now.
    fn record_child(&self, token: &GenerationToken, pid: u32) {
        let Ok(entry) = self.entry(&token.session_id) else {
            return;
        };
        let Ok(mut guard) = entry.lock() else {
            return;
        };
        guard.pending_children = guard.pending_children.saturating_sub(1);
        let saved = self.required(&token.session_id).and_then(|mut record| {
            record.owned_children.push(OwnedProcess {
                pid,
                start_time: process_start_time(pid),
            });
            record.unresolved_categories.insert(SHELL_CHILDREN.into());
            self.save_next(&mut record)
        });
        if saved.is_err() {
            guard.fail_closed = true;
        }
        guard.changed.notify_waiters();
    }

    fn release_pending_child(&self, id: &str) {
        if let Ok(entry) = self.entry(id)
            && let Ok(mut guard) = entry.lock()
        {
            guard.pending_children = guard.pending_children.saturating_sub(1);
        }
        let _ = self.reconcile_children(id);
    }

    fn child_listed(&self, id: &str, pid: u32) -> bool {
        self.inspect(id)
            .ok()
            .flatten()
            .is_some_and(|record| record.owned_children.iter().any(|c| c.pid == pid))
    }

    /// Drop children proven gone and clear `shell_children` once none remain
    /// and none are still spawning. Never clears anything else.
    pub(crate) fn reconcile_children(&self, id: &str) -> Result<()> {
        let entry = self.entry(id)?;
        let mut guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        let Some(mut record) = self.inspect(id)? else {
            return Ok(());
        };
        let before = record.owned_children.len();
        record.owned_children.retain(|child| !process_gone(child));
        let cleared = record.owned_children.is_empty()
            && guard.pending_children == 0
            && record.unresolved_categories.remove(SHELL_CHILDREN);
        if record.owned_children.len() != before || cleared {
            if let Err(error) = self.save_next(&mut record) {
                guard.fail_closed = true;
                return Err(error);
            }
            guard.changed.notify_waiters();
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
            // The /proc scan and record write stay off the async worker.
            let (store, owned_id) = (self.clone(), id.to_owned());
            tokio::task::spawn_blocking(move || store.reconcile_children(&owned_id))
                .await
                .map_err(|error| anyhow::anyhow!("Child reconciliation task failed: {error}"))??;
            let record = self.required(id)?;
            // Only owned shell children are expected to finish on their own.
            // Any other category is unknown external work.
            if record
                .unresolved_categories
                .iter()
                .any(|category| category != SHELL_CHILDREN)
            {
                return self.stop_state(id, ControlState::Unresolved);
            }
            if outstanding == 0 && record.unresolved_categories.is_empty() {
                return self.settle_rereading(id, record.revision);
            }
            // Children that exit without a watcher (for example after a daemon
            // restart) are only noticed by re-checking, so poll while any are listed.
            let wake = if record.owned_children.is_empty() {
                expires
            } else {
                expires.min(tokio::time::Instant::now() + std::time::Duration::from_millis(250))
            };
            let _ = tokio::time::timeout_at(wake, notified).await;
            if tokio::time::Instant::now() >= expires {
                return self.stop_state(id, ControlState::Unresolved);
            }
        }
    }

    /// Settle against `revision`, re-reading a bounded number of times when a
    /// concurrent record write (a report stamp, a child update) moved the
    /// revision after it was observed. Every other refusal is returned as is.
    fn settle_rereading(&self, id: &str, mut revision: u64) -> Result<ControlRecord> {
        let mut attempts = 0;
        loop {
            match self.settle(id, revision) {
                Err(error)
                    if attempts < 3
                        && error.to_string().contains("Stale session control revision") =>
                {
                    attempts += 1;
                    revision = self.required(id)?.revision;
                }
                other => return other,
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

    /// Admit explicit, authorized new work (resume, wake, retry, reassign,
    /// assign) for `target`. A settled stop is replaced by a fresh monotonic
    /// generation. Concurrent callers race on the per-session lock, so exactly
    /// one observes `Resumed`; the rest see `AlreadyRunnable`. An unsettled stop,
    /// an unauthorized requester or an untrusted record is refused and nothing
    /// is written. Nothing else (restart, recovery, queued wakes) calls this.
    pub(crate) fn admit_new_work(
        &self,
        requester: &str,
        target: &str,
        force: bool,
    ) -> Result<NewWorkAdmission> {
        let entry = self.entry(target)?;
        let mut guard = entry
            .lock()
            .map_err(|_| anyhow::anyhow!("Session control lock poisoned"))?;
        ensure!(!guard.fail_closed, "Session control persistence unresolved");
        // Sessions that predate control records are not revoked.
        let Some(mut record) = self.inspect(target)? else {
            return Ok(NewWorkAdmission::AlreadyRunnable);
        };
        match record.state {
            ControlState::Runnable => Ok(NewWorkAdmission::AlreadyRunnable),
            ControlState::Stopped => {
                self.authorized(requester, target, force)?;
                ensure!(
                    guard.outstanding.is_empty(),
                    "Predecessor work remains unresolved"
                );
                record.state = ControlState::Runnable;
                let token = self.allocate(&mut record)?;
                guard.pre_admitted = Some(token.generation);
                Ok(NewWorkAdmission::Resumed(token.generation))
            }
            state => bail!("Session '{target}' stop has not settled: {state:?}"),
        }
    }
}

/// Outcome of [`ControlStore::admit_new_work`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NewWorkAdmission {
    AlreadyRunnable,
    Resumed(u64),
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

/// Compatibility floor for a candidate daemon binary: with persisted control
/// records, the candidate must report a control reader version at or above every
/// record's `min_reader_version`. An unreported version (a build that predates
/// durable stop and would ignore revocation) is refused, never assumed.
pub(crate) fn ensure_reader_floor(floor: Option<u32>, candidate: Option<u32>) -> Result<()> {
    let Some(floor) = floor else { return Ok(()) };
    match candidate {
        Some(version) if version >= floor => Ok(()),
        Some(version) => bail!(
            "candidate control reader version {version} is below persisted session control floor {floor}; \
             it would serve state while ignoring stop revocation"
        ),
        None => bail!(
            "candidate reports no control reader version (predates durable stop control) but session control \
             records exist; it would serve state while ignoring stop revocation"
        ),
    }
}

pub(crate) fn store() -> ControlStore {
    ControlStore::new(crate::storage::durable_state_dir().join("session-control"))
}

tokio::task_local! { static EXECUTION: GenerationToken; }

tokio::task_local! { static REPORTED_GENERATION: Option<u64>; }

/// Run a report projection for a report that names its own generation on the
/// wire (`None`: the reporter did not name one, so nothing is attributed).
pub(crate) async fn with_reported_generation<F: Future>(
    generation: Option<u64>,
    future: F,
) -> F::Output {
    REPORTED_GENERATION.scope(generation, future).await
}

/// Generation of the running tool scope, if the caller is inside one.
pub(crate) fn current_generation(session_id: &str) -> Option<u64> {
    current_token()
        .filter(|token| token.session_id == session_id)
        .map(|token| token.generation)
}

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
            .mark_external_effect(&external, "external_effects")
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
        assert!(unknown.unresolved_categories.contains("external_effects"));
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

    #[test]
    fn every_native_mutation_tool_is_denied_after_revocation_with_bytes_unchanged() {
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
            let file = root.path().join("target.txt");
            std::fs::write(&file, "original\n").unwrap();
            let doomed = root.path().join("doomed.txt");
            std::fs::write(&doomed, "keep\n").unwrap();
            let ctx = crate::tool::ToolContext {
                session_id: "worker".into(),
                message_id: "message".into(),
                tool_call_id: "mutate".into(),
                working_dir: Some(root.path().to_owned()),
                stdin_request_tx: None,
                graceful_shutdown_signal: None,
                execution_mode: crate::tool::ToolExecutionMode::AgentTurn,
            };
            let unified = "--- a/target.txt\n+++ b/target.txt\n@@ -1 +1 @@\n-original\n+patched\n";
            let codex = "*** Begin Patch\n*** Update File: target.txt\n@@\n-original\n+codex\n*** Add File: created.txt\n+new\n*** Delete File: doomed.txt\n*** End Patch";
            let calls = [
                ("write", serde_json::json!({"file_path": file, "content": "w"})),
                (
                    "edit",
                    serde_json::json!({"file_path": file, "old_string": "original", "new_string": "edited"}),
                ),
                ("patch", serde_json::json!({"patch_text": unified})),
                ("apply_patch", serde_json::json!({"patch_text": codex})),
            ];
            // Positive control: identical inputs succeed for a runnable session, so
            // the denials below are caused by revocation and not malformed inputs.
            store.register("control-worker", None).unwrap();
            let control_token = store.admit("control-worker").unwrap();
            let control_ctx = crate::tool::ToolContext {
                session_id: "control-worker".into(),
                ..ctx.clone()
            };
            for (name, input) in calls.iter().cloned() {
                scope(
                    Some(control_token.clone()),
                    registry.execute(name, input, control_ctx.clone()),
                )
                .await
                .unwrap_or_else(|error| panic!("control {name} failed: {error:#}"));
                std::fs::write(&file, "original\n").unwrap();
                std::fs::write(&doomed, "keep\n").unwrap();
                let _ = std::fs::remove_file(root.path().join("created.txt"));
            }
            store
                .request_stop("worker", "worker", false, "stop")
                .unwrap();
            for (name, input) in calls {
                let result = scope(
                    Some(token.clone()),
                    registry.execute(name, input, ctx.clone()),
                )
                .await;
                assert!(result.is_err(), "revoked {name} succeeded");
                assert_eq!(
                    std::fs::read_to_string(&file).unwrap(),
                    "original\n",
                    "revoked {name} changed target bytes"
                );
                assert_eq!(std::fs::read_to_string(&doomed).unwrap(), "keep\n");
                assert!(
                    !root.path().join("created.txt").exists(),
                    "revoked {name} created a file"
                );
            }
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
    fn newer_reader_floor_fails_closed_preserves_bytes_and_legacy_records_upgrade_on_write() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("future", None).unwrap();
        let path = store.path("future").unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        value["min_reader_version"] =
            serde_json::json!(jcode_swarm_core::control::CONTROL_MIN_READER_VERSION + 1);
        value["schema_version"] =
            serde_json::json!(jcode_swarm_core::control::CONTROL_SCHEMA_VERSION + 1);
        let future_bytes = serde_json::to_vec(&value).unwrap();
        std::fs::write(&path, &future_bytes).unwrap();
        assert!(store.inspect("future").is_err());
        assert!(store.admit("future").is_err());
        assert!(store.runnable("future").is_err());
        assert!(
            store
                .request_stop("future", "future", false, "stop")
                .is_err()
        );
        assert!(store.predecessor_unsettled("future"));
        assert_eq!(
            std::fs::read(&path).unwrap(),
            future_bytes,
            "refused record was rewritten"
        );
        let refused = store.incompatible_records();
        assert_eq!(refused.len(), 1);
        assert_eq!(refused[0].0, "future");
        assert!(refused[0].1.contains("bytes preserved"), "{refused:?}");

        let legacy = r#"{"schema_version":1,"session_id":"legacy","owner_session_id":null,
            "revision":3,"last_allocated_generation":1,"admitted_generation":null,
            "revoked_through":0,"state":"runnable","latest_stop_request":null}"#;
        std::fs::write(store.path("legacy").unwrap(), legacy).unwrap();
        let before = store.inspect("legacy").unwrap().unwrap();
        assert_eq!((before.schema_version, before.min_reader_version), (1, 1));
        store.admit("legacy").unwrap();
        let after = store.inspect("legacy").unwrap().unwrap();
        assert_eq!(
            after.schema_version,
            jcode_swarm_core::control::CONTROL_SCHEMA_VERSION
        );
        assert_eq!(
            after.min_reader_version,
            jcode_swarm_core::control::CONTROL_MIN_READER_VERSION
        );
        assert_eq!(after.last_allocated_generation, 2);
    }

    #[tokio::test]
    async fn settle_rereads_after_a_concurrent_revision_bump_but_keeps_other_refusals() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let token = store.admit("worker").unwrap();
        let stop = store
            .request_stop("worker", "worker", false, "stop-1")
            .unwrap();
        let _ = token;
        // A report stamp lands after the drain observed `stop.revision`.
        let mut record = store.inspect("worker").unwrap().unwrap();
        store.save_next(&mut record).unwrap();
        assert!(store.settle("worker", stop.revision).is_err());
        let settled = store.settle_rereading("worker", stop.revision).unwrap();
        assert_eq!(settled.state, ControlState::Stopped);
        // Outstanding work is still refused, not retried into success.
        store.register("busy", None).unwrap();
        let busy = store.admit("busy").unwrap();
        let held = busy.reserve().unwrap();
        let requested = store.request_stop("busy", "busy", false, "stop-2").unwrap();
        assert!(store.settle_rereading("busy", requested.revision).is_err());
        drop(held);
    }

    #[test]
    fn unauthorized_or_unknown_stop_installs_no_barrier() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", Some("coord")).unwrap();
        let token = store.admit("worker").unwrap();
        let before = store.inspect("worker").unwrap().unwrap();
        assert!(
            store
                .request_stop("stranger", "worker", false, "stop-1")
                .is_err()
        );
        assert!(
            store
                .request_stop("stranger", "ghost", false, "stop-2")
                .is_err()
        );
        assert_eq!(store.inspect("worker").unwrap().unwrap(), before);
        assert!(
            store.reserve(&token).is_ok(),
            "unauthorized stop revoked work"
        );
        // The unknown target can still register and run afterwards.
        store.register("ghost", None).unwrap();
        assert!(store.admit("ghost").is_ok());
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

    fn spawn_group_leader(script: &str) -> std::process::Child {
        use std::os::unix::process::CommandExt;
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(script).process_group(0);
        cmd.spawn().unwrap()
    }

    #[tokio::test]
    async fn shell_child_clears_only_after_proven_exit_and_then_stop_settles() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let token = store.admit("worker").unwrap();
        let mut owned = token.own_shell_child().unwrap();
        assert!(
            store
                .inspect("worker")
                .unwrap()
                .unwrap()
                .unresolved_categories
                .contains(SHELL_CHILDREN)
        );
        let mut child = spawn_group_leader("sleep 1");
        owned.set_pid(child.id());
        // Still alive: the category must remain.
        store.reconcile_children("worker").unwrap();
        let record = store.inspect("worker").unwrap().unwrap();
        assert!(record.unresolved_categories.contains(SHELL_CHILDREN));
        assert_eq!(record.owned_children.len(), 1);
        store
            .request_stop("worker", "worker", false, "stop-1")
            .unwrap();
        let drain = {
            let store = store.clone();
            tokio::spawn(async move {
                store
                    .drain("worker", std::time::Duration::from_secs(10))
                    .await
            })
        };
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(!drain.is_finished(), "stop must wait for the live child");
        child.wait().unwrap();
        drop(owned);
        let settled = drain.await.unwrap().unwrap();
        assert_eq!(settled.state, ControlState::Stopped);
        assert!(settled.owned_children.is_empty());
        assert!(settled.unresolved_categories.is_empty());
    }

    #[tokio::test]
    async fn surviving_grandchild_or_unreaped_child_keeps_stop_unresolved() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let token = store.admit("worker").unwrap();
        let mut owned = token.own_shell_child().unwrap();
        // The leader exits at once but leaves a backgrounded group member.
        let mut child = spawn_group_leader("sleep 30 & exit 0");
        let pgid = child.id();
        owned.set_pid(pgid);
        child.wait().unwrap();
        drop(owned);
        store
            .request_stop("worker", "worker", false, "stop-1")
            .unwrap();
        let record = store
            .drain("worker", std::time::Duration::from_millis(600))
            .await
            .unwrap();
        unsafe { libc::kill(-(pgid as i32), libc::SIGKILL) };
        assert_eq!(record.state, ControlState::Unresolved);
        assert!(record.unresolved_categories.contains(SHELL_CHILDREN));
    }

    #[test]
    fn explicit_new_work_allocates_one_fresh_generation_and_refuses_unsettled_or_unauthorized() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("coord", None).unwrap();
        store.register("worker", Some("coord")).unwrap();
        store.register("stranger", None).unwrap();
        let old = store.admit("worker").unwrap();
        // Runnable: no new generation, no write.
        let before = store.inspect("worker").unwrap().unwrap();
        assert_eq!(
            store.admit_new_work("coord", "worker", false).unwrap(),
            NewWorkAdmission::AlreadyRunnable
        );
        assert_eq!(store.inspect("worker").unwrap().unwrap(), before);
        // Unsettled stop refuses even the owner.
        let work = store.reserve(&old).unwrap();
        store
            .request_stop("coord", "worker", false, "stop-1")
            .unwrap();
        assert!(store.admit_new_work("coord", "worker", false).is_err());
        drop(work);
        let stopped = store.settle("worker", store.inspect("worker").unwrap().unwrap().revision);
        assert_eq!(stopped.unwrap().state, ControlState::Stopped);
        // An unrelated requester cannot resume and nothing is written.
        let revision = store.inspect("worker").unwrap().unwrap().revision;
        assert!(store.admit_new_work("stranger", "worker", false).is_err());
        assert_eq!(store.inspect("worker").unwrap().unwrap().revision, revision);
        // Many concurrent explicit requests: exactly one winner.
        let winners = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let store = store.clone();
                    scope.spawn(move || store.admit_new_work("coord", "worker", false))
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .filter(|r| matches!(r, Ok(NewWorkAdmission::Resumed(_))))
                .count()
        });
        assert_eq!(winners, 1);
        let record = store.inspect("worker").unwrap().unwrap();
        assert_eq!(record.state, ControlState::Runnable);
        assert_eq!(record.admitted_generation, Some(old.generation + 1));
        // The old generation never gains the new authority.
        assert!(store.reserve(&old).is_err());
        // A restart-style reload does not resume a stopped record.
        store
            .request_stop("coord", "worker", false, "stop-2")
            .unwrap();
        let revision = store.inspect("worker").unwrap().unwrap().revision;
        store.settle("worker", revision).unwrap();
        let reloaded = ControlStore::new(root.path().to_owned());
        assert_eq!(
            reloaded.inspect("worker").unwrap().unwrap().state,
            ControlState::Stopped
        );
        assert!(reloaded.runnable("worker").is_err());
    }

    #[tokio::test]
    async fn stale_generation_report_cannot_project_onto_the_resumed_generation() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let old = store.admit("worker").unwrap();
        let stop = store
            .request_stop("worker", "worker", false, "stop-1")
            .unwrap();
        store.settle("worker", stop.revision).unwrap();
        let fresh = match store.admit_new_work("worker", "worker", false).unwrap() {
            NewWorkAdmission::Resumed(generation) => generation,
            other => panic!("expected resume, got {other:?}"),
        };
        assert_eq!(fresh, old.generation + 1);
        // The claim release in `update_member_status` runs inside this closure,
        // so a rejected stale report must never execute it.
        let released = std::cell::Cell::new(false);
        let stale = scope(Some(old.clone()), async {
            store.project("worker", "completed", || released.set(true))
        })
        .await;
        assert!(stale.is_err());
        assert!(
            !released.get(),
            "stale report released the new generation's claims"
        );
        // Unscoped server-side projections and the current generation still work.
        assert!(store.project("worker", "running", || ()).is_ok());
        let current = GenerationToken {
            generation: fresh,
            ..old
        };
        assert!(
            scope(Some(current), async {
                store.project("worker", "completed", || ())
            })
            .await
            .is_ok()
        );
    }

    #[tokio::test]
    async fn report_stamps_only_the_producing_generation_and_binds_waits() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        // Idle before any turn: a wait binds to the next generation.
        assert_eq!(store.awaited_generation("worker"), Some(1));
        let first = store.admit("worker").unwrap();
        let work = first.reserve().unwrap();
        // Outstanding work: the wait binds to the running generation.
        assert_eq!(store.awaited_generation("worker"), Some(1));
        scope(Some(first.clone()), async {
            store
                .project_report("worker", "ready", true, || ())
                .unwrap();
        })
        .await;
        drop(work);
        let record = store.inspect("worker").unwrap().unwrap();
        assert_eq!(record.report_generation, Some(1));
        // Settled: the next wait is for generation 2, which has no report yet.
        assert_eq!(store.awaited_generation("worker"), Some(2));
        // Non-report projections never stamp.
        let second = store.admit("worker").unwrap();
        scope(Some(second), async {
            store.project("worker", "running", || ()).unwrap();
        })
        .await;
        assert_eq!(
            store.inspect("worker").unwrap().unwrap().report_generation,
            Some(1)
        );
        // A stale generation cannot stamp a report onto the newer generation.
        let stale = scope(Some(first), async {
            store.project_report("worker", "ready", true, || ())
        })
        .await;
        assert!(stale.is_err());
        assert_eq!(
            store.inspect("worker").unwrap().unwrap().report_generation,
            Some(1)
        );
    }

    #[tokio::test]
    async fn resume_then_turn_admission_allocates_one_generation_that_waits_bind_to() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let old = store.admit("worker").unwrap();
        let stop = store
            .request_stop("worker", "worker", false, "stop-1")
            .unwrap();
        store.settle("worker", stop.revision).unwrap();
        let fresh = match store.admit_new_work("worker", "worker", false).unwrap() {
            NewWorkAdmission::Resumed(generation) => generation,
            other => panic!("expected resume, got {other:?}"),
        };
        // A wait bound between resume and the turn naming the same generation.
        assert_eq!(store.awaited_generation("worker"), Some(fresh));
        let token = store.admit("worker").unwrap();
        assert_eq!(
            token.generation, fresh,
            "turn must adopt the resume's generation"
        );
        let record = store.inspect("worker").unwrap().unwrap();
        assert_eq!(record.last_allocated_generation, old.generation + 1);
        // Once the turn runs, the wait still names that generation.
        let work = token.reserve().unwrap();
        assert_eq!(store.awaited_generation("worker"), Some(fresh));
        drop(work);
        // A stop between resume and the turn drops the pre-admission: the
        // adopted generation is never reused after revocation.
        let stop = store
            .request_stop("worker", "worker", false, "stop-2")
            .unwrap();
        store.settle("worker", stop.revision).unwrap();
        let again = match store.admit_new_work("worker", "worker", false).unwrap() {
            NewWorkAdmission::Resumed(generation) => generation,
            other => panic!("expected resume, got {other:?}"),
        };
        assert_eq!(again, fresh + 1);
        let stop = store
            .request_stop("worker", "worker", false, "stop-3")
            .unwrap();
        store.settle("worker", stop.revision).unwrap();
        assert!(store.admit("worker").is_err());
    }

    #[tokio::test]
    async fn report_generation_is_the_works_own_never_ambient_admitted_state() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("worker", None).unwrap();
        let first = store.admit("worker").unwrap();
        // Turn-end report (no scope, no wire generation) belongs to the last run.
        store
            .project_report("worker", "ready", true, || ())
            .unwrap();
        assert_eq!(
            store.inspect("worker").unwrap().unwrap().report_generation,
            Some(first.generation)
        );
        // Stop and resume: generation 2 is admitted but has not run.
        let stop = store
            .request_stop("worker", "worker", false, "stop-1")
            .unwrap();
        store.settle("worker", stop.revision).unwrap();
        store.admit_new_work("worker", "worker", false).unwrap();
        // A late generation-1 turn-end report (last run = 1) is rejected and
        // cannot be attributed to generation 2.
        assert!(
            store
                .project_report("worker", "ready", true, || ())
                .is_err()
        );
        // A late wire report naming generation 1 is rejected too.
        let stale = with_reported_generation(Some(first.generation), async {
            store.project_report("worker", "ready", true, || ())
        })
        .await;
        assert!(stale.is_err());
        assert_eq!(
            store.inspect("worker").unwrap().unwrap().report_generation,
            Some(first.generation)
        );
        // A wire report that names no generation (old client) is never bound.
        let second = store.admit("worker").unwrap();
        let unnamed = with_reported_generation(None, async {
            store.project_report("worker", "ready", true, || ())
        })
        .await;
        assert!(unnamed.is_ok());
        assert_eq!(
            store.inspect("worker").unwrap().unwrap().report_generation,
            Some(first.generation)
        );
        // Naming the live generation binds it.
        let named = with_reported_generation(Some(second.generation), async {
            store.project_report("worker", "ready", true, || ())
        })
        .await;
        assert!(named.is_ok());
        assert_eq!(
            store.inspect("worker").unwrap().unwrap().report_generation,
            Some(second.generation)
        );
    }

    #[tokio::test]
    async fn multi_attachment_stop_cancels_every_active_turn_and_settles_only_when_all_end() {
        let root = tempfile::tempdir().unwrap();
        let store = ControlStore::new(root.path().to_owned());
        store.register("multi_worker", None).unwrap();
        let token = store.admit("multi_worker").unwrap();
        let (work_a, work_b) = (token.reserve().unwrap(), token.reserve().unwrap());
        let (sig_a, sig_b) = (
            jcode_agent_runtime::InterruptSignal::new(),
            jcode_agent_runtime::InterruptSignal::new(),
        );
        let active_a =
            crate::turn_cancel_registry::register_active_turn("multi_worker", sig_a.clone());
        let active_b =
            crate::turn_cancel_registry::register_active_turn("multi_worker", sig_b.clone());
        store
            .request_stop("multi_worker", "multi_worker", false, "stop")
            .unwrap();
        let waiting = store.clone();
        let task = tokio::spawn(async move {
            waiting
                .drain("multi_worker", std::time::Duration::from_secs(5))
                .await
        });
        for signal in [&sig_a, &sig_b] {
            tokio::time::timeout(std::time::Duration::from_secs(1), signal.notified())
                .await
                .expect("every attached turn is cancelled");
        }
        drop(active_a);
        drop(work_a);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(
            !task.is_finished(),
            "one settled turn must not settle the session"
        );
        drop(active_b);
        drop(work_b);
        assert_eq!(task.await.unwrap().unwrap().state, ControlState::Stopped);
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
