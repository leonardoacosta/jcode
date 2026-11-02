//! `council` tool: a bounded, read-only debate between swarm seats.
//!
//! Thin orchestrator over the swarm socket primitives (spawn, message, read
//! context, stop). Seats are spawned with a Rust-side tool allowlist so they
//! cannot edit files, every round ends on a barrier (all replies or timeout),
//! and a separate verdict seat returns PASS/PARTIAL with a tally and dissent.

use super::communicate::transport::send_request;
use super::{Tool, ToolContext, ToolOutput};
use crate::protocol::{CommDeliveryMode, HistoryMessage, Request, ServerEvent};
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

const REQUEST_ID: u64 = 1;
const MIN_SEATS: usize = 2;
const MAX_SEATS: usize = 4;
const MAX_ROUNDS: u32 = 3;
const DEFAULT_TOTAL_TIMEOUT_SECS: u64 = 600;
const MIN_ROUND_TIMEOUT_SECS: u64 = 60;
const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// Tools a seat may call. Enforced by the registry executor, not the prompt.
pub(crate) const SEAT_ALLOWED_TOOLS: [&str; 4] = ["read", "agentgrep", "webfetch", "websearch"];

#[derive(Debug, Clone, Deserialize)]
struct SeatSpec {
    name: String,
    brief: String,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CouncilInput {
    action: String,
    #[serde(default)]
    council_id: Option<String>,
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    seats: Vec<SeatSpec>,
    #[serde(default)]
    rounds: Option<u32>,
    #[serde(default)]
    timeout_secs: Option<u64>,
    #[serde(default)]
    max_calls: Option<u32>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Turn {
    round: u32,
    seat: String,
    text: String,
    vote: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct Verdict {
    status: String,
    tally: BTreeMap<String, usize>,
    dissent: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Seat {
    name: String,
    session_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct Council {
    id: String,
    owner_session: String,
    topic: String,
    seats: Vec<Seat>,
    rounds: u32,
    rounds_done: u32,
    round_timeout_secs: u64,
    max_calls: u32,
    calls: u32,
    transcript: Vec<Turn>,
    notes: Vec<String>,
    converged: bool,
    verdict: Option<Verdict>,
}

static COUNCILS: LazyLock<Mutex<HashMap<String, Council>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn councils() -> std::sync::MutexGuard<'static, HashMap<String, Council>> {
    COUNCILS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Validate caps before any seat is spawned. Returns (rounds, max_calls).
pub(crate) fn validate_caps(
    seat_count: usize,
    rounds: Option<u32>,
    max_calls: Option<u32>,
) -> Result<(u32, u32)> {
    if !(MIN_SEATS..=MAX_SEATS).contains(&seat_count) {
        bail!("council needs {MIN_SEATS}-{MAX_SEATS} seats, got {seat_count}");
    }
    let rounds = rounds.unwrap_or(1);
    if rounds > MAX_ROUNDS {
        bail!("council rounds must be 0-{MAX_ROUNDS}, got {rounds}");
    }
    let default_calls = seat_count as u32 * (rounds + 1) + 1;
    let max_calls = max_calls.unwrap_or(default_calls);
    // Opening statements plus the verdict must always fit.
    let floor = seat_count as u32 + 1;
    if max_calls < floor {
        bail!("max_calls {max_calls} is below the minimum {floor} (openings + verdict)");
    }
    Ok((rounds, max_calls))
}

pub(crate) fn round_timeout_secs(total_secs: u64, rounds: u32) -> u64 {
    (total_secs / (rounds as u64 + 1)).max(MIN_ROUND_TIMEOUT_SECS)
}

/// Last `VOTE: <option>` line, normalized. Missing vote = abstain (None).
pub(crate) fn parse_vote(text: &str) -> Option<String> {
    text.lines().rev().find_map(|line| {
        let line = line.trim().trim_matches(|c| c == '*' || c == '`').trim();
        let rest = line
            .get(..5)
            .filter(|p| p.eq_ignore_ascii_case("vote:"))
            .map(|_| &line[5..])?;
        let vote = rest.trim().trim_matches(|c| c == '*' || c == '`').trim();
        (!vote.is_empty()).then(|| vote.to_ascii_lowercase())
    })
}

/// True only if every seat cast a vote and all votes match.
pub(crate) fn unanimous(votes: &[Option<String>]) -> bool {
    let Some(Some(first)) = votes.first() else {
        return false;
    };
    votes.iter().all(|v| v.as_deref() == Some(first.as_str()))
}

/// Parse verdict-seat output. Anything unparseable is PARTIAL.
pub(crate) fn parse_verdict(text: &str, fallback_tally: &BTreeMap<String, usize>) -> Verdict {
    let mut status = None;
    let mut tally = BTreeMap::new();
    let mut dissent = Vec::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches(['-', '*', ' ']);
        let upper = line.to_ascii_uppercase();
        if let Some(rest) = upper.strip_prefix("VERDICT:") {
            status = match rest.trim() {
                s if s.starts_with("PASS") => Some("PASS"),
                s if s.starts_with("PARTIAL") => Some("PARTIAL"),
                _ => None,
            };
        } else if upper.starts_with("TALLY:") {
            for part in line[6..].split(',') {
                if let Some((opt, n)) = part.split_once('=')
                    && let Ok(n) = n.trim().parse::<usize>()
                {
                    tally.insert(opt.trim().to_ascii_lowercase(), n);
                }
            }
        } else if upper.starts_with("DISSENT:") {
            let d = line[8..].trim();
            if !d.is_empty() && !d.eq_ignore_ascii_case("none") {
                dissent.push(d.to_string());
            }
        }
    }
    Verdict {
        status: status.unwrap_or("PARTIAL").to_string(),
        tally: if tally.is_empty() {
            fallback_tally.clone()
        } else {
            tally
        },
        dissent,
    }
}

fn tally_votes(turns: &[Turn], round: u32) -> BTreeMap<String, usize> {
    let mut tally = BTreeMap::new();
    for turn in turns.iter().filter(|t| t.round == round) {
        let key = turn.vote.clone().unwrap_or_else(|| "abstain".to_string());
        *tally.entry(key).or_insert(0) += 1;
    }
    tally
}

fn seat_task(seat: &SeatSpec, council_id: &str, topic: &str) -> String {
    format!(
        "{brief}\n\nYou are seat \"{name}\" in council {council_id}. You are read-only: you may \
         read, search and fetch, never edit. Each message you receive is the next turn of the \
         debate. Take a clear position, argue with concrete evidence, and change your mind if \
         another seat out-argues you. Reply in under 200 words. End every reply with a final \
         line `VOTE: <short option>`.\n\nTopic: {topic}",
        brief = seat.brief.trim(),
        name = seat.name,
    )
}

fn format_transcript(turns: &[Turn]) -> String {
    turns
        .iter()
        .map(|t| format!("[round {} | {}]\n{}", t.round, t.seat, t.text.trim()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn check_error(response: &ServerEvent) -> Result<()> {
    if let ServerEvent::Error { message, .. } = response {
        bail!("{message}");
    }
    Ok(())
}

async fn spawn_seat(
    ctx: &ToolContext,
    council_id: &str,
    label: String,
    model: Option<String>,
) -> Result<String> {
    let nonce = format!(
        "council-{}-{}",
        ctx.tool_call_id,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let request = Request::CommSpawn {
        id: REQUEST_ID,
        session_id: ctx.session_id.clone(),
        council_id: Some(council_id.to_string()),
        working_dir: ctx.working_dir.as_ref().map(|p| p.display().to_string()),
        initial_message: None,
        request_nonce: Some(nonce),
        spawn_mode: Some("headless".to_string()),
        model: model.or_else(|| crate::config::config().agents.roles.council.clone()),
        effort: None,
        label: Some(label),
    };
    match send_request(request).await? {
        ServerEvent::CommSpawnResponse { new_session_id, .. } if !new_session_id.is_empty() => {
            Ok(new_session_id)
        }
        other => {
            check_error(&other)?;
            Err(anyhow!("council seat spawn returned no session id"))
        }
    }
}

async fn send_to(ctx: &ToolContext, target: &str, message: String) -> Result<()> {
    let request = Request::CommMessage {
        id: REQUEST_ID,
        from_session: ctx.session_id.clone(),
        message,
        to_session: Some(target.to_string()),
        channel: None,
        wake: Some(true),
        delivery: Some(CommDeliveryMode::Wake),
        tldr: Some("council turn".to_string()),
        to_swarm: None,
        require_target: true,
    };
    check_error(&send_request(request).await?)
}

async fn read_history(ctx: &ToolContext, target: &str) -> Result<Vec<HistoryMessage>> {
    let request = Request::CommReadContext {
        id: REQUEST_ID,
        session_id: ctx.session_id.clone(),
        target_session: target.to_string(),
    };
    match send_request(request).await? {
        ServerEvent::CommContextHistory { messages, .. } => Ok(messages),
        other => {
            check_error(&other)?;
            Ok(Vec::new())
        }
    }
}

fn assistant_count(messages: &[HistoryMessage]) -> usize {
    messages.iter().filter(|m| m.role == "assistant").count()
}

async fn stop_seats(ctx: &ToolContext, session_ids: &[String]) {
    for target in session_ids {
        let _ = send_request(Request::CommStop {
            id: REQUEST_ID,
            session_id: ctx.session_id.clone(),
            target_session: target.clone(),
            force: Some(true),
        })
        .await;
        super::clear_session_tool_policy_entry(target);
    }
}

/// Send a prompt to every seat and wait until each replies or the timeout fires.
/// Returns replies in seat order; `None` marks a late or errored seat.
async fn barrier(
    ctx: &ToolContext,
    seats: &[Seat],
    prompts: Vec<String>,
    timeout: Duration,
) -> Vec<Option<String>> {
    let mut baseline = Vec::with_capacity(seats.len());
    for seat in seats {
        baseline.push(
            read_history(ctx, &seat.session_id)
                .await
                .map(|m| assistant_count(&m))
                .unwrap_or(0),
        );
    }
    let mut replies: Vec<Option<String>> = vec![None; seats.len()];
    let mut failed = vec![false; seats.len()];
    for (i, (seat, prompt)) in seats.iter().zip(prompts).enumerate() {
        if send_to(ctx, &seat.session_id, prompt).await.is_err() {
            failed[i] = true;
        }
    }
    let deadline = Instant::now() + timeout;
    loop {
        for (i, seat) in seats.iter().enumerate() {
            if replies[i].is_some() || failed[i] {
                continue;
            }
            if let Ok(messages) = read_history(ctx, &seat.session_id).await
                && assistant_count(&messages) > baseline[i]
                && let Some(text) = crate::protocol::latest_assistant_comm_report(&messages)
            {
                replies[i] = Some(text);
            }
        }
        let pending = replies
            .iter()
            .zip(&failed)
            .any(|(r, f)| r.is_none() && !f);
        if !pending || Instant::now() >= deadline {
            return replies;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

fn persist(council: &Council) {
    let Ok(path) = crate::session::session_path(&council.owner_session) else {
        return;
    };
    let Some(dir) = path.parent() else { return };
    let file = dir.join(format!(
        "{}.council-{}.json",
        council.owner_session, council.id
    ));
    if let Ok(body) = serde_json::to_vec_pretty(council) {
        let _ = std::fs::write(file, body);
    }
}

fn record_round(council: &mut Council, round: u32, replies: Vec<Option<String>>) -> Vec<String> {
    let mut late = Vec::new();
    for (seat, reply) in council.seats.iter().zip(replies) {
        match reply {
            Some(text) => {
                council.calls += 1;
                council.transcript.push(Turn {
                    round,
                    seat: seat.name.clone(),
                    vote: parse_vote(&text),
                    text,
                });
            }
            None => {
                council.calls += 1;
                late.push(seat.name.clone());
                council.transcript.push(Turn {
                    round,
                    seat: seat.name.clone(),
                    text: "(no reply before timeout)".to_string(),
                    vote: None,
                });
            }
        }
    }
    let votes: Vec<_> = council
        .transcript
        .iter()
        .filter(|t| t.round == round)
        .map(|t| t.vote.clone())
        .collect();
    council.converged = unanimous(&votes);
    if !late.is_empty() {
        council
            .notes
            .push(format!("round {round}: late/errored seats: {}", late.join(", ")));
    }
    late
}

fn take(id: &str) -> Result<Council> {
    councils()
        .remove(id)
        .ok_or_else(|| anyhow!("unknown council_id '{id}'"))
}

fn put(council: Council) {
    persist(&council);
    councils().insert(council.id.clone(), council);
}

fn summary(council: &Council) -> String {
    let last = council.rounds_done;
    let tally = tally_votes(&council.transcript, last);
    let mut out = format!(
        "council {} | round {}/{} | calls {}/{} | converged: {}\ntally: {:?}",
        council.id,
        council.rounds_done,
        council.rounds,
        council.calls,
        council.max_calls,
        council.converged,
        tally
    );
    for note in &council.notes {
        out.push_str(&format!("\nnote: {note}"));
    }
    out
}

async fn open(ctx: &ToolContext, input: &CouncilInput) -> Result<Council> {
    let topic = input
        .topic
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or_else(|| anyhow!("council open requires a topic"))?
        .to_string();
    let seats: Vec<SeatSpec> = input
        .seats
        .iter()
        .filter(|s| !s.name.trim().is_empty() && !s.brief.trim().is_empty())
        .cloned()
        .collect();
    let (rounds, max_calls) = validate_caps(seats.len(), input.rounds, input.max_calls)?;
    let id = format!("c{}", &uuid_like(ctx));
    let timeout = round_timeout_secs(
        input.timeout_secs.unwrap_or(DEFAULT_TOTAL_TIMEOUT_SECS),
        rounds,
    );
    let mut spawned: Vec<Seat> = Vec::new();
    let mut guard = SeatGuard {
        ctx: ctx.clone(),
        ids: Vec::new(),
    };
    for spec in &seats {
        match spawn_seat(
            ctx,
            &id,
            format!("council {id}: {}", spec.name),
            spec.model.clone(),
        )
        .await
        {
            Ok(session_id) => {
                guard.ids.push(session_id.clone());
                spawned.push(Seat {
                    name: spec.name.clone(),
                    session_id,
                });
            }
            Err(e) => {
                let ids: Vec<_> = spawned.iter().map(|s| s.session_id.clone()).collect();
                stop_seats(ctx, &ids).await;
                guard.ids.clear();
                bail!("council seat '{}' failed to spawn: {e}", spec.name);
            }
        }
    }
    let mut council = Council {
        id: id.clone(),
        owner_session: ctx.session_id.clone(),
        topic: topic.clone(),
        seats: spawned,
        rounds,
        rounds_done: 0,
        round_timeout_secs: timeout,
        max_calls,
        calls: 0,
        transcript: Vec::new(),
        notes: Vec::new(),
        converged: false,
        verdict: None,
    };
    let prompts = seats
        .iter()
        .map(|s| {
            format!(
                "{}\n\nGive your opening statement.",
                seat_task(s, &id, &topic)
            )
        })
        .collect();
    let replies = barrier(ctx, &council.seats, prompts, Duration::from_secs(timeout)).await;
    record_round(&mut council, 0, replies);
    guard.ids.clear();
    Ok(council)
}

fn uuid_like(ctx: &ToolContext) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    ctx.tool_call_id.hash(&mut h);
    std::time::SystemTime::now().hash(&mut h);
    format!("{:08x}", h.finish() as u32)
}

/// Runs one debate round. Returns false when refused (cap or rounds exhausted).
async fn round(ctx: &ToolContext, council: &mut Council) -> bool {
    if council.rounds_done >= council.rounds {
        return false;
    }
    let need = council.seats.len() as u32;
    // Reserve one call for the verdict.
    if council.calls + need + 1 > council.max_calls {
        council
            .notes
            .push("round refused: call cap reached; proceeding to verdict".to_string());
        return false;
    }
    let k = council.rounds_done + 1;
    let prior = format_transcript(&council.transcript);
    let prompts = council
        .seats
        .iter()
        .map(|s| {
            format!(
                "Round {k}. Debate so far:\n\n{prior}\n\nYou are {}. Respond to the other seats \
                 and end with `VOTE: <option>`.",
                s.name
            )
        })
        .collect();
    let replies = barrier(
        ctx,
        &council.seats,
        prompts,
        Duration::from_secs(council.round_timeout_secs),
    )
    .await;
    record_round(council, k, replies);
    council.rounds_done = k;
    true
}

async fn verdict(ctx: &ToolContext, council: &mut Council) -> Result<Verdict> {
    let tally = tally_votes(&council.transcript, council.rounds_done);
    let judge = spawn_seat(
        ctx,
        &council.id,
        format!("council {}: verdict", council.id),
        None,
    )
    .await?;
    let mut guard = SeatGuard {
        ctx: ctx.clone(),
        ids: vec![judge.clone()],
    };
    council.calls += 1;
    let prompt = format!(
        "You are the neutral judge of council {}. Topic: {}\n\nTranscript:\n\n{}\n\n\
         Reply with exactly these lines:\nVERDICT: PASS (seats converged) or PARTIAL (open)\n\
         TALLY: option=count, option=count\nDISSENT: <seat>: <reason> (one line per dissenter, \
         or `DISSENT: none`)",
        council.id,
        council.topic,
        format_transcript(&council.transcript)
    );
    let seat = Seat {
        name: "verdict".to_string(),
        session_id: judge.clone(),
    };
    let reply = barrier(
        ctx,
        std::slice::from_ref(&seat),
        vec![prompt],
        Duration::from_secs(council.round_timeout_secs),
    )
    .await;
    stop_seats(ctx, &[judge]).await;
    guard.ids.clear();
    let v = match reply.into_iter().next().flatten() {
        Some(text) => parse_verdict(&text, &tally),
        None => Verdict {
            status: "PARTIAL".to_string(),
            tally,
            dissent: vec!["verdict: judge timed out".to_string()],
        },
    };
    council.verdict = Some(v.clone());
    Ok(v)
}

async fn close(ctx: &ToolContext, mut council: Council) -> Result<ToolOutput> {
    let mut guard = SeatGuard::for_council(ctx, &council);
    let result = verdict(ctx, &mut council).await;
    let ids: Vec<_> = council.seats.iter().map(|s| s.session_id.clone()).collect();
    stop_seats(ctx, &ids).await;
    guard.ids.clear();
    persist(&council);
    let v = result?;
    Ok(ToolOutput::new(format!(
        "{}\nVERDICT: {}\ntally: {:?}\ndissent: {}\nseats stopped: {}",
        summary(&council),
        v.status,
        v.tally,
        if v.dissent.is_empty() {
            "none".to_string()
        } else {
            v.dissent.join("; ")
        },
        ids.len()
    ))
    .with_metadata(json!({ "council": council })))
}

/// Stops a council's seats unless disarmed. Covers errors and cancellation in `run`.
struct SeatGuard {
    ctx: ToolContext,
    ids: Vec<String>,
}

impl SeatGuard {
    fn for_council(ctx: &ToolContext, council: &Council) -> Self {
        Self {
            ctx: ctx.clone(),
            ids: council
                .seats
                .iter()
                .map(|seat| seat.session_id.clone())
                .collect(),
        }
    }
}

impl Drop for SeatGuard {
    fn drop(&mut self) {
        if self.ids.is_empty() {
            return;
        }
        let ctx = self.ctx.clone();
        let ids = std::mem::take(&mut self.ids);
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move { stop_seats(&ctx, &ids).await });
        }
    }
}

pub struct CouncilTool;

impl CouncilTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CouncilTool {
    fn name(&self) -> &str {
        "council"
    }

    fn description(&self) -> &str {
        "Debate a decision with 2-4 read-only seats; returns a PASS/PARTIAL verdict."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["action"],
            "properties": {
                "intent": super::intent_schema_property(),
                "action": {"type": "string", "enum": ["run", "open", "round", "say", "log", "close"]},
                "council_id": {"type": "string", "description": "Council ID from open (round/say/log/close)."},
                "topic": {"type": "string", "description": "Decision under debate (open/run)."},
                "seats": {
                    "type": "array",
                    "description": "2-4 seats (open/run).",
                    "items": {
                        "type": "object",
                        "required": ["name", "brief"],
                        "properties": {
                            "name": {"type": "string"},
                            "brief": {"type": "string", "description": "Stance or perspective."},
                            "model": {"type": "string"}
                        }
                    }
                },
                "rounds": {"type": "integer", "minimum": 0, "maximum": 3, "description": "Debate rounds after openings. Default 1."},
                "timeout_secs": {"type": "integer", "description": "Total time budget. Default 600."},
                "max_calls": {"type": "integer", "description": "Seat-reply cap. Default seats*(rounds+1)+1."},
                "message": {"type": "string", "description": "Convener message for say."}
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let input: CouncilInput = serde_json::from_value(input)?;
        let id = || {
            input
                .council_id
                .clone()
                .ok_or_else(|| anyhow!("council_id is required for '{}'", input.action))
        };
        match input.action.as_str() {
            "run" => {
                let council = open(&ctx, &input).await?;
                let mut guard = SeatGuard::for_council(&ctx, &council);
                let mut council = council;
                persist(&council);
                while !council.converged && round(&ctx, &mut council).await {
                    persist(&council);
                }
                if council.converged && council.rounds_done < council.rounds {
                    council.notes.push(format!(
                        "early stop: unanimous after round {}",
                        council.rounds_done
                    ));
                }
                let result = close(&ctx, council).await;
                guard.ids.clear();
                result
            }
            "open" => {
                let council = open(&ctx, &input).await?;
                let out = format!("{}\n\n{}", summary(&council), format_transcript(&council.transcript));
                put(council);
                Ok(ToolOutput::new(out))
            }
            "round" => {
                let mut council = take(&id()?)?;
                let mut guard = SeatGuard::for_council(&ctx, &council);
                let ran = round(&ctx, &mut council).await;
                let out = if ran {
                    let k = council.rounds_done;
                    let turns: Vec<_> = council.transcript.iter().filter(|t| t.round == k).cloned().collect();
                    format!("{}\n\n{}", summary(&council), format_transcript(&turns))
                } else {
                    format!("round refused (rounds or call cap exhausted); call close.\n{}", summary(&council))
                };
                put(council);
                guard.ids.clear();
                Ok(ToolOutput::new(out))
            }
            "say" => {
                let council = take(&id()?)?;
                let mut guard = SeatGuard::for_council(&ctx, &council);
                let msg = input.message.clone().unwrap_or_default();
                let mut failed = Vec::new();
                for seat in &council.seats {
                    if send_to(&ctx, &seat.session_id, format!("Convener: {msg}")).await.is_err() {
                        failed.push(seat.name.clone());
                    }
                }
                put(council);
                guard.ids.clear();
                Ok(ToolOutput::new(if failed.is_empty() {
                    "sent to all seats".to_string()
                } else {
                    format!("send failed for: {}", failed.join(", "))
                }))
            }
            "log" => {
                let id = id()?;
                let map = councils();
                let council = map.get(&id).ok_or_else(|| anyhow!("unknown council_id '{id}'"))?;
                Ok(ToolOutput::new(format!(
                    "{}\n\n{}",
                    summary(council),
                    format_transcript(&council.transcript)
                )))
            }
            "close" => close(&ctx, take(&id()?)?).await,
            other => bail!("unknown council action '{other}'"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn rejects_bad_seat_counts_and_rounds() {
        assert!(validate_caps(1, None, None).is_err());
        assert!(validate_caps(5, None, None).is_err());
        assert!(validate_caps(3, Some(4), None).is_err());
        assert_eq!(validate_caps(3, Some(1), None).unwrap(), (1, 7));
        assert_eq!(validate_caps(2, Some(0), None).unwrap(), (0, 3));
    }

    #[test]
    fn call_cap_below_floor_is_rejected() {
        assert!(validate_caps(3, Some(2), Some(3)).is_err());
        assert_eq!(validate_caps(3, Some(2), Some(4)).unwrap(), (2, 4));
    }

    fn council(max_calls: u32) -> Council {
        Council {
            id: "t".into(),
            owner_session: "o".into(),
            topic: "x".into(),
            seats: vec![
                Seat {
                    name: "a".into(),
                    session_id: "sa".into(),
                },
                Seat {
                    name: "b".into(),
                    session_id: "sb".into(),
                },
            ],
            rounds: 3,
            rounds_done: 0,
            round_timeout_secs: 60,
            max_calls,
            calls: 0,
            transcript: vec![],
            notes: vec![],
            converged: false,
            verdict: None,
        }
    }

    #[tokio::test]
    async fn round_refused_when_cap_reached() {
        let mut c = council(3);
        c.calls = 2;
        let ctx = ToolContext {
            session_id: "o".into(),
            message_id: "m".into(),
            tool_call_id: "t".into(),
            working_dir: None,
            stdin_request_tx: None,
            graceful_shutdown_signal: None,
            execution_mode: crate::tool::ToolExecutionMode::Direct,
        };
        assert!(!round(&ctx, &mut c).await);
        assert!(c.notes[0].contains("call cap"));
    }

    #[test]
    fn timeout_names_late_seats_and_blocks_early_stop() {
        let mut c = council(9);
        let late = record_round(&mut c, 0, vec![Some("ok\nVOTE: A".into()), None]);
        assert_eq!(late, vec!["b".to_string()]);
        assert!(!c.converged);
        assert!(c.notes[0].contains("b"));
    }

    #[test]
    fn unanimous_votes_converge() {
        let mut c = council(9);
        record_round(
            &mut c,
            0,
            vec![
                Some("x\nVOTE: Ship".into()),
                Some("y\n**VOTE: ship**".into()),
            ],
        );
        assert!(c.converged);
        assert_eq!(parse_vote("no vote here"), None);
        assert!(!unanimous(&[Some("a".into()), None]));
    }

    #[test]
    fn split_vote_verdict_is_partial_with_dissent() {
        let mut fallback = BTreeMap::new();
        fallback.insert("a".into(), 1);
        let v = parse_verdict(
            "VERDICT: PARTIAL\nTALLY: a=2, b=1\nDISSENT: skeptic: risk too high",
            &fallback,
        );
        assert_eq!(v.status, "PARTIAL");
        assert_eq!(v.tally.get("b"), Some(&1));
        assert_eq!(v.dissent, vec!["skeptic: risk too high".to_string()]);
        let junk = parse_verdict("I think it's fine", &fallback);
        assert_eq!(junk.status, "PARTIAL");
        assert_eq!(junk.tally, fallback);
        assert_eq!(parse_verdict("VERDICT: PASS\nDISSENT: none", &fallback).status, "PASS");
    }

    #[test]
    fn seat_policy_rejects_edit_tools() {
        let id = "council-test-seat-policy";
        crate::tool::set_session_tool_policy(
            id,
            Some(SEAT_ALLOWED_TOOLS.iter().map(|s| s.to_string()).collect()),
            HashSet::new(),
        );
        assert_eq!(crate::tool::session_tool_policy_allows_tool_for_test(id, "edit"), Some(false));
        assert_eq!(crate::tool::session_tool_policy_allows_tool_for_test(id, "bash"), Some(false));
        assert_eq!(crate::tool::session_tool_policy_allows_tool_for_test(id, "read"), Some(true));
        crate::tool::clear_session_tool_policy_entry(id);
    }
}
