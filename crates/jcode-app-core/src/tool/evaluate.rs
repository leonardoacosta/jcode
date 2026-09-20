//! Jev evaluate tool — typed probabilistic judgments from TypeSafe's System One model.
//!
//! Sends `POST /v1/systemone` with `state` and typed `questions`; returns structured
//! answers with probabilities that agent code can branch on directly.
//!
//! Uses the deterministic jcode_jev_cache to avoid re-billing identical decisions.

use super::{Tool, ToolContext, ToolOutput};
use crate::tool::jev_cache::JevCache;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::Duration;

const TYPESAFE_API_URL: &str = "https://api.typesafe.ai/v1/systemone";
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/alpha/decisions";
const DEFAULT_MODEL: &str = "jev-latest";
const OPENROUTER_MODEL: &str = "~typesafe/jev-latest";
const REQUEST_TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> = std::sync::LazyLock::new(|| {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()
        .expect("build reqwest client")
});

pub struct EvaluateTool {
    cache: Arc<JevCache>,
}

impl EvaluateTool {
    pub fn new(cache: Arc<JevCache>) -> Self {
        Self { cache }
    }
}

/// Input shape for the evaluate tool.
#[derive(Debug, Deserialize)]
struct EvaluateInput {
    #[serde(default)]
    #[allow(dead_code)]
    intent: Option<String>,
    /// The content to judge: plain text, or a structured JSON object/array.
    state: Value,
    /// Map of question IDs to typed questions.
    questions: std::collections::HashMap<String, Question>,
    /// Model override. Defaults to `jev-latest` (or `~typesafe/jev-latest` on OpenRouter).
    #[serde(default)]
    model: Option<String>,
}

/// A single typed question for Jev.
#[derive(Debug, Serialize, Deserialize)]
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

/// Request body sent to the TypeSafe API.
#[derive(Debug, Serialize)]
struct TypeSafeRequest {
    state: Value,
    model: String,
    questions: std::collections::HashMap<String, Question>,
}

/// Response from the TypeSafe API.
#[derive(Debug, Serialize, Deserialize)]
struct TypeSafeResponse {
    answers: std::collections::HashMap<String, Value>,
    model: String,
    #[serde(default)]
    usage: Option<Value>,
}

fn api_config() -> Result<(&'static str, String, String)> {
    if let Ok(key) = std::env::var("TYPESAFE_API_KEY") {
        if key.trim().is_empty() {
            // fall through to OpenRouter
        } else {
            return Ok((TYPESAFE_API_URL, key, DEFAULT_MODEL.to_string()));
        }
    }
    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        if key.trim().is_empty() {
            anyhow::bail!(
                "No TypeSafe or OpenRouter API key configured. Set TYPESAFE_API_KEY or OPENROUTER_API_KEY."
            );
        }
        return Ok((OPENROUTER_API_URL, key, OPENROUTER_MODEL.to_string()));
    }
    anyhow::bail!(
        "No TypeSafe or OpenRouter API key configured. Set TYPESAFE_API_KEY or OPENROUTER_API_KEY."
    )
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
                    "description": "Model override. Defaults to jev-latest."
                }
            }
        })
    }

    async fn execute(&self, input: Value, _ctx: ToolContext) -> Result<ToolOutput> {
        let params: EvaluateInput = serde_json::from_value(input)?;

        // Validate locally: questions must not be empty.
        if params.questions.is_empty() {
            anyhow::bail!("evaluate requires at least one question");
        }

        let (api_url, api_key, default_model) = api_config()?;
        let model = params.model.unwrap_or(default_model);
        let model_for_cache = if model == OPENROUTER_MODEL {
            DEFAULT_MODEL.to_string()
        } else {
            model.clone()
        };
        let questions_json = serde_json::to_value(&params.questions)?;

        // Check cache.
        let fp = JevCache::fingerprint(&model_for_cache, &questions_json, &params.state, "");
        if let Some(cached) = self.cache.check(&fp) {
            return Ok(ToolOutput::new(format!(
                "{cached}"
            )));
        }

        self.cache.record_call();

        let body = TypeSafeRequest {
            state: params.state,
            model,
            questions: params.questions,
        };

        let response = send_with_retry(api_url, &api_key, &body, MAX_RETRIES).await?;

        // Validate: every requested question must have an answer.
        let answer_ids: Vec<&String> = body.questions.keys().collect();
        for id in &answer_ids {
            if !response.answers.contains_key(*id) {
                anyhow::bail!(
                    "evaluate: missing answer for question '{}' in TypeSafe response",
                    id
                );
            }
        }

        // Cache the answer.
        let answer_value = serde_json::to_value(&response)?;
        let answer_for_cache = serde_json::to_value(&response.answers)?;
        let _ = self
            .cache
            .store(&fp, &model_for_cache, &answer_for_cache);

        Ok(ToolOutput::new(format!("{answer_value}")))
    }
}

async fn send_with_retry(
    url: &str,
    api_key: &str,
    body: &TypeSafeRequest,
    max_retries: u32,
) -> Result<TypeSafeResponse> {
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 0..=max_retries {
        match send_once(url, api_key, body).await {
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

async fn send_once(url: &str, api_key: &str, body: &TypeSafeRequest) -> Result<TypeSafeResponse> {
    let response = HTTP_CLIENT
        .post(url)
        .header("Authorization", format!("Bearer {api_key}"))
        .json(body)
        .send()
        .await
        .context("evaluate: failed to reach TypeSafe/OpenRouter API")?;

    let status = response.status();
    if !status.is_success() {
        let status_text = response.text().await.unwrap_or_default();
        anyhow::bail!("evaluate: API returned HTTP {status}: {status_text}");
    }

    let parsed: TypeSafeResponse = response
        .json()
        .await
        .context("evaluate: failed to parse TypeSafe response")?;

    Ok(parsed)
}