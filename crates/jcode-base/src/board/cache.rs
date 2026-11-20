//! In-memory board cache, isolated per binding.
//!
//! An entry is returned only for the exact [`CacheKey`] it was stored under:
//! verified world, backend, auth identity, team and project. The auth
//! identity is the secret-free MCP server identity, so two connections to the
//! same server name with different endpoints or credentials never share data.
//!
//! There is no fallback. A fetch error drops that key's entry and surfaces the
//! error; stale data is never served as current success, and one key's data is
//! never served for another.

use super::binding::BoardScope;
use super::provider::{BoardError, BoardIssue, BoardProvider, WorkflowState};
use jcode_config_types::BoardBackend;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Secret-free identity of the MCP connection a board is read through.
pub fn auth_identity(server_name: &str, config: &crate::mcp::McpServerConfig) -> String {
    crate::mcp::tasks::server_identity(server_name, config)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub world: String,
    pub backend: BoardBackend,
    pub auth_identity: String,
    pub team: String,
    pub project: Option<String>,
}

impl CacheKey {
    pub fn new(scope: &BoardScope, auth_identity: &str) -> Self {
        Self {
            world: scope.world.clone(),
            backend: scope.backend,
            auth_identity: auth_identity.to_string(),
            team: scope.team.clone(),
            project: scope.project.clone(),
        }
    }
}

/// One successful board read: columns and cards from the same fetch.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardSnapshot {
    pub states: Vec<WorkflowState>,
    pub issues: Vec<BoardIssue>,
}

pub struct BoardCache {
    ttl: Duration,
    entries: Mutex<HashMap<CacheKey, (Instant, Arc<BoardSnapshot>)>>,
}

impl BoardCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<CacheKey, (Instant, Arc<BoardSnapshot>)>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Fresh entry for exactly this key. An expired entry is dropped.
    pub fn get(&self, key: &CacheKey) -> Option<Arc<BoardSnapshot>> {
        let mut entries = self.lock();
        match entries.get(key) {
            Some((at, snapshot)) if at.elapsed() < self.ttl => Some(Arc::clone(snapshot)),
            Some(_) => {
                entries.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn put(&self, key: CacheKey, snapshot: BoardSnapshot) -> Arc<BoardSnapshot> {
        let snapshot = Arc::new(snapshot);
        let mut entries = self.lock();
        entries.retain(|_, (at, _)| at.elapsed() < self.ttl);
        entries.insert(key, (Instant::now(), Arc::clone(&snapshot)));
        snapshot
    }

    pub fn invalidate(&self, key: &CacheKey) {
        self.lock().remove(key);
    }

    pub fn clear(&self) {
        self.lock().clear();
    }

    /// Cached board for `scope`, or a live read through `provider`. `refresh`
    /// bypasses a fresh entry. On any provider error the key's entry is
    /// dropped and the error returned: no stale or foreign data is served.
    pub async fn load(
        &self,
        scope: &BoardScope,
        auth_identity: &str,
        provider: &dyn BoardProvider,
        refresh: bool,
    ) -> Result<Arc<BoardSnapshot>, BoardError> {
        let key = CacheKey::new(scope, auth_identity);
        if !refresh && let Some(hit) = self.get(&key) {
            return Ok(hit);
        }
        let fetched = async {
            let states = provider.workflow_states(scope).await?;
            let issues = provider.issues(scope).await?;
            Ok::<_, BoardError>(BoardSnapshot { states, issues })
        }
        .await;
        match fetched {
            Ok(snapshot) => Ok(self.put(key, snapshot)),
            Err(error) => {
                self.invalidate(&key);
                Err(error)
            }
        }
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
