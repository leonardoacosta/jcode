//! Jev-driven background observer for swarm worker supervision.
//!
//! Periodically assesses active swarm workers with 10 Noul questions per
//! observation cycle. Phase 1 is passive only — assessments are logged,
//! never acted upon. Phase 2 (intervention) is deferred pending validation
//! of assessment quality.
//!
//! Adapted from `thruwire/foreman` (MIT).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configurable thresholds for foreman assessment.
#[derive(Debug, Clone)]
pub struct ForemanConfig {
    /// Whether the observer is enabled.
    pub enabled: bool,
    /// Interval between observation cycles in seconds.
    pub interval_seconds: u64,
    /// Maximum observation state characters before truncation.
    pub max_state_chars: usize,
}

impl Default for ForemanConfig {
    fn default() -> Self {
        Self {
            enabled: false, // opt-in for Phase 1
            interval_seconds: 30,
            max_state_chars: 8000,
        }
    }
}

/// A single worker observation snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationState {
    pub worker_id: String,
    pub job_description: String,
    pub worker_status: String,
    pub recent_history: Vec<String>,
    pub output_tail: String,
    pub elapsed_seconds: u64,
    pub attempt_count: u32,
    pub failure_count: u32,
}

/// Result of a 10-dimension Jev assessment.
#[derive(Debug, Clone, Default)]
pub struct AssessmentResult {
    pub implementation_complete: f64,
    pub tests_sufficient: f64,
    pub requirements_satisfied: f64,
    pub needs_verification: f64,
    pub ready_to_finish: f64,
    pub meaningful_progress: f64,
    pub worker_stuck: f64,
    pub work_off_track: f64,
    pub agents_md_drift: f64,
    pub needs_human: f64,
}

/// Background observer that periodically assesses swarm workers.
pub struct ForemanObserver {
    config: ForemanConfig,
    /// Map of worker_id → last assessment. Shared across assessment tasks.
    last_assessments: Arc<RwLock<std::collections::HashMap<String, Vec<AssessmentResult>>>>,
}

impl ForemanObserver {
    pub fn new(config: ForemanConfig) -> Self {
        Self {
            config,
            last_assessments: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Start the observation loop. Runs until the cancellation token fires.
    /// Phase 1: assessments are logged, never acted upon.
    pub async fn run(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
        get_active_workers: impl Fn() -> Vec<ObservationState> + Send + Sync + 'static,
    ) {
        if !self.config.enabled {
            crate::logging::debug("Foreman observer: disabled, not starting");
            return;
        }

        crate::logging::info(&format!(
            "Foreman observer: starting with {}s interval",
            self.config.interval_seconds
        ));

        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(self.config.interval_seconds));

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    crate::logging::info("Foreman observer: shutdown signal received");
                    break;
                }
                _ = interval.tick() => {
                    let workers = get_active_workers();
                    if workers.is_empty() {
                        continue;
                    }

                    let assessments = self.last_assessments.clone();
                    let max_chars = self.config.max_state_chars;

                    for worker in workers {
                        let worker_id = worker.worker_id.clone();
                        let result = Self::assess_worker(&worker, max_chars).await;
                        match result {
                            Ok(assessment) => {
                                crate::logging::info(&format!(
                                    "Foreman: {} stuck={:.2} off_track={:.2} progress={:.2} needs_human={:.2}",
                                    worker_id,
                                    assessment.worker_stuck,
                                    assessment.work_off_track,
                                    assessment.meaningful_progress,
                                    assessment.needs_human,
                                ));

                                // Store for trend analysis.
                                let mut map = assessments.write().await;
                                map.entry(worker_id.clone())
                                    .or_default()
                                    .push(assessment);
                                // Keep only last 5 cycles.
                                if let Some(history) = map.get_mut(&worker_id) {
                                    if history.len() > 5 {
                                        history.remove(0);
                                    }
                                }
                            }
                            Err(e) => {
                                crate::logging::warn(&format!(
                                    "Foreman: assessment failed for {}: {e}",
                                    worker_id
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Run a 10-dimension assessment on a single worker via Jev.
    async fn assess_worker(
        worker: &ObservationState,
        max_chars: usize,
    ) -> Result<AssessmentResult> {
        let systemone = crate::systemone::resolve().context("Foreman: resolve System One route")?;

        // Build observation state, truncated to max_chars.
        let state_json = serde_json::to_string(worker)?;
        let state_text = if state_json.len() > max_chars {
            format!("{}...", &state_json[..max_chars])
        } else {
            state_json
        };

        let request = serde_json::json!({
            "state": { "worker": state_text },
            "model": systemone.default_model,
            "questions": {
                "implementation_complete": {
                    "type": "noul",
                    "instructions": "Is the required implementation work for this coding task complete?"
                },
                "tests_sufficient": {
                    "type": "noul",
                    "instructions": "Are tests adequate for this change?"
                },
                "requirements_satisfied": {
                    "type": "noul",
                    "instructions": "Does the work done satisfy the original requirements?"
                },
                "needs_verification": {
                    "type": "noul",
                    "instructions": "Should an independent verification pass be run?"
                },
                "ready_to_finish": {
                    "type": "noul",
                    "instructions": "Is the coding task ready to be considered complete?"
                },
                "meaningful_progress": {
                    "type": "noul",
                    "instructions": "Is the current worker making meaningful progress on the task?"
                },
                "worker_stuck": {
                    "type": "noul",
                    "instructions": "Is the worker looping, failing repeatedly, or unable to advance?"
                },
                "work_off_track": {
                    "type": "noul",
                    "instructions": "Is the work drifting away from the original job/task?"
                },
                "agents_md_drift": {
                    "type": "noul",
                    "instructions": "Is worker behavior inconsistent with documented instructions (AGENTS.md)?"
                },
                "needs_human": {
                    "type": "noul",
                    "instructions": "Does this situation need human judgment, credentials, or clarification?"
                }
            }
        });

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .http2_prior_knowledge()
            .build()
            .context("Foreman: cannot create HTTP client")?;

        let resp = client
            .post(&systemone.endpoint_url)
            .header("Authorization", format!("Bearer {}", systemone.api_key))
            .json(&request)
            .send()
            .await
            .context("Foreman: API unreachable")?;

        if !resp.status().is_success() {
            anyhow::bail!("Foreman: HTTP {}", resp.status());
        }

        let body: Value = resp
            .json()
            .await
            .context("Foreman: invalid JSON response")?;

        let answers = body["answers"]
            .as_object()
            .context("Foreman: missing answers")?;

        Ok(AssessmentResult {
            implementation_complete: extract_noul(answers, "implementation_complete"),
            tests_sufficient: extract_noul(answers, "tests_sufficient"),
            requirements_satisfied: extract_noul(answers, "requirements_satisfied"),
            needs_verification: extract_noul(answers, "needs_verification"),
            ready_to_finish: extract_noul(answers, "ready_to_finish"),
            meaningful_progress: extract_noul(answers, "meaningful_progress"),
            worker_stuck: extract_noul(answers, "worker_stuck"),
            work_off_track: extract_noul(answers, "work_off_track"),
            agents_md_drift: extract_noul(answers, "agents_md_drift"),
            needs_human: extract_noul(answers, "needs_human"),
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
