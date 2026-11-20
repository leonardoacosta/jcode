//! Background facts for the header's `linear:` and `openspec:` rows.
//!
//! The header renders on the UI thread, so nothing here runs a process or a
//! network call inline: a row reads its cached text, and a stale or missing
//! entry starts one background refresh. [`generation`] changes when a refresh
//! lands so the prepared-header cache repaints instead of waiting out its TTL.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime};

static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Bumped whenever a background refresh changes a row.
pub(crate) fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|p| p.into_inner())
}

/// Cached row per key, plus keys with a refresh in flight.
struct RowCache<K> {
    rows: HashMap<K, (Instant, Option<String>)>,
    in_flight: HashSet<K>,
}

impl<K> Default for RowCache<K> {
    fn default() -> Self {
        Self {
            rows: HashMap::new(),
            in_flight: HashSet::new(),
        }
    }
}

impl<K: std::hash::Hash + Eq + Clone + Send + 'static> RowCache<K> {
    /// Cached text, and whether a refresh should start now.
    fn read(
        cache: &Mutex<Self>,
        key: &K,
        fresh: impl Fn(Instant) -> bool,
    ) -> (Option<String>, bool) {
        let mut guard = lock(cache);
        let (text, stale) = match guard.rows.get(key) {
            Some((at, text)) => (text.clone(), !fresh(*at)),
            None => (None, true),
        };
        let start = stale && guard.in_flight.insert(key.clone());
        (text, start)
    }

    fn store(cache: &Mutex<Self>, key: K, text: Option<String>) {
        let mut guard = lock(cache);
        guard.in_flight.remove(&key);
        let changed = guard.rows.get(&key).map(|(_, t)| t) != Some(&text);
        guard.rows.insert(key, (Instant::now(), text));
        if changed {
            GENERATION.fetch_add(1, Ordering::Relaxed);
        }
    }
}

// ------------------------------------------------------------------ openspec

/// Summary of one change, from `openspec list --json` and `validate --json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChangeFacts {
    pub completed_tasks: u64,
    pub total_tasks: u64,
    pub complete: bool,
    pub valid: bool,
}

/// `openspec: N not ready | N ready | N active`. Complete changes are not
/// counted. A change with checked tasks is active; an unstarted change is
/// ready only when it validates and has at least one task.
pub(crate) fn openspec_text(changes: &[ChangeFacts]) -> String {
    let (mut not_ready, mut ready, mut active) = (0, 0, 0);
    for change in changes.iter().filter(|c| !c.complete) {
        if change.completed_tasks > 0 {
            active += 1;
        } else if change.valid && change.total_tasks > 0 {
            ready += 1;
        } else {
            not_ready += 1;
        }
    }
    format!("openspec: {not_ready} not ready | {ready} ready | {active} active")
}

/// Joins the two CLI outputs by change name. `None` on any unreadable shape.
pub(crate) fn parse_openspec(list: &str, validate: &str) -> Option<Vec<ChangeFacts>> {
    let list: serde_json::Value = serde_json::from_str(list).ok()?;
    let validate: serde_json::Value = serde_json::from_str(validate).ok()?;
    let valid: HashMap<&str, bool> = validate
        .get("items")?
        .as_array()?
        .iter()
        .filter_map(|item| Some((item.get("id")?.as_str()?, item.get("valid")?.as_bool()?)))
        .collect();
    list.get("changes")?
        .as_array()?
        .iter()
        .map(|change| {
            let name = change.get("name")?.as_str()?;
            Some(ChangeFacts {
                completed_tasks: change.get("completedTasks")?.as_u64()?,
                total_tasks: change.get("totalTasks")?.as_u64()?,
                complete: change.get("status")?.as_str()? == "complete",
                valid: valid.get(name).copied().unwrap_or(false),
            })
        })
        .collect()
}

/// Directory mtime plus every `tasks.md` mtime: changes on add, remove, or a
/// checked task, without reading any file.
fn changes_stamp(changes_dir: &Path) -> Option<Vec<SystemTime>> {
    let mut stamp = vec![std::fs::metadata(changes_dir).ok()?.modified().ok()?];
    for entry in std::fs::read_dir(changes_dir).ok()?.flatten() {
        if let Ok(modified) = entry
            .path()
            .join("tasks.md")
            .metadata()
            .and_then(|m| m.modified())
        {
            stamp.push(modified);
        }
    }
    stamp.sort();
    Some(stamp)
}

type OpenspecKey = (PathBuf, Vec<SystemTime>);
static OPENSPEC: LazyLock<Mutex<RowCache<OpenspecKey>>> = LazyLock::new(Default::default);
type StampEntry = (Instant, Option<Vec<SystemTime>>);
static OPENSPEC_STAMPS: LazyLock<Mutex<HashMap<PathBuf, StampEntry>>> =
    LazyLock::new(Default::default);
/// Filesystem stamps are re-read at most this often per directory.
const STAMP_TTL: Duration = Duration::from_secs(5);

fn run_json(dir: &Path, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new("openspec")
        .args(args)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    // `validate` exits nonzero when a change is invalid but still prints JSON.
    String::from_utf8(output.stdout).ok()
}

/// The row for `dir`, or `None` when `dir/openspec/changes` is absent, the CLI
/// is missing, or its output is unreadable. Never blocks.
pub(crate) fn openspec_row(dir: &Path) -> Option<String> {
    let changes_dir = dir.join("openspec").join("changes");
    let stamp = {
        let mut stamps = lock(&OPENSPEC_STAMPS);
        match stamps.get(dir) {
            Some((at, stamp)) if at.elapsed() < STAMP_TTL => stamp.clone(),
            _ => {
                let stamp = changes_stamp(&changes_dir);
                stamps.insert(dir.to_path_buf(), (Instant::now(), stamp.clone()));
                stamp
            }
        }
    }?;
    let key = (dir.to_path_buf(), stamp);
    // Keyed by stamp: a cached row is valid until the files change.
    let (text, start) = RowCache::read(&OPENSPEC, &key, |_| true);
    if start {
        let dir = dir.to_path_buf();
        std::thread::spawn(move || {
            let text = run_json(&dir, &["list", "--json"])
                .zip(run_json(&dir, &["validate", "--changes", "--json"]))
                .and_then(|(list, validate)| parse_openspec(&list, &validate))
                .map(|changes| openspec_text(&changes));
            RowCache::store(&OPENSPEC, key, text);
        });
    }
    text
}

// -------------------------------------------------------------------- linear

const LINEAR_TTL: Duration = Duration::from_secs(60);
static LINEAR: LazyLock<Mutex<RowCache<PathBuf>>> = LazyLock::new(Default::default);

/// `linear: {team} · {state} {n} · ...` in board order. Done-like columns
/// (`completed`, `canceled`, `duplicate`) and empty columns are left out.
pub(crate) fn linear_text(
    scope: &crate::board::BoardScope,
    snapshot: &crate::board::BoardSnapshot,
) -> String {
    let team = snapshot
        .issues
        .iter()
        .find_map(|i| i.team_name.clone())
        .unwrap_or_else(|| scope.team.clone());
    let mut text = format!("linear: {team}");
    if let Some(project) = &scope.project {
        text.push_str(&format!(" · {project}"));
    }
    for state in &snapshot.states {
        if matches!(state.kind.as_str(), "completed" | "canceled" | "duplicate") {
            continue;
        }
        let n = snapshot
            .issues
            .iter()
            .filter(|i| i.state_id == state.id)
            .count();
        if n > 0 {
            text.push_str(&format!(" · {} {n}", state.name));
        }
    }
    text
}

/// One board read for `dir`. `None` when the world has no Linear binding;
/// `Some("linear: unavailable")` when the binding exists but the read fails.
async fn fetch_linear(dir: PathBuf) -> Option<String> {
    use crate::board::{BoardBackend, BoardCache, LinearMcpProvider, McpManagerCaller};
    use std::sync::Arc;
    // resolve_board runs git and reads config: keep it off the async workers.
    let (scope, config) = tokio::task::spawn_blocking(move || {
        let scope = crate::board::resolve_board(&dir).ok()?;
        let config = crate::mcp::McpConfig::load_for_dir(Some(&dir));
        Some((scope, config))
    })
    .await
    .ok()??;
    if scope.backend != BoardBackend::Linear {
        return None;
    }
    const UNAVAILABLE: &str = "linear: unavailable";
    // Same guard as `/board`: only a server from the global config is used.
    let server = match config.servers.get(&scope.server) {
        Some(server)
            if server.workspaces.is_empty()
                && config.server_scope(&scope.server) == Some("global") =>
        {
            server
        }
        _ => return Some(UNAVAILABLE.into()),
    };
    let auth = crate::board::auth_identity(&scope.server, server);
    let manager = Arc::new(crate::mcp::McpManager::with_config(config));
    let provider = LinearMcpProvider::new(Arc::new(McpManagerCaller::new(manager)));
    let cache = BoardCache::new(LINEAR_TTL);
    Some(match cache.load(&scope, &auth, &provider, true).await {
        Ok(snapshot) => linear_text(&scope, &snapshot),
        Err(_) => UNAVAILABLE.into(),
    })
}

/// Cached `linear:` row for `dir`, refreshed in the background every
/// [`LINEAR_TTL`]. Never blocks; `None` until the first read lands.
pub(crate) fn linear_row(dir: &Path) -> Option<String> {
    let key = dir.to_path_buf();
    let (text, start) = RowCache::read(&LINEAR, &key, |at| at.elapsed() < LINEAR_TTL);
    if start {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    let text = fetch_linear(key.clone()).await;
                    RowCache::store(&LINEAR, key, text);
                });
            }
            Err(_) => RowCache::store(&LINEAR, key, None),
        }
    }
    text
}

#[cfg(test)]
#[path = "header_facts_tests.rs"]
mod tests;
