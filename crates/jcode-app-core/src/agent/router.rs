//! Jev-driven model tier selection for cost-optimized routing.
//!
//! On each fresh user turn, the prompt text is sent to Jev for complexity
//! classification. Jev returns a tier recommendation (Fast/Balanced/Strong/
//! Long), and the provider dispatcher routes to the configured model for
//! that tier. Simple queries route to cheap models; complex tasks use
//! stronger models.
//!
//! Adapted from `gargpratyush/jev-router` (MIT).

use anyhow::{Context, Result};
use serde_json::Value;

/// Tier taxonomy for model selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelTier {
    /// Cheapest models for trivial queries.
    Fast = 0,
    /// Default tier for normal tasks.
    Balanced = 1,
    /// Strong models for complex tasks.
    Strong = 2,
    /// Extended-context models for very large conversations.
    Long = 3,
}

impl ModelTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelTier::Fast => "fast",
            ModelTier::Balanced => "balanced",
            ModelTier::Strong => "strong",
            ModelTier::Long => "long",
        }
    }
}

/// Result of a Jev routing classification.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub tier: ModelTier,
    pub confidence: f64,
    /// Raw Jev complexity score (0 = trivial, 4 = expert).
    pub complexity_score: f64,
    pub reasoning_required: f64,
    pub tool_complexity: f64,
    pub context_size: f64,
}

/// Jev-driven model router.
pub struct JevRouter;

impl JevRouter {
    /// Classify a user prompt to determine the best model tier.
    ///
    /// Sends ONLY the prompt text to Jev (no source code, tool results,
    /// conversation history, or system prompts). Returns KeepCurrent on
    /// Jev failure (fail-open).
    pub async fn classify(prompt: &str) -> RoutingDecision {
        match Self::classify_inner(prompt).await {
            Ok(decision) => decision,
            Err(e) => {
                crate::logging::warn(&format!(
                    "JevRouter: classification failed ({e}), keeping current model"
                ));
                RoutingDecision {
                    tier: ModelTier::Balanced,
                    confidence: 0.0,
                    complexity_score: 2.0,
                    reasoning_required: 0.5,
                    tool_complexity: 0.5,
                    context_size: 0.5,
                }
            }
        }
    }

    async fn classify_inner(prompt: &str) -> Result<RoutingDecision> {
        let systemone =
            crate::systemone::resolve().context("JevRouter: resolve System One route")?;

        let request = serde_json::json!({
            "state": { "prompt": prompt },
            "model": systemone.default_model,
            "questions": {
                "complexity": {
                    "type": "score",
                    "instructions": "Classify the complexity of this coding task.",
                    "criteria": ["trivial", "simple", "moderate", "complex", "expert"]
                },
                "reasoning_required": {
                    "type": "noul",
                    "instructions": "Does this task require multi-step reasoning across multiple files or systems?"
                },
                "tool_complexity": {
                    "type": "noul",
                    "instructions": "Does this task need multiple coordinated tool calls (e.g., search, then read, then edit)?"
                },
                "context_size": {
                    "type": "noul",
                    "instructions": "Does this task require understanding a large amount of context (many files, long conversation)?"
                }
            }
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .http2_prior_knowledge()
            .build()
            .context("JevRouter: cannot create HTTP client")?;

        let resp = client
            .post(&systemone.endpoint_url)
            .header("Authorization", format!("Bearer {}", systemone.api_key))
            .json(&request)
            .send()
            .await
            .context("JevRouter: API unreachable")?;

        if !resp.status().is_success() {
            anyhow::bail!("JevRouter: HTTP {}", resp.status());
        }

        let body: Value = resp
            .json()
            .await
            .context("JevRouter: invalid JSON response")?;

        let answers = body["answers"]
            .as_object()
            .context("JevRouter: missing answers in response")?;

        // Extract complexity score (Score type returns {score: N, probabilities: {...}}).
        let complexity_score = answers
            .get("complexity")
            .and_then(|a| a.get("score"))
            .and_then(|s| s.as_f64())
            .unwrap_or(2.0);

        // Extract Noul probabilities.
        let reasoning_required = extract_noul(&answers, "reasoning_required");
        let tool_complexity = extract_noul(&answers, "tool_complexity");
        let context_size = extract_noul(&answers, "context_size");

        // Map complexity score (0-4) to tier.
        let tier = if complexity_score <= 0.5 {
            ModelTier::Fast
        } else if complexity_score <= 1.5 {
            ModelTier::Fast // borderline: score says simple
        } else if complexity_score <= 2.5 {
            ModelTier::Balanced
        } else if complexity_score <= 3.5 {
            ModelTier::Strong
        } else {
            ModelTier::Long
        };

        // Confidence: how much the other signals agree.
        let confidence =
            if tier == ModelTier::Fast && (reasoning_required > 0.3 || tool_complexity > 0.3) {
                0.4
            } else if tier == ModelTier::Balanced && reasoning_required > 0.6 {
                0.5
            } else {
                0.75
            };

        Ok(RoutingDecision {
            tier,
            confidence,
            complexity_score,
            reasoning_required,
            tool_complexity,
            context_size,
        })
    }
}

fn extract_noul(answers: &serde_json::Map<String, Value>, key: &str) -> f64 {
    answers
        .get(key)
        .and_then(|a| a.get("noul"))
        .and_then(|n| n.as_f64())
        .or_else(|| {
            answers
                .get(key)
                .and_then(|a| a.get("probability"))
                .and_then(|p| p.as_f64())
        })
        .unwrap_or(0.5)
        .clamp(0.0, 1.0)
}

/// Apply deterministic routing policy to a Jev routing decision.
///
/// Returns the recommended tier, or None to keep the current model.
pub fn apply_routing_policy(
    decision: &RoutingDecision,
    user_explicitly_selected_model: bool,
    token_usage_ratio: f64,
    current_tier: Option<ModelTier>,
) -> Option<ModelTier> {
    // 1. Explicit user choice always wins.
    if user_explicitly_selected_model {
        return None;
    }

    // 2. Jev failure (confidence 0) -> keep current.
    if decision.confidence == 0.0 {
        return None;
    }

    // 3. Low confidence: never downgrade, cap upgrade at Balanced.
    if decision.confidence < 0.6 {
        if let Some(cur) = current_tier {
            if decision.tier < cur {
                return None; // no downgrade
            }
        }
        if decision.tier > ModelTier::Balanced {
            return Some(ModelTier::Balanced);
        }
    }

    // 4. Large conversation: refuse downgrades.
    if token_usage_ratio > 0.5 {
        if let Some(cur) = current_tier {
            if decision.tier < cur {
                return None;
            }
        }
    }

    // 5. Normal routing.
    Some(decision.tier)
}
