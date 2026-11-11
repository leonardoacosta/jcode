//! Automatic, bounded, visible model fallback on transient errors.
//!
//! Runs after per-request retries and same-provider account failover are
//! exhausted. Permanent errors (auth, invalid request, config) never switch.

use std::collections::{HashMap, VecDeque};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

pub(super) const COOL_FAILURES: usize = 3;
pub(super) const COOL_WINDOW: Duration = Duration::from_secs(60);
pub(super) const MAX_CYCLES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ErrorClass {
    Transient,
    Permanent,
}

impl ErrorClass {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::Permanent => "permanent",
        }
    }
}

/// Classify a turn error. Unknown errors are permanent so we never switch
/// models on something we do not understand.
pub(super) fn classify(message: &str) -> ErrorClass {
    // An uncertain refresh failure is not revocation, but changing models must
    // not escape the selected login's failed authentication transaction.
    if message.to_ascii_lowercase().contains("refresh") {
        return ErrorClass::Permanent;
    }
    match jcode_provider_core::classify_provider_failure(message) {
        jcode_provider_core::ProviderFailureKind::Transient
        | jcode_provider_core::ProviderFailureKind::RateLimited => ErrorClass::Transient,
        _ => ErrorClass::Permanent,
    }
}

/// Configured chain for `model`, without blanks or self-references.
pub(super) fn chain_for(map: &HashMap<String, Vec<String>>, model: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for c in map.get(model).into_iter().flatten() {
        let c = c.trim();
        if !c.is_empty() && c != model && !out.iter().any(|o| o == c) {
            out.push(c.to_string());
        }
    }
    out
}

#[derive(Default)]
pub(super) struct Health {
    failures: HashMap<String, VecDeque<Instant>>,
}

impl Health {
    pub(super) fn record_failure(&mut self, model: &str, now: Instant) {
        let q = self.failures.entry(model.to_string()).or_default();
        q.push_back(now);
        while q.len() > COOL_FAILURES {
            q.pop_front();
        }
    }

    pub(super) fn record_success(&mut self, model: &str) {
        self.failures.remove(model);
    }

    pub(super) fn is_cooling(&self, model: &str, now: Instant) -> bool {
        self.failures.get(model).is_some_and(|q| {
            q.iter().filter(|t| now.duration_since(**t) < COOL_WINDOW).count() >= COOL_FAILURES
        })
    }

    /// Healthy first, cooling last, config order preserved within each group.
    pub(super) fn order(&self, chain: &[String], now: Instant) -> Vec<(String, bool)> {
        let (cool, warm): (Vec<_>, Vec<_>) =
            chain.iter().cloned().partition(|m| self.is_cooling(m, now));
        warm.into_iter().map(|m| (m, false)).chain(cool.into_iter().map(|m| (m, true))).collect()
    }
}

pub(super) static HEALTH: LazyLock<Mutex<Health>> = LazyLock::new(Default::default);

pub(super) fn health() -> std::sync::MutexGuard<'static, Health> {
    HEALTH.lock().unwrap_or_else(|p| p.into_inner())
}

/// Ordered attempt plan: up to `MAX_CYCLES` passes over the ordered chain.
pub(super) fn plan(chain: &[String], health: &Health, now: Instant) -> Vec<(String, bool)> {
    let ordered = health.order(chain, now);
    (0..MAX_CYCLES).flat_map(|_| ordered.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier() {
        for t in ["HTTP 429 Too Many Requests", "503 Service Unavailable", "status 529 overloaded", "connection reset by peer", "stream ended unexpectedly", "HTTP 408"] {
            assert_eq!(classify(t), ErrorClass::Transient, "{t}");
        }
        for p in ["401 Unauthorized", "400 invalid_request_error", "provider not configured", "403 forbidden rate limit", "weird"] {
            assert_eq!(classify(p), ErrorClass::Permanent, "{p}");
        }
        assert_eq!(classify("port 14290"), ErrorClass::Permanent);
    }

    #[test]
    fn shared_taxonomy_never_substitutes_after_quota_or_refresh_failure() {
        for message in [
            "429 insufficient_quota",
            "429 billing exhausted",
            "invalid_grant connection reset",
            "OpenAI token refresh failed: HTTP 503 service unavailable",
        ] {
            assert_eq!(classify(message), ErrorClass::Permanent, "{message}");
        }
    }

    #[test]
    fn chain_drops_blank_and_self() {
        let map = HashMap::from([("a".into(), vec!["".into(), "a".into(), " b ".into(), "b".into(), "c".into()])]);
        assert_eq!(chain_for(&map, "a"), vec!["b", "c"]);
        assert!(chain_for(&map, "z").is_empty());
    }

    #[test]
    fn cooling_window_and_reset() {
        let mut h = Health::default();
        let t0 = Instant::now();
        h.record_failure("b", t0);
        h.record_failure("b", t0);
        assert!(!h.is_cooling("b", t0));
        h.record_failure("b", t0);
        assert!(h.is_cooling("b", t0));
        assert!(!h.is_cooling("b", t0 + COOL_WINDOW));
        h.record_success("b");
        assert!(!h.is_cooling("b", t0));
    }

    #[test]
    fn ordering_and_cycle_cap() {
        let mut h = Health::default();
        let t0 = Instant::now();
        for _ in 0..3 {
            h.record_failure("b", t0);
        }
        let chain = vec!["b".to_string(), "c".to_string(), "d".to_string()];
        let order: Vec<_> = h.order(&chain, t0);
        assert_eq!(order, vec![("c".into(), false), ("d".into(), false), ("b".into(), true)]);
        assert_eq!(plan(&chain, &h, t0).len(), 3 * MAX_CYCLES);
    }
}
