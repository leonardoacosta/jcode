use super::{Tool, ToolContext, ToolOutput};
use crate::ambient::{
    AmbientCycleResult, AmbientManager, AmbientState, CycleStatus, ItemState, Priority,
    ScheduleRequest, ScheduleTarget, ScheduledItem,
};
use crate::ambient_runner::AmbientRunnerHandle;
use crate::safety::{self, PermissionRequest, PermissionResult, SafetySystem, Urgency};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};

// ---------------------------------------------------------------------------
// Global state for ambient tools
// ---------------------------------------------------------------------------

/// Global ambient cycle result, set by EndAmbientCycleTool for the ambient
/// runner to collect after the cycle completes.
static AMBIENT_CYCLE_RESULT: OnceLock<Mutex<Option<AmbientCycleResult>>> = OnceLock::new();

fn cycle_result_slot() -> &'static Mutex<Option<AmbientCycleResult>> {
    AMBIENT_CYCLE_RESULT.get_or_init(|| Mutex::new(None))
}

/// Store a cycle result for the ambient runner to pick up.
pub fn store_cycle_result(result: AmbientCycleResult) {
    if let Ok(mut slot) = cycle_result_slot().lock() {
        *slot = Some(result);
    }
}

/// Take the stored cycle result (returns None if not set or already taken).
pub fn take_cycle_result() -> Option<AmbientCycleResult> {
    cycle_result_slot()
        .lock()
        .ok()
        .and_then(|mut slot| slot.take())
}

/// Global SafetySystem instance shared with ambient tools.
static SAFETY_SYSTEM: OnceLock<Arc<SafetySystem>> = OnceLock::new();
/// Shared schedule/ambient runner handle used to wake the background loop after
/// queue changes.
static SCHEDULE_RUNNER: OnceLock<Mutex<Option<AmbientRunnerHandle>>> = OnceLock::new();
/// Session IDs currently allowed to use ambient-only permission workflows.
static AMBIENT_SESSION_IDS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

pub fn init_safety_system(system: Arc<SafetySystem>) {
    let _ = SAFETY_SYSTEM.set(system);
}

pub fn init_schedule_runner(handle: AmbientRunnerHandle) {
    if let Ok(mut slot) = SCHEDULE_RUNNER.get_or_init(|| Mutex::new(None)).lock() {
        *slot = Some(handle);
    }
}

fn get_safety_system() -> Arc<SafetySystem> {
    SAFETY_SYSTEM
        .get()
        .cloned()
        .unwrap_or_else(|| Arc::new(SafetySystem::new()))
}

fn ambient_session_ids() -> &'static Mutex<HashSet<String>> {
    AMBIENT_SESSION_IDS.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Mark a session ID as ambient-enabled for ambient-only tooling.
pub fn register_ambient_session(session_id: impl Into<String>) {
    if let Ok(mut ids) = ambient_session_ids().lock() {
        ids.insert(session_id.into());
    }
}

/// Remove a session ID from the ambient-enabled set.
pub fn unregister_ambient_session(session_id: &str) {
    if let Ok(mut ids) = ambient_session_ids().lock() {
        ids.remove(session_id);
    }
}

fn is_ambient_session_registered(session_id: &str) -> bool {
    ambient_session_ids()
        .lock()
        .map(|ids| ids.contains(session_id))
        .unwrap_or(false)
}

fn ensure_ambient_session(ctx: &ToolContext) -> Result<()> {
    if is_ambient_session_registered(&ctx.session_id) {
        Ok(())
    } else {
        anyhow::bail!(
            "request_permission is only available to ambient sessions (session '{}')",
            ctx.session_id
        )
    }
}

// ===========================================================================
// EndAmbientCycleTool
// ===========================================================================

pub struct EndAmbientCycleTool;

impl Default for EndAmbientCycleTool {
    fn default() -> Self {
        Self::new()
    }
}

impl EndAmbientCycleTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct EndCycleInput {
    summary: String,
    #[serde(deserialize_with = "super::serde_coerce::u32_from_string_or_number")]
    memories_modified: u32,
    #[serde(deserialize_with = "super::serde_coerce::u32_from_string_or_number")]
    compactions: u32,
    #[serde(default)]
    proactive_work: Option<String>,
    #[serde(default)]
    next_schedule: Option<NextScheduleInput>,
}

#[derive(Deserialize)]
struct NextScheduleInput {
    #[serde(
        default,
        deserialize_with = "super::serde_coerce::opt_u32_from_string_or_number"
    )]
    wake_in_minutes: Option<u32>,
    #[serde(default)]
    context: Option<String>,
    #[serde(default)]
    priority: Option<String>,
}

#[async_trait]
impl Tool for EndAmbientCycleTool {
    fn name(&self) -> &str {
        "end_ambient_cycle"
    }

    fn description(&self) -> &str {
        "End the current ambient cycle."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["summary", "memories_modified", "compactions"],
            "properties": {
                "intent": super::intent_schema_property(),
                "summary": {
                    "type": "string",
                    "description": "Human-readable summary of what was done this cycle"
                },
                "memories_modified": {
                    "type": "integer",
                    "description": "Count of memories created, merged, pruned, or updated"
                },
                "compactions": {
                    "type": "integer",
                    "description": "Number of context compactions during this cycle"
                },
                "proactive_work": {
                    "type": "string",
                    "description": "Description of proactive code changes, if any"
                },
                "next_schedule": {
                    "type": "object",
                    "description": "When to wake next and what to do",
                    "properties": {
                        "wake_in_minutes": {
                            "type": "integer",
                            "description": "Minutes until next wake"
                        },
                        "context": {
                            "type": "string",
                            "description": "What to do next cycle"
                        },
                        "priority": {
                            "type": "string",
                            "enum": ["low", "normal", "high"],
                            "description": "Priority for next cycle"
                        }
                    }
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: EndCycleInput = serde_json::from_value(input)?;

        let next_schedule = params.next_schedule.map(|ns| ScheduleRequest {
            wake_in_minutes: ns.wake_in_minutes,
            wake_at: None,
            context: ns.context.unwrap_or_default(),
            priority: parse_priority(ns.priority.as_deref()),
            target: ScheduleTarget::Ambient,
            created_by_session: ctx.session_id.clone(),
            working_dir: None,
            task_description: None,
            relevant_files: Vec::new(),
            git_branch: None,
            additional_context: None,
        });

        let now = Utc::now();
        let result = AmbientCycleResult {
            summary: params.summary.clone(),
            memories_modified: params.memories_modified,
            compactions: params.compactions,
            proactive_work: params.proactive_work,
            next_schedule: next_schedule.clone(),
            started_at: now, // approximate; the runner will override if it tracks start time
            ended_at: now,
            status: CycleStatus::Complete,
            conversation: None, // populated by the runner after cycle completes
        };

        // Store for the ambient runner to pick up
        store_cycle_result(result);

        // Also persist state immediately so a crash after this tool but before
        // the runner collects won't lose the cycle.
        if let Ok(mut state) = AmbientState::load() {
            let next_desc = if let Some(ref sched) = next_schedule {
                let mins = sched.wake_in_minutes.unwrap_or(30);
                format!("~{}", crate::ambient::format_minutes_human(mins))
            } else {
                "system default".to_string()
            };

            state.last_run = Some(now);
            state.last_summary = Some(params.summary.clone());
            state.last_compactions = Some(params.compactions);
            state.last_memories_modified = Some(params.memories_modified);
            state.total_cycles += 1;
            let _ = state.save();

            Ok(ToolOutput::new(format!(
                "Ambient cycle ended. Memories modified: {}, compactions: {}. Next wake: {}",
                params.memories_modified, params.compactions, next_desc
            ))
            .with_title("ambient cycle ended".to_string()))
        } else {
            Ok(ToolOutput::new(format!(
                "Ambient cycle ended (state save failed). Summary: {}",
                params.summary
            ))
            .with_title("ambient cycle ended".to_string()))
        }
    }
}

// ===========================================================================
// ScheduleAmbientTool
// ===========================================================================

pub struct ScheduleAmbientTool;

impl Default for ScheduleAmbientTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleAmbientTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct ScheduleInput {
    #[serde(
        default,
        deserialize_with = "super::serde_coerce::opt_u32_from_string_or_number"
    )]
    wake_in_minutes: Option<u32>,
    #[serde(default)]
    wake_at: Option<String>,
    context: String,
    #[serde(default)]
    priority: Option<String>,
}

#[async_trait]
impl Tool for ScheduleAmbientTool {
    fn name(&self) -> &str {
        "schedule_ambient"
    }

    fn description(&self) -> &str {
        "Schedule an ambient task."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["context"],
            "properties": {
                "intent": super::intent_schema_property(),
                "wake_in_minutes": {
                    "type": "integer",
                    "description": "Minutes from now to wake"
                },
                "wake_at": {
                    "type": "string",
                    "description": "ISO 8601 timestamp for when to wake (alternative to wake_in_minutes)"
                },
                "context": {
                    "type": "string",
                    "description": "What to do when waking — stored in the scheduled queue"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "normal", "high"],
                    "description": "Priority for this scheduled task (default: normal)"
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: ScheduleInput = serde_json::from_value(input)?;

        let wake_at = if let Some(ref ts) = params.wake_at {
            Some(
                ts.parse::<chrono::DateTime<Utc>>()
                    .map_err(|e| anyhow::anyhow!("Invalid wake_at timestamp: {}", e))?,
            )
        } else {
            None
        };

        let request = ScheduleRequest {
            wake_in_minutes: params.wake_in_minutes,
            wake_at,
            context: params.context.clone(),
            priority: parse_priority(params.priority.as_deref()),
            target: ScheduleTarget::Ambient,
            created_by_session: ctx.session_id,
            working_dir: None,
            task_description: None,
            relevant_files: Vec::new(),
            git_branch: None,
            additional_context: None,
        };

        let mut manager = AmbientManager::new()?;
        let id = manager.schedule(request)?;
        nudge_schedule_runner();

        let when = if let Some(ref ts) = params.wake_at {
            ts.clone()
        } else if let Some(mins) = params.wake_in_minutes {
            format!("in {}", crate::ambient::format_minutes_human(mins))
        } else {
            "in 30m (default)".to_string()
        };

        Ok(
            ToolOutput::new(format!("Scheduled ambient task {} for {}", id, when))
                .with_title(format!("scheduled: {}", params.context)),
        )
    }
}

// ===========================================================================
// RequestPermissionTool
// ===========================================================================

pub struct RequestPermissionTool;

impl Default for RequestPermissionTool {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestPermissionTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct RequestPermissionInput {
    action: String,
    description: String,
    rationale: String,
    #[serde(default)]
    urgency: Option<String>,
    #[serde(
        default = "default_false",
        deserialize_with = "super::serde_coerce::bool_from_string_or_bool"
    )]
    wait: bool,
    #[serde(default)]
    context: Option<Value>,
}

fn default_false() -> bool {
    false
}

fn extract_context_string(map: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        map.get(*key).and_then(|value| {
            value.as_str().and_then(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            })
        })
    })
}

fn extract_context_list(map: &Map<String, Value>, keys: &[&str]) -> Vec<String> {
    for key in keys {
        let Some(value) = map.get(*key) else {
            continue;
        };

        if let Some(items) = value.as_array() {
            let list: Vec<String> = items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToString::to_string)
                .collect();
            if !list.is_empty() {
                return list;
            }
        } else if let Some(single) = value.as_str() {
            let trimmed = single.trim();
            if !trimmed.is_empty() {
                return vec![trimmed.to_string()];
            }
        }
    }
    Vec::new()
}

fn build_permission_review_context(
    action: &str,
    description: &str,
    rationale: &str,
    context: Option<&Value>,
) -> Value {
    let context_obj = context.and_then(Value::as_object);

    let summary = context_obj
        .and_then(|m| extract_context_string(m, &["summary", "what", "activity_summary"]))
        .unwrap_or_else(|| description.to_string());

    let why_permission_needed = context_obj
        .and_then(|m| {
            extract_context_string(
                m,
                &[
                    "why_permission_needed",
                    "why",
                    "reason",
                    "rationale",
                    "justification",
                ],
            )
        })
        .unwrap_or_else(|| rationale.to_string());

    let mut review = Map::new();
    review.insert("summary".to_string(), Value::String(summary));
    review.insert(
        "why_permission_needed".to_string(),
        Value::String(why_permission_needed),
    );
    review.insert(
        "requested_action".to_string(),
        Value::String(action.to_string()),
    );

    let string_fields: [(&str, &[&str]); 4] = [
        (
            "current_activity",
            &["current_activity", "activity", "task", "current_task"],
        ),
        (
            "expected_outcome",
            &["expected_outcome", "outcome", "success_criteria", "success"],
        ),
        ("impact", &["impact", "user_impact"]),
        ("rollback_plan", &["rollback_plan", "rollback"]),
    ];

    if let Some(map) = context_obj {
        for (field_name, keys) in string_fields {
            if let Some(value) = extract_context_string(map, keys) {
                review.insert(field_name.to_string(), Value::String(value));
            }
        }

        let list_fields: [(&str, &[&str]); 4] = [
            (
                "planned_steps",
                &["planned_steps", "steps", "plan", "checklist"],
            ),
            ("files", &["files", "file_paths", "planned_files"]),
            ("commands", &["commands", "planned_commands"]),
            ("risks", &["risks", "risk", "safety_risks"]),
        ];

        for (field_name, keys) in list_fields {
            let items = extract_context_list(map, keys);
            if !items.is_empty() {
                review.insert(
                    field_name.to_string(),
                    Value::Array(items.into_iter().map(Value::String).collect()),
                );
            }
        }
    }

    if let Some(raw) = context
        && !raw.is_object()
    {
        review.insert("notes".to_string(), raw.clone());
    }

    Value::Object(review)
}

#[async_trait]
impl Tool for RequestPermissionTool {
    fn name(&self) -> &str {
        "request_permission"
    }

    fn description(&self) -> &str {
        "Request user permission."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["action", "description", "rationale"],
            "properties": {
                "intent": super::intent_schema_property(),
                "action": {
                    "type": "string",
                    "description": "The action requiring permission (e.g., 'create_pull_request', 'push', 'edit')"
                },
                "description": {
                    "type": "string",
                    "description": "What the action will do"
                },
                "rationale": {
                    "type": "string",
                    "description": "Why this action is beneficial"
                },
                "urgency": {
                    "type": "string",
                    "enum": ["low", "normal", "high"],
                    "description": "How urgent the permission request is (default: normal)"
                },
                "wait": {
                    "type": "boolean",
                    "description": "If true, block until user decides (with timeout). If false, queue and continue."
                },
                "context": {
                    "type": "object",
                    "description": "Structured reviewer context. Include summary of current work and why permission is needed.",
                    "properties": {
                        "summary": {
                            "type": "string",
                            "description": "One-paragraph summary of what you are currently doing"
                        },
                        "why_permission_needed": {
                            "type": "string",
                            "description": "Why this action needs user approval right now"
                        },
                        "current_activity": {
                            "type": "string",
                            "description": "Current task or ambient objective"
                        },
                        "planned_steps": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Short ordered plan of intended steps"
                        },
                        "files": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Files expected to be created/modified"
                        },
                        "commands": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Commands expected to be executed"
                        },
                        "risks": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Known risks or side effects"
                        },
                        "rollback_plan": {
                            "type": "string",
                            "description": "How to back out changes if needed"
                        },
                        "expected_outcome": {
                            "type": "string",
                            "description": "What successful completion should look like"
                        }
                    },
                    "additionalProperties": true
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        ensure_ambient_session(&ctx)?;

        let params: RequestPermissionInput = serde_json::from_value(input)?;

        let urgency = match params.urgency.as_deref() {
            Some("low") => Urgency::Low,
            Some("high") => Urgency::High,
            _ => Urgency::Normal,
        };

        let request_id = safety::new_request_id();
        let now = Utc::now();
        let review = build_permission_review_context(
            &params.action,
            &params.description,
            &params.rationale,
            params.context.as_ref(),
        );
        let mut request_context = json!({
            "session_id": ctx.session_id,
            "message_id": ctx.message_id,
            "tool_call_id": ctx.tool_call_id,
            "working_dir": ctx.working_dir.as_ref().map(|p| p.display().to_string()),
            "requested_at": now.to_rfc3339(),
        });
        if let Some(obj) = request_context.as_object_mut() {
            obj.insert("review".to_string(), review);
            if let Some(user_context) = params.context {
                obj.insert("details".to_string(), user_context);
            }
        }

        let request = PermissionRequest {
            id: request_id.clone(),
            action: params.action.clone(),
            description: params.description.clone(),
            rationale: params.rationale.clone(),
            urgency,
            wait: params.wait,
            created_at: now,
            context: Some(request_context),
        };

        let system = get_safety_system();
        let result = system.request_permission(request);

        let output = match result {
            PermissionResult::Approved { ref message } => {
                let msg = message.as_deref().unwrap_or("no message");
                format!("Permission approved: {}", msg)
            }
            PermissionResult::Denied { ref reason } => {
                let reason = reason.as_deref().unwrap_or("no reason given");
                format!("Permission denied: {}", reason)
            }
            PermissionResult::Queued { ref request_id } => {
                format!(
                    "Permission request queued (id: {}). \
                     Action '{}' is pending user review.",
                    request_id, params.action
                )
            }
            PermissionResult::Timeout => {
                "Permission request timed out. The user did not respond in time.".to_string()
            }
        };

        Ok(ToolOutput::new(output).with_title(format!("permission: {}", params.action)))
    }
}

// ===========================================================================
// ScheduleTool — available to normal sessions to queue future ambient tasks
// ===========================================================================

pub struct ScheduleTool;

impl Default for ScheduleTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct ScheduleToolInput {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    schedule_id: Option<String>,
    #[serde(default)]
    task: Option<String>,
    #[serde(
        default,
        deserialize_with = "super::serde_coerce::opt_u32_from_string_or_number"
    )]
    wake_in_minutes: Option<u32>,
    #[serde(default)]
    wake_at: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    relevant_files: Vec<String>,
    #[serde(default)]
    background_context: Option<String>,
    #[serde(default)]
    success_criteria: Option<String>,
    #[serde(default)]
    target: Option<String>,
    // Recurring (cron) schedules.
    #[serde(default)]
    cron: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    missed_policy: Option<String>,
    #[serde(default)]
    catch_up_limit: Option<u32>,
    #[serde(default)]
    max_active: Option<u32>,
    #[serde(default)]
    token_cap: Option<u64>,
    #[serde(default)]
    spend_cap_usd: Option<f64>,
    #[serde(default)]
    count: Option<usize>,
}

#[async_trait]
impl Tool for ScheduleTool {
    fn name(&self) -> &str {
        "schedule"
    }

    fn description(&self) -> &str {
        "Schedule independent future tasks."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property(),
                "action": {
                    "type": "string",
                    "enum": ["create", "list", "cancel", "retry", "pause", "resume", "preview"],
                    "description": "Defaults to create. retry re-queues an uncertain or missed task; it may already have run. pause/resume/preview act on a recurring schedule."
                },
                "schedule_id": {
                    "type": "string",
                    "description": "Scheduled task ID. Required for cancel, retry, pause and resume; preview takes it or cron+timezone. A recurring id starts with cron_."
                },
                "cron": {
                    "type": "string",
                    "description": "Recurring: five-field cron (minute hour day-of-month month day-of-week; *, N, A-B, A,B, */S). With create it needs timezone and replaces wake_in_minutes/wake_at."
                },
                "timezone": { "type": "string", "description": "IANA zone for cron, e.g. America/New_York. Required with cron." },
                "name": { "type": "string" },
                "missed_policy": {
                    "type": "string",
                    "enum": ["skip", "run-once", "catch-up"],
                    "description": "Required with cron. What to do with occurrences missed during downtime or a pause."
                },
                "catch_up_limit": { "type": "integer", "description": "Positive. Required for missed_policy=catch-up." },
                "max_active": { "type": "integer", "description": "Per-schedule active run limit (default 1)." },
                "token_cap": { "type": "integer", "description": "Token cap per schedule (input+output+cache-creation), enforced from provider usage; reaching it cancels the run. One response may overshoot." },
                "spend_cap_usd": { "type": "number", "description": "Spend cap. Not enforceable today: the schedule reports budget_unavailable and does not start." },
                "count": { "type": "integer", "description": "Occurrences to preview (default 5, max 50)." },
                "task": {
                    "type": "string",
                    "description": "Task. Required for action=create."
                },
                "wake_in_minutes": { "type": "integer" },
                "wake_at": { "type": "string" },
                "priority": {
                    "type": "string",
                    "enum": ["low", "normal", "high"]
                },
                "relevant_files": {
                    "type": "array",
                    "items": { "type": "string" }
                },
                "background_context": {
                    "type": "string",
                    "description": "Optional background context for the scheduled task."
                },
                "success_criteria": { "type": "string" },
                "target": {
                    "type": "string",
                    "enum": ["resume", "spawn", "ambient"],
                    "description": "Delivery target. Defaults to resuming this session; 'spawn' runs one new child session."
                }
            }
        })
    }

    async fn execute(&self, input: Value, ctx: ToolContext) -> Result<ToolOutput> {
        let params: ScheduleToolInput = serde_json::from_value(input)?;

        match params.action.as_deref().unwrap_or("create") {
            "create" => self.execute_create(params, ctx).await,
            "list" => self.execute_list().await,
            "cancel" => self.execute_cancel(params).await,
            "retry" => self.execute_retry(params).await,
            "pause" => self.execute_pause_resume(params, true).await,
            "resume" => self.execute_pause_resume(params, false).await,
            "preview" => self.execute_preview(params).await,
            other => anyhow::bail!(
                "Invalid action '{}'. Expected one of: create, list, cancel, retry, pause, resume, preview",
                other
            ),
        }
    }
}

impl ScheduleTool {
    async fn execute_create(
        &self,
        params: ScheduleToolInput,
        ctx: ToolContext,
    ) -> Result<ToolOutput> {
        let task = params
            .task
            .clone()
            .ok_or_else(|| anyhow::anyhow!("task is required for action=create"))?;

        if params.cron.is_some() {
            if params.wake_in_minutes.is_some() || params.wake_at.is_some() {
                anyhow::bail!("cron cannot be combined with wake_in_minutes or wake_at");
            }
        } else if params.timezone.is_some()
            || params.name.is_some()
            || params.missed_policy.is_some()
            || params.catch_up_limit.is_some()
            || params.max_active.is_some()
            || params.token_cap.is_some()
            || params.spend_cap_usd.is_some()
        {
            anyhow::bail!(
                "timezone, name, missed_policy, catch_up_limit, max_active, token_cap and spend_cap_usd apply only to a recurring schedule (set cron)"
            );
        } else if params.wake_in_minutes.is_none() && params.wake_at.is_none() {
            anyhow::bail!(
                "Either wake_in_minutes or wake_at is required. \
                 This tool is for scheduling future tasks."
            );
        }

        let wake_at = if let Some(ref ts) = params.wake_at {
            Some(
                ts.parse::<chrono::DateTime<Utc>>()
                    .map_err(|e| anyhow::anyhow!("Invalid wake_at timestamp: {}", e))?,
            )
        } else {
            None
        };

        let working_dir = ctx.working_dir.as_ref().map(|p| p.display().to_string());

        let git_branch = ctx
            .working_dir
            .as_ref()
            .and_then(|wd| {
                std::process::Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .current_dir(wd)
                    .output()
                    .ok()
            })
            .and_then(|out| {
                if out.status.success() {
                    String::from_utf8(out.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            });

        let target = parse_schedule_target(params.target.as_deref(), &ctx.session_id)?;
        let target_summary = format_schedule_target(&target);

        let request = ScheduleRequest {
            wake_in_minutes: params.wake_in_minutes,
            wake_at,
            context: task.clone(),
            priority: parse_priority(params.priority.as_deref()),
            target,
            created_by_session: ctx.session_id.clone(),
            working_dir: working_dir.clone(),
            task_description: Some(task.clone()),
            relevant_files: params.relevant_files.clone(),
            git_branch,
            additional_context: {
                let mut parts = Vec::new();
                if let Some(ref bg) = params.background_context {
                    parts.push(format!("Background: {}", bg));
                }
                if let Some(ref sc) = params.success_criteria {
                    parts.push(format!("Success criteria: {}", sc));
                }
                parts.push(format!("Scheduled by session: {}", ctx.session_id));
                Some(parts.join("\n"))
            },
        };

        if params.cron.is_some() {
            return self.create_recurring(&params, request, &task, &target_summary);
        }

        let mut manager = AmbientManager::new()?;
        let id = manager.schedule(request)?;
        nudge_schedule_runner();

        let when = if let Some(ref ts) = params.wake_at {
            ts.clone()
        } else if let Some(mins) = params.wake_in_minutes {
            format!("in {}", crate::ambient::format_minutes_human(mins))
        } else {
            "unspecified".to_string()
        };

        let mut summary = format!("Scheduled task '{}' for {} (id: {})", task, when, id);
        if let Some(ref wd) = working_dir {
            summary.push_str(&format!("\nWorking directory: {}", wd));
        }
        if !params.relevant_files.is_empty() {
            summary.push_str(&format!(
                "\nRelevant files: {}",
                params.relevant_files.join(", ")
            ));
        }
        summary.push_str(&format!("\nTarget: {}", target_summary));

        Ok(ToolOutput::new(summary).with_title(format!("scheduled: {}", task)))
    }

    fn create_recurring(
        &self,
        params: &ScheduleToolInput,
        template: ScheduleRequest,
        task: &str,
        target_summary: &str,
    ) -> Result<ToolOutput> {
        use crate::ambient::recurring::{MissedPolicy, NewSchedule, RecurringStore};
        let missed = match params.missed_policy.as_deref() {
            Some("skip") => MissedPolicy::Skip,
            Some("run-once") => MissedPolicy::RunOnce,
            Some("catch-up") => MissedPolicy::CatchUp {
                limit: params
                    .catch_up_limit
                    .ok_or_else(|| anyhow::anyhow!("catch_up_limit is required for catch-up"))?,
            },
            Some(other) => anyhow::bail!(
                "Invalid missed_policy '{other}'. Expected one of: skip, run-once, catch-up"
            ),
            None => anyhow::bail!("missed_policy is required for a recurring schedule"),
        };
        if params.catch_up_limit.is_some() && !matches!(missed, MissedPolicy::CatchUp { .. }) {
            anyhow::bail!("catch_up_limit only applies to missed_policy=catch-up");
        }
        let zone = params
            .timezone
            .clone()
            .ok_or_else(|| anyhow::anyhow!("timezone (an IANA zone) is required with cron"))?;
        let store = RecurringStore::open_default()?;
        let schedule = store.create(
            NewSchedule {
                name: params.name.clone(),
                cron: params.cron.clone().unwrap_or_default(),
                zone,
                missed,
                max_active: params.max_active.unwrap_or(1),
                token_cap: params.token_cap,
                spend_cap_usd: params.spend_cap_usd,
                template,
            },
            Utc::now(),
        )?;
        nudge_schedule_runner();
        let next = format_next_occurrences(&schedule, 1);
        Ok(ToolOutput::new(format!(
            "Scheduled recurring task '{task}' (id: {})\n{}\nTarget: {target_summary}{next}",
            schedule.id,
            describe_recurring(&schedule)
        ))
        .with_title(format!("recurring: {task}")))
    }

    async fn execute_pause_resume(
        &self,
        params: ScheduleToolInput,
        paused: bool,
    ) -> Result<ToolOutput> {
        let verb = if paused { "pause" } else { "resume" };
        let id = params
            .schedule_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("schedule_id is required for action={verb}"))?;
        let store = crate::ambient::recurring::RecurringStore::open_default()?;
        let schedule = store.set_paused(id, paused)?;
        nudge_schedule_runner();
        let note = if paused {
            "New occurrences are not admitted. Runs already delivered are not stopped."
        } else {
            "Occurrences missed while paused follow the missed-run policy on the next pass."
        };
        Ok(ToolOutput::new(format!(
            "{} recurring schedule {id}. {note}\n{}",
            if paused { "Paused" } else { "Resumed" },
            describe_recurring(&schedule)
        ))
        .with_title(format!("{verb}: {id}")))
    }

    /// Read-only: never touches the store or the queue.
    async fn execute_preview(&self, params: ScheduleToolInput) -> Result<ToolOutput> {
        use crate::ambient::cron::{CronExpr, next_occurrences, parse_zone};
        let count = params
            .count
            .unwrap_or(5)
            .clamp(1, crate::ambient::recurring::MAX_PREVIEW);
        let text = if let Some(id) = params.schedule_id.as_deref() {
            let store = crate::ambient::recurring::RecurringStore::open_default()?;
            let schedule = store
                .get(id)?
                .ok_or_else(|| anyhow::anyhow!("no recurring schedule '{id}'"))?;
            format!(
                "{}{}",
                describe_recurring(&schedule),
                format_next_occurrences(&schedule, count)
            )
        } else {
            let expr = CronExpr::parse(params.cron.as_deref().ok_or_else(|| {
                anyhow::anyhow!("preview needs schedule_id, or cron and timezone")
            })?)?;
            let zone = parse_zone(
                params
                    .timezone
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("timezone is required with cron"))?,
            )?;
            let mut out = format!("Preview of '{}' in {}:", expr.as_str(), zone.name());
            for occ in next_occurrences(&expr, zone, Utc::now(), None, count) {
                out.push_str(&format!(
                    "\n- {} local | {} UTC",
                    occ.local.format("%Y-%m-%d %H:%M"),
                    occ.at.format("%Y-%m-%d %H:%M")
                ));
            }
            out
        };
        Ok(ToolOutput::new(text).with_title("schedule preview"))
    }

    async fn execute_list(&self) -> Result<ToolOutput> {
        let recurring = crate::ambient::recurring::RecurringStore::open_default()
            .and_then(|store| store.list())?;
        let manager = AmbientManager::new()?;
        let manager_queue_state = |key: &str| -> String {
            let queue = manager.queue();
            queue
                .items()
                .iter()
                .chain(queue.in_flight())
                .chain(queue.recent_history(usize::MAX))
                .find(|item| item.id == key)
                .and_then(|item| serde_json::to_value(item.delivery.state).ok())
                .and_then(|v| v.as_str().map(str::to_string))
                .unwrap_or_else(|| "not in queue".to_string())
        };
        let queue = manager.queue();
        let mut items: Vec<&ScheduledItem> =
            queue.items().iter().chain(queue.in_flight()).collect();
        items.sort_by_key(|item| item.scheduled_for);
        let history = queue.recent_history(SCHEDULE_HISTORY_LIMIT);

        if items.is_empty() && history.is_empty() && recurring.is_empty() {
            return Ok(ToolOutput::new("No scheduled tasks."));
        }

        let mut summary = String::new();
        if !recurring.is_empty() {
            summary.push_str(&format!("{} recurring schedule(s):", recurring.len()));
            for schedule in &recurring {
                summary.push_str(&format!(
                    "\n- {}{}",
                    schedule.id,
                    format_next_occurrences(schedule, 1)
                ));
                summary.push_str(&format!("\n  {}", describe_recurring(schedule)));
                let recent: Vec<String> = schedule
                    .occurrences
                    .iter()
                    .rev()
                    .take(5)
                    .map(|o| {
                        let mut state = serde_json::to_value(o.state)
                            .ok()
                            .and_then(|v| v.as_str().map(str::to_string))
                            .unwrap_or_default();
                        // An enqueued occurrence's outcome is its queue
                        // delivery state, which is acceptance, not task success.
                        if o.state == crate::ambient::recurring::OccurrenceState::Enqueued {
                            let queue = manager_queue_state(&o.key);
                            state = format!("enqueued ({queue})");
                        }
                        format!("{} {state}", o.local.format("%m-%d %H:%M"))
                    })
                    .collect();
                if !recent.is_empty() {
                    summary.push_str(&format!("\n  recent: {}", recent.join(", ")));
                }
                let runs: Vec<String> = schedule
                    .runs
                    .iter()
                    .rev()
                    .take(5)
                    .map(|r| {
                        let state = serde_json::to_value(r.state)
                            .ok()
                            .and_then(|v| v.as_str().map(str::to_string))
                            .unwrap_or_default();
                        format!("{} {state}", r.started_at.format("%m-%d %H:%M"))
                    })
                    .collect();
                if !runs.is_empty() {
                    summary.push_str(&format!("\n  runs: {}", runs.join(", ")));
                }
            }
            summary.push('\n');
        }
        summary.push_str(&if items.is_empty() {
            "No pending scheduled tasks.".to_string()
        } else {
            format!("{} scheduled task(s):", items.len())
        });
        for item in items {
            summary.push('\n');
            summary.push_str(&format_scheduled_item(item));
            if item.delivery.state == ItemState::Dispatched {
                summary.push_str(" | dispatched, awaiting acknowledgement");
            }
        }
        if !history.is_empty() {
            summary.push_str(&format!("\nRecent history ({}):", history.len()));
            for item in history {
                summary.push('\n');
                summary.push_str(&format_history_item(item));
            }
        }

        Ok(ToolOutput::new(summary).with_title("scheduled tasks"))
    }

    async fn execute_retry(&self, params: ScheduleToolInput) -> Result<ToolOutput> {
        let id = params
            .schedule_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("schedule_id is required for action=retry"))?;

        let mut manager = AmbientManager::new()?;
        let Some(item) = manager.retry_schedule(id)? else {
            anyhow::bail!(
                "No uncertain or missed scheduled task found with id '{}'",
                id
            );
        };
        nudge_schedule_runner();

        let task = item.task_description.as_deref().unwrap_or(&item.context);
        let message = if item.delivery.attempts.is_empty() {
            format!(
                "Retrying missed scheduled task '{task}' (id: {}). It was never dispatched, so \
                 no earlier effects are expected.",
                item.id
            )
        } else {
            format!(
                "Retrying scheduled task '{task}' (id: {}). Warning: attempt {} may already have \
                 run, so its effects may already exist and this retry can duplicate them. \
                 Retrying does not prove the first attempt failed. Prior attempt history is kept.",
                item.id,
                item.delivery.attempts.len()
            )
        };
        Ok(ToolOutput::new(message).with_title(format!("retry: {}", item.id)))
    }

    /// Remove a recurring definition and cancel its still-pending occurrences.
    /// Delivered or uncertain history is kept.
    fn cancel_recurring(&self, id: &str) -> Result<ToolOutput> {
        let store = crate::ambient::recurring::RecurringStore::open_default()?;
        if store.remove(id)?.is_none() {
            anyhow::bail!("No recurring schedule found with id '{id}'");
        }
        let mut manager = AmbientManager::new()?;
        let prefix = format!("cron:{id}:");
        let pending: Vec<String> = manager
            .queue()
            .items()
            .iter()
            .filter(|item| item.id.starts_with(&prefix))
            .map(|item| item.id.clone())
            .collect();
        for key in &pending {
            manager.cancel_schedule(key)?;
        }
        nudge_schedule_runner();
        Ok(ToolOutput::new(format!(
            "Cancelled recurring schedule {id}; {} pending occurrence(s) cancelled. Already delivered runs are not stopped.",
            pending.len()
        ))
        .with_title(format!("cancelled: {id}")))
    }

    async fn execute_cancel(&self, params: ScheduleToolInput) -> Result<ToolOutput> {
        let id = params
            .schedule_id
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("schedule_id is required for action=cancel"))?;

        if id.starts_with("cron_") {
            return self.cancel_recurring(id);
        }

        let mut manager = AmbientManager::new()?;
        let Some(item) = manager.cancel_schedule(id)? else {
            anyhow::bail!(
                "No pending, uncertain or missed scheduled task found with id '{}'",
                id
            );
        };
        nudge_schedule_runner();

        Ok(ToolOutput::new(format!(
            "Cancelled scheduled task '{}' for {} (id: {})",
            item.task_description.as_deref().unwrap_or(&item.context),
            item.scheduled_for,
            item.id
        ))
        .with_title(format!("cancelled: {}", item.id)))
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

fn parse_priority(s: Option<&str>) -> Priority {
    match s {
        Some("low") => Priority::Low,
        Some("high") => Priority::High,
        _ => Priority::Normal,
    }
}

fn parse_schedule_target(s: Option<&str>, session_id: &str) -> Result<ScheduleTarget> {
    Ok(match s {
        Some("ambient") => ScheduleTarget::Ambient,
        Some("spawn") => ScheduleTarget::Spawn {
            parent_session_id: session_id.to_string(),
        },
        Some("resume") | None => ScheduleTarget::Session {
            session_id: session_id.to_string(),
        },
        Some(other) => anyhow::bail!(
            "Invalid target '{}'. Expected one of: resume, spawn, ambient",
            other
        ),
    })
}

fn format_schedule_target(target: &ScheduleTarget) -> String {
    match target {
        ScheduleTarget::Ambient => "ambient agent".to_string(),
        ScheduleTarget::Session { session_id } => format!("resume session {}", session_id),
        ScheduleTarget::Spawn { parent_session_id } => {
            format!("spawn one child session from {}", parent_session_id)
        }
    }
}

/// Terminal items shown by `schedule list`.
const SCHEDULE_HISTORY_LIMIT: usize = 10;

fn format_history_item(item: &ScheduledItem) -> String {
    let record = &item.delivery;
    let mut line = format!(
        "- {} | {} | {} | {}",
        item.id,
        record.state.as_str(),
        item.scheduled_for,
        item.task_description.as_deref().unwrap_or(&item.context)
    );
    match record.state {
        ItemState::Missed => {
            if let Some(secs) = record.missed_delay_secs {
                let mins = u32::try_from(secs / 60).unwrap_or(u32::MAX);
                line.push_str(&format!(
                    " | {} late, not delivered; retry explicitly with action=retry",
                    crate::ambient::format_minutes_human(mins)
                ));
            }
        }
        ItemState::Uncertain => line.push_str(
            " | acknowledgement missing, it may have run; retry explicitly with action=retry",
        ),
        ItemState::Delivered => line.push_str(" | target accepted the message (not task success)"),
        _ => {}
    }
    if record.attempts.len() > 1 {
        line.push_str(&format!(" | {} attempts", record.attempts.len()));
    }
    line
}

fn format_scheduled_item(item: &ScheduledItem) -> String {
    format!(
        "- {} | {} | {:?} | {} | {}",
        item.id,
        item.scheduled_for,
        item.priority,
        format_schedule_target(&item.target),
        item.task_description.as_deref().unwrap_or(&item.context)
    )
}

fn describe_recurring(schedule: &crate::ambient::recurring::Schedule) -> String {
    use crate::ambient::recurring::MissedPolicy;
    let missed = match schedule.missed {
        MissedPolicy::Skip => "skip".to_string(),
        MissedPolicy::RunOnce => "run-once".to_string(),
        MissedPolicy::CatchUp { limit } => format!("catch-up({limit})"),
    };
    let mut line = format!(
        "cron '{}' in {} | policy {missed} | max_active {} | {}",
        schedule.cron,
        schedule.zone,
        schedule.max_active,
        if schedule.paused { "paused" } else { "active" }
    );
    if let Some(cap) = schedule.token_cap {
        line.push_str(&format!(" | tokens {}/{cap}", schedule.tokens_used));
    }
    if let Some(cap) = schedule.spend_cap_usd {
        line.push_str(&format!(
            " | spend ${:.2}/${cap:.2}",
            schedule.spend_used_usd
        ));
    }
    if schedule.spend_cap_usd.is_some() {
        line.push_str(" | spend caps cannot be enforced (no price or reservation source): runs report budget_unavailable and do not start");
    } else if schedule.token_cap.is_some() {
        line.push_str(" | token cap enforced from provider usage; one response may overshoot");
    }
    line
}

fn format_next_occurrences(schedule: &crate::ambient::recurring::Schedule, n: usize) -> String {
    let mut out = String::new();
    for occ in schedule.upcoming(Utc::now(), n) {
        out.push_str(&format!(
            "\n  next {} local | {} UTC",
            occ.local.format("%Y-%m-%d %H:%M"),
            occ.at.format("%Y-%m-%d %H:%M")
        ));
    }
    out
}

/// Wake the runner from a non-async hook (turn end).
pub(crate) fn wake_schedule_runner() {
    nudge_schedule_runner();
}

pub(super) fn nudge_schedule_runner() {
    let runner = SCHEDULE_RUNNER
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|slot| slot.clone());
    if let Some(runner) = runner {
        runner.nudge();
    }
}

// ---------------------------------------------------------------------------
// SendChannelMessageTool — send messages via any configured channel
// ---------------------------------------------------------------------------

pub struct SendChannelMessageTool;

impl Default for SendChannelMessageTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SendChannelMessageTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SendChannelMessageTool {
    fn name(&self) -> &str {
        "send_message"
    }

    fn description(&self) -> &str {
        "Send a user message."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "intent": super::intent_schema_property(),
                "message": {
                    "type": "string",
                    "description": "The message text to send"
                },
                "channel": {
                    "type": "string",
                    "description": "Optional: specific channel to send to (e.g. 'telegram', 'discord'). Omit to send to all."
                }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, args: Value, _context: ToolContext) -> Result<ToolOutput> {
        let message = args
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: message"))?;

        let channel_name = args.get("channel").and_then(|v| v.as_str());

        let config = crate::config::config();
        let registry = crate::channel::ChannelRegistry::from_config(&config.safety);

        if let Some(name) = channel_name {
            match registry.find_by_name(name) {
                Some(ch) => match ch.send(message).await {
                    Ok(()) => Ok(ToolOutput::new(format!("Message sent via {}.", name))),
                    Err(e) => Ok(ToolOutput::new(format!(
                        "Failed to send via {}: {}",
                        name, e
                    ))),
                },
                None => {
                    let available = registry.channel_names();
                    Ok(ToolOutput::new(format!(
                        "Channel '{}' not found. Available: {}",
                        name,
                        if available.is_empty() {
                            "none configured".to_string()
                        } else {
                            available.join(", ")
                        }
                    )))
                }
            }
        } else {
            let channels = registry.send_enabled();
            if channels.is_empty() {
                return Ok(ToolOutput::new(
                    "No messaging channels configured. Enable telegram or discord in config.",
                ));
            }
            let mut results = Vec::new();
            for ch in &channels {
                match ch.send(message).await {
                    Ok(()) => results.push(format!("✓ {}", ch.name())),
                    Err(e) => results.push(format!("✗ {}: {}", ch.name(), e)),
                }
            }
            Ok(ToolOutput::new(format!(
                "Message sent: {}",
                results.join(", ")
            )))
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
#[path = "ambient/tests.rs"]
mod tests;
