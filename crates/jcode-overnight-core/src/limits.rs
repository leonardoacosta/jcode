//! Enforced overnight run limits: pure evaluation, no IO.
//!
//! Token totals come from the deduplicated usage ledger, which is an
//! observation made after each provider response. A limit check therefore
//! never authorizes dispatch and never replaces a pre-send monetary
//! reservation; it can only stop a run that is already over its bound.

use super::{UsageObservation, UsageSource, dedup_observations};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result of stopping a run's owned work through durable session control.
/// `Settled` is recorded only when every owned session reached `stopped`; a
/// drain deadline or an unreadable record is `Unresolved`, never success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum StopSettlement {
    Settled {
        at: DateTime<Utc>,
    },
    Unresolved {
        at: DateTime<Utc>,
        /// `session id: control state` for each session not confirmed stopped.
        sessions: Vec<String>,
        reason: String,
    },
}

impl StopSettlement {
    pub fn is_settled(&self) -> bool {
        matches!(self, Self::Settled { .. })
    }
}

/// Live helper workers a new run may own unless the user says otherwise,
/// matching the "one coordinator plus at most one helper" run contract.
pub const DEFAULT_MAX_AGENTS: u32 = 1;

/// Per-run bounds. `None` means unbounded. Manifests written before limits
/// existed deserialize to all-`None`, so old runs are never newly restricted.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OvernightLimits {
    /// Most live workers the coordinator may have spawned at once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_agents: Option<u32>,
    /// Longest a single coordinator turn may run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turn_minutes: Option<u32>,
    /// Most tokens (input + output + cache creation) the run and its owned
    /// workers may consume, as measured from the usage ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_run_tokens: Option<u64>,
}

impl OvernightLimits {
    pub fn is_unset(&self) -> bool {
        *self == Self::default()
    }

    /// Limits applied to a newly started run when none are given.
    pub fn standard() -> Self {
        Self {
            max_agents: Some(DEFAULT_MAX_AGENTS),
            ..Self::default()
        }
    }

    pub fn summary(&self) -> String {
        let part = |label: &str, value: Option<u64>| {
            value.map_or_else(|| format!("{label} unlimited"), |v| format!("{label} {v}"))
        };
        format!(
            "{} · {} · {}",
            part("agents", self.max_agents.map(u64::from)),
            part("turn minutes", self.max_turn_minutes.map(u64::from)),
            part("tokens", self.max_run_tokens)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LimitKind {
    Agents,
    TurnTime,
    RunTokens,
}

impl LimitKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Agents => "agent",
            Self::TurnTime => "turn time",
            Self::RunTokens => "token",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitBreach {
    pub kind: LimitKind,
    pub limit: u64,
    pub observed: u64,
    /// How `observed` was measured. `Estimated` is a lower bound.
    pub source: UsageSource,
    pub message: String,
}

/// Run-wide token total with an explicit measurement basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTokenUsage {
    pub tokens: u64,
    pub requests: u64,
    /// Responses that omitted input or output counters; the total is short by
    /// whatever they consumed.
    pub incomplete_requests: u64,
    pub unreadable_lines: usize,
    /// `Reported`: every counted response carried full counters.
    /// `Estimated`: a lower bound. `Unavailable`: nothing was reported.
    pub source: UsageSource,
}

/// Sum the run's deduplicated ledger. Replays and mirrors of one response
/// (same `event_id`) count once, whichever session or generation wrote them.
pub fn aggregate_run_tokens(
    run_id: &str,
    observations: &[UsageObservation],
    unreadable_lines: usize,
) -> RunTokenUsage {
    let unique = dedup_observations(run_id, observations);
    let mut tokens = 0u64;
    let mut incomplete = 0u64;
    for o in &unique {
        tokens = tokens
            .saturating_add(o.input_tokens.unwrap_or(0))
            .saturating_add(o.output_tokens.unwrap_or(0))
            .saturating_add(o.cache_creation_input_tokens.unwrap_or(0));
        if o.input_tokens.is_none() || o.output_tokens.is_none() {
            incomplete += 1;
        }
    }
    let source = if unique.is_empty() {
        UsageSource::Unavailable
    } else if incomplete > 0 || unreadable_lines > 0 {
        UsageSource::Estimated
    } else {
        UsageSource::Reported
    };
    RunTokenUsage {
        tokens,
        requests: unique.len() as u64,
        incomplete_requests: incomplete,
        unreadable_lines,
        source,
    }
}

/// A token breach. An `Estimated` total is a lower bound, so exceeding the cap
/// is still certain; `Unavailable` proves nothing and never breaches.
pub fn check_run_tokens(limits: &OvernightLimits, usage: &RunTokenUsage) -> Option<LimitBreach> {
    let limit = limits.max_run_tokens?;
    if usage.source == UsageSource::Unavailable || usage.tokens < limit {
        return None;
    }
    let message = match usage.source {
        UsageSource::Reported => format!(
            "Token limit {limit} reached: {} tokens reported across {} response(s)",
            usage.tokens, usage.requests
        ),
        _ => format!(
            "Token limit {limit} reached: at least {} tokens (estimated lower bound; {} response(s) lacked counters, {} ledger line(s) unreadable)",
            usage.tokens, usage.incomplete_requests, usage.unreadable_lines
        ),
    };
    Some(LimitBreach {
        kind: LimitKind::RunTokens,
        limit,
        observed: usage.tokens,
        source: usage.source,
        message,
    })
}

pub fn check_turn_time(limits: &OvernightLimits, elapsed_secs: u64) -> Option<LimitBreach> {
    let minutes = u64::from(limits.max_turn_minutes?);
    (elapsed_secs >= minutes.saturating_mul(60)).then(|| LimitBreach {
        kind: LimitKind::TurnTime,
        limit: minutes,
        observed: elapsed_secs / 60,
        source: UsageSource::Reported,
        message: format!(
            "Turn time limit {minutes}m reached after {}m; stopping the run",
            elapsed_secs / 60
        ),
    })
}

/// A spawn is refused once `live_workers` already fills the cap.
pub fn check_agent_spawn(limits: &OvernightLimits, live_workers: usize) -> Option<LimitBreach> {
    let limit = u64::from(limits.max_agents?);
    (live_workers as u64 >= limit).then(|| LimitBreach {
        kind: LimitKind::Agents,
        limit,
        observed: live_workers as u64,
        source: UsageSource::Reported,
        message: format!(
            "Overnight agent limit reached (max {limit}); {live_workers} worker(s) are already live. Let one finish or stop it before spawning another."
        ),
    })
}

const FLAG_USAGE: &str = "Limit flags go between the duration and the mission: `--max-agents <n|none>`, `--max-turn-minutes <n|none>`, `--max-tokens <n|none>`.";

/// Split leading `--max-*` flags off the text after the duration. Returns the
/// limits and the remaining mission text.
pub fn parse_limit_flags(rest: &str) -> Result<(OvernightLimits, Option<String>), String> {
    let mut limits = OvernightLimits::standard();
    let mut remaining = rest.trim_start();
    while remaining.starts_with("--max-") {
        let (token, after) = remaining
            .split_once(char::is_whitespace)
            .unwrap_or((remaining, ""));
        let (flag, inline) = match token.split_once('=') {
            Some((flag, value)) => (flag, Some(value)),
            None => (token, None),
        };
        let (value, after) = match inline {
            Some(value) => (value, after),
            None => {
                let after = after.trim_start();
                after.split_once(char::is_whitespace).unwrap_or((after, ""))
            }
        };
        let unlimited = matches!(value, "none" | "unlimited");
        let number = |name: &str| -> Result<Option<u64>, String> {
            if unlimited {
                return Ok(None);
            }
            value
                .parse::<u64>()
                .map(Some)
                .map_err(|_| format!("Invalid value `{value}` for {name}. {FLAG_USAGE}"))
        };
        let positive = |name: &str| -> Result<Option<u64>, String> {
            match number(name)? {
                Some(0) => Err(format!("{name} must be greater than zero. {FLAG_USAGE}")),
                other => Ok(other),
            }
        };
        let narrow = |name: &str, value: Option<u64>| -> Result<Option<u32>, String> {
            value
                .map(|v| u32::try_from(v).map_err(|_| format!("{name} is too large. {FLAG_USAGE}")))
                .transpose()
        };
        match flag {
            "--max-agents" => limits.max_agents = narrow(flag, number(flag)?)?,
            "--max-turn-minutes" => limits.max_turn_minutes = narrow(flag, positive(flag)?)?,
            "--max-tokens" => limits.max_run_tokens = positive(flag)?,
            other => return Err(format!("Unknown overnight flag `{other}`. {FLAG_USAGE}")),
        }
        remaining = after.trim_start();
    }
    let mission = Some(remaining.trim().to_string()).filter(|value| !value.is_empty());
    Ok((limits, mission))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn obs(
        event: &str,
        run: &str,
        input: Option<u64>,
        output: Option<u64>,
        creation: Option<u64>,
    ) -> UsageObservation {
        UsageObservation {
            event_id: event.into(),
            run_id: run.into(),
            session_id: "s".into(),
            generation: None,
            provider: "p".into(),
            account: None,
            model: "m".into(),
            input_tokens: input,
            output_tokens: output,
            cache_read_input_tokens: Some(1_000_000),
            cache_creation_input_tokens: creation,
            observed_at: Utc::now(),
        }
    }

    #[test]
    fn aggregate_dedupes_replays_and_other_runs_and_excludes_cache_reads() {
        let rows = vec![
            obs("a", "r", Some(10), Some(5), Some(1)),
            obs("a", "r", Some(10), Some(5), Some(1)),
            obs("b", "r", Some(100), Some(50), None),
            obs("c", "other", Some(9_999), Some(9_999), None),
        ];
        let usage = aggregate_run_tokens("r", &rows, 0);
        assert_eq!(usage.tokens, 166);
        assert_eq!(usage.requests, 2);
        assert_eq!(usage.source, UsageSource::Reported);
    }

    #[test]
    fn partial_counters_or_unreadable_lines_make_a_lower_bound() {
        let partial = aggregate_run_tokens("r", &[obs("a", "r", Some(10), None, None)], 0);
        assert_eq!(partial.source, UsageSource::Estimated);
        assert_eq!(partial.incomplete_requests, 1);
        let unreadable = aggregate_run_tokens("r", &[obs("a", "r", Some(1), Some(1), None)], 2);
        assert_eq!(unreadable.source, UsageSource::Estimated);
    }

    #[test]
    fn token_cap_distinguishes_reported_estimated_and_unavailable() {
        let limits = OvernightLimits {
            max_run_tokens: Some(100),
            ..Default::default()
        };
        let reported = aggregate_run_tokens("r", &[obs("a", "r", Some(80), Some(30), None)], 0);
        let breach = check_run_tokens(&limits, &reported).unwrap();
        assert_eq!(breach.source, UsageSource::Reported);
        assert!(breach.message.contains("110 tokens reported"));

        let lower = aggregate_run_tokens("r", &[obs("a", "r", Some(150), None, None)], 0);
        let breach = check_run_tokens(&limits, &lower).unwrap();
        assert_eq!(breach.source, UsageSource::Estimated);
        assert!(breach.message.contains("at least 150"));

        let none = aggregate_run_tokens("r", &[], 0);
        assert_eq!(none.source, UsageSource::Unavailable);
        assert!(check_run_tokens(&limits, &none).is_none());

        let under = aggregate_run_tokens("r", &[obs("a", "r", Some(40), Some(30), None)], 0);
        assert!(check_run_tokens(&limits, &under).is_none());
        assert!(check_run_tokens(&OvernightLimits::default(), &reported).is_none());
    }

    #[test]
    fn turn_time_and_agent_caps() {
        let limits = OvernightLimits {
            max_turn_minutes: Some(2),
            max_agents: Some(1),
            ..Default::default()
        };
        assert!(check_turn_time(&limits, 119).is_none());
        assert_eq!(
            check_turn_time(&limits, 120).unwrap().kind,
            LimitKind::TurnTime
        );
        assert!(check_agent_spawn(&limits, 0).is_none());
        let breach = check_agent_spawn(&limits, 1).unwrap();
        assert_eq!(breach.kind, LimitKind::Agents);
        assert!(breach.message.contains("max 1"));
        assert!(check_agent_spawn(&OvernightLimits::default(), 50).is_none());
    }

    #[test]
    fn flags_parse_before_the_mission() {
        let (limits, mission) =
            parse_limit_flags("--max-agents 3 --max-tokens=5000 --max-turn-minutes 90 fix bugs")
                .unwrap();
        assert_eq!(limits.max_agents, Some(3));
        assert_eq!(limits.max_run_tokens, Some(5000));
        assert_eq!(limits.max_turn_minutes, Some(90));
        assert_eq!(mission.as_deref(), Some("fix bugs"));

        let (limits, mission) = parse_limit_flags("fix --max-agents 9").unwrap();
        assert_eq!(limits, OvernightLimits::standard());
        assert_eq!(mission.as_deref(), Some("fix --max-agents 9"));

        let (limits, mission) = parse_limit_flags("--max-agents none").unwrap();
        assert_eq!(limits.max_agents, None);
        assert_eq!(mission, None);
        assert_eq!(
            parse_limit_flags("").unwrap().0,
            OvernightLimits::standard()
        );
    }

    #[test]
    fn bad_flags_are_rejected() {
        for bad in [
            "--max-agents",
            "--max-agents x",
            "--max-tokens 0",
            "--max-turn-minutes 0",
            "--max-bogus 1",
        ] {
            assert!(parse_limit_flags(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn legacy_manifest_fragment_has_no_limits() {
        let limits: OvernightLimits = serde_json::from_str("{}").unwrap();
        assert_eq!(limits, OvernightLimits::default());
    }
}
