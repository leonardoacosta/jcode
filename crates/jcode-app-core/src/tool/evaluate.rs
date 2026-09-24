//! Jev evaluate tool — typed probabilistic judgments from TypeSafe's System One model.
//!
//! Delegates to `jcode_system_one::SystemOneService` for provider resolution and
//! HTTP transport, preserving the tool's own cache, retry policy, and schema.
//!
//! Uses the deterministic jcode_jev_cache to avoid re-billing identical decisions.

use super::{Tool, ToolContext, ToolOutput};
use crate::tool::jev_cache::JevCache;
use anyhow::Result;
use async_trait::async_trait;
use jcode_system_one::{resolve_service, LiveSystemOneService, SystemOneResponse, SystemOneService};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

const REQUEST_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

pub struct EvaluateTool {
    service: Arc<dyn SystemOneService>,
    cache: Arc<JevCache>,
}

impl EvaluateTool {
    pub fn new(cache: Arc<JevCache>) -> Self {
        let config = resolve_service(None)
            .expect("EvaluateTool: no System One credential available at construction");
        let service = Arc::new(LiveSystemOneService::with_timeout(
            config,
            Duration::from_secs(REQUEST_TIMEOUT_SECS),
        ));
        Self { service, cache }
    }

    #[cfg(test)]
    pub fn with_service(cache: Arc<JevCache>, service: Arc<dyn SystemOneService>) -> Self {
        Self { service, cache }
    }
}

/// Input shape for the evaluate tool.
#[derive(Debug, Deserialize)]
struct EvaluateInput {
    #[serde(default)]
    #[allow(dead_code)]
    intent: Option<String>,
    state: Value,
    questions: std::collections::HashMap<String, Question>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, serde::Serialize, Deserialize)]
#[serde(tag = "type")]
enum Question {
    #[serde(rename = "noul")]
    Noul { instructions: Value },
    #[serde(rename = "choice")]
    Choice {
        instructions: Value,
        criteria: std::collections::HashMap<String, Value>,
    },
    #[serde(rename = "score")]
    Score {
        instructions: Value,
        criteria: Vec<Value>,
    },
}

#[async_trait]
impl Tool for EvaluateTool {
    fn name(&self) -> &str {
        "evaluate"
    }

    fn description(&self) -> &str {
        "Call TypeSafe Jev for typed probabilistic judgments. Use for yes/no questions (noul), \
         choosing from defined options (choice), or rating on ordered levels (score). \
         Send state (text or JSON) plus typed questions; get back probabilities your code \
         can branch on directly. Ask one narrow judgment per question. Include a no-match \
         option in choice criteria when nothing may fit. Keep policy in code — Jev returns \
         probabilities, you set thresholds."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["state", "questions"],
            "properties": {
                "intent": super::intent_schema_property(),
                "state": {
                    "type": ["string", "object", "array"],
                    "description": "Content to judge: plain text, or structured JSON with named fields. Raw evidence, not your conclusion about it."
                },
                "questions": {
                    "type": "object",
                    "description": "Map of question IDs to typed questions. Each question has 'type' (noul/choice/score) and 'instructions'. Choice also needs 'criteria' (map of option→description). Score also needs 'criteria' (ordered levels).",
                    "additionalProperties": {
                        "type": "object",
                        "required": ["type", "instructions"],
                        "properties": {
                            "type": {
                                "type": "string",
                                "enum": ["noul", "choice", "score"]
                            },
                            "instructions": {
                                "type": ["string", "object", "array"],
                                "description": "The question to evaluate. For noul: yes/no question. For choice: which option fits? For score: where on the scale?"
                            },
                            "criteria": {
                                "description": "For choice: map of option → description. For score: ordered list of level descriptions."
                            }
                        }
                    }
                },
                "model": {
                    "type": "string",
                    "description": "Model override. Defaults to the resolver's configured model."
                }
            }
        })
    }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let params: EvaluateInput = serde_json::from_value(input)?;

        if params.questions.is_empty() {
            anyhow::bail!("evaluate requires at least one question");
        }

        let questions_json = serde_json::to_value(&params.questions)?;
        let cache_model = params.model.as_deref().unwrap_or("");
        let fp = JevCache::fingerprint(cache_model, &questions_json, &params.state, "");
        if let Some(cached) = self.cache.check(&fp) {
            crate::logging::debug(&format!("Jev cache HIT fp={fp}"));
            return Ok(ToolOutput::new(format!("{cached}")));
        }
        crate::logging::debug(&format!("Jev cache MISS fp={fp}"));

        self.cache.record_call();

        let questions: std::collections::HashMap<String, Value> = params
            .questions
            .into_iter()
            .map(|(id, q)| {
                let v = serde_json::to_value(&q).expect("serialize Question");
                (id, v)
            })
            .collect();

        let model_override = params.model.as_deref();
        let response = send_with_retry(
            self.service.as_ref(),
            &params.state,
            &questions,
            model_override,
            MAX_RETRIES,
        )
        .await?;

        for id in questions.keys() {
            if !response.answers.contains_key(id) {
                anyhow::bail!("evaluate: missing answer for question '{id}' in response");
            }
        }

        let answer_value = serde_json::to_value(&response)?;
        let answer_for_cache = serde_json::to_value(&response.answers)?;
        let _ = self.cache.store(&fp, cache_model, &answer_for_cache);

        Ok(ToolOutput::new(format!("{answer_value}")))
    }
}

async fn send_with_retry(
    service: &dyn SystemOneService,
    state: &Value,
    questions: &std::collections::HashMap<String, Value>,
    model_override: Option<&str>,
    max_retries: u32,
) -> Result<SystemOneResponse> {
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 0..=max_retries {
        match service
            .evaluate(state.clone(), questions.clone(), model_override)
            .await
        {
            Ok(response) => return Ok(response),
            Err(e) => {
                let status_is_retryable =
                    e.to_string().contains("429") || e.to_string().contains("529");
                if status_is_retryable && attempt < max_retries {
                    let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                    crate::logging::debug(&format!(
                        "evaluate: retryable error, attempt {}/{max_retries}, waiting {}ms",
                        attempt + 1,
                        delay.as_millis()
                    ));
                    tokio::time::sleep(delay).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("evaluate: unknown error")))
}