//! Per-run usage reconciliation: projected range versus observed usage.
//!
//! Everything here is a post-request observation. It never reserves spend and
//! never proves a hard cap. A missing counter is `None` and is reported as
//! `estimated` or `unavailable`, never as zero.

use super::{UsageProjection, UsageProviderSnapshot, html_escape};
use chrono::{DateTime, Utc};
use jcode_usage_types::limit_bar;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub const USAGE_RECONCILED_VERSION: u32 = 1;
pub const USAGE_BASIS: &str = "post_request_observation";

/// One provider-reported response, as appended to the run ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsageObservation {
    /// Unique per upstream response attempt. Replays and mirrors share it.
    pub event_id: String,
    pub run_id: String,
    pub session_id: String,
    /// Execution generation of the session when the usage arrived.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<u64>,
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageSource {
    Reported,
    Estimated,
    Unavailable,
}

impl UsageSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Reported => "reported",
            Self::Estimated => "estimated",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Sums over reported counters only. `None` means no response reported it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenTotals {
    pub requests: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
}

fn add(total: &mut Option<u64>, value: Option<u64>) {
    if let Some(value) = value {
        *total = Some(total.unwrap_or(0).saturating_add(value));
    }
}

impl TokenTotals {
    fn push(&mut self, o: &UsageObservation) {
        self.requests += 1;
        add(&mut self.input_tokens, o.input_tokens);
        add(&mut self.output_tokens, o.output_tokens);
        add(&mut self.cache_read_input_tokens, o.cache_read_input_tokens);
        add(
            &mut self.cache_creation_input_tokens,
            o.cache_creation_input_tokens,
        );
    }

    fn summary(&self) -> String {
        let part = |label: &str, value: Option<u64>| {
            value.map_or_else(|| format!("{label} n/a"), |v| format!("{v} {label}"))
        };
        format!(
            "{} req · {} · {} · {}",
            self.requests,
            part("in", self.input_tokens),
            part("out", self.output_tokens),
            part("cached", self.cache_read_input_tokens)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenerationUsage {
    pub session_id: String,
    /// `None`: the session had no control generation when usage arrived.
    pub generation: Option<u64>,
    pub totals: TokenTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderUsageReconciliation {
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    pub source: UsageSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub totals: Option<TokenTotals>,
    pub note: String,
}

/// Control-plane view of one run-owned session at run end.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSessionState {
    pub session_id: String,
    pub control_state: String,
    /// Stop requested, draining or unresolved: work may still be consuming.
    pub unsettled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionUsage {
    pub session_id: String,
    pub control_state: String,
    pub unsettled: bool,
    pub source: UsageSource,
    pub generations: Vec<GenerationUsage>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LimitReconciliation {
    pub provider_name: String,
    pub limit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_percent: Option<f32>,
    /// Provider-reported limit percentage at run end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_percent: Option<f32>,
    pub source: UsageSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectedUsage {
    pub delta_min_percent: Option<f32>,
    pub delta_max_percent: Option<f32>,
    pub end_min_percent: Option<f32>,
    pub end_max_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageReconciled {
    pub version: u32,
    pub run_id: String,
    pub reconciled_at: DateTime<Utc>,
    /// Always [`USAGE_BASIS`]: observation after the request, not a reservation.
    pub basis: String,
    pub projected: ProjectedUsage,
    pub providers: Vec<ProviderUsageReconciliation>,
    pub limits: Vec<LimitReconciliation>,
    pub sessions: Vec<SessionUsage>,
    pub notes: Vec<String>,
}

pub struct ReconcileInput<'a> {
    pub run_id: &'a str,
    /// Provider the run was started on, so missing usage there is visible.
    pub expected_provider: &'a str,
    pub preflight: Option<&'a UsageProjection>,
    pub observations: &'a [UsageObservation],
    pub sessions: &'a [RunSessionState],
    pub end_limits: &'a [UsageProviderSnapshot],
    pub reconciled_at: DateTime<Utc>,
}

/// Drop replayed or mirrored copies of one response, keeping the first.
pub fn dedup_observations<'a>(
    run_id: &str,
    observations: &'a [UsageObservation],
) -> Vec<&'a UsageObservation> {
    let mut seen = HashSet::new();
    observations
        .iter()
        .filter(|o| o.run_id == run_id && seen.insert(o.event_id.as_str()))
        .collect()
}

pub fn reconcile_usage(input: ReconcileInput<'_>) -> UsageReconciled {
    let observed = dedup_observations(input.run_id, input.observations);
    let projected = ProjectedUsage {
        delta_min_percent: input.preflight.and_then(|p| p.projected_delta_min_percent),
        delta_max_percent: input.preflight.and_then(|p| p.projected_delta_max_percent),
        end_min_percent: input.preflight.and_then(|p| p.projected_end_min_percent),
        end_max_percent: input.preflight.and_then(|p| p.projected_end_max_percent),
    };
    let has_projection = projected.end_min_percent.is_some() && projected.end_max_percent.is_some();

    // Provider and account stay separate: one login's usage never hides in another's.
    let mut groups: BTreeMap<(String, Option<String>), TokenTotals> = BTreeMap::new();
    for o in &observed {
        groups
            .entry((o.provider.to_ascii_lowercase(), o.account.clone()))
            .or_default()
            .push(o);
    }
    let mut providers: Vec<ProviderUsageReconciliation> = groups
        .into_iter()
        .map(
            |((provider, account), totals)| ProviderUsageReconciliation {
                note: totals.summary(),
                provider,
                account,
                source: UsageSource::Reported,
                totals: Some(totals),
            },
        )
        .collect();
    let expected = input.expected_provider.to_ascii_lowercase();
    if !expected.is_empty() && !providers.iter().any(|p| p.provider == expected) {
        let (source, note) = if has_projection {
            (
                UsageSource::Estimated,
                "No usage reported; only the preflight projection applies",
            )
        } else {
            (
                UsageSource::Unavailable,
                "No usage reported and no projection available",
            )
        };
        providers.push(ProviderUsageReconciliation {
            provider: expected,
            account: None,
            source,
            totals: None,
            note: note.to_string(),
        });
    }

    let mut per_session: BTreeMap<&str, BTreeMap<Option<u64>, TokenTotals>> = BTreeMap::new();
    for o in &observed {
        per_session
            .entry(o.session_id.as_str())
            .or_default()
            .entry(o.generation)
            .or_default()
            .push(o);
    }
    let known: BTreeSet<&str> = input
        .sessions
        .iter()
        .map(|s| s.session_id.as_str())
        .collect();
    let mut sessions: Vec<SessionUsage> = input
        .sessions
        .iter()
        .map(|s| (s.session_id.as_str(), s.control_state.as_str(), s.unsettled))
        .chain(
            // Usage from a session the control plane no longer lists stays attributed.
            per_session
                .keys()
                .filter(|id| !known.contains(*id))
                .map(|id| (*id, "unknown", false)),
        )
        .map(|(id, state, unsettled)| {
            let generations: Vec<GenerationUsage> = per_session
                .get(id)
                .into_iter()
                .flatten()
                .map(|(generation, totals)| GenerationUsage {
                    session_id: id.to_string(),
                    generation: *generation,
                    totals: totals.clone(),
                })
                .collect();
            let reported = !generations.is_empty();
            let note = match (reported, unsettled) {
                (true, true) => "Reported so far; unsettled work may consume more",
                (true, false) => "Reported",
                (false, true) => "Unsettled work; consumption unknown, not zero",
                (false, false) => "No usage reported; consumption unknown, not zero",
            };
            SessionUsage {
                session_id: id.to_string(),
                control_state: state.to_string(),
                unsettled,
                source: if reported {
                    UsageSource::Reported
                } else {
                    UsageSource::Unavailable
                },
                generations,
                note: note.to_string(),
            }
        })
        .collect();
    sessions.sort_by(|a, b| a.session_id.cmp(&b.session_id));

    let mut limits = Vec::new();
    for provider in input.preflight.iter().flat_map(|p| &p.providers) {
        for start in &provider.limits {
            let end = input
                .end_limits
                .iter()
                .filter(|p| p.provider_name == provider.provider_name)
                .flat_map(|p| &p.limits)
                .find(|l| l.name == start.name);
            limits.push(LimitReconciliation {
                provider_name: provider.provider_name.clone(),
                limit: start.name.clone(),
                start_percent: Some(start.usage_percent),
                end_percent: end.map(|l| l.usage_percent),
                source: match end {
                    Some(_) => UsageSource::Reported,
                    None if has_projection => UsageSource::Estimated,
                    None => UsageSource::Unavailable,
                },
            });
        }
    }

    let mut notes = vec![
        "Post-request observation. Not a spend reservation and not proof of a hard cap."
            .to_string(),
        "Counts provider stream responses only; compaction, summaries and other non-stream requests are not included.".to_string(),
    ];
    if sessions.iter().any(|s| s.unsettled) {
        notes.push(
            "Some run-owned sessions were unsettled at run end; their usage is a lower bound."
                .to_string(),
        );
    }
    if input.end_limits.is_empty() {
        notes.push("End-of-run provider limit percentages were unavailable.".to_string());
    }

    UsageReconciled {
        version: USAGE_RECONCILED_VERSION,
        run_id: input.run_id.to_string(),
        reconciled_at: input.reconciled_at,
        basis: USAGE_BASIS.to_string(),
        projected,
        providers,
        limits,
        sessions,
        notes,
    }
}

fn range(min: Option<f32>, max: Option<f32>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("{min:.0}% to {max:.0}%"),
        _ => "unknown".to_string(),
    }
}

pub fn usage_reconciled_summary(usage: &UsageReconciled) -> String {
    let sources = |s: UsageSource| usage.providers.iter().filter(|p| p.source == s).count();
    format!(
        "Usage reconciled: projected end {}; providers reported {}, estimated {}, unavailable {} (post-request observation, not a spend cap)",
        range(
            usage.projected.end_min_percent,
            usage.projected.end_max_percent
        ),
        sources(UsageSource::Reported),
        sources(UsageSource::Estimated),
        sources(UsageSource::Unavailable),
    )
}

/// One limit bar row. Glyphs, colors and used/left convention come from
/// `jcode_usage_types::limit_bar`, the same source as the info widget.
fn limit_bar_html(used: f32) -> String {
    let used_pct = used.round().clamp(0.0, 100.0) as u8;
    let (filled, empty) = limit_bar::cells(used_pct, limit_bar::MAX_CELLS);
    let (r, g, b) = limit_bar::color_rgb(100 - used_pct);
    format!(
        "<span class=\"limit-bar\" style=\"color: rgb({r},{g},{b})\">{}{}</span> {used_pct}% used",
        limit_bar::FILLED.to_string().repeat(filled),
        limit_bar::EMPTY.to_string().repeat(empty),
    )
}

pub fn render_usage_reconciled_html(usage: Option<&UsageReconciled>) -> String {
    let Some(usage) = usage else {
        return "<p class=\"meta\">Usage reconciliation pending: it is recorded when the run ends.</p>".to_string();
    };
    let mut html = format!(
        "<p>Projected end: <strong>{}</strong> <span class=\"badge\">estimated</span></p>\n",
        html_escape(&range(
            usage.projected.end_min_percent,
            usage.projected.end_max_percent
        ))
    );
    html.push_str("<ul>\n");
    for l in &usage.limits {
        let observed = match l.end_percent {
            Some(end) => limit_bar_html(end),
            None => "no end-of-run value".to_string(),
        };
        html.push_str(&format!(
            "<li>{} / {}: start {} · observed {} <span class=\"badge\">{}</span></li>\n",
            html_escape(&l.provider_name),
            html_escape(&l.limit),
            l.start_percent
                .map_or_else(|| "n/a".to_string(), |p| format!("{p:.0}%")),
            observed,
            l.source.label(),
        ));
    }
    for p in &usage.providers {
        html.push_str(&format!(
            "<li>{}{}: {} <span class=\"badge\">{}</span></li>\n",
            html_escape(&p.provider),
            p.account
                .as_deref()
                .map(|a| format!(" ({})", html_escape(a)))
                .unwrap_or_default(),
            html_escape(&p.note),
            p.source.label(),
        ));
    }
    html.push_str("</ul>\n<ul>\n");
    for s in &usage.sessions {
        let gens = s
            .generations
            .iter()
            .map(|g| {
                format!(
                    "gen {}: {}",
                    g.generation
                        .map_or("unbound".to_string(), |g| g.to_string()),
                    g.totals.summary()
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        html.push_str(&format!(
            "<li><code>{}</code> [{}] {} {} <span class=\"badge\">{}</span></li>\n",
            html_escape(&s.session_id),
            html_escape(&s.control_state),
            html_escape(&gens),
            html_escape(&s.note),
            s.source.label(),
        ));
    }
    html.push_str("</ul>\n");
    for note in &usage.notes {
        html.push_str(&format!("<p class=\"meta\">{}</p>\n", html_escape(note)));
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UsageLimitSnapshot;

    fn obs(
        event: &str,
        session: &str,
        generation: Option<u64>,
        provider: &str,
    ) -> UsageObservation {
        UsageObservation {
            event_id: event.into(),
            run_id: "run".into(),
            session_id: session.into(),
            generation,
            provider: provider.into(),
            account: None,
            model: "m".into(),
            input_tokens: Some(100),
            output_tokens: Some(10),
            cache_read_input_tokens: None,
            cache_creation_input_tokens: None,
            observed_at: Utc::now(),
        }
    }

    fn projection() -> UsageProjection {
        UsageProjection {
            captured_at: Utc::now(),
            risk: "low".into(),
            confidence: "medium".into(),
            projected_delta_min_percent: Some(3.0),
            projected_delta_max_percent: Some(7.0),
            projected_end_min_percent: Some(53.0),
            projected_end_max_percent: Some(57.0),
            providers: vec![UsageProviderSnapshot {
                provider_name: "OpenAI".into(),
                hard_limit_reached: false,
                error: None,
                limits: vec![UsageLimitSnapshot {
                    name: "5h".into(),
                    usage_percent: 50.0,
                    resets_at: None,
                }],
                extra_info: vec![],
            }],
            notes: vec![],
        }
    }

    fn run<'a>(
        observations: &'a [UsageObservation],
        sessions: &'a [RunSessionState],
        preflight: Option<&'a UsageProjection>,
        end: &'a [UsageProviderSnapshot],
    ) -> UsageReconciled {
        reconcile_usage(ReconcileInput {
            run_id: "run",
            expected_provider: "openai",
            preflight,
            observations,
            sessions,
            end_limits: end,
            reconciled_at: Utc::now(),
        })
    }

    #[test]
    fn duplicate_event_ids_count_once_and_other_runs_are_ignored() {
        let mut foreign = obs("z", "w", Some(1), "openai");
        foreign.run_id = "other".into();
        let all = [
            obs("a", "w", Some(1), "openai"),
            obs("a", "w", Some(1), "openai"),
            obs("b", "w", Some(1), "openai"),
            foreign,
        ];
        let out = run(&all, &[], None, &[]);
        let totals = out.providers[0].totals.as_ref().unwrap();
        assert_eq!((totals.requests, totals.input_tokens), (2, Some(200)));
    }

    #[test]
    fn generations_stay_separate_and_unbound_is_not_a_generation() {
        let all = [
            obs("a", "w", Some(1), "openai"),
            obs("b", "w", Some(2), "openai"),
            obs("c", "w", Some(2), "openai"),
            obs("d", "w", None, "openai"),
        ];
        let out = run(&all, &[], None, &[]);
        let gens: Vec<_> = out.sessions[0]
            .generations
            .iter()
            .map(|g| (g.generation, g.totals.requests))
            .collect();
        assert_eq!(gens, vec![(None, 1), (Some(1), 1), (Some(2), 2)]);
    }

    #[test]
    fn provider_and_account_never_merge() {
        let mut a = obs("a", "w", Some(1), "openai");
        a.account = Some("one".into());
        let mut b = obs("b", "w", Some(1), "openai");
        b.account = Some("two".into());
        let c = obs("c", "w", Some(1), "claude");
        let out = run(&[a, b, c], &[], None, &[]);
        let keys: Vec<_> = out
            .providers
            .iter()
            .map(|p| (p.provider.as_str(), p.account.as_deref()))
            .collect();
        assert_eq!(
            keys,
            vec![
                ("claude", None),
                ("openai", Some("one")),
                ("openai", Some("two"))
            ]
        );
    }

    #[test]
    fn missing_usage_is_estimated_or_unavailable_never_zero() {
        let projection = projection();
        let estimated = run(&[], &[], Some(&projection), &[]);
        assert_eq!(estimated.providers[0].source, UsageSource::Estimated);
        assert!(estimated.providers[0].totals.is_none());
        let unavailable = run(&[], &[], None, &[]);
        assert_eq!(unavailable.providers[0].source, UsageSource::Unavailable);
        assert!(unavailable.providers[0].totals.is_none());
        // A counter nobody reported stays None, not 0.
        let all = [obs("a", "w", Some(1), "openai")];
        let totals = run(&all, &[], None, &[]).providers[0]
            .totals
            .clone()
            .unwrap();
        assert_eq!(totals.cache_read_input_tokens, None);
    }

    #[test]
    fn unsettled_worker_without_usage_is_unknown_not_zero() {
        let sessions = [RunSessionState {
            session_id: "w".into(),
            control_state: "draining".into(),
            unsettled: true,
        }];
        let all = [obs("a", "coord", Some(1), "openai")];
        let out = run(&all, &sessions, None, &[]);
        let worker = out.sessions.iter().find(|s| s.session_id == "w").unwrap();
        assert_eq!(worker.source, UsageSource::Unavailable);
        assert!(worker.generations.is_empty());
        assert!(worker.note.contains("not zero"));
        assert!(out.notes.iter().any(|n| n.contains("lower bound")));
        // Unsettled with partial usage keeps what was reported, marked as such.
        let partial = [obs("a", "w", Some(1), "openai")];
        let out = run(&partial, &sessions, None, &[]);
        assert_eq!(out.sessions[0].source, UsageSource::Reported);
        assert!(out.sessions[0].note.contains("unsettled"));
    }

    #[test]
    fn reported_limits_carry_projection_and_labels_never_claim_authority() {
        let projection = projection();
        let end = [UsageProviderSnapshot {
            provider_name: "OpenAI".into(),
            hard_limit_reached: false,
            error: None,
            limits: vec![UsageLimitSnapshot {
                name: "5h".into(),
                usage_percent: 56.0,
                resets_at: None,
            }],
            extra_info: vec![],
        }];
        let out = run(&[], &[], Some(&projection), &end);
        assert_eq!(out.limits[0].source, UsageSource::Reported);
        assert_eq!(out.limits[0].end_percent, Some(56.0));
        assert_eq!(out.basis, USAGE_BASIS);
        let html = render_usage_reconciled_html(Some(&out));
        assert!(html.contains("reported"));
        assert!(html.contains("53% to 57%"));
        assert!(html.contains(limit_bar::FILLED));
        assert!(html.contains("Not a spend reservation"));
        assert!(!html.to_lowercase().contains("authoritative"));
        // No end-of-run percent: estimated, not a made-up zero.
        let missing = run(&[], &[], Some(&projection), &[]);
        assert_eq!(missing.limits[0].source, UsageSource::Estimated);
        assert_eq!(missing.limits[0].end_percent, None);
        assert!(usage_reconciled_summary(&missing).contains("post-request observation"));
    }

    #[test]
    fn html_escapes_untrusted_names() {
        let mut o = obs("a", "<s>", Some(1), "openai");
        o.account = Some("<b>".into());
        let html = render_usage_reconciled_html(Some(&run(&[o], &[], None, &[])));
        assert!(!html.contains("<b>") && !html.contains("<s>"));
    }
}
