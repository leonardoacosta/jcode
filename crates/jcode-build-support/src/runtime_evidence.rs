use jcode_selfdev_types::RuntimeIdentity;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceVerdict {
    Matched,
    Mismatched,
    Unknown,
}

/// A point-in-time comparison. Missing observations never become a clean match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEvidence {
    pub source_git_hash: Option<String>,
    pub source_dirty: Option<bool>,
    pub client_git_hash: String,
    pub client_dirty: bool,
    pub server_git_hash: Option<String>,
    pub server_dirty: Option<bool>,
    pub server_binary_path: Option<PathBuf>,
    pub verdict: EvidenceVerdict,
}

fn valid_hash(hash: &str) -> bool {
    (7..=64).contains(&hash.len()) && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn same_revision(left: &str, right: &str) -> bool {
    let shared_len = left.len().min(right.len());
    left[..shared_len].eq_ignore_ascii_case(&right[..shared_len])
}

impl RuntimeEvidence {
    pub fn new(
        source_git_hash: Option<String>,
        source_dirty: Option<bool>,
        client_git_hash: String,
        client_dirty: bool,
        server: Option<RuntimeIdentity>,
    ) -> Self {
        let mut evidence = Self {
            source_git_hash,
            source_dirty,
            client_git_hash,
            client_dirty,
            server_git_hash: server.as_ref().map(|identity| identity.git_hash.clone()),
            server_dirty: server
                .as_ref()
                .filter(|identity| identity.cleanliness_verified)
                .map(|identity| identity.dirty),
            server_binary_path: server.and_then(|identity| identity.binary_path),
            verdict: EvidenceVerdict::Unknown,
        };
        let known_revision_mismatch = evidence
            .source_git_hash
            .as_deref()
            .filter(|hash| valid_hash(hash))
            .is_some_and(|source| {
                (valid_hash(&evidence.client_git_hash)
                    && !same_revision(source, &evidence.client_git_hash))
                    || evidence
                        .server_git_hash
                        .as_deref()
                        .filter(|hash| valid_hash(hash))
                        .is_some_and(|server| !same_revision(source, server))
            });
        if known_revision_mismatch
            || evidence.source_dirty == Some(true)
            || evidence.client_dirty
            || evidence.server_dirty == Some(true)
        {
            evidence.verdict = EvidenceVerdict::Mismatched;
            return evidence;
        }
        if let (Some(source), Some(source_dirty), Some(server), Some(server_dirty)) = (
            evidence.source_git_hash.as_deref(),
            evidence.source_dirty,
            evidence.server_git_hash.as_deref(),
            evidence.server_dirty,
        ) && valid_hash(source)
            && valid_hash(&evidence.client_git_hash)
            && valid_hash(server)
        {
            evidence.verdict = if !source_dirty
                && !client_dirty
                && !server_dirty
                && same_revision(source, &evidence.client_git_hash)
                && same_revision(source, server)
            {
                EvidenceVerdict::Matched
            } else {
                EvidenceVerdict::Mismatched
            };
        }
        evidence
    }

    pub fn capture(
        repo_dir: &Path,
        client_hash: &str,
        client_dirty: bool,
        server: Option<RuntimeIdentity>,
    ) -> Self {
        let source = super::current_git_hash_full(repo_dir).ok();
        let filters_absent = Command::new("git")
            .args([
                "-c",
                "core.fsmonitor=false",
                "config",
                "--get-regexp",
                "^filter\\.",
            ])
            .current_dir(repo_dir)
            .output()
            .ok()
            .is_some_and(|output| output.status.code() == Some(1) && output.stdout.is_empty());
        let dirty = filters_absent
            .then(|| {
                Command::new("git")
                    .args([
                        "-c",
                        "core.fsmonitor=false",
                        "status",
                        "--porcelain=v1",
                        "--untracked-files=all",
                    ])
                    .current_dir(repo_dir)
                    .output()
            })
            .and_then(Result::ok)
            .filter(|output| output.status.success())
            .map(|output| !output.stdout.is_empty());
        Self::new(source, dirty, client_hash.to_owned(), client_dirty, server)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jcode_selfdev_types::RuntimeIdentity;

    #[test]
    fn runtime_evidence_capture_never_executes_repository_filters_or_fsmonitor() {
        let repo = tempfile::tempdir().unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .args(args)
                .current_dir(repo.path())
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        };
        git(&["init", "-q"]);
        git(&["config", "core.fsmonitor", "touch fsmonitor-sentinel"]);
        let report = RuntimeEvidence::capture(repo.path(), "abcdef012", false, None);
        assert!(report.source_dirty.is_some());
        assert!(!repo.path().join("fsmonitor-sentinel").exists());
        git(&["config", "filter.fixture.clean", "touch filter-sentinel"]);
        std::fs::write(repo.path().join(".gitattributes"), "* filter=fixture\n").unwrap();
        let report = RuntimeEvidence::capture(repo.path(), "abcdef012", false, None);
        assert_eq!(report.source_dirty, None);
        assert!(!repo.path().join("filter-sentinel").exists());
    }

    fn evidence(
        source_dirty: bool,
        client_dirty: bool,
        server: Option<RuntimeIdentity>,
    ) -> RuntimeEvidence {
        RuntimeEvidence::new(
            Some("abcdef0123456789012345678901234567890123".into()),
            Some(source_dirty),
            "abcdef012".into(),
            client_dirty,
            server,
        )
    }

    fn server(hash: &str, dirty: bool) -> RuntimeIdentity {
        RuntimeIdentity {
            cleanliness_verified: true,
            git_hash: hash.into(),
            dirty,
            binary_path: Some("/resolved/jcode".into()),
        }
    }

    #[test]
    fn runtime_evidence_matched_clean_hashes_and_abbreviations() {
        let report = evidence(false, false, Some(server("abcdef012", false)));
        assert_eq!(report.verdict, EvidenceVerdict::Matched);
        assert_eq!(
            report.server_binary_path.as_deref(),
            Some(std::path::Path::new("/resolved/jcode"))
        );
    }

    #[test]
    fn runtime_evidence_mismatch_names_stale_server() {
        let report = evidence(false, false, Some(server("123456789", false)));
        assert_eq!(report.verdict, EvidenceVerdict::Mismatched);
        assert_eq!(report.server_git_hash.as_deref(), Some("123456789"));
        assert!(report.source_git_hash.unwrap().starts_with("abcdef012"));
    }

    #[test]
    fn runtime_evidence_dirty_never_matches() {
        for (source, client, daemon) in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            assert_eq!(
                evidence(source, client, Some(server("abcdef012", daemon))).verdict,
                EvidenceVerdict::Mismatched
            );
        }
    }

    #[test]
    fn runtime_evidence_unknown_never_matches() {
        let mut unverified = server("abcdef012", false);
        unverified.cleanliness_verified = false;
        assert_eq!(
            evidence(false, false, Some(unverified.clone())).verdict,
            EvidenceVerdict::Unknown
        );
        unverified.git_hash = "123456789".into();
        assert_eq!(
            evidence(false, false, Some(unverified)).verdict,
            EvidenceVerdict::Mismatched
        );
        assert_eq!(
            evidence(false, false, None).verdict,
            EvidenceVerdict::Unknown
        );
        for hash in ["unknown", "", "a", "abcdefg", "abc1234-dirty"] {
            assert_eq!(
                evidence(false, false, Some(server(hash, false))).verdict,
                EvidenceVerdict::Unknown
            );
        }
        let report = RuntimeEvidence::capture(
            std::path::Path::new("/nonexistent/jcode-runtime-evidence"),
            "abcdef012",
            false,
            Some(server("abcdef012", false)),
        );
        assert_eq!(report.source_dirty, None);
        assert_eq!(report.verdict, EvidenceVerdict::Unknown);
    }
}
