//! Deterministic decision cache for Jev evaluate calls.
//!
//! Jev decisions are approximately pure functions of (model, schema, state).
//! This module memoizes by a content-addressed fingerprint so the same
//! decision never runs — or bills — twice. A local hit returns in ~0ms for
//! $0; a miss routes to the backend and caches the result.
//!
//! Design follows the architecture of <https://jevcache.sh>.
//!
//! NB: dead_code warnings are expected until the evaluate tool integrates
//! this module (Phase 2 of jcode-jevy-evaluate-cache).
#![allow(dead_code)]

use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Fingerprint → cached answer.
type Index = HashMap<String, serde_json::Value>;

/// Telemetry counters.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total evaluate calls (including cache checks).
    pub calls: u64,
    /// Cache hits (answer served from cache, no API call).
    pub hits: u64,
    /// Estimated cost saved, based on `cost_per_call`.
    pub cost_saved: f64,
}

/// Deterministic decision cache.
pub struct JevCache {
    /// In-memory index for ~0ms lookups.
    index: RwLock<Index>,
    /// Append-only ledger path on disk.
    ledger_path: PathBuf,
    /// Optional per-schema salts (schema_name → salt).
    salts: HashMap<String, String>,
    /// Telemetry.
    stats: RwLock<CacheStats>,
    /// Estimated USD cost per evaluate call (default $0.0004).
    cost_per_call: f64,
    /// When true, all operations are skipped (corrupt/unwritable ledger, or
    /// JEVCACHE_DISABLE).
    disabled: bool,
}

impl JevCache {
    /// Create a new cache, loading the ledger from disk.
    ///
    /// Checks `JEVCACHE_DISABLE` env var — when set to `"true"` or `"1"`,
    /// the cache is disabled and all operations become no-ops.
    pub fn new(ledger_dir: &Path) -> Result<Self> {
        let disabled = std::env::var("JEVCACHE_DISABLE").map_or(false, |v| v == "true" || v == "1");

        if disabled {
            crate::logging::debug(&"Jev cache: disabled (JEVCACHE_DISABLE set)".to_string());
        }

        Self::new_inner(ledger_dir, disabled)
    }

    /// Create a cache, explicitly controlling whether it is disabled.
    fn new_inner(ledger_dir: &Path, disabled: bool) -> Result<Self> {
        if disabled {
            return Ok(Self::disabled(ledger_dir));
        }

        let cost_per_call = std::env::var("JEVCACHE_ESTIMATED_COST_PER_CALL")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0004);

        let ledger_path = ledger_dir.join("ledger.jsonl");

        // Load salts from env.
        let mut salts = HashMap::new();
        for (key, value) in std::env::vars() {
            if let Some(name) = key.strip_prefix("JEVCACHE_SALT_") {
                if !name.is_empty() {
                    salts.insert(name.to_string(), value);
                }
            }
        }

        // Load existing ledger entries.
        let mut index: Index = HashMap::new();
        let mut load_errors = 0u64;

        if let Ok(f) = std::fs::File::open(&ledger_path) {
            let reader = std::io::BufReader::new(f);
            for line in reader.lines() {
                match line {
                    Ok(line) if line.trim().is_empty() => continue,
                    Ok(line) => match serde_json::from_str::<serde_json::Value>(&line) {
                        Ok(entry) => {
                            if let (Some(fp), Some(answer)) = (
                                entry.get("fp").and_then(|v| v.as_str()),
                                entry.get("answer"),
                            ) {
                                index.insert(fp.to_string(), answer.clone());
                            }
                        }
                        Err(_) => {
                            load_errors += 1;
                        }
                    },
                    Err(_) => {
                        load_errors += 1;
                    }
                }
            }
        }

        if load_errors > 0 {
            crate::logging::warn(&format!(
                "Jev cache: {load_errors} unparseable lines in ledger; skipping them"
            ));
        }

        crate::logging::debug(&format!(
            "Jev cache loaded {} entries from ledger",
            index.len()
        ));

        Ok(Self {
            index: RwLock::new(index),
            ledger_path,
            salts,
            stats: RwLock::new(CacheStats::default()),
            cost_per_call,
            disabled: false,
        })
    }

    /// Return a cache that is permanently disabled.
    fn disabled(ledger_dir: &Path) -> Self {
        Self {
            index: RwLock::new(HashMap::new()),
            ledger_path: ledger_dir.join("ledger.jsonl"),
            salts: HashMap::new(),
            stats: RwLock::new(CacheStats::default()),
            cost_per_call: 0.0004,
            disabled: true,
        }
    }

    /// Compute a deterministic fingerprint for an evaluate call.
    ///
    /// `fingerprint(model, schema, state, salt)` returns hex-encoded SHA-256
    /// of `model \0 canonical_schema \0 canonical(redact(state)) \0 salt`.
    pub fn fingerprint(
        model: &str,
        schema: &serde_json::Value,
        state: &serde_json::Value,
        salt: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(model.as_bytes());
        hasher.update(b"\0");
        hasher.update(&canonicalize_json(schema));
        hasher.update(b"\0");

        // Redact before canonicalization: strip volatile fields so
        // logically identical states produce identical keys.
        let redacted = redact_state(state);
        hasher.update(&canonicalize_json(&redacted));
        hasher.update(b"\0");
        hasher.update(salt.as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Check the cache. Returns `Some(answer)` on hit, `None` on miss.
    pub fn check(&self, fp: &str) -> Option<serde_json::Value> {
        if self.disabled {
            return None;
        }
        let index = self.index.read().expect("JevCache index lock poisoned");
        let hit = index.get(fp).cloned();
        if hit.is_some() {
            if let Ok(mut stats) = self.stats.write() {
                stats.hits += 1;
                stats.cost_saved += self.cost_per_call;
            }
        }
        hit
    }

    /// Store a new answer in the cache.
    ///
    /// Appends to the ledger file and updates the in-memory index.
    /// Returns `Ok(())` even if the ledger write fails (the in-memory
    /// index is always updated first).
    pub fn store(&self, fp: &str, model: &str, answer: &serde_json::Value) -> Result<()> {
        if self.disabled {
            return Ok(());
        }

        // Update in-memory index first — this always succeeds.
        {
            let mut index = self.index.write().expect("JevCache index lock poisoned");
            index.insert(fp.to_string(), answer.clone());
        }

        // Append to ledger (best-effort).
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let line = serde_json::json!({
            "fp": fp,
            "ts": ts,
            "model": model,
            "answer": answer,
        });

        match self.append_ledger_line(&line) {
            Ok(()) => {}
            Err(e) => {
                crate::logging::warn(&format!(
                    "Jev cache: failed to persist entry to ledger: {e}"
                ));
            }
        }

        Ok(())
    }

    /// Record an evaluate call that was not a cache hit.
    ///
    /// Separated from `check` so that `JEVCACHE_DISABLE` bypasses call
    /// correctly increment this counter too.
    pub fn record_call(&self) {
        if let Ok(mut stats) = self.stats.write() {
            stats.calls += 1;
        }
    }

    /// Return current cache statistics.
    pub fn stats(&self) -> CacheStats {
        self.stats.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Log a one-line cache stats summary at info level.
    pub fn log_stats(&self) {
        let s = self.stats();
        let hit_rate = if s.calls > 0 {
            (s.hits as f64 / s.calls as f64) * 100.0
        } else {
            0.0
        };
        crate::logging::info(&format!(
            "Jev cache: {} calls, {} hits ({:.1}%), ${:.4} saved",
            s.calls, s.hits, hit_rate, s.cost_saved,
        ));
    }

    /// Return the salt configured for a schema name, if any.
    ///
    /// Used by the evaluate tool to include the salt when computing
    /// fingerprints for sensitive schemas.
    pub fn salt_for_schema(&self, name: &str) -> &str {
        self.salts.get(name).map(|s| s.as_str()).unwrap_or("")
    }

    // ── private helpers ──

    fn append_ledger_line(&self, value: &serde_json::Value) -> Result<()> {
        if let Some(parent) = self.ledger_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create cache dir: {}", parent.display()))?;
        }

        let mut buf = serde_json::to_vec(value)?;
        buf.push(b'\n');

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.ledger_path)
            .with_context(|| format!("open ledger: {}", self.ledger_path.display()))?;

        file.write_all(&buf)
            .with_context(|| format!("write ledger: {}", self.ledger_path.display()))?;

        Ok(())
    }
}

// ── fingerprinting helpers ──

/// Redact PII-like patterns from state before hashing so logically
/// identical states (differing only in volatile fields) produce the
/// same fingerprint.
fn redact_state(state: &serde_json::Value) -> serde_json::Value {
    match state {
        serde_json::Value::String(s) => serde_json::Value::String(redact_string(s)),
        // For structured values, redact recursively.
        serde_json::Value::Object(map) => {
            let redacted: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), redact_state(v)))
                .collect();
            serde_json::Value::Object(redacted)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(redact_state).collect())
        }
        other => other.clone(),
    }
}

/// Redact PII patterns from a string:
/// - Email addresses → `[EMAIL]`
/// - Phone numbers → `[PHONE]`
/// - UUIDs → `[UUID]`
/// - ISO timestamps → `[TIMESTAMP]`
/// - Long digit runs (>6 consecutive) → `[DIGITS]`
fn redact_string(s: &str) -> String {
    // Order matters: UUID before long digit runs, timestamp before phone.
    let s = replace_pattern(
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
        "[EMAIL]",
        s,
    );
    // UUID-like patterns (8-4-4-4-12 hex, optional prefix like id=)
    let s = replace_pattern(
        r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}",
        "[UUID]",
        &s,
    );
    let s = replace_pattern(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}", "[TIMESTAMP]", &s);
    // Phone-like: 7–15 digits with optional +, spaces, dashes, parens.
    let s = replace_pattern(r"\+?[\d\s\-().]{7,}", "[PHONE]", &s);
    // Long isolated digit runs (>6 consecutive digits).
    let s = replace_pattern(r"\b\d{7,}\b", "[DIGITS]", &s);
    s
}

/// Apply a regex replacement over all matches in the string.
fn replace_pattern(pattern: &str, replacement: &str, s: &str) -> String {
    match Regex::new(pattern) {
        Ok(re) => re.replace_all(s, replacement).into_owned(),
        Err(e) => {
            crate::logging::warn(&format!("Jev cache: invalid redaction pattern: {e}"));
            s.to_string()
        }
    }
}

/// Canonicalize a JSON value for deterministic hashing.
///
/// `serde_json::to_vec` on `serde_json::Value` produces sorted keys
/// (unless the `preserve_order` feature is enabled — it is not in this
/// crate) and compact representation with no extra whitespace. Numbers
/// are formatted via `serde_json`'s own `ryu`-based formatter.
fn canonicalize_json(value: &serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap_or_else(|_| b"null".to_vec())
}

// ── tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn fingerprint_deterministic() {
        let state = serde_json::json!({"task": "deploy", "env": "production"});
        let schema = serde_json::json!({"q": {"type": "noul", "instructions": "Is this urgent?"}});

        let fp1 = JevCache::fingerprint("jev-latest", &schema, &state, "");
        let fp2 = JevCache::fingerprint("jev-latest", &schema, &state, "");
        assert_eq!(fp1, fp2, "same inputs must produce same fingerprint");
    }

    #[test]
    fn fingerprint_different_model() {
        let state = serde_json::json!("test");
        let schema = serde_json::json!({"q": {"type": "noul"}});

        let fp1 = JevCache::fingerprint("jev-latest", &schema, &state, "");
        let fp2 = JevCache::fingerprint("jev-2", &schema, &state, "");
        assert_ne!(
            fp1, fp2,
            "different models must produce different fingerprints"
        );
    }

    #[test]
    fn fingerprint_different_schema() {
        let state = serde_json::json!("test");
        let s1 = serde_json::json!({"q": {"type": "noul"}});
        let s2 = serde_json::json!({"q": {"type": "choice"}});

        let fp1 = JevCache::fingerprint("jev-latest", &s1, &state, "");
        let fp2 = JevCache::fingerprint("jev-latest", &s2, &state, "");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn fingerprint_salt_changes() {
        let state = serde_json::json!("test");
        let schema = serde_json::json!({"q": {"type": "noul"}});

        let fp1 = JevCache::fingerprint("jev-latest", &schema, &state, "");
        let fp2 = JevCache::fingerprint("jev-latest", &schema, &state, "my_salt");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn redact_state_string_uuid() {
        let s1 = serde_json::Value::String(
            "Error in request abc12345-6789-4abc-def0-1234567890ab: timeout".into(),
        );
        let s2 = serde_json::Value::String(
            "Error in request ffffffff-ffff-ffff-ffff-ffffffffffff: timeout".into(),
        );
        let schema = serde_json::json!({"q": {"type": "noul"}});

        let fp1 = JevCache::fingerprint("jev-latest", &schema, &s1, "");
        let fp2 = JevCache::fingerprint("jev-latest", &schema, &s2, "");
        assert_eq!(fp1, fp2, "UUID differences must be redacted away");
    }

    #[test]
    fn redact_state_string_timestamp() {
        let s1 = serde_json::Value::String("Deploy at 2026-09-19T18:30:00 failed".into());
        let s2 = serde_json::Value::String("Deploy at 2025-01-01T00:00:00 failed".into());
        let schema = serde_json::json!({"q": {"type": "noul"}});

        let fp1 = JevCache::fingerprint("jev-latest", &schema, &s1, "");
        let fp2 = JevCache::fingerprint("jev-latest", &schema, &s2, "");
        assert_eq!(fp1, fp2, "timestamp differences must be redacted away");
    }

    #[test]
    fn redact_state_object_recursive() {
        let s1 = serde_json::json!({
            "user": "alice@example.com",
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "action": "deploy",
            "nested": {"email": "bob@test.com"}
        });
        let s2 = serde_json::json!({
            "user": "carol@other.org",
            "id": "00000000-0000-0000-0000-000000000000",
            "action": "deploy",
            "nested": {"email": "dave@other.org"}
        });
        let schema = serde_json::json!({"q": {"type": "noul"}});

        let fp1 = JevCache::fingerprint("jev-latest", &schema, &s1, "");
        let fp2 = JevCache::fingerprint("jev-latest", &schema, &s2, "");
        assert_eq!(
            fp1, fp2,
            "email and UUID differences must be redacted away in structured objects"
        );
    }

    #[test]
    fn canonicalize_json_sorted_keys() {
        let v1 = serde_json::json!({"b": 2, "a": 1});
        let v2 = serde_json::json!({"a": 1, "b": 2});
        assert_eq!(canonicalize_json(&v1), canonicalize_json(&v2));
    }

    #[test]
    fn cache_hit_and_miss() -> Result<()> {
        let dir = tempdir()?;

        let cache = JevCache::new_inner(dir.path(), false)?;

        let fp = "test_fp_001";

        // First check: miss.
        assert!(cache.check(fp).is_none(), "first check should be miss");

        // Store.
        cache.store(fp, "jev-latest", &serde_json::json!({"noul": 0.97}))?;

        // Second check: hit.
        let hit = cache.check(fp);
        assert!(hit.is_some(), "second check should be hit");
        assert_eq!(hit.unwrap()["noul"], 0.97);

        Ok(())
    }

    #[test]
    fn cache_disabled_by_env() -> Result<()> {
        let dir = tempdir()?;

        // Use new_inner directly to avoid env var leakage between parallel tests.
        let cache = JevCache::new_inner(dir.path(), true)?;

        // Store should succeed but be a no-op.
        cache.store("fp", "jev-latest", &serde_json::json!({"noul": 1.0}))?;

        // Check should always miss.
        assert!(cache.check("fp").is_none());

        Ok(())
    }

    #[test]
    fn cache_corrupt_ledger_is_graceful() -> Result<()> {
        let dir = tempdir()?;
        let ledger_path = dir.path().join("ledger.jsonl");

        // Write a corrupt line after a valid one.
        std::fs::write(
            &ledger_path,
            "{\"fp\":\"good\",\"ts\":1,\"model\":\"jev-latest\",\"answer\":{\"noul\":1.0}}\nnot valid json\n",
        )?;

        let cache = JevCache::new(dir.path())?;

        // Valid entry should be loaded.
        assert!(cache.check("good").is_some());

        // Cache should still be functional (not disabled).
        cache.store("new", "jev-latest", &serde_json::json!({"noul": 0.5}))?;
        let hit = cache.check("new");
        assert!(hit.is_some());
        assert_eq!(hit.unwrap()["noul"], 0.5);

        Ok(())
    }

    #[test]
    fn cache_cross_session_reload() -> Result<()> {
        let dir = tempdir()?;

        // Session 1: store.
        {
            let cache = JevCache::new_inner(dir.path(), false)?;
            cache.store("fp_a", "jev-latest", &serde_json::json!({"score": 3}))?;
        }

        // Session 2: reload.
        {
            let cache = JevCache::new_inner(dir.path(), false)?;
            let hit = cache.check("fp_a");
            assert!(hit.is_some(), "cross-session reload should hit");
            assert_eq!(hit.unwrap()["score"], 3);
        }

        Ok(())
    }

    #[test]
    fn telemetry_counts() -> Result<()> {
        let dir = tempdir()?;

        let cache = JevCache::new_inner(dir.path(), false)?;

        // 3 calls, 3 hits.
        cache.store("fp1", "jev-latest", &serde_json::json!({"a": 1}))?;
        cache.record_call(); // this check would have been a call
        cache.record_call(); // another call
        cache.check("fp1"); // hit
        cache.record_call(); // another call
        cache.check("fp1"); // second hit
        cache.check("fp1"); // third hit

        let stats = cache.stats();
        assert_eq!(stats.calls, 3, "3 non-cache calls recorded");
        assert_eq!(stats.hits, 3, "3 cache hits");
        assert!(stats.cost_saved > 0.0, "cost saved should be > 0 for hits");

        Ok(())
    }

    #[test]
    fn concurrent_check_and_store_stress() -> Result<()> {
        let dir = tempdir()?;
        let cache = std::sync::Arc::new(JevCache::new_inner(dir.path(), false)?);
        let threads = 4;
        let ops_per_thread = 50;

        std::thread::scope(|s| {
            for t in 0..threads {
                let cache = cache.clone();
                s.spawn(move || {
                    for i in 0..ops_per_thread {
                        let fp = format!("cfp_{t}_{i}");
                        if i % 2 == 0 {
                            let _ = cache.check(&fp);
                        } else {
                            let _ = cache.store(&fp, "jev-latest", &serde_json::json!({"v": i}));
                        }
                    }
                });
            }
        });

        // No panics is success. Verifies RwLock doesn't deadlock/poison
        // under concurrent check+store from multiple threads.
        Ok(())
    }
}
