//! Modular System One (Jev) service.
//!
//! Provides a `SystemOneService` trait for typed probabilistic judgments,
//! a shared provider/credential resolver, and a live HTTP implementation.
//! Callers inject the trait; this crate has no dependency on jcode-app-core,
//! jcode-tui, or jcode-protocol.

pub mod resolver;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

pub use resolver::{Provider, ServiceConfig};

/// Default request timeout for the live service.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// A typed response from the System One API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemOneResponse {
    pub answers: HashMap<String, Value>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Value>,
}

/// The System One evaluation service trait.
///
/// Implementations may be live (HTTP), cached, mocked, or debug.
#[async_trait]
pub trait SystemOneService: Send + Sync {
    /// Send state and typed questions to the System One model.
    ///
    /// `state` is raw evidence — plain text or structured JSON.
    /// `questions` maps question IDs to their typed question bodies
    /// (already serialized as Value by the caller).
    /// `model_override` replaces the resolver's default model when set.
    async fn evaluate(
        &self,
        state: Value,
        questions: HashMap<String, Value>,
        model_override: Option<&str>,
    ) -> Result<SystemOneResponse>;
}

// ---------------------------------------------------------------------------
// Live HTTP service
// ---------------------------------------------------------------------------

/// A live System One service that sends requests to the resolved endpoint.
pub struct LiveSystemOneService {
    client: reqwest::Client,
    config: ServiceConfig,
}

impl LiveSystemOneService {
    /// Create a live service from the resolved config with the default timeout.
    pub fn new(config: ServiceConfig) -> Self {
        Self::with_timeout(config, DEFAULT_REQUEST_TIMEOUT)
    }

    /// Create a live service with a caller-specified timeout.
    pub fn with_timeout(config: ServiceConfig, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .timeout(timeout)
            .build()
            .expect("build reqwest client for System One service");
        Self { client, config }
    }
}

#[async_trait]
impl SystemOneService for LiveSystemOneService {
    async fn evaluate(
        &self,
        state: Value,
        questions: HashMap<String, Value>,
        model_override: Option<&str>,
    ) -> Result<SystemOneResponse> {
        let model = model_override.unwrap_or(&self.config.default_model);
        let body = serde_json::json!({
            "state": state,
            "model": model,
            "questions": questions,
        });

        let response = self
            .client
            .post(&self.config.endpoint_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let status_text = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "System One service: HTTP {status} from {}: {status_text}",
                self.config.endpoint_url
            );
        }

        let parsed: SystemOneResponse = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("System One service: invalid response: {e}"))?;

        // Validate: every requested question must have an answer.
        let body_map = body
            .as_object()
            .and_then(|o| o.get("questions"))
            .and_then(|q| q.as_object());
        if let Some(questions_obj) = body_map {
            for key in questions_obj.keys() {
                if !parsed.answers.contains_key(key) {
                    anyhow::bail!(
                        "System One service: missing answer for question '{key}' in response"
                    );
                }
            }
        }

        Ok(parsed)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_roundtrip() {
        let response = SystemOneResponse {
            answers: {
                let mut m = HashMap::new();
                m.insert(
                    "q".into(),
                    serde_json::json!({"type": "noul", "noul": 0.95}),
                );
                m
            },
            model: "jev-latest".into(),
            usage: Some(serde_json::json!({"input_tokens": 100})),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: SystemOneResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, parsed);
        assert_eq!(parsed.answers["q"]["noul"], 0.95);
    }

    #[test]
    fn test_response_no_usage() {
        let response = SystemOneResponse {
            answers: HashMap::new(),
            model: "jev-latest".into(),
            usage: None,
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: SystemOneResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.usage, None);
    }

    /// A mock service that always returns a fixed response.
    struct MockService {
        response: SystemOneResponse,
    }

    #[async_trait]
    impl SystemOneService for MockService {
        async fn evaluate(
            &self,
            _state: Value,
            _questions: HashMap<String, Value>,
            _model_override: Option<&str>,
        ) -> Result<SystemOneResponse> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_mock_service_trait() {
        let svc = MockService {
            response: SystemOneResponse {
                answers: {
                    let mut m = HashMap::new();
                    m.insert(
                        "a".into(),
                        serde_json::json!({"type": "choice", "choice": "yes"}),
                    );
                    m
                },
                model: "test".into(),
                usage: None,
            },
        };
        let result = svc
            .evaluate(serde_json::json!("test"), HashMap::new(), None)
            .await
            .unwrap();
        assert_eq!(result.answers["a"]["choice"], "yes");
        assert_eq!(result.model, "test");
    }

    #[tokio::test]
    async fn test_mock_service_model_override_ignored() {
        let svc = MockService {
            response: SystemOneResponse {
                answers: HashMap::new(),
                model: "built-in".into(),
                usage: None,
            },
        };
        let result = svc
            .evaluate(
                serde_json::json!("state"),
                HashMap::new(),
                Some("override-model"),
            )
            .await
            .unwrap();
        // The mock ignores the override; live implementations use it.
        assert_eq!(result.model, "built-in");
    }
}
