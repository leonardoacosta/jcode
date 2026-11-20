//! `[workspaces.rules.hindsight]`: per-world remote Hindsight memory route.
//!
//! Lives only in the user's global config (workspace rules cannot come from a
//! project checkout). Holds a credential *reference* (`env:NAME`), never the
//! credential itself, so serializing or logging a route leaks no secret.

use serde::{Deserialize, Serialize};

pub const DEFAULT_HINDSIGHT_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_HINDSIGHT_MAX_RESULTS: usize = 8;
pub const DEFAULT_HINDSIGHT_MAX_QUERY_CHARS: usize = 2_000;
pub const DEFAULT_HINDSIGHT_QUEUE_CAPACITY: usize = 4;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HindsightRoute {
    /// Base URL of the Hindsight API, for example `https://hindsight.example`.
    pub endpoint: String,
    /// Bank identifier owned by this world. Routing data, not authorization.
    pub bank: String,
    /// Credential reference, `env:VARIABLE_NAME`. Resolved only at the
    /// transport boundary. Must differ from every other world's reference.
    pub credential: String,
    /// Data scopes whose automatic retain is allowed (`explicit`,
    /// `transcript`). Empty (the default) means automatic retain is off.
    pub retain_scopes: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub max_results: Option<usize>,
    pub max_query_chars: Option<usize>,
    pub queue_capacity: Option<usize>,
}

impl HindsightRoute {
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_HINDSIGHT_TIMEOUT_MS)
            .min(120_000)
    }

    pub fn max_results(&self) -> usize {
        self.max_results
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_HINDSIGHT_MAX_RESULTS)
            .min(50)
    }

    pub fn max_query_chars(&self) -> usize {
        self.max_query_chars
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_HINDSIGHT_MAX_QUERY_CHARS)
            .min(20_000)
    }

    pub fn queue_capacity(&self) -> usize {
        self.queue_capacity
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_HINDSIGHT_QUEUE_CAPACITY)
            .min(64)
    }

    /// Environment variable named by `credential`, `None` for any other form.
    pub fn credential_env_var(&self) -> Option<&str> {
        self.credential
            .trim()
            .strip_prefix("env:")
            .map(str::trim)
            .filter(|name| !name.is_empty())
    }

    pub fn retains(&self, scope: &str) -> bool {
        self.retain_scopes
            .iter()
            .any(|s| s.trim().eq_ignore_ascii_case(scope))
    }

    /// Problems that make this route unusable (the adapter denies it).
    pub fn warnings(&self, label: &str) -> Vec<String> {
        let mut out = Vec::new();
        if self.endpoint.trim().is_empty() || self.bank.trim().is_empty() {
            out.push(format!(
                "{label}: hindsight needs `endpoint` and `bank`; Hindsight memory is denied"
            ));
        }
        if let Some(rest) = self.endpoint.trim().strip_prefix("http://") {
            let host = rest.split(['/', ':']).next().unwrap_or("");
            if !(host.eq_ignore_ascii_case("localhost")
                || host.starts_with("127.")
                || rest.starts_with("[::1]"))
            {
                out.push(format!(
                    "{label}: hindsight `endpoint` must use https (http is allowed only for loopback); Hindsight memory is denied"
                ));
            }
        }
        if self.credential_env_var().is_none() {
            out.push(format!(
                "{label}: hindsight `credential` must be a reference like `env:HINDSIGHT_KEY`; Hindsight memory is denied"
            ));
        }
        for scope in &self.retain_scopes {
            if !matches!(
                scope.trim().to_ascii_lowercase().as_str(),
                "explicit" | "transcript"
            ) {
                out.push(format!(
                    "{label}: unknown hindsight retain scope {scope:?} (expected explicit or transcript)"
                ));
            }
        }
        out
    }
}
