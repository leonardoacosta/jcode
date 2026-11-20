//! Linear over an MCP server, behind a Rust-side allowlist.
//!
//! This is the only place that names MCP tools. The provider can call exactly
//! three tools (`list_issue_statuses`, `list_issues`, `save_issue`); the tool
//! name is a private enum, never a caller-supplied string, and `save_issue`
//! arguments are built from an issue id and a state id only, so a label,
//! assignee, comment or description cannot be expressed.
//!
//! UNVERIFIED: the real Linear MCP response shapes and `save_issue`'s
//! idempotency contract were not confirmed (no live calls are allowed in this
//! work). The parser is therefore tolerant about shape but strict about
//! meaning: anything it cannot read is `BoardError::Provider`, never a partial
//! result. Scope is enforced twice: ids from the binding go into the request,
//! and every returned issue is re-checked client-side (an issue whose team, or
//! bound project, is absent or different is dropped), so a connection that can
//! also see other teams (for example BrownandBrown) cannot leak them. An
//! issue that carries no team id (or, with a project binding, no project id)
//! cannot be verified, so the whole read fails instead of dropping it: a
//! response shape the parser does not understand must not look like an empty
//! board.

use super::binding::BoardScope;
use super::provider::{BoardError, BoardIssue, BoardProvider, WorkflowState, sort_states};
use async_trait::async_trait;
use serde_json::{Map, Value, json};
use std::sync::Arc;

/// Issues requested per page.
const PAGE_LIMIT: u64 = 100;
/// Hard cap on pages. Hitting it is an error, not a truncated board.
const MAX_PAGES: usize = 20;

/// The narrow MCP surface this module needs. Implementations must return the
/// tool's text content.
#[async_trait]
pub trait McpCaller: Send + Sync {
    async fn call(&self, server: &str, tool: &str, args: Value) -> anyhow::Result<String>;
}

/// The server answered with a tool error (`isError`): it processed the call
/// and reported failure, so the write is known not to have been dispatched
/// successfully. Other caller errors (transport) are ambiguous.
#[derive(Debug)]
pub struct ToolError(pub String);

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}

/// Real caller over the MCP manager.
pub struct McpManagerCaller {
    manager: Arc<crate::mcp::McpManager>,
}

impl McpManagerCaller {
    pub fn new(manager: Arc<crate::mcp::McpManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl McpCaller for McpManagerCaller {
    async fn call(&self, server: &str, tool: &str, args: Value) -> anyhow::Result<String> {
        let result = self.manager.call_tool(server, tool, args).await?;
        tool_result_text(result)
    }
}

/// Text blocks joined by newlines; `isError` becomes a [`ToolError`].
fn tool_result_text(result: crate::mcp::ToolCallResult) -> anyhow::Result<String> {
    let text = result
        .content
        .iter()
        .filter_map(|block| match block {
            crate::mcp::ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    if result.is_error {
        return Err(ToolError(text).into());
    }
    Ok(text)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    ListIssueStatuses,
    ListIssues,
    SaveIssue,
}

impl Tool {
    fn name(self) -> &'static str {
        match self {
            Self::ListIssueStatuses => "list_issue_statuses",
            Self::ListIssues => "list_issues",
            Self::SaveIssue => "save_issue",
        }
    }
}

fn statuses_args(scope: &BoardScope) -> Value {
    json!({ "team": scope.team })
}

fn list_issues_args(scope: &BoardScope, cursor: Option<&str>) -> Value {
    let mut args = Map::new();
    args.insert("team".into(), json!(scope.team));
    if let Some(project) = &scope.project {
        args.insert("project".into(), json!(project));
    }
    args.insert("limit".into(), json!(PAGE_LIMIT));
    if let Some(cursor) = cursor {
        args.insert("cursor".into(), json!(cursor));
    }
    Value::Object(args)
}

/// Exactly `{id, state}`. No other key can be produced.
fn save_issue_args(issue_id: &str, state_id: &str) -> Value {
    json!({ "id": issue_id, "state": state_id })
}

pub struct LinearMcpProvider {
    caller: Arc<dyn McpCaller>,
}

impl LinearMcpProvider {
    pub fn new(caller: Arc<dyn McpCaller>) -> Self {
        Self { caller }
    }

    async fn call(
        &self,
        scope: &BoardScope,
        tool: Tool,
        args: Value,
    ) -> Result<String, anyhow::Error> {
        self.caller.call(&scope.server, tool.name(), args).await
    }

    async fn read(
        &self,
        scope: &BoardScope,
        tool: Tool,
        args: Value,
    ) -> Result<String, BoardError> {
        self.call(scope, tool, args).await.map_err(|error| {
            let text = format!("{error:#}");
            if looks_unauthorized(&text) {
                BoardError::Unauthorized(text)
            } else if error.downcast_ref::<ToolError>().is_some() {
                BoardError::Provider(text)
            } else {
                BoardError::Unavailable(text)
            }
        })
    }

    /// All in-scope raw issues across pages. Truncation is an error.
    async fn scoped_issues(&self, scope: &BoardScope) -> Result<Vec<RawIssue>, BoardError> {
        let mut out = Vec::new();
        let mut other_team = 0usize;
        let mut cursor: Option<String> = None;
        for _ in 0..MAX_PAGES {
            let text = self
                .read(
                    scope,
                    Tool::ListIssues,
                    list_issues_args(scope, cursor.as_deref()),
                )
                .await?;
            let (items, next) = parse_issue_page(&text)?;
            for issue in items {
                if issue.team.as_deref().is_some_and(|t| t != scope.team) {
                    other_team += 1;
                }
                if issue.in_scope(scope)? {
                    out.push(issue);
                }
            }
            match next {
                None if out.is_empty() && other_team > 0 => {
                    // Fail closed, but say why: a binding written as a team key
                    // or name matches no `teamId`, which would otherwise read
                    // as an empty board.
                    return Err(BoardError::Provider(format!(
                        "{other_team} issues came back but none belong to the bound team; \
                         the binding's `team` must be the Linear team id (uuid), not its key or name"
                    )));
                }
                None => return Ok(out),
                Some(next) if cursor.as_deref() == Some(next.as_str()) => {
                    return Err(BoardError::Provider("issue cursor did not advance".into()));
                }
                Some(next) => cursor = Some(next),
            }
        }
        Err(BoardError::Provider(format!(
            "issue list exceeds {MAX_PAGES} pages; refusing a truncated board"
        )))
    }

    async fn resolve(
        &self,
        scope: &BoardScope,
        raw: Vec<RawIssue>,
    ) -> Result<Vec<BoardIssue>, BoardError> {
        let states = if raw.iter().any(|i| i.state_id.is_none()) {
            self.workflow_states(scope).await?
        } else {
            Vec::new()
        };
        raw.into_iter()
            .map(|issue| {
                let state_id = match (&issue.state_id, &issue.state_name) {
                    (Some(id), _) => id.clone(),
                    (None, Some(name)) => states
                        .iter()
                        .find(|s| &s.name == name)
                        .map(|s| s.id.clone())
                        .ok_or_else(|| {
                            BoardError::Provider(format!(
                                "issue {} is in a state not in the team workflow",
                                issue.identifier
                            ))
                        })?,
                    (None, None) => {
                        return Err(BoardError::Provider(format!(
                            "issue {} has no state",
                            issue.identifier
                        )));
                    }
                };
                Ok(BoardIssue {
                    id: issue.id,
                    identifier: issue.identifier,
                    title: issue.title,
                    state_id,
                })
            })
            .collect()
    }
}

#[async_trait]
impl BoardProvider for LinearMcpProvider {
    async fn workflow_states(&self, scope: &BoardScope) -> Result<Vec<WorkflowState>, BoardError> {
        let text = self
            .read(scope, Tool::ListIssueStatuses, statuses_args(scope))
            .await?;
        parse_states(&text, &scope.team)
    }

    async fn issues(&self, scope: &BoardScope) -> Result<Vec<BoardIssue>, BoardError> {
        let raw = self.scoped_issues(scope).await?;
        self.resolve(scope, raw).await
    }

    async fn issue_state(&self, scope: &BoardScope, issue_id: &str) -> Result<String, BoardError> {
        // No single-issue tool is allowlisted: look the issue up inside the
        // bound scope, so an issue of another team or project is not found.
        let raw = self.scoped_issues(scope).await?;
        let issue = raw
            .into_iter()
            .find(|i| i.id == issue_id)
            .ok_or_else(|| BoardError::Provider("issue is not in the bound board scope".into()))?;
        let mut resolved = self.resolve(scope, vec![issue]).await?;
        Ok(resolved.remove(0).state_id)
    }

    /// Sends `save_issue {id, state}` after confirming the issue is inside the
    /// bound scope. A transport failure is `Uncertain`: Linear's idempotency
    /// for `save_issue` is unverified, so callers must not retry blindly.
    async fn set_issue_state(
        &self,
        scope: &BoardScope,
        issue_id: &str,
        state_id: &str,
    ) -> Result<(), BoardError> {
        self.issue_state(scope, issue_id).await?;
        // The target must be a column of this team's workflow, so a state id
        // from another team (or a made-up one) never reaches `save_issue`.
        if !self
            .workflow_states(scope)
            .await?
            .iter()
            .any(|state| state.id == state_id)
        {
            return Err(BoardError::Provider(
                "requested state is not in the team workflow".into(),
            ));
        }
        match self
            .call(scope, Tool::SaveIssue, save_issue_args(issue_id, state_id))
            .await
        {
            Ok(_) => Ok(()),
            Err(error) => {
                let text = format!("{error:#}");
                Err(if looks_unauthorized(&text) {
                    BoardError::Unauthorized(text)
                } else if error.downcast_ref::<ToolError>().is_some() || failed_before_send(&text) {
                    BoardError::Provider(text)
                } else {
                    BoardError::Uncertain(text)
                })
            }
        }
    }
}

/// Manager errors raised before any request left this process: the write was
/// certainly not sent, so it is a definitive failure, not an uncertain one.
fn failed_before_send(text: &str) -> bool {
    [
        "failed to connect",
        "not connected",
        "out of scope for this workspace",
        "did not connect within",
    ]
    .iter()
    .any(|marker| text.contains(marker))
}

fn looks_unauthorized(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    // Status codes match as whole tokens: "issue ABC-4015" is not a 401.
    lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .any(|token| token == "401" || token == "403")
        || [
            "unauthor",
            "forbidden",
            "not signed in",
            "not authenticated",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
}

// ------------------------------------------------------------------ parsing

struct RawIssue {
    id: String,
    identifier: String,
    title: String,
    state_id: Option<String>,
    state_name: Option<String>,
    team: Option<String>,
    project: Option<String>,
}

impl RawIssue {
    /// `Ok(false)`: provably another team or project, drop it. An issue with
    /// no team id (or no project id under a project binding) cannot be
    /// verified: error, never a silent drop.
    fn in_scope(&self, scope: &BoardScope) -> Result<bool, BoardError> {
        let Some(team) = self.team.as_deref() else {
            return Err(BoardError::Provider(
                "cannot verify issue team; response shape unsupported".into(),
            ));
        };
        if team != scope.team {
            return Ok(false);
        }
        let Some(want) = scope.project.as_deref() else {
            return Ok(true);
        };
        match self.project.as_deref() {
            Some(project) => Ok(project == want),
            None => Err(BoardError::Provider(
                "cannot verify issue project; response shape unsupported".into(),
            )),
        }
    }
}

fn parse_json(text: &str) -> Result<Value, BoardError> {
    serde_json::from_str(text.trim()).map_err(|_| {
        if looks_unauthorized(text) {
            BoardError::Unauthorized(truncate(text))
        } else {
            BoardError::Provider("response is not JSON".into())
        }
    })
}

fn truncate(text: &str) -> String {
    text.chars().take(200).collect()
}

/// An array, or an object holding the array under one of `keys`.
fn items<'a>(value: &'a Value, keys: &[&str]) -> Result<&'a Vec<Value>, BoardError> {
    match value {
        Value::Array(items) => Ok(items),
        Value::Object(map) => keys
            .iter()
            .chain(["nodes"].iter())
            .find_map(|key| map.get(*key).and_then(Value::as_array))
            .ok_or_else(|| BoardError::Provider("response has no item list".into())),
        _ => Err(BoardError::Provider("response has no item list".into())),
    }
}

fn str_field(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// `id` of `key` when it is an object, or the string itself when `id_key`
/// style scalar is given separately.
fn nested_id(map: &Map<String, Value>, object_key: &str, scalar_key: &str) -> Option<String> {
    map.get(object_key)
        .and_then(Value::as_object)
        .and_then(|o| str_field(o, "id"))
        .or_else(|| str_field(map, scalar_key))
}

fn parse_states(text: &str, team: &str) -> Result<Vec<WorkflowState>, BoardError> {
    let value = parse_json(text)?;
    let list = items(&value, &["statuses", "states", "issueStatuses"])?;
    struct Row {
        id: String,
        name: String,
        position: Option<f64>,
        rank: Option<f64>,
        kind: String,
    }
    let mut rows = Vec::new();
    for item in list {
        let map = item
            .as_object()
            .ok_or_else(|| BoardError::Provider("workflow state is not an object".into()))?;
        let id = str_field(map, "id")
            .ok_or_else(|| BoardError::Provider("workflow state has no id".into()))?;
        let name = str_field(map, "name")
            .ok_or_else(|| BoardError::Provider("workflow state has no name".into()))?;
        // A state that names another team is not ours. Absent is accepted:
        // the request itself was scoped to the team.
        if nested_id(map, "team", "teamId").is_some_and(|t| t != team) {
            continue;
        }
        rows.push(Row {
            id,
            name,
            position: map.get("position").and_then(Value::as_f64),
            rank: str_field(map, "type").and_then(|t| type_rank(&t)),
            kind: str_field(map, "type")
                .map(|t| t.to_ascii_lowercase())
                .unwrap_or_default(),
        });
    }
    // Provider order: `position` when every state has it, else the state
    // `type` ladder, else the order the provider listed them.
    let by_position = rows.iter().all(|r| r.position.is_some());
    let by_rank = rows.iter().all(|r| r.rank.is_some());
    let mut states: Vec<WorkflowState> = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| WorkflowState {
            position: if by_position {
                row.position.unwrap_or_default()
            } else if by_rank {
                // Same-type states (for example two `started` columns) keep the
                // order the provider listed them.
                row.rank.unwrap_or_default() + (index as f64) / 1000.0
            } else {
                index as f64
            },
            id: row.id,
            name: row.name,
            kind: row.kind,
        })
        .collect();
    sort_states(&mut states);
    Ok(states)
}

fn type_rank(kind: &str) -> Option<f64> {
    Some(match kind.to_ascii_lowercase().as_str() {
        "triage" => 0.0,
        "backlog" => 1.0,
        "unstarted" => 2.0,
        "started" => 3.0,
        "completed" => 4.0,
        "canceled" | "cancelled" => 5.0,
        // Seen in a real capture: a closed state that is neither completed nor
        // canceled. It sorts with the other terminal states.
        "duplicate" => 6.0,
        _ => return None,
    })
}

fn parse_issue_page(text: &str) -> Result<(Vec<RawIssue>, Option<String>), BoardError> {
    let value = parse_json(text)?;
    let list = items(&value, &["issues"])?;
    let mut out = Vec::with_capacity(list.len());
    for item in list {
        let map = item
            .as_object()
            .ok_or_else(|| BoardError::Provider("issue is not an object".into()))?;
        let id =
            str_field(map, "id").ok_or_else(|| BoardError::Provider("issue has no id".into()))?;
        let (state_id, state_name) = match map.get("state").or_else(|| map.get("status")) {
            Some(Value::Object(state)) => (str_field(state, "id"), str_field(state, "name")),
            Some(Value::String(name)) => (str_field(map, "stateId"), Some(name.trim().to_string())),
            _ => (str_field(map, "stateId"), None),
        };
        out.push(RawIssue {
            identifier: str_field(map, "identifier").unwrap_or_else(|| id.clone()),
            title: str_field(map, "title").unwrap_or_default(),
            id,
            state_id,
            state_name,
            team: nested_id(map, "team", "teamId"),
            project: nested_id(map, "project", "projectId"),
        });
    }
    let next = next_cursor(&value, out.len())?;
    Ok((out, next))
}

/// The cursor of the next page, `None` only when the response positively says
/// this is the last page. A full page with no paging information at all, or
/// `hasNextPage` without a cursor, might be truncated: error.
fn next_cursor(value: &Value, count: usize) -> Result<Option<String>, BoardError> {
    let map = value.as_object();
    if let Some(page) = map
        .and_then(|m| m.get("pageInfo"))
        .and_then(Value::as_object)
    {
        return match page.get("hasNextPage").and_then(Value::as_bool) {
            Some(true) => str_field(page, "endCursor")
                .map(Some)
                .ok_or_else(|| BoardError::Provider("next page without cursor".into())),
            Some(false) => Ok(None),
            // `pageInfo` without a usable flag proves nothing about a full page.
            None if (count as u64) < PAGE_LIMIT => Ok(None),
            None => Err(BoardError::Provider(
                "paging information without hasNextPage on a full page; refusing a possibly truncated board".into(),
            )),
        };
    }
    if let Some(map) = map {
        // A top-level `hasNextPage` is an explicit end signal too.
        match map.get("hasNextPage").and_then(Value::as_bool) {
            Some(true) => {
                return ["nextCursor", "cursor", "endCursor"]
                    .iter()
                    .find_map(|key| str_field(map, key))
                    .map(Some)
                    .ok_or_else(|| BoardError::Provider("next page without cursor".into()));
            }
            Some(false) => return Ok(None),
            None => {}
        }
        if let Some(cursor) = ["nextCursor", "cursor"]
            .iter()
            .find_map(|key| str_field(map, key))
        {
            return Ok(Some(cursor));
        }
        // Only `nextCursor: null` is a positive end signal. A bare `cursor`
        // may merely echo the request, so a null one proves nothing.
        if map.contains_key("nextCursor") {
            return Ok(None);
        }
    }
    if count as u64 >= PAGE_LIMIT {
        return Err(BoardError::Provider(
            "full page without paging information; refusing a possibly truncated board".into(),
        ));
    }
    Ok(None)
}

#[cfg(test)]
#[path = "linear_tests.rs"]
mod tests;
