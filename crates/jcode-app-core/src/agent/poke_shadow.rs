//! Shadow assessment for poke — typed recommendations from the System One service.
//!
//! Observes turn-end snapshots and returns recommendations without changing
//! poke behavior. Enabled only via explicit `/poke shadow on` consent.

use anyhow::Result;
use jcode_system_one::SystemOneResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

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

/// Resolve the provider name for display in the TUI.
/// Returns the provider string (e.g. "openrouter" or "typesafe") or
/// a message indicating no credentials are configured.
pub fn shadow_provider_name() -> String {
    jcode_system_one::resolve_service(None)
        .ok()
        .map(|cfg| cfg.provider.to_string())
        .unwrap_or_else(|| "no credentials configured".to_string())
}

// ---------------------------------------------------------------------------
// Evidence builder
// ---------------------------------------------------------------------------

/// Maximum UTF-8 byte length for a shadow assessment evidence payload.
pub const MAX_EVIDENCE_BYTES: usize = 8192;

/// Bounded evidence snapshot sent to the System One service.
#[derive(Debug, Clone, Serialize)]
pub struct EvidenceSnapshot {
    /// Shortened user request (first 200 chars).
    pub user_request: String,
    /// Relevant todos: id, content (first 120 chars), status.
    pub todos: Vec<TodoEvidence>,
    /// Names of tools called in the last turn (max 8).
    pub recent_tools: Vec<String>,
    /// Whether a verification pass has run recently.
    pub verification_fresh: bool,
    /// Known background work (max 3 items).
    pub background_work: Vec<String>,
    /// Explicit user-input wait detected.
    pub awaiting_user: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TodoEvidence {
    pub id: String,
    pub content: String,
    pub status: String,
}

/// Known credential sentinels — substrings that indicate a secret may be
/// present in text. Text containing any of these is refused by the builder.
const CREDENTIAL_SENTINELS: &[&str] = &[
    "TYPESAFE_API_KEY",
    "OPENROUTER_API_KEY",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "api_key",
    "Bearer ",
    "Authorization:",
    "sk-",
    "sk-ant-",
    "AIza",
];

/// Build a sanitized evidence snapshot from turn-end state.
///
/// Returns `None` if the evidence cannot be safely captured (e.g. credential
/// sentinels detected in the request text, or the payload exceeds the byte cap
/// after minimization).
pub fn build_evidence_snapshot(
    user_request: &str,
    todos: &[crate::todo::TodoItem],
    recent_tools: &[String],
    verification_fresh: bool,
    background_work: &[String],
    awaiting_user: bool,
) -> Option<EvidenceSnapshot> {
    // Redact user request: check for credential sentinels first.
    let lower_req = user_request.to_lowercase();
    for sentinel in CREDENTIAL_SENTINELS {
        if lower_req.contains(&sentinel.to_lowercase()) {
            crate::logging::debug(&format!(
                "poke_shadow: evidence rejected — credential sentinel '{sentinel}' in request"
            ));
            return None;
        }
    }

    // Truncate user request to first 200 chars.
    let request_text: String = user_request.chars().take(200).collect();

    // Collect todo evidence, max 12 items, each content capped at 120 chars.
    let todo_evidence: Vec<TodoEvidence> = todos
        .iter()
        .take(12)
        .map(|t| TodoEvidence {
            id: t.id.clone(),
            content: t.content.chars().take(120).collect(),
            status: t.status.clone(),
        })
        .collect();

    // Cap tool names.
    let tools: Vec<String> = recent_tools.iter().take(8).cloned().collect();

    // Cap background work.
    let bg: Vec<String> = background_work.iter().take(3).cloned().collect();

    let snapshot = EvidenceSnapshot {
        user_request: request_text,
        todos: todo_evidence,
        recent_tools: tools,
        verification_fresh,
        background_work: bg,
        awaiting_user,
    };

    // Serialize and check the byte cap.
    let serialized = match serde_json::to_vec(&snapshot) {
        Ok(v) => v,
        Err(_) => return None,
    };

    if serialized.len() > MAX_EVIDENCE_BYTES {
        crate::logging::debug(&format!(
            "poke_shadow: evidence rejected — {} bytes exceeds {MAX_EVIDENCE_BYTES} cap",
            serialized.len()
        ));
        return None;
    }

    Some(snapshot)
}

// ---------------------------------------------------------------------------
// Assessment service lifecycle
// ---------------------------------------------------------------------------

/// Maximum number of launched requests per consent session.
pub const MAX_REQUESTS_PER_SESSION: u32 = 20;

/// Overall deadline for an assessment request (queuing + provider time).
pub const ASSESSMENT_DEADLINE_SECS: u64 = 2;

/// The result of an assessment attempt.
#[derive(Debug, Clone)]
pub enum AssessmentOutcome {
    /// A valid recommendation was produced.
    Recommendation(PokeAssessment),
    /// The assessment could not produce a recommendation.
    Abstained(AbstainReason),
}

/// An in-flight or completed assessment request, keyed by evidence revision.
#[derive(Debug, Clone)]
pub struct AssessmentState {
    /// Current shadow consent generation. Incremented on cancel/disable.
    pub generation: u64,
    /// Whether shadow mode is currently enabled.
    pub enabled: bool,
    /// Number of launched requests in this consent session.
    pub launched: u32,
    /// Last evidence revision that was (or is being) assessed.
    pub last_revision: Option<String>,
    /// The most recent outcome, if any.
    pub last_outcome: Option<AssessmentOutcome>,
    /// When the current in-flight request started, for deadline tracking.
    pub in_flight_since: Option<Instant>,
}

impl Default for AssessmentState {
    fn default() -> Self {
        Self {
            generation: 1,
            enabled: false,
            launched: 0,
            last_revision: None,
            last_outcome: None,
            in_flight_since: None,
        }
    }
}

impl AssessmentState {
    /// Enable shadow mode, returning the disclosure message.
    pub fn enable(&mut self, provider: &str) -> String {
        if self.enabled {
            return format!("Shadow assessment is already enabled. Provider: {provider}. Launched: {}/{}.",
                self.launched, MAX_REQUESTS_PER_SESSION);
        }
        self.enabled = true;
        // Reset budget on fresh enable. Re-enabling within the same session
        // does not reset the budget per the design.
        format!(
            "Shadow assessment enabled.\n\nProvider: {provider}\n\nData sent: bounded task text, \
             todo summaries, recent tool names, sanitized outcome summaries. No raw command \
             arguments, output, environment values, file bodies, or credentials.\n\nRequests \
             may incur provider charges. Max {MAX_REQUESTS_PER_SESSION} requests per session, \
             {ASSESSMENT_DEADLINE_SECS}s deadline per request.\n\nUse /poke shadow off to disable."
        )
    }

    /// Disable shadow mode.
    pub fn disable(&mut self) -> &'static str {
        if !self.enabled {
            return "Shadow assessment is already disabled.";
        }
        self.enabled = false;
        self.generation = self.generation.wrapping_add(1);
        self.in_flight_since = None;
        "Shadow assessment disabled."
    }

    /// Check whether a new assessment can be launched.
    pub fn can_launch(&self) -> Option<AbstainReason> {
        if !self.enabled {
            return Some(AbstainReason::MissingCredentials); // reuse as "not enabled"
        }
        if self.in_flight_since.is_some() {
            // One in-flight at a time.
            return None; // None means "skip, already in flight" — not an error
        }
        if self.launched >= MAX_REQUESTS_PER_SESSION {
            return Some(AbstainReason::BudgetExhausted);
        }
        None // can launch
    }

    /// Record that a request has been launched for the given revision.
    pub fn record_launch(&mut self, revision: &str) {
        self.launched = self.launched.saturating_add(1);
        self.last_revision = Some(revision.to_string());
        self.in_flight_since = Some(Instant::now());
    }

    /// Record that the in-flight request completed (or timed out / was cancelled).
    pub fn record_complete(
        &mut self,
        outcome: AssessmentOutcome,
        current_generation: u64,
    ) {
        if self.generation != current_generation {
            // Stale: a new generation started while this request was in flight.
            return;
        }
        self.in_flight_since = None;
        self.last_outcome = Some(outcome);
    }

    /// Check whether the current in-flight request has exceeded the deadline.
    pub fn deadline_exceeded(&self) -> bool {
        match self.in_flight_since {
            Some(start) => start.elapsed().as_secs() >= ASSESSMENT_DEADLINE_SECS,
            None => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Live evaluation
// ---------------------------------------------------------------------------

/// Run a shadow assessment against the live System One service.
///
/// Builds the evidence into a choice question, calls Jev through the injected
/// service, parses the response, and returns an `AssessmentOutcome`.
/// This is the only function that makes a hosted call.
pub async fn evaluate_shadow(
    service: &dyn jcode_system_one::SystemOneService,
    evidence: &EvidenceSnapshot,
    thresholds: &DisplayThresholds,
) -> AssessmentOutcome {
    let state = serde_json::to_value(evidence).unwrap_or(serde_json::Value::Null);

    let mut questions = std::collections::HashMap::new();
    questions.insert(
        "poke_action".to_string(),
        serde_json::json!({
            "type": "choice",
            "instructions": "Given this turn-end snapshot of a coding agent, what should happen next?",
            "criteria": {
                "continue": "The agent should continue working — there is a clear, actionable next step.",
                "verify": "The agent claims completion but verification evidence is thin or missing. It should run tests or checks.",
                "replan": "The agent appears stuck or repeating failed approaches. It should step back and replan.",
                "wait_for_user": "The agent needs human input, permission, credentials, or a decision before proceeding.",
                "unknown": "The situation is ambiguous or none of the above clearly applies."
            }
        }),
    );

    let model = Some("jev-latest");
    let response = match tokio::time::timeout(
        std::time::Duration::from_secs(ASSESSMENT_DEADLINE_SECS),
        service.evaluate(state, questions, model),
    )
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            crate::logging::debug(&format!("poke_shadow: service error: {e}"));
            return AssessmentOutcome::Abstained(AbstainReason::ProviderError);
        }
        Err(_elapsed) => {
            crate::logging::debug("poke_shadow: deadline exceeded");
            return AssessmentOutcome::Abstained(AbstainReason::DeadlineExceeded);
        }
    };

    match parse_poke_assessment(&response, thresholds) {
        Some(Ok(assessment)) => AssessmentOutcome::Recommendation(assessment),
        Some(Err(reason)) => {
            crate::logging::debug(&format!("poke_shadow: abstained: {:?}", reason));
            AssessmentOutcome::Abstained(reason)
        }
        None => {
            crate::logging::debug("poke_shadow: no valid answer in response");
            AssessmentOutcome::Abstained(AbstainReason::MalformedResponse)
        }
    }
}

/// Convenience entry point for the TUI: build evidence, resolve the service,
/// evaluate, and return a display string for the status line.
///
/// Returns `None` if the evidence cannot be built safely (credential sentinel
/// or oversized payload) or if no System One credential is configured.
pub async fn run_shadow_assessment(
    user_request: &str,
    todos: &[crate::todo::TodoItem],
    recent_tools: &[String],
    verification_fresh: bool,
    background_work: &[String],
    awaiting_user: bool,
) -> Option<String> {
    let evidence = build_evidence_snapshot(
        user_request, todos, recent_tools,
        verification_fresh, background_work, awaiting_user,
    )?;

    let config = jcode_system_one::resolve_service(None).ok()?;
    let service = jcode_system_one::LiveSystemOneService::with_timeout(
        config,
        std::time::Duration::from_secs(ASSESSMENT_DEADLINE_SECS),
    );
    let thresholds = DisplayThresholds::default();

    let outcome = evaluate_shadow(&service, &evidence, &thresholds).await;

    Some(match outcome {
        AssessmentOutcome::Recommendation(a) => {
            format!("Shadow: → {} (conf: {:.0}%)",
                a.recommendation,
                a.provider_confidence.unwrap_or(0.0) * 100.0)
        }
        AssessmentOutcome::Abstained(r) => {
            format!("Shadow: — ({})", r.as_reason())
        }
    })
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

    // --- evidence builder tests ---

    fn make_test_todo(id: &str, content: &str, status: &str) -> crate::todo::TodoItem {
        crate::todo::TodoItem {
            id: id.to_string(),
            content: content.to_string(),
            status: status.to_string(),
            priority: "medium".to_string(),
            group: None,
            blocked_by: vec![],
            assigned_to: None,
            confidence: None,
            completion_confidence: None,
            confidence_history: vec![],
        }
    }

    #[test]
    fn test_evidence_builder_normal() {
        let todos = vec![
            make_test_todo("1", "Implement feature X", "in_progress"),
            make_test_todo("2", "Fix bug in parser", "pending"),
        ];
        let tools = vec!["bash".into(), "agentgrep".into()];
        let snapshot = build_evidence_snapshot(
            "Please implement the new feature",
            &todos,
            &tools,
            true,
            &[],
            false,
        )
        .unwrap();
        assert_eq!(snapshot.todos.len(), 2);
        assert_eq!(snapshot.todos[0].id, "1");
        assert_eq!(snapshot.recent_tools.len(), 2);
        assert!(snapshot.verification_fresh);
    }

    #[test]
    fn test_evidence_builder_credential_sentinel_rejected() {
        let snapshot = build_evidence_snapshot(
            "Use OPENAI_API_KEY=sk-abc123 for the request",
            &[],
            &[],
            false,
            &[],
            false,
        );
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_evidence_builder_bearer_token_rejected() {
        let snapshot = build_evidence_snapshot(
            "curl -H 'Authorization: Bearer xyz' https://api.example.com",
            &[],
            &[],
            false,
            &[],
            false,
        );
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_evidence_builder_truncates_long_request() {
        let long_request = "A".repeat(500);
        let snapshot = build_evidence_snapshot(&long_request, &[], &[], false, &[], false).unwrap();
        assert!(snapshot.user_request.len() <= 200);
        assert!(snapshot.user_request.chars().all(|c| c == 'A'));
    }

    #[test]
    fn test_evidence_builder_caps_todos_and_tools() {
        let todos: Vec<_> = (0..20)
            .map(|i| make_test_todo(&format!("{i}"), &format!("task {i}"), "pending"))
            .collect();
        let tools: Vec<_> = (0..15).map(|i| format!("tool{i}")).collect();
        let snapshot = build_evidence_snapshot("test", &todos, &tools, false, &[], false).unwrap();
        assert!(snapshot.todos.len() <= 12);
        assert!(snapshot.recent_tools.len() <= 8);
    }

    // --- assessment lifecycle tests ---

    #[test]
    fn test_assessment_state_default_disabled() {
        let state = AssessmentState::default();
        assert!(!state.enabled);
        assert_eq!(state.launched, 0);
        assert!(state.last_revision.is_none());
    }

    #[test]
    fn test_assessment_state_enable_disable() {
        let mut state = AssessmentState::default();
        let msg = state.enable("test-provider");
        assert!(msg.contains("Shadow assessment enabled"));
        assert!(msg.contains("test-provider"));
        assert!(state.enabled);

        let msg = state.disable();
        assert!(msg.contains("disabled"));
        assert!(!state.enabled);
    }

    #[test]
    fn test_assessment_state_budget() {
        let mut state = AssessmentState::default();
        state.enable("test");
        // Launch MAX_REQUESTS_PER_SESSION requests.
        for i in 0..MAX_REQUESTS_PER_SESSION {
            assert!(state.can_launch().is_none(), "should allow launch {i}");
            state.record_launch(&format!("rev{i}"));
            state.record_complete(
                AssessmentOutcome::Abstained(AbstainReason::Uncertain),
                state.generation,
            );
        }
        // Next launch should be blocked.
        assert_eq!(
            state.can_launch(),
            Some(AbstainReason::BudgetExhausted)
        );
    }

    #[test]
    fn test_assessment_state_generation_rejects_stale() {
        let mut state = AssessmentState::default();
        state.enable("test");
        let gen_val = state.generation;
        state.record_launch("rev1");
        state.disable(); // increments generation
        // Record with old generation — should be silently ignored.
        state.record_complete(
            AssessmentOutcome::Recommendation(PokeAssessment {
                recommendation: PokeRecommendation::Continue,
                probabilities: HashMap::new(),
                provider_confidence: Some(0.9),
                model: "test".into(),
            }),
            gen_val,
        );
        assert!(state.last_outcome.is_none());
    }

    #[test]
    fn test_assessment_state_deadline() {
        let mut state = AssessmentState::default();
        state.enable("test");
        state.record_launch("rev1");
        // Immediately after launch, deadline should not be exceeded.
        assert!(!state.deadline_exceeded());
    }

    #[test]
    fn test_assessment_state_in_flight_blocks_new_launch() {
        let mut state = AssessmentState::default();
        state.enable("test");
        state.record_launch("rev1");
        // can_launch returns None when already in flight (not an error, just skip).
        assert!(state.can_launch().is_none());
    }
}