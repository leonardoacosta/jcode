//! Lifecycle event hooks: `[[hooks.PreToolUse]]`, `[[hooks.Stop]]`, ...
//!
//! Each event has a list of [`HookEntry`](crate::config::HookEntry) tables.
//! A hook receives a Claude Code compatible JSON object on stdin and may
//! answer with an exit code (0 allow, 2 block) or JSON on stdout. How the
//! answer is used depends on the entry's mode:
//!
//! - `blocking`: awaited, decisions honored (only `PreToolUse` and `Stop`).
//! - `observe`: awaited with a timeout, output logged, decisions ignored.
//! - `async`: detached, never awaited, output discarded; the timeout still
//!   kills it.
//!
//! Commands run with the same direct-argv, `JCODE_HOOKS_DISABLED=1` contract
//! as the legacy hooks in the parent module. Child processes lead their own
//! process group so a timeout kills the whole tree, and callers can hand in a
//! [`ChildOwnership`] so the session's durable generation accounts for every
//! spawned child until it is proven gone.

use super::{
    BLOCK_REASON_LIMIT, GATE_CONTEXT_LIMIT, HookEvent, build_hook_process, hooks_suppressed,
    parse_decision_stdout, strict_hooks_config, truncate_bytes,
};
use crate::config::{HookEntry, HooksConfig};
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

/// Maximum bytes kept from each of a hook's stdout and stderr.
const OUTPUT_LIMIT: usize = 64 * 1024;
/// Maximum chars of hook output written to a log line.
const LOG_OUTPUT_LIMIT: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    SessionStart,
    PreToolUse,
    PostToolUse,
    SubagentStop,
    Stop,
    PreCompact,
}

impl LifecycleEvent {
    pub const ALL: [Self; 6] = [
        Self::SessionStart,
        Self::PreToolUse,
        Self::PostToolUse,
        Self::SubagentStop,
        Self::Stop,
        Self::PreCompact,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::SessionStart => "SessionStart",
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::SubagentStop => "SubagentStop",
            Self::Stop => "Stop",
            Self::PreCompact => "PreCompact",
        }
    }

    pub fn default_mode(self) -> HookMode {
        match self {
            Self::PreToolUse | Self::Stop => HookMode::Blocking,
            Self::SessionStart | Self::PreCompact => HookMode::Observe,
            Self::PostToolUse | Self::SubagentStop => HookMode::Async,
        }
    }

    /// Only these events act on a hook's decision.
    pub fn can_block(self) -> bool {
        matches!(self, Self::PreToolUse | Self::Stop)
    }

    /// The lifecycle event a legacy snake_case hook key is an alias of.
    pub(super) fn from_legacy(legacy: &str) -> Option<Self> {
        match legacy {
            "session_start" => Some(Self::SessionStart),
            "pre_tool" => Some(Self::PreToolUse),
            "post_tool" => Some(Self::PostToolUse),
            _ => None,
        }
    }

    /// Name of the env/payload field that carries the matcher subject.
    fn subject_field(self) -> Option<&'static str> {
        match self {
            Self::PreToolUse | Self::PostToolUse => Some("TOOL_NAME"),
            Self::SessionStart => Some("SOURCE"),
            Self::PreCompact => Some("TRIGGER"),
            Self::SubagentStop | Self::Stop => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookMode {
    Blocking,
    Observe,
    Async,
}

impl HookMode {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "blocking" => Some(Self::Blocking),
            "observe" => Some(Self::Observe),
            "async" => Some(Self::Async),
            _ => None,
        }
    }
}

/// A validated entry. Invalid entries never become one.
#[derive(Debug, Clone)]
pub struct ResolvedHook {
    pub command: String,
    pub timeout: Duration,
    pub fail_closed: bool,
    pub mode: HookMode,
    matcher: Option<Regex>,
    path: Option<Regex>,
}

impl ResolvedHook {
    /// Whether this hook applies to a call. `subject` is the tool name,
    /// session-start source or compaction trigger; events without a subject
    /// ignore `matcher`. When `path` is set, some path argument must match it
    /// (absolute, or relative to `cwd`).
    pub fn applies(&self, subject: Option<&str>, paths: &[String], cwd: Option<&str>) -> bool {
        if let (Some(matcher), Some(subject)) = (&self.matcher, subject)
            && !matcher.is_match(subject)
        {
            return false;
        }
        let Some(glob) = &self.path else {
            return true;
        };
        paths.iter().any(|path| {
            let path = path.replace('\\', "/");
            if glob.is_match(&path) {
                return true;
            }
            cwd.map(|cwd| cwd.replace('\\', "/"))
                .and_then(|cwd| {
                    path.strip_prefix(cwd.trim_end_matches('/'))
                        .map(|rest| rest.trim_start_matches('/').to_string())
                })
                .is_some_and(|relative| glob.is_match(&relative))
        })
    }
}

/// Whole-string, case-insensitive tool-name/source/trigger matcher. Empty and
/// `*` match everything.
fn compile_matcher(matcher: &str) -> Result<Option<Regex>, String> {
    let matcher = matcher.trim();
    if matcher.is_empty() || matcher == "*" {
        return Ok(None);
    }
    Regex::new(&format!("(?i)^(?:{matcher})$"))
        .map(Some)
        .map_err(|error| format!("invalid matcher {matcher:?}: {error}"))
}

/// Compile a path glob: `*` and `?` stay inside one path segment, `**` crosses
/// segments, `[...]` is a character class (`[!...]` negates).
fn compile_glob(glob: &str) -> Result<Regex, String> {
    let mut re = String::from("(?s)^");
    let mut chars = glob.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    chars.next();
                    re.push_str("(?:.*/)?");
                } else {
                    re.push_str(".*");
                }
            }
            '*' => re.push_str("[^/]*"),
            '?' => re.push_str("[^/]"),
            '[' => {
                let mut class = String::new();
                let mut closed = false;
                if matches!(chars.peek(), Some('!' | '^')) {
                    chars.next();
                    class.push('^');
                }
                for inner in chars.by_ref() {
                    if inner == ']' && !class.trim_start_matches('^').is_empty() {
                        closed = true;
                        break;
                    }
                    if matches!(inner, '\\' | '[') {
                        class.push('\\');
                    }
                    class.push(inner);
                }
                if !closed {
                    return Err(format!("invalid path glob {glob:?}: unclosed '['"));
                }
                re.push('[');
                re.push_str(&class);
                re.push(']');
            }
            other => re.push_str(&regex::escape(&other.to_string())),
        }
    }
    re.push('$');
    Regex::new(&re).map_err(|error| format!("invalid path glob {glob:?}: {error}"))
}

fn resolve_entry(
    event: LifecycleEvent,
    entry: &HookEntry,
) -> Result<(ResolvedHook, Option<String>), String> {
    let command = entry.command.trim();
    if command.is_empty() {
        return Err("empty command".to_string());
    }
    let mut note = None;
    let mut mode = match entry.mode.as_deref() {
        None => event.default_mode(),
        Some(raw) => HookMode::parse(raw).ok_or_else(|| {
            format!("unknown mode {raw:?} (expected blocking, observe, or async)")
        })?,
    };
    if mode == HookMode::Blocking && !event.can_block() {
        note = Some(format!(
            "mode \"blocking\" is not supported for {}; running as observe",
            event.name()
        ));
        mode = HookMode::Observe;
    }
    let matcher = entry
        .matcher
        .as_deref()
        .map(compile_matcher)
        .transpose()?
        .flatten();
    let path = entry.path.as_deref().map(compile_glob).transpose()?;
    Ok((
        ResolvedHook {
            command: command.to_string(),
            timeout: Duration::from_secs(entry.timeout.max(1)),
            fail_closed: entry.fail_closed,
            mode,
            matcher,
            path,
        },
        note,
    ))
}

/// Validated hooks for `event` plus one warning per rejected or adjusted
/// entry. A bad entry is disabled; the rest still load.
pub fn resolve_entries(
    hooks: &HooksConfig,
    event: LifecycleEvent,
) -> (Vec<ResolvedHook>, Vec<String>) {
    let mut resolved = Vec::new();
    let mut warnings = Vec::new();
    for (index, entry) in hooks.lifecycle_entries(event.name()).iter().enumerate() {
        let label = format!("hooks.{}[{index}]", event.name());
        match resolve_entry(event, entry) {
            Ok((hook, note)) => {
                warnings.extend(note.map(|note| format!("{label}: {note}")));
                resolved.push(hook);
            }
            Err(reason) => warnings.push(format!("{label} disabled: {reason}")),
        }
    }
    (resolved, warnings)
}

/// Every warning [`resolve_entries`] would produce for `hooks`.
pub fn lifecycle_config_warnings(hooks: &HooksConfig) -> Vec<String> {
    LifecycleEvent::ALL
        .into_iter()
        .flat_map(|event| resolve_entries(hooks, event).1)
        .collect()
}

/// Whether any lifecycle entry could run for `event`. Cheap; hot paths use it
/// to skip payload construction.
pub fn lifecycle_configured(event: LifecycleEvent) -> bool {
    if hooks_suppressed() {
        return false;
    }
    strict_hooks_config().is_some_and(|hooks| !hooks.lifecycle_entries(event.name()).is_empty())
        // A workspace rule may add entries for sessions whose cwd it matches.
        || crate::config::config_strict()
            .is_ok_and(|config| config.workspaces.has_lifecycle_entries(event.name()))
}

/// Drop legacy commands that a lifecycle entry already runs for the same
/// event, so configuring one hook under both spellings dispatches it once.
pub(super) fn drop_aliased_legacy(
    hooks: &HooksConfig,
    legacy_event: &str,
    commands: &mut Vec<String>,
) {
    let Some(event) = LifecycleEvent::from_legacy(legacy_event) else {
        return;
    };
    let lifecycle: HashSet<&str> = hooks
        .lifecycle_entries(event.name())
        .iter()
        .map(|entry| entry.command.trim())
        .collect();
    commands.retain(|command| !lifecycle.contains(command.as_str()));
}

/// Lets the caller account a spawned hook child against a durable owner.
pub trait ChildOwnership: Send + Sync {
    /// Claim a child before it spawns. An error (for example a revoked
    /// generation) means the hook must not run.
    fn own(&self) -> anyhow::Result<Box<dyn ChildLease>>;
}

/// One claimed child. Dropping it releases the claim; the owner decides when
/// the process is really gone.
pub trait ChildLease: Send {
    fn set_pid(&mut self, pid: u32);
}

/// One lifecycle event to deliver.
#[derive(Clone)]
pub struct LifecycleCall {
    pub event: LifecycleEvent,
    pub session_id: String,
    pub cwd: Option<String>,
    pub transcript_path: Option<String>,
    /// Tool name, session-start source or compaction trigger.
    pub subject: Option<String>,
    /// File paths named by a tool call, for `path` filters.
    pub paths: Vec<String>,
    /// Event fields merged into the stdin payload.
    pub fields: Map<String, Value>,
    pub owner: Option<Arc<dyn ChildOwnership>>,
}

impl LifecycleCall {
    pub fn new(event: LifecycleEvent, session_id: impl Into<String>) -> Self {
        Self {
            event,
            session_id: session_id.into(),
            cwd: None,
            transcript_path: None,
            subject: None,
            paths: Vec::new(),
            fields: Map::new(),
            owner: None,
        }
    }

    pub fn cwd(mut self, cwd: Option<String>) -> Self {
        self.cwd = cwd;
        self
    }

    pub fn transcript_path(mut self, path: Option<String>) -> Self {
        self.transcript_path = path;
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn paths(mut self, paths: Vec<String>) -> Self {
        self.paths = paths;
        self
    }

    pub fn field(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.fields.insert(key.to_string(), value.into());
        self
    }

    pub fn owner(mut self, owner: Option<Arc<dyn ChildOwnership>>) -> Self {
        self.owner = owner;
        self
    }

    /// The JSON object a hook reads on stdin.
    pub fn payload(&self) -> Value {
        let mut map = Map::new();
        map.insert("session_id".into(), self.session_id.clone().into());
        map.insert(
            "transcript_path".into(),
            self.transcript_path
                .clone()
                .map_or(Value::Null, Value::String),
        );
        map.insert(
            "cwd".into(),
            self.cwd.clone().map_or(Value::Null, Value::String),
        );
        map.insert("hook_event_name".into(), self.event.name().into());
        for (key, value) in &self.fields {
            map.insert(key.clone(), value.clone());
        }
        Value::Object(map)
    }

    fn env_event(&self) -> HookEvent {
        let mut event = HookEvent::new(self.event.name()).session_id(self.session_id.clone());
        if let Some(cwd) = &self.cwd {
            event = event.cwd(cwd.clone());
        }
        if let (Some(field), Some(subject)) = (self.event.subject_field(), &self.subject) {
            event = event.field(field, subject.clone());
        }
        event
    }
}

/// What the blocking hooks of one event decided.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LifecycleOutcome {
    /// `PreToolUse`: the tool call is denied (exit 2, JSON deny/`ask`, or
    /// `fail_closed` failure). `Stop`: the reason to continue with.
    pub block: Option<String>,
    /// Replacement tool input (`PreToolUse`, only when not blocked).
    pub updated_input: Option<Value>,
    /// Extra model context from `additionalContext`/`systemMessage`.
    pub additional_context: Vec<String>,
    /// `continue: false` with its `stopReason`.
    pub halt: Option<String>,
    /// A hook child could not be claimed (revoked generation). Callers must
    /// not admit new work after this.
    pub admission_error: Option<String>,
}

enum RunResult {
    Exited {
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    TimedOut,
    Failed(String),
}

fn redact(text: &str, limit: usize) -> String {
    truncate_bytes(&crate::message::redact_secrets(text), limit).to_string()
}

async fn read_bounded<R: tokio::io::AsyncRead + Unpin>(reader: Option<R>) -> Vec<u8> {
    use tokio::io::AsyncReadExt;
    let Some(mut reader) = reader else {
        return Vec::new();
    };
    let mut kept = Vec::new();
    let mut buffer = [0u8; 8192];
    // Keep reading past the cap so a chatty hook never blocks on a full pipe.
    while let Ok(read) = reader.read(&mut buffer).await {
        if read == 0 {
            break;
        }
        let room = OUTPUT_LIMIT.saturating_sub(kept.len());
        kept.extend_from_slice(&buffer[..read.min(room)]);
    }
    kept
}

fn kill_group(pid: Option<u32>) {
    if let Some(pid) = pid {
        #[cfg(unix)]
        let signal = libc::SIGKILL;
        #[cfg(not(unix))]
        let signal = 9;
        let _ = crate::platform::signal_detached_process_group(pid, signal);
    }
}

async fn run_command(
    hook: &ResolvedHook,
    call: &LifecycleCall,
    payload: &str,
    mut lease: Option<Box<dyn ChildLease>>,
) -> RunResult {
    let std_cmd = match build_hook_process(&hook.command, &call.env_event()) {
        Ok(cmd) => cmd,
        Err(error) => return RunResult::Failed(format!("invalid command: {error}")),
    };
    let mut cmd = tokio::process::Command::from(std_cmd);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    // Own process group (pgid == pid): timeouts kill the tree and the owner
    // can prove the whole group gone.
    #[cfg(unix)]
    cmd.process_group(0);
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(error) => return RunResult::Failed(format!("failed to start: {error}")),
    };
    let pid = child.id();
    if let (Some(lease), Some(pid)) = (lease.as_mut(), pid) {
        lease.set_pid(pid);
    }
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let work = async {
        let write = async move {
            if let Some(mut stdin) = stdin {
                use tokio::io::AsyncWriteExt;
                // A hook that exits without reading stdin closes the pipe.
                let _ = stdin.write_all(payload.as_bytes()).await;
            }
        };
        let (_, out, err, status) = tokio::join!(
            write,
            read_bounded(stdout),
            read_bounded(stderr),
            child.wait()
        );
        (out, err, status)
    };
    let result = match tokio::time::timeout(hook.timeout, work).await {
        Ok((out, err, Ok(status))) => RunResult::Exited {
            code: status.code(),
            stdout: String::from_utf8_lossy(&out).into_owned(),
            stderr: String::from_utf8_lossy(&err).into_owned(),
        },
        Ok((_, _, Err(error))) => RunResult::Failed(format!("wait failed: {error}")),
        Err(_) => {
            kill_group(pid);
            let _ = child.start_kill();
            let _ = child.wait().await;
            RunResult::TimedOut
        }
    };
    // Grandchildren that outlive the hook stay accounted by the lease owner.
    drop(lease);
    result
}

fn failure_reason(result: &RunResult, hook: &ResolvedHook) -> String {
    match result {
        RunResult::TimedOut => format!("timed out after {}s", hook.timeout.as_secs()),
        RunResult::Failed(reason) => redact(reason, BLOCK_REASON_LIMIT),
        RunResult::Exited { code, .. } => format!("exited with {code:?}"),
    }
}

fn log_observed(event: LifecycleEvent, hook: &ResolvedHook, result: &RunResult) {
    match result {
        RunResult::Exited {
            code: Some(0),
            stdout,
            ..
        } => {
            let verdict = parse_decision_stdout(stdout, event.name());
            if !verdict.suppress_output && !stdout.trim().is_empty() {
                crate::logging::debug(&format!(
                    "Hook '{}' output: {}",
                    event.name(),
                    redact(stdout.trim(), LOG_OUTPUT_LIMIT)
                ));
            }
        }
        other => crate::logging::warn(&format!(
            "Hook '{}' command '{}' {}",
            event.name(),
            redact(&hook.command, LOG_OUTPUT_LIMIT),
            failure_reason(other, hook)
        )),
    }
}

/// Run every matching hook of `call.event` in declaration order.
pub async fn run_lifecycle(call: LifecycleCall) -> LifecycleOutcome {
    if hooks_suppressed() {
        return LifecycleOutcome::default();
    }
    let Some(hooks) = strict_hooks_config() else {
        return LifecycleOutcome::default();
    };
    // Per-workspace entries append after the global ones, only for a cwd that
    // resolves to that workspace by its own git origin.
    // Skip the blocking hop entirely when no rule could add an entry.
    let event = call.event.name();
    if crate::config::config_strict().is_ok_and(|c| c.workspaces.has_lifecycle_entries(event)) {
        let cwd = call.cwd.clone();
        let merged = tokio::task::spawn_blocking(move || {
            crate::workspace::lifecycle_hooks_for(cwd.as_deref(), event)
        })
        .await
        .ok()
        .flatten();
        if let Some(merged) = merged {
            return run_lifecycle_with(&merged, call).await;
        }
    }
    run_lifecycle_with(hooks, call).await
}

/// [`run_lifecycle`] against an explicit config.
pub async fn run_lifecycle_with(hooks: &HooksConfig, mut call: LifecycleCall) -> LifecycleOutcome {
    let mut outcome = LifecycleOutcome::default();
    // Invalid entries (global and workspace) were already reported when the
    // config loaded.
    let (entries, _warnings) = resolve_entries(hooks, call.event);
    for hook in entries {
        if !hook.applies(call.subject.as_deref(), &call.paths, call.cwd.as_deref()) {
            continue;
        }
        let lease = match call.owner.as_ref().map(|owner| owner.own()).transpose() {
            Ok(lease) => lease,
            Err(error) => {
                outcome.admission_error = Some(format!("{error:#}"));
                break;
            }
        };
        let payload = call.payload().to_string();
        match hook.mode {
            HookMode::Async => {
                let call = call.clone();
                tokio::spawn(async move {
                    let result = run_command(&hook, &call, &payload, lease).await;
                    log_observed(call.event, &hook, &result);
                });
            }
            HookMode::Observe => {
                let result = run_command(&hook, &call, &payload, lease).await;
                log_observed(call.event, &hook, &result);
            }
            HookMode::Blocking => {
                let result = run_command(&hook, &call, &payload, lease).await;
                // `call.paths` is deliberately not recomputed after a rewrite:
                // `path` filters judge the tool call as the model issued it.
                if apply_blocking(&hook, call.event, &result, &mut outcome)
                    && let (Some(updated), None) = (&outcome.updated_input, &outcome.block)
                {
                    call.fields.insert("tool_input".into(), updated.clone());
                }
            }
        }
    }
    if outcome.block.is_some() {
        outcome.updated_input = None;
    }
    outcome
}

/// Fold one blocking hook's result into `outcome`. Returns true when the
/// hook completed normally (exit 0), so later hooks should see its rewrite.
fn apply_blocking(
    hook: &ResolvedHook,
    event: LifecycleEvent,
    result: &RunResult,
    outcome: &mut LifecycleOutcome,
) -> bool {
    let label = event.name();
    let mut block = |reason: String| {
        if outcome.block.is_none() {
            outcome.block = Some(reason);
        }
    };
    match result {
        RunResult::Exited {
            code: Some(0),
            stdout,
            ..
        } => {
            let verdict = parse_decision_stdout(stdout, label);
            if let Some(reason) = verdict.block {
                block(redact(&reason, BLOCK_REASON_LIMIT));
            } else if outcome.block.is_none()
                && let Some(updated) = verdict.updated_input
                && event == LifecycleEvent::PreToolUse
            {
                outcome.updated_input = Some(updated);
            }
            if let Some(context) = verdict.context {
                outcome
                    .additional_context
                    .push(redact(&context, GATE_CONTEXT_LIMIT));
            }
            if outcome.halt.is_none() {
                outcome.halt = verdict.halt;
            }
            true
        }
        RunResult::Exited {
            code: Some(2),
            stderr,
            ..
        } => {
            let reason = stderr.trim();
            block(if reason.is_empty() {
                format!("blocked by {label} hook")
            } else {
                redact(reason, BLOCK_REASON_LIMIT)
            });
            false
        }
        other => {
            let reason = failure_reason(other, hook);
            let command = redact(&hook.command, LOG_OUTPUT_LIMIT);
            // `fail_closed` only gates tool calls. A failed `Stop` hook must
            // never become a continuation instruction.
            if hook.fail_closed && event == LifecycleEvent::PreToolUse {
                crate::logging::warn(&format!(
                    "Hook '{label}' command '{command}' {reason} (fail_closed: blocking)"
                ));
                block(format!(
                    "{label} hook failed ({reason}); blocked by fail_closed"
                ));
            } else {
                crate::logging::warn(&format!(
                    "Hook '{label}' command '{command}' {reason} (continuing)"
                ));
            }
            false
        }
    }
}

/// Fire `call` without waiting, for sync call sites. Skipped (with a debug
/// log) outside a Tokio runtime.
pub fn dispatch_lifecycle_detached(call: LifecycleCall) {
    if hooks_suppressed() || !lifecycle_configured(call.event) {
        return;
    }
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(async move {
                run_lifecycle(call).await;
            });
        }
        Err(_) => crate::logging::debug(&format!(
            "Hook '{}' skipped: no async runtime",
            call.event.name()
        )),
    }
}

#[cfg(test)]
#[path = "hooks_lifecycle_tests.rs"]
mod tests;
