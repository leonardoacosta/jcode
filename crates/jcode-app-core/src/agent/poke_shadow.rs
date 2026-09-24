//! Shadow assessment for poke — typed recommendations from the System One service.
//!
//! Observes turn-end snapshots and returns recommendations without changing
//! poke behavior. Enabled only via explicit `/poke shadow on` consent.

use anyhow::Result;
use jcode_system_one::SystemOneResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Recommendation from the shadow assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PokeRecommendation {
    Continue,
    Verify,
    Replan,
    WaitForUser,
    Unknown,
}

impl PokeRecommendation {
    pub const ALL: &[Self] = &[
        Self::Continue,
        Self::Verify,
        Self::Replan,
        Self::WaitForUser,
        Self::Unknown,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Verify => "verify",
            Self::Replan => "replan",
            Self::WaitForUser => "wait_for_user",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for PokeRecommendation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A parsed, validated recommendation from a System One choice response.
#[derive(Debug, Clone)]
pub struct PokeAssessment {
    pub recommendation: PokeRecommendation,
    pub probabilities: HashMap<String, f64>,
    /// Provider-reported confidence (concentration, not correctness).
    pub provider_confidence: Option<f64>,
    pub model: String,
}

/// Abstention reasons when no recommendation can be produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbstainReason {
    /// Provider returned an error or was unreachable.
    ProviderError,
    /// Required credentials are missing.
    MissingCredentials,
    /// The response had an invalid shape or unknown label.
    MalformedResponse,
    /// Top probability is below the display threshold.
    Uncertain,
    /// Top probability margin to the second is too narrow.
    InsufficientMargin,
    /// Request deadline expired.
    DeadlineExceeded,
    /// Session request budget exhausted.
    BudgetExhausted,
}

impl AbstainReason {
    pub fn as_reason(&self) -> &'static str {
        match self {
            Self::ProviderError => "provider_error",
            Self::MissingCredentials => "missing_credentials",
            Self::MalformedResponse => "malformed_response",
            Self::Uncertain => "uncertain",
            Self::InsufficientMargin => "insufficient_margin",
            Self::DeadlineExceeded => "deadline_exceeded",
            Self::BudgetExhausted => "budget_exhausted",
        }
    }
}

/// Display thresholds for a recommendation.
#[derive(Debug, Clone)]
pub struct DisplayThresholds {
    /// Minimum top probability to show a non-unknown recommendation.
    pub min_top_probability: f64,
    /// Minimum margin between top and second probability.
    pub min_probability_margin: f64,
}

impl Default for DisplayThresholds {
    fn default() -> Self {
        Self {
            min_top_probability: 0.80,
            min_probability_margin: 0.20,
        }
    }
}

/// Parse a System One choice response into a PokeAssessment.
///
/// Returns `None` with an `AbstainReason` when the response cannot produce
/// a valid recommendation or when the result fails display thresholds.
pub fn parse_poke_assessment(
    response: &SystemOneResponse,
    thresholds: &DisplayThresholds,
) -> Option<Result<PokeAssessment, AbstainReason>> {
    let answer = response.answers.get("poke_action")?;
    let answer_obj = answer.as_object()?;

    // Expect a choice response.
    if answer_obj.get("type").and_then(|v| v.as_str()) != Some("choice") {
        return Some(Err(AbstainReason::MalformedResponse));
    }

    let label = answer_obj.get("choice").and_then(|v| v.as_str()).unwrap_or("");

    let recommendation = match label {
        "continue" => PokeRecommendation::Continue,
        "verify" => PokeRecommendation::Verify,
        "replan" => PokeRecommendation::Replan,
        "wait_for_user" => PokeRecommendation::WaitForUser,
        "unknown" => PokeRecommendation::Unknown,
        _ => return Some(Err(AbstainReason::MalformedResponse)),
    };

    // Parse probabilities.
    let probs_map = answer_obj.get("probabilities").and_then(|v| v.as_object());
    let probabilities: HashMap<String, f64> = match probs_map {
        Some(map) => {
            let mut probs = HashMap::new();
            for (k, v) in map {
                let p = v.as_f64().unwrap_or(0.0);
                if p.is_finite() && p >= 0.0 && p <= 1.0 {
                    probs.insert(k.clone(), p);
                } else {
                    return Some(Err(AbstainReason::MalformedResponse));
                }
            }
            probs
        }
        None => return Some(Err(AbstainReason::MalformedResponse)),
    };

    // Validate the distribution sums to ~1.0.
    let total: f64 = probabilities.values().sum();
    if (total - 1.0).abs() > 0.01 {
        return Some(Err(AbstainReason::MalformedResponse));
    }

    // Find top two probabilities.
    let mut sorted: Vec<(&String, f64)> = probabilities.iter().map(|(k, v)| (k, *v)).collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let unknown_str = "unknown".to_string();
    let (top_label, top_prob) = sorted.first().copied().unwrap_or((&unknown_str, 0.0));
    let second_prob = sorted.get(1).map(|(_, p)| *p).unwrap_or(0.0);

    // Apply display thresholds.
    if *top_label != recommendation.as_str() {
        return Some(Err(AbstainReason::MalformedResponse));
    }
    if top_prob < thresholds.min_top_probability {
        return Some(Err(AbstainReason::Uncertain));
    }
    if top_prob - second_prob < thresholds.min_probability_margin {
        return Some(Err(AbstainReason::InsufficientMargin));
    }

    let provider_confidence = answer_obj.get("confidence").and_then(|v| v.as_f64());

    Some(Ok(PokeAssessment {
        recommendation,
        probabilities,
        provider_confidence,
        model: response.model.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_choice_answer(label: &str, probs: Vec<(&str, f64)>, confidence: Option<f64>) -> Value {
        let mut probabilities = serde_json::Map::new();
        for (k, v) in probs {
            probabilities.insert(k.to_string(), serde_json::json!(v));
        }
        let mut obj = serde_json::json!({
            "type": "choice",
            "choice": label,
            "probabilities": probabilities,
        });
        if let Some(c) = confidence {
            obj["confidence"] = serde_json::json!(c);
        }
        obj
    }

    fn make_response(answers: HashMap<String, Value>) -> SystemOneResponse {
        SystemOneResponse {
            answers,
            model: "jev-latest".to_string(),
            usage: None,
        }
    }

    #[test]
    fn test_valid_continue() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("continue", vec![("continue", 0.90), ("verify", 0.05), ("replan", 0.02), ("wait_for_user", 0.02), ("unknown", 0.01)], Some(0.85)),
        );
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap().unwrap();
        assert_eq!(result.recommendation, PokeRecommendation::Continue);
        assert_eq!(result.probabilities.len(), 5);
    }

    #[test]
    fn test_valid_verify() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("verify", vec![("verify", 0.92), ("continue", 0.03), ("replan", 0.02), ("wait_for_user", 0.02), ("unknown", 0.01)], None),
        );
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap().unwrap();
        assert_eq!(result.recommendation, PokeRecommendation::Verify);
    }

    #[test]
    fn test_valid_wait_for_user() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("wait_for_user", vec![("wait_for_user", 0.88), ("continue", 0.05), ("verify", 0.03), ("replan", 0.02), ("unknown", 0.02)], Some(0.82)),
        );
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap().unwrap();
        assert_eq!(result.recommendation, PokeRecommendation::WaitForUser);
    }

    #[test]
    fn test_abstain_uncertain_low_top_prob() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("continue", vec![("continue", 0.75), ("verify", 0.10), ("replan", 0.05), ("wait_for_user", 0.05), ("unknown", 0.05)], None),
        );
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap();
        assert_eq!(result.unwrap_err(), AbstainReason::Uncertain);
    }

    #[test]
    fn test_abstain_insufficient_margin() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("continue", vec![("continue", 0.55), ("verify", 0.40), ("replan", 0.02), ("wait_for_user", 0.02), ("unknown", 0.01)], None),
        );
        let response = make_response(answers);
        // Relax thresholds so top=0.55 passes min_top but margin=0.15 fails.
        let thresholds = DisplayThresholds {
            min_top_probability: 0.50,
            min_probability_margin: 0.20,
        };
        let result = parse_poke_assessment(&response, &thresholds).unwrap();
        assert_eq!(result.unwrap_err(), AbstainReason::InsufficientMargin);
    }

    #[test]
    fn test_abstain_unknown_label() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("invalid_label", vec![("invalid_label", 0.90), ("continue", 0.05), ("unknown", 0.05)], None),
        );
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap();
        assert_eq!(result.unwrap_err(), AbstainReason::MalformedResponse);
    }

    #[test]
    fn test_abstain_missing_answer() {
        let response = make_response(HashMap::new());
        assert!(parse_poke_assessment(&response, &DisplayThresholds::default()).is_none());
    }

    #[test]
    fn test_abstain_wrong_type() {
        let mut answers = HashMap::new();
        answers.insert("poke_action".to_string(), serde_json::json!({"type": "noul", "noul": 0.95}));
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap();
        assert_eq!(result.unwrap_err(), AbstainReason::MalformedResponse);
    }

    #[test]
    fn test_abstain_nonfinite_probability() {
        let mut answers = HashMap::new();
        let mut probs = serde_json::Map::new();
        probs.insert("continue".to_string(), serde_json::json!(f64::INFINITY));
        probs.insert("verify".to_string(), serde_json::json!(0.5));
        let obj = serde_json::json!({
            "type": "choice",
            "choice": "continue",
            "probabilities": probs,
        });
        answers.insert("poke_action".to_string(), obj);
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap();
        assert_eq!(result.unwrap_err(), AbstainReason::MalformedResponse);
    }

    #[test]
    fn test_abstain_bad_distribution_sum() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("continue", vec![("continue", 0.50), ("verify", 0.30), ("unknown", 0.10)], None),
        );
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap();
        assert_eq!(result.unwrap_err(), AbstainReason::MalformedResponse);
    }

    #[test]
    fn test_mismatched_top_label() {
        let mut answers = HashMap::new();
        answers.insert(
            "poke_action".to_string(),
            make_choice_answer("verify", vec![("continue", 0.90), ("verify", 0.05), ("unknown", 0.05)], None),
        );
        let response = make_response(answers);
        let result = parse_poke_assessment(&response, &DisplayThresholds::default()).unwrap();
        assert_eq!(result.unwrap_err(), AbstainReason::MalformedResponse);
    }
}