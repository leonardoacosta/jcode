//! Inspect and purge stored memories.
//!
//! Purge is the only bulk deletion. It reports what matches without deleting
//! unless asked to apply, and it only removes entries that match every filter
//! it was given. Removed ids are kept as tombstones so a writer that loaded
//! the graph before the purge cannot save a purged entry back: every graph
//! save drops entries that match a tombstone of that same graph file, under
//! the same lock the purge holds. A tombstone records the purged version
//! (`updated_at`), so only that version or an older one is dropped. A newer
//! write of the same id, such as a goal sync after an edit, is kept.

use super::MemoryManager;
use crate::memory_graph::MemoryGraph;
use crate::memory_types::{MemoryEntry, MemoryScope};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const TOMBSTONE_FILE: &str = "purged-ids.json";
const LOCK_NAME: &str = "graphs.json";
/// A stale snapshot older than this is not a realistic concurrent writer.
const TOMBSTONE_KEEP_DAYS: i64 = 90;
const PREVIEW_CHARS: usize = 80;

/// Which entries a purge or a listing selects. Every field narrows.
#[derive(Debug, Clone)]
pub struct PurgeFilter {
    pub scope: MemoryScope,
    /// Only entries that are no longer active (superseded).
    pub inactive_only: bool,
    /// Only entries last updated at least this many days ago.
    pub older_than_days: Option<u64>,
    pub now: DateTime<Utc>,
}

impl PurgeFilter {
    pub fn new(scope: MemoryScope) -> Self {
        Self {
            scope,
            inactive_only: false,
            older_than_days: None,
            now: Utc::now(),
        }
    }

    /// A purge with no narrowing filter would remove everything: refused.
    pub fn is_narrowed(&self) -> bool {
        self.inactive_only || self.older_than_days.is_some()
    }

    fn matches(&self, entry: &MemoryEntry) -> bool {
        if self.inactive_only && entry.active {
            return false;
        }
        match self.older_than_days {
            Some(days) => age_cutoff(self.now, days).is_some_and(|c| entry.updated_at <= c),
            None => true,
        }
    }
}

/// The instant `days` before `now`. `None` when that is not representable, so
/// an absurd age matches nothing instead of wrapping into "everything".
pub fn age_cutoff(now: DateTime<Utc>, days: u64) -> Option<DateTime<Utc>> {
    let days = i64::try_from(days).ok()?;
    now.checked_sub_signed(Duration::try_days(days)?)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeCandidate {
    pub id: String,
    pub scope: MemoryScope,
    pub category: String,
    /// First characters of the content, one line.
    pub preview: String,
    pub age_days: i64,
    pub active: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeReport {
    pub matched: Vec<PurgeCandidate>,
    /// Entries actually removed. Zero for a dry run.
    pub removed: usize,
}

fn preview(content: &str) -> String {
    let one_line: String = content
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    // Entries stored before the secret gate existed may still hold secrets.
    let one_line = crate::message::redact_preview_secrets(&one_line);
    let mut out: String = one_line.chars().take(PREVIEW_CHARS).collect();
    if one_line.chars().count() > PREVIEW_CHARS {
        out.push('…');
    }
    out
}

fn candidate(entry: &MemoryEntry, scope: MemoryScope, now: DateTime<Utc>) -> PurgeCandidate {
    PurgeCandidate {
        id: entry.id.clone(),
        scope,
        category: entry.category.to_string(),
        preview: preview(&entry.content),
        age_days: (now - entry.updated_at).num_days().max(0),
        active: entry.active,
    }
}

/// One purged entry: when it was purged and which version was removed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Tombstone {
    purged_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Graph file path -> entry id -> tombstone. Ids are not unique across
/// graphs (goal and skill ids are slugs), so a tombstone only applies to the
/// graph file it was written for.
type Tombstones = BTreeMap<String, BTreeMap<String, Tombstone>>;

fn graph_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

impl MemoryManager {
    fn tombstone_path(&self) -> Result<PathBuf> {
        Ok(self.global_memory_path()?.with_file_name(TOMBSTONE_FILE))
    }

    fn read_tombstones(&self) -> Result<Tombstones> {
        let path = self.tombstone_path()?;
        if !path.exists() {
            return Ok(Tombstones::new());
        }
        crate::storage::read_json(&path).map_err(|error| {
            anyhow::anyhow!(
                "memory tombstone file {} is unreadable ({error}); repair or delete it",
                path.display()
            )
        })
    }

    /// Ids that a legacy-note import must not bring back into this graph.
    pub(super) fn purged_in(&self, graph_path: &Path, id: &str, updated_at: DateTime<Utc>) -> bool {
        self.read_tombstones().is_ok_and(|all| {
            all.get(&graph_key(graph_path))
                .and_then(|graph| graph.get(id))
                .is_some_and(|t| updated_at <= t.updated_at)
        })
    }

    /// Cross-process lock over the memory graph files and the tombstones.
    pub(super) fn lock_graphs(&self) -> Result<crate::auth::account_store::StoreLockGuard> {
        let dir = self
            .global_memory_path()?
            .parent()
            .map(PathBuf::from)
            .ok_or_else(|| anyhow::anyhow!("memory directory has no parent"))?;
        crate::auth::account_store::lock_store(&dir, LOCK_NAME)
    }

    /// The graph without any purged id, borrowed when there is nothing to drop.
    pub(super) fn without_purged<'a>(
        &self,
        graph: &'a MemoryGraph,
        graph_path: &Path,
    ) -> Cow<'a, MemoryGraph> {
        let tombstones = match self.read_tombstones() {
            Ok(all) => all,
            Err(error) => {
                // Fail open: dropping live memories on a damaged file is worse.
                crate::logging::info(&format!("memory purge filter skipped: {error}"));
                return Cow::Borrowed(graph);
            }
        };
        let Some(for_graph) = tombstones.get(&graph_key(graph_path)) else {
            return Cow::Borrowed(graph);
        };
        let doomed: Vec<&String> = graph
            .memories
            .iter()
            .filter(|(id, entry)| {
                for_graph
                    .get(*id)
                    .is_some_and(|t| entry.updated_at <= t.updated_at)
            })
            .map(|(id, _)| id)
            .collect();
        if doomed.is_empty() {
            return Cow::Borrowed(graph);
        }
        let mut cleaned = graph.clone();
        for id in doomed {
            cleaned.remove_memory(id);
        }
        Cow::Owned(cleaned)
    }

    fn matching(
        &self,
        graph: &MemoryGraph,
        scope: MemoryScope,
        filter: &PurgeFilter,
    ) -> Vec<PurgeCandidate> {
        let mut out: Vec<PurgeCandidate> = graph
            .all_memories()
            .filter(|entry| filter.matches(entry))
            .map(|entry| candidate(entry, scope, filter.now))
            .collect();
        out.sort_by(|a, b| b.age_days.cmp(&a.age_days).then(a.id.cmp(&b.id)));
        out
    }

    /// What a purge with `filter` would remove. Reads only, deletes nothing.
    pub fn purge_candidates(&self, filter: &PurgeFilter) -> Result<Vec<PurgeCandidate>> {
        self.ensure_scope_available(filter)?;
        let mut out = Vec::new();
        if filter.scope.includes_project() {
            out.extend(self.matching(&self.load_project_graph()?, MemoryScope::Project, filter));
        }
        if filter.scope.includes_global() {
            out.extend(self.matching(&self.load_global_graph()?, MemoryScope::Global, filter));
        }
        Ok(out)
    }

    fn ensure_scope_available(&self, filter: &PurgeFilter) -> Result<()> {
        anyhow::ensure!(
            filter.scope != MemoryScope::Project || self.project_memory_path()?.is_some(),
            "project scope needs a project directory"
        );
        Ok(())
    }

    /// Remove every entry matching `filter`. Refuses a filter that does not
    /// narrow, and refuses to run without the cross-process lock: a purge that
    /// could race another writer must not delete.
    pub fn purge_apply(&self, filter: &PurgeFilter) -> Result<PurgeReport> {
        anyhow::ensure!(
            filter.is_narrowed(),
            "refusing to purge without a filter: pass --inactive and/or --older-than"
        );
        self.ensure_scope_available(filter)?;
        let _guard = self.lock_graphs().map_err(|error| {
            anyhow::anyhow!("cannot lock memory storage, nothing purged: {error}")
        })?;
        let mut tombstones = self.read_tombstones()?;
        let keep_from = filter.now - Duration::days(TOMBSTONE_KEEP_DAYS);
        for per_graph in tombstones.values_mut() {
            per_graph.retain(|_, t| t.purged_at >= keep_from);
        }
        tombstones.retain(|_, per_graph| !per_graph.is_empty());
        let mut report = PurgeReport::default();

        if filter.scope.includes_project()
            && let Some(path) = self.project_memory_path()?
        {
            let graph = self.load_project_graph()?;
            self.purge_graph(
                &graph,
                &path,
                MemoryScope::Project,
                filter,
                &mut tombstones,
                &mut report,
                |g| self.save_project_graph(g),
            )?;
        }
        if filter.scope.includes_global() {
            let path = self.global_memory_path()?;
            let graph = self.load_global_graph()?;
            self.purge_graph(
                &graph,
                &path,
                MemoryScope::Global,
                filter,
                &mut tombstones,
                &mut report,
                |g| self.save_global_graph(g),
            )?;
        }
        Ok(report)
    }

    /// Save first, then tombstone: a failed save leaves no tombstone behind to
    /// delete a live entry later. Both happen under the caller's lock.
    #[allow(clippy::too_many_arguments)]
    fn purge_graph(
        &self,
        graph: &MemoryGraph,
        path: &Path,
        scope: MemoryScope,
        filter: &PurgeFilter,
        tombstones: &mut Tombstones,
        report: &mut PurgeReport,
        save: impl FnOnce(&MemoryGraph) -> Result<()>,
    ) -> Result<()> {
        let hits = self.matching(graph, scope, filter);
        if hits.is_empty() {
            return Ok(());
        }
        let mut cleaned = graph.clone();
        let per_graph = tombstones.entry(graph_key(path)).or_default();
        for hit in &hits {
            if let Some(entry) = cleaned.get_memory(&hit.id) {
                per_graph.insert(
                    hit.id.clone(),
                    Tombstone {
                        purged_at: filter.now,
                        updated_at: entry.updated_at,
                    },
                );
            }
            cleaned.remove_memory(&hit.id);
        }
        save(&cleaned)?;
        let removed = hits.len();
        report.matched.extend(hits);
        report.removed += removed;
        self.write_tombstones(tombstones).map_err(|error| {
            anyhow::anyhow!(
                "removed {} memories but could not record the purge: {error}",
                report.removed
            )
        })
    }

    fn write_tombstones(&self, tombstones: &Tombstones) -> Result<()> {
        crate::storage::write_json(&self.tombstone_path()?, tombstones)
    }

    /// The bounded policy used by startup auto-purge. `None` unless the user
    /// explicitly enabled it: age or supersession alone deletes nothing.
    pub fn retention_filter(
        policy: &jcode_config_types::MemoryRetentionConfig,
        now: DateTime<Utc>,
    ) -> Option<PurgeFilter> {
        policy.auto_purge_active().then_some(PurgeFilter {
            // Startup has no project directory, so only the global store is
            // reachable. Project memories are purged with `jcode memory purge`.
            scope: MemoryScope::Global,
            inactive_only: true,
            older_than_days: Some(policy.inactive_max_age_days),
            now,
        })
    }

    /// Startup hook: purge per `[memory.retention]`. Does nothing unless the
    /// policy is explicitly on. Errors are logged, never fatal.
    pub fn auto_purge(&self, policy: &jcode_config_types::MemoryRetentionConfig) {
        let Some(filter) = Self::retention_filter(policy, Utc::now()) else {
            return;
        };
        match self.purge_apply(&filter) {
            Ok(report) if report.removed > 0 => crate::logging::info(&format!(
                "memory retention purged {} inactive memories",
                report.removed
            )),
            Ok(_) => {}
            Err(error) => crate::logging::info(&format!("memory retention skipped: {error}")),
        }
    }
}

#[cfg(test)]
#[path = "retention_tests.rs"]
mod tests;
