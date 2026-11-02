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
    std::fs::canonicalize(path)
        .or_else(|_| {
            // New files: canonicalize the parent, keep the file name.
            let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
            match (parent, path.file_name()) {
                (Some(p), Some(name)) => std::fs::canonicalize(p).map(|p| p.join(name)),
                _ => Err(std::io::Error::other("no parent")),
            }
        })
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
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

#[cfg(test)]
mod claim_tests {
    use super::*;

    #[test]
    fn claim_warn_block_release_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let persist = dir.path().join("claims.json");
        let file = dir.path().join("lib.rs");
        let mut store = FileClaimStore::load(Some(persist.clone()));

        assert_eq!(store.claim("a", &file, FileClaimMode::Warn, false), ClaimOutcome::Granted);
        assert_eq!(store.claim("a", &file, FileClaimMode::Block, false), ClaimOutcome::Granted);
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
        assert_eq!(store.claim("b", &file, FileClaimMode::Off, false), ClaimOutcome::Granted);

        // Survives reload.
        let mut reloaded = FileClaimStore::load(Some(persist.clone()));
        assert_eq!(reloaded.active().len(), 1);
        assert_eq!(reloaded.active()[0].1, "a");

        // Stop/orphan release frees the path for B.
        assert_eq!(reloaded.release_session("a").len(), 1);
        assert_eq!(reloaded.claim("b", &file, FileClaimMode::Block, false), ClaimOutcome::Granted);
        assert_eq!(FileClaimStore::load(Some(persist)).active()[0].1, "b");
    }

    #[test]
    fn mode_parse_defaults_to_warn() {
        assert_eq!(FileClaimMode::parse("block"), FileClaimMode::Block);
        assert_eq!(FileClaimMode::parse("OFF"), FileClaimMode::Off);
        assert_eq!(FileClaimMode::parse(""), FileClaimMode::Warn);
    }
}
