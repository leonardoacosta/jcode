//! `[overnight]` section of the config: per-role model chains.

use serde::{Deserialize, Serialize};

pub const DEFAULT_COORDINATOR_MODEL_CHAIN: &[&str] = &[
    "claude-opus-5.5",
    "gpt-6-sol",
    "claude-sonnet-5.5",
    "gpt-6-luna",
    "DeepSeek-V4.1-Flash",
    "qwen3.5-35b",
];

pub const DEFAULT_WORKER_MODEL_CHAIN: &[&str] = &[
    "claude-sonnet-5.5",
    "gpt-6-luna",
    "DeepSeek-V4.1-Flash",
    "qwen3.5-35b",
];

/// Ordered model fallback chains for overnight runs. Every step stays on the
/// session's selected provider route and account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct OvernightConfig {
    /// Models the overnight coordinator tries, in order.
    pub coordinator_model_chain: Vec<String>,
    /// Models workers spawned by an overnight run try, in order.
    pub worker_model_chain: Vec<String>,
}

impl Default for OvernightConfig {
    fn default() -> Self {
        Self {
            coordinator_model_chain: DEFAULT_COORDINATOR_MODEL_CHAIN
                .iter()
                .map(|m| (*m).to_string())
                .collect(),
            worker_model_chain: DEFAULT_WORKER_MODEL_CHAIN
                .iter()
                .map(|m| (*m).to_string())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec_lists() {
        let config = OvernightConfig::default();
        assert_eq!(
            config.coordinator_model_chain,
            [
                "claude-opus-5.5",
                "gpt-6-sol",
                "claude-sonnet-5.5",
                "gpt-6-luna",
                "DeepSeek-V4.1-Flash",
                "qwen3.5-35b"
            ]
        );
        assert_eq!(
            config.worker_model_chain,
            [
                "claude-sonnet-5.5",
                "gpt-6-luna",
                "DeepSeek-V4.1-Flash",
                "qwen3.5-35b"
            ]
        );
    }

    #[test]
    fn parses_without_section_and_with_overrides() {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Root {
            overnight: OvernightConfig,
        }
        let none: Root = serde_json::from_str("{}").unwrap();
        assert_eq!(none.overnight, OvernightConfig::default());
        let partial: Root =
            serde_json::from_str(r#"{"overnight":{"worker_model_chain":["a","b"]}}"#).unwrap();
        assert_eq!(partial.overnight.worker_model_chain, ["a", "b"]);
        assert_eq!(
            partial.overnight.coordinator_model_chain,
            OvernightConfig::default().coordinator_model_chain
        );
    }
}
