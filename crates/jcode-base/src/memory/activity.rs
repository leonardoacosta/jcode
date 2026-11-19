use super::*;
use crate::memory_types::PipelineState;
use std::sync::Mutex;
use std::time::Duration;

/// Global memory activity state - updated by sidecar, read by info widget
static MEMORY_ACTIVITY: Mutex<Option<MemoryActivity>> = Mutex::new(None);

/// Maximum number of recent events to keep
const MAX_RECENT_EVENTS: usize = 10;

/// Staleness timeout: auto-reset to Idle if state has been non-Idle for this long
const STALENESS_TIMEOUT_SECS: u64 = 10;

/// Get current memory activity state
pub fn get_activity() -> Option<MemoryActivity> {
    MEMORY_ACTIVITY.lock().ok().and_then(|guard| guard.clone())
}

pub fn activity_snapshot() -> Option<crate::protocol::MemoryActivitySnapshot> {
    get_activity().as_ref().map(memory_activity_snapshot)
}

pub fn apply_remote_activity_snapshot(snapshot: &crate::protocol::MemoryActivitySnapshot) {
    if let Ok(mut guard) = MEMORY_ACTIVITY.lock() {
        let recent_events = guard
            .as_ref()
            .map(|activity| activity.recent_events.clone())
            .unwrap_or_default();
        let now = Instant::now();
        let state_since = now
            .checked_sub(Duration::from_millis(snapshot.state_age_ms))
            .unwrap_or(now);

        *guard = Some(MemoryActivity {
            state: from_snapshot_state(&snapshot.state),
            state_since,
            pipeline: snapshot.pipeline.as_ref().map(from_snapshot_pipeline),
            recent_events,
        });
    }
}

/// Update the memory activity state
pub fn set_state(state: MemoryState) {
    if let Ok(mut guard) = MEMORY_ACTIVITY.lock() {
        if let Some(activity) = guard.as_mut() {
            activity.state = state;
            activity.state_since = Instant::now();
        } else {
            *guard = Some(MemoryActivity {
                state,
                state_since: Instant::now(),
                pipeline: None,
                recent_events: Vec::new(),
            });
        }
    }
}

/// Add an event to the activity log
pub fn add_event(kind: MemoryEventKind) {
    crate::memory_log::log_event(&kind);

    if let Ok(mut guard) = MEMORY_ACTIVITY.lock() {
        let event = MemoryEvent {
            kind,
            timestamp: Instant::now(),
            detail: None,
        };

        if let Some(activity) = guard.as_mut() {
            activity.recent_events.insert(0, event);
            activity.recent_events.truncate(MAX_RECENT_EVENTS);
        } else {
            *guard = Some(MemoryActivity {
                state: MemoryState::Idle,
                state_since: Instant::now(),
                pipeline: None,
                recent_events: vec![event],
            });
        }
    }
}

/// Start a new pipeline run (called at the beginning of each memory check)
pub fn pipeline_start() {
    if let Ok(mut guard) = MEMORY_ACTIVITY.lock() {
        if let Some(activity) = guard.as_mut() {
            activity.pipeline = Some(PipelineState::new());
        } else {
            *guard = Some(MemoryActivity {
                state: MemoryState::Idle,
                state_since: Instant::now(),
                pipeline: Some(PipelineState::new()),
                recent_events: Vec::new(),
            });
        }
    }
}

/// Update pipeline step status
#[expect(
    clippy::collapsible_if,
    reason = "Memory activity updates keep optional state transitions explicit"
)]
pub fn pipeline_update(f: impl FnOnce(&mut PipelineState)) {
    if let Ok(mut guard) = MEMORY_ACTIVITY.lock() {
        if let Some(activity) = guard.as_mut() {
            if let Some(pipeline) = activity.pipeline.as_mut() {
                f(pipeline);
            }
        }
    }
}

/// Check for staleness and auto-reset if needed.
/// Returns true if state was reset due to staleness.
#[expect(
    clippy::collapsible_if,
    reason = "Memory activity timeout checks keep nested optional state explicit"
)]
pub fn check_staleness() -> bool {
    if let Ok(mut guard) = MEMORY_ACTIVITY.lock() {
        if let Some(activity) = guard.as_mut() {
            if !matches!(activity.state, MemoryState::Idle)
                && activity.state_since.elapsed().as_secs() >= STALENESS_TIMEOUT_SECS
            {
                crate::logging::info(&format!(
                    "Memory state stale ({:?} for {}s), auto-resetting to Idle",
                    activity.state,
                    activity.state_since.elapsed().as_secs()
                ));
                activity.state = MemoryState::Idle;
                activity.state_since = Instant::now();
                return true;
            }
        }
    }
    false
}

/// Clear activity (reset to idle with no events)
pub fn clear_activity() {
    if let Ok(mut guard) = MEMORY_ACTIVITY.lock() {
        *guard = None;
    }
}

/// Record that a memory payload was injected into model context, from the
/// rendered text alone. Degraded path for publishers and remote servers that
/// carry no structured items: items are re-parsed from the text and only a
/// visible short id (never source/trust/updated_at) can be attached.
pub fn record_injected_prompt(prompt: &str, count: usize, age_ms: u64) {
    record_injected(prompt, count, age_ms, Vec::new(), 0);
}

/// Record an injection with its structured items and dropped count, exactly as
/// the formatter produced them. Empty `items` falls back to parsing `prompt`.
/// This feeds the memory info widget and the memory event log.
pub fn record_injected(
    prompt: &str,
    count: usize,
    age_ms: u64,
    items: Vec<InjectedMemoryItem>,
    dropped: usize,
) {
    crate::telemetry::record_memory_injected(count, age_ms);
    let preview = prompt_preview(prompt, 72);
    add_event(injected_event_kind(
        prompt,
        count,
        age_ms,
        preview.clone(),
        items,
        dropped,
    ));
    add_event(MemoryEventKind::MemorySurfaced {
        memory_preview: preview,
    });
}

fn injected_event_kind(
    prompt: &str,
    count: usize,
    age_ms: u64,
    preview: String,
    items: Vec<InjectedMemoryItem>,
    dropped: usize,
) -> MemoryEventKind {
    let (items, dropped) = if items.is_empty() {
        parse_injected_items(prompt, MAX_INJECTED_ITEMS)
    } else {
        (items, dropped)
    };
    MemoryEventKind::MemoryInjected {
        count,
        prompt_chars: prompt.chars().count(),
        age_ms,
        preview,
        items,
        dropped,
    }
}

/// Split `[mem <short id>] text` into the short id and the text.
fn split_mem_ref(content: &str) -> (Option<&str>, &str) {
    if let Some(rest) = content.strip_prefix("[mem ")
        && let Some((id, text)) = rest.split_once("] ")
        && !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':' | '.'))
    {
        return (Some(id), text.trim());
    }
    (None, content)
}

/// Parse a rendered prompt into items, plus how many items were left out: those
/// the formatter reported omitted and any beyond `max_items` here.
fn parse_injected_items(prompt: &str, max_items: usize) -> (Vec<InjectedMemoryItem>, usize) {
    let mut items: Vec<InjectedMemoryItem> = Vec::new();
    let mut section = String::from("Memory");
    let mut dropped = 0usize;
    let mut excess = 0usize;

    // The omission trailer is only trusted as the final line, so a multi-line
    // memory body cannot inflate the count by imitating it.
    let last_line = prompt.lines().map(str::trim).rfind(|l| !l.is_empty());
    for raw_line in prompt.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line == "# Memory" {
            continue;
        }
        if Some(line) == last_line
            && let Some(n) = line
                .strip_prefix('(')
                .and_then(|l| l.strip_suffix(" more memories omitted)"))
                .and_then(|n| n.parse::<usize>().ok())
        {
            dropped += n;
            continue;
        }
        if let Some(header) = line.strip_prefix("## ") {
            let header = header.trim();
            if !header.is_empty() {
                section = header.to_string();
            }
            continue;
        }

        let content = if let Some(rest) = line.strip_prefix("- ") {
            Some(rest.trim())
        } else if let Some((prefix, rest)) = line.split_once(". ") {
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                Some(rest.trim())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(content) = content {
            if content.is_empty() {
                continue;
            }
            if items.len() >= max_items {
                excess += 1;
                continue;
            }
            let (short_id, text) = split_mem_ref(content);
            items.push(InjectedMemoryItem {
                section: section.clone(),
                content: text.to_string(),
                id: short_id.map(str::to_string),
                source: None,
                trust: None,
                updated_at: None,
            });
        }
    }

    if items.is_empty() {
        let fallback = prompt
            .lines()
            .map(str::trim)
            .filter(|line| {
                !line.is_empty()
                    && !line.starts_with('#')
                    && !line.starts_with("## ")
                    && !line.starts_with("- ")
            })
            .collect::<Vec<_>>()
            .join(" ");
        if !fallback.is_empty() {
            items.push(InjectedMemoryItem {
                section,
                content: fallback,
                id: None,
                source: None,
                trust: None,
                updated_at: None,
            });
        }
    }

    (items, dropped + excess)
}

fn prompt_preview(prompt: &str, max_chars: usize) -> String {
    let bullet = prompt
        .lines()
        .map(str::trim)
        .find_map(|line| {
            if line.starts_with("- ") {
                Some(line.trim_start_matches("- ").trim())
            } else if let Some((prefix, rest)) = line.split_once(". ") {
                if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                    Some(rest.trim())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| prompt.trim());
    let bullet = split_mem_ref(bullet).1;

    if bullet.chars().count() <= max_chars {
        bullet.to_string()
    } else {
        let mut out = String::new();
        for (i, ch) in bullet.chars().enumerate() {
            if i >= max_chars.saturating_sub(3) {
                break;
            }
            out.push(ch);
        }
        out.push_str("...");
        out
    }
}

fn memory_activity_snapshot(activity: &MemoryActivity) -> crate::protocol::MemoryActivitySnapshot {
    crate::protocol::MemoryActivitySnapshot {
        state: snapshot_state(&activity.state),
        state_age_ms: activity.state_since.elapsed().as_millis() as u64,
        pipeline: activity.pipeline.as_ref().map(snapshot_pipeline),
    }
}

fn snapshot_state(state: &MemoryState) -> crate::protocol::MemoryStateSnapshot {
    match state {
        MemoryState::Idle => crate::protocol::MemoryStateSnapshot::Idle,
        MemoryState::Embedding => crate::protocol::MemoryStateSnapshot::Embedding,
        MemoryState::SidecarChecking { count } => {
            crate::protocol::MemoryStateSnapshot::SidecarChecking { count: *count }
        }
        MemoryState::FoundRelevant { count } => {
            crate::protocol::MemoryStateSnapshot::FoundRelevant { count: *count }
        }
        MemoryState::Extracting { reason } => crate::protocol::MemoryStateSnapshot::Extracting {
            reason: reason.clone(),
        },
        MemoryState::Maintaining { phase } => crate::protocol::MemoryStateSnapshot::Maintaining {
            phase: phase.clone(),
        },
        MemoryState::ToolAction { action, detail } => {
            crate::protocol::MemoryStateSnapshot::ToolAction {
                action: action.clone(),
                detail: detail.clone(),
            }
        }
    }
}

fn snapshot_pipeline(pipeline: &PipelineState) -> crate::protocol::MemoryPipelineSnapshot {
    crate::protocol::MemoryPipelineSnapshot {
        search: snapshot_step_status(&pipeline.search),
        search_result: pipeline.search_result.as_ref().map(snapshot_step_result),
        verify: snapshot_step_status(&pipeline.verify),
        verify_result: pipeline.verify_result.as_ref().map(snapshot_step_result),
        verify_progress: pipeline.verify_progress,
        inject: snapshot_step_status(&pipeline.inject),
        inject_result: pipeline.inject_result.as_ref().map(snapshot_step_result),
        maintain: snapshot_step_status(&pipeline.maintain),
        maintain_result: pipeline.maintain_result.as_ref().map(snapshot_step_result),
    }
}

fn snapshot_step_status(status: &StepStatus) -> crate::protocol::MemoryStepStatusSnapshot {
    match status {
        StepStatus::Pending => crate::protocol::MemoryStepStatusSnapshot::Pending,
        StepStatus::Running => crate::protocol::MemoryStepStatusSnapshot::Running,
        StepStatus::Done => crate::protocol::MemoryStepStatusSnapshot::Done,
        StepStatus::Error => crate::protocol::MemoryStepStatusSnapshot::Error,
        StepStatus::Skipped => crate::protocol::MemoryStepStatusSnapshot::Skipped,
    }
}

fn snapshot_step_result(result: &StepResult) -> crate::protocol::MemoryStepResultSnapshot {
    crate::protocol::MemoryStepResultSnapshot {
        summary: result.summary.clone(),
        latency_ms: result.latency_ms,
    }
}

fn from_snapshot_state(snapshot: &crate::protocol::MemoryStateSnapshot) -> MemoryState {
    match snapshot {
        crate::protocol::MemoryStateSnapshot::Idle => MemoryState::Idle,
        crate::protocol::MemoryStateSnapshot::Embedding => MemoryState::Embedding,
        crate::protocol::MemoryStateSnapshot::SidecarChecking { count } => {
            MemoryState::SidecarChecking { count: *count }
        }
        crate::protocol::MemoryStateSnapshot::FoundRelevant { count } => {
            MemoryState::FoundRelevant { count: *count }
        }
        crate::protocol::MemoryStateSnapshot::Extracting { reason } => MemoryState::Extracting {
            reason: reason.clone(),
        },
        crate::protocol::MemoryStateSnapshot::Maintaining { phase } => MemoryState::Maintaining {
            phase: phase.clone(),
        },
        crate::protocol::MemoryStateSnapshot::ToolAction { action, detail } => {
            MemoryState::ToolAction {
                action: action.clone(),
                detail: detail.clone(),
            }
        }
    }
}

fn from_snapshot_pipeline(snapshot: &crate::protocol::MemoryPipelineSnapshot) -> PipelineState {
    PipelineState {
        search: from_snapshot_step_status(&snapshot.search),
        search_result: snapshot
            .search_result
            .as_ref()
            .map(from_snapshot_step_result),
        verify: from_snapshot_step_status(&snapshot.verify),
        verify_result: snapshot
            .verify_result
            .as_ref()
            .map(from_snapshot_step_result),
        verify_progress: snapshot.verify_progress,
        inject: from_snapshot_step_status(&snapshot.inject),
        inject_result: snapshot
            .inject_result
            .as_ref()
            .map(from_snapshot_step_result),
        maintain: from_snapshot_step_status(&snapshot.maintain),
        maintain_result: snapshot
            .maintain_result
            .as_ref()
            .map(from_snapshot_step_result),
        started_at: Instant::now(),
    }
}

fn from_snapshot_step_status(snapshot: &crate::protocol::MemoryStepStatusSnapshot) -> StepStatus {
    match snapshot {
        crate::protocol::MemoryStepStatusSnapshot::Pending => StepStatus::Pending,
        crate::protocol::MemoryStepStatusSnapshot::Running => StepStatus::Running,
        crate::protocol::MemoryStepStatusSnapshot::Done => StepStatus::Done,
        crate::protocol::MemoryStepStatusSnapshot::Error => StepStatus::Error,
        crate::protocol::MemoryStepStatusSnapshot::Skipped => StepStatus::Skipped,
    }
}

fn from_snapshot_step_result(snapshot: &crate::protocol::MemoryStepResultSnapshot) -> StepResult {
    StepResult {
        summary: snapshot.summary.clone(),
        latency_ms: snapshot.latency_ms,
    }
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    use crate::memory_types::{
        MemoryCategory, MemoryEntry, TrustLevel, format_relevant_prompt_with_items,
    };

    fn injected(kind: MemoryEventKind) -> (Vec<InjectedMemoryItem>, usize) {
        match kind {
            MemoryEventKind::MemoryInjected { items, dropped, .. } => (items, dropped),
            other => panic!("unexpected event {other:?}"),
        }
    }

    #[test]
    fn structured_items_reach_the_event_with_full_provenance() {
        let mut entry = MemoryEntry::new(MemoryCategory::Fact, "prefers terse answers");
        entry.source = Some("user".into());
        entry.trust = TrustLevel::High;
        let f = format_relevant_prompt_with_items(std::slice::from_ref(&entry), 5).unwrap();
        let kind = injected_event_kind(&f.text, 1, 0, String::new(), f.items, f.dropped);
        let (items, dropped) = injected(kind);
        assert_eq!(dropped, 0);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content, "prefers terse answers");
        assert_eq!(items[0].id.as_deref(), Some(entry.id.as_str()));
        assert_eq!(items[0].source.as_deref(), Some("user"));
        assert_eq!(items[0].trust, Some(TrustLevel::High));
        assert_eq!(items[0].updated_at, Some(entry.updated_at));
        assert_eq!(prompt_preview(&f.text, 72), "prefers terse answers");
    }

    #[test]
    fn event_records_item_cap_and_char_cap_drops() {
        let entries: Vec<_> = (0..10)
            .map(|i| MemoryEntry::new(MemoryCategory::Fact, format!("fact number {i}")))
            .collect();
        let f = format_relevant_prompt_with_items(&entries, 100).unwrap();
        let (items, dropped) = injected(injected_event_kind(
            &f.text,
            8,
            0,
            String::new(),
            f.items,
            f.dropped,
        ));
        assert_eq!((items.len(), dropped), (MAX_INJECTED_ITEMS, 2));

        let big = "x".repeat(crate::memory_types::MAX_INJECTED_CHARS - 10);
        let entries = [
            MemoryEntry::new(MemoryCategory::Fact, big),
            MemoryEntry::new(MemoryCategory::Fact, "over the char budget"),
        ];
        let f = format_relevant_prompt_with_items(&entries, 5).unwrap();
        let (items, dropped) = injected(injected_event_kind(
            &f.text,
            1,
            0,
            String::new(),
            f.items,
            f.dropped,
        ));
        assert_eq!((items.len(), dropped), (1, 1));
    }

    #[test]
    fn memory_text_imitating_the_omission_trailer_does_not_inflate_dropped() {
        let entries = [
            MemoryEntry::new(MemoryCategory::Fact, "(500 more memories omitted)"),
            MemoryEntry::new(MemoryCategory::Fact, "ordinary"),
        ];
        let f = format_relevant_prompt_with_items(&entries, 5).unwrap();
        // Structured path: exact.
        let (_, dropped) = injected(injected_event_kind(
            &f.text,
            2,
            0,
            String::new(),
            f.items.clone(),
            f.dropped,
        ));
        assert_eq!(dropped, 0);
        // Text-only fallback: the imitation is an item line (`N. [mem ..] (..)`),
        // never the final trailer line, so it still counts as nothing dropped.
        let (_, dropped) = injected(injected_event_kind(
            &f.text,
            2,
            0,
            String::new(),
            Vec::new(),
            0,
        ));
        assert_eq!(dropped, 0);
    }

    #[test]
    fn text_only_fallback_keeps_short_id_without_inventing_provenance() {
        let entries: Vec<_> = (0..10)
            .map(|i| MemoryEntry::new(MemoryCategory::Fact, format!("fact number {i}")))
            .collect();
        let f = format_relevant_prompt_with_items(&entries, 100).unwrap();
        let (items, dropped) = parse_injected_items(&f.text, MAX_INJECTED_ITEMS);
        assert_eq!((items.len(), dropped), (MAX_INJECTED_ITEMS, 2));
        assert!(items.iter().all(|i| i.id.is_some()
            && i.source.is_none()
            && i.trust.is_none()
            && i.updated_at.is_none()));
        let (items, dropped) = parse_injected_items("# Memory\n\n## Facts\n1. old style", 8);
        assert_eq!(dropped, 0);
        assert_eq!(items[0].content, "old style");
        assert!(items[0].id.is_none());
    }
}
