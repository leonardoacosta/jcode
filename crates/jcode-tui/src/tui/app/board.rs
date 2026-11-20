//! `/board`: the issue board of the active repository's verified world.
//!
//! The core (`crate::board`) resolves the world, enforces the binding, reads
//! through an allowlisted Linear provider and performs the one allowed write.
//! This module only connects it to the overlay: it fetches off the UI thread,
//! delivers results over a channel polled on every tick, and turns a confirmed
//! move into `execute` with an audit record written before dispatch.

use super::App;
use crate::board::{
    AuditLog, BoardCache, BoardError, BoardProvider, BoardScope, BoardSnapshot, ChangeOutcome,
    Confirmation, LinearMcpProvider, McpManagerCaller, ReconcileOutcome, StatusChangeRequest,
    execute, provider_for, reconcile, resolve_board,
};
use crate::tui::board_overlay::{
    BoardAction, BoardColumn, BoardData, BoardIssue, BoardLoad, MoveRequest, MoveVerdict,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

const CACHE_TTL: Duration = Duration::from_secs(60);
/// Text of `Refusal::WorldChanged`, used to recognise that outcome.
const WORLD_CHANGED_MARK: &str = "world changed before write";

/// Result of a background board task, tagged with the open it belongs to so a
/// late result can never land on a newer overlay or another world's board.
pub(super) struct BoardEvent {
    generation: u64,
    pub(super) kind: BoardEventKind,
}

pub(super) enum BoardEventKind {
    /// World resolved and provider built off the UI thread.
    Ready(Box<BoardRuntime>),
    /// A load or a move finished.
    Loaded {
        load: BoardLoad,
        /// One-row result of a move, shown after the board reloads.
        notice: Option<String>,
    },
    /// A move finished. `fresh` is the re-read board when that read worked.
    MoveDone {
        verdict: MoveVerdict,
        text: String,
        fresh: Option<BoardLoad>,
    },
}

/// Everything `/board` keeps between key presses. Built off the UI thread
/// because resolving the world may run git.
pub(super) struct BoardRuntime {
    scope: BoardScope,
    provider: Arc<dyn BoardProvider>,
    cache: Arc<BoardCache>,
    auth: String,
    cwd: PathBuf,
}

#[derive(Default)]
pub(super) struct BoardState {
    /// Bumped on every `/board`; events of older opens are dropped.
    generation: u64,
    runtime: Option<BoardRuntime>,
    events: Option<(Sender<BoardEvent>, Receiver<BoardEvent>)>,
}

pub(super) fn handle_board_command(app: &mut App, trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("/board") else {
        return false;
    };
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return false;
    }
    if !rest.trim().is_empty() {
        app.push_display_message(super::DisplayMessage::error("Usage: /board"));
        return true;
    }
    // No session working directory means no verified repository: no board.
    // The process cwd is never a substitute (a remote session's repository is
    // not this client's).
    let Some(cwd) = app.session.working_dir.clone().map(PathBuf::from) else {
        app.board.runtime = None;
        app.open_board_overlay(BoardLoad::Unavailable(
            "this session has no working directory, so its world cannot be verified".into(),
        ));
        return true;
    };
    let Ok(handle) = tokio::runtime::Handle::try_current() else {
        app.open_board_overlay(BoardLoad::Error("no async runtime".into()));
        return true;
    };
    // A fresh open: a new generation and channel so nothing from an earlier
    // open (even one still running) can reach this overlay.
    app.board.generation = app.board.generation.wrapping_add(1);
    app.board.runtime = None;
    app.board.events = Some(channel());
    let generation = app.board.generation;
    let tx = app.board_sender();
    app.open_board_overlay(BoardLoad::Loading);
    handle.spawn(async move {
        // resolve_board may run git and read config files: keep it off the UI
        // thread and off the async workers.
        let built = tokio::task::spawn_blocking(move || prepare_runtime(cwd)).await;
        let kind = match built {
            Ok(Ok(runtime)) => BoardEventKind::Ready(Box::new(runtime)),
            Ok(Err(load)) => BoardEventKind::Loaded { load, notice: None },
            Err(error) => BoardEventKind::Loaded {
                load: BoardLoad::Error(format!("board setup failed: {error}")),
                notice: None,
            },
        };
        let _ = tx.send(BoardEvent { generation, kind });
    });
    true
}

/// The board reads and writes through this MCP server, so a repository must
/// not be able to supply it: project-local MCP config can define any command
/// under any name. Only a server from the user's global config is used.
pub(super) fn check_board_server(
    scope: &BoardScope,
    config: &crate::mcp::McpConfig,
) -> Result<(), BoardLoad> {
    if scope.backend != crate::board::BoardBackend::Linear {
        return Ok(());
    }
    // The board's manager has no project directory, so a server limited to
    // `workspaces` would always read as out of scope. Say so instead.
    if config
        .servers
        .get(&scope.server)
        .is_some_and(|server| !server.workspaces.is_empty())
    {
        return Err(BoardLoad::Unavailable(format!(
            "MCP server '{}' is limited to specific workspaces; the board cannot use a workspace-scoped server",
            scope.server
        )));
    }
    match config.server_scope(&scope.server) {
        Some("global") => Ok(()),
        Some(other) => Err(BoardLoad::Unavailable(format!(
            "MCP server '{}' comes from {other} config; the board only uses a server from your global config",
            scope.server
        ))),
        None => Err(BoardLoad::Unavailable(format!(
            "MCP server '{}' is not configured",
            scope.server
        ))),
    }
}

/// Resolve the world and build the provider. Every refusal is returned as the
/// state to show; nothing is queried and no other binding is tried.
fn prepare_runtime(cwd: PathBuf) -> Result<BoardRuntime, BoardLoad> {
    let scope = resolve_board(&cwd).map_err(|why| BoardLoad::Unavailable(why.to_string()))?;
    // One config load feeds both the provider and the cache identity, so the
    // identity always describes the server actually used.
    let config = crate::mcp::McpConfig::load_for_dir(Some(&cwd));
    check_board_server(&scope, &config)?;
    let auth = match config.servers.get(&scope.server) {
        Some(server) => crate::board::auth_identity(&scope.server, server),
        None => format!("unconfigured:{}", scope.server),
    };
    let manager = Arc::new(crate::mcp::McpManager::with_config(config));
    let linear = Arc::new(LinearMcpProvider::new(Arc::new(McpManagerCaller::new(
        manager,
    ))));
    let provider = provider_for(&scope, linear);
    Ok(BoardRuntime {
        scope,
        provider,
        cache: Arc::new(BoardCache::new(CACHE_TTL)),
        auth,
        cwd,
    })
}

/// Columns in provider order. An issue whose state is not a known column goes
/// to a trailing "Other" column so nothing silently disappears.
pub(super) fn to_board_load(scope: &BoardScope, snapshot: &BoardSnapshot) -> BoardLoad {
    if snapshot.issues.is_empty() {
        return BoardLoad::Empty;
    }
    let mut columns: Vec<BoardColumn> = snapshot
        .states
        .iter()
        .map(|state| BoardColumn {
            id: state.id.clone(),
            name: state.name.clone(),
            kind: state.kind.clone(),
            issues: Vec::new(),
        })
        .collect();
    let mut other = Vec::new();
    for issue in &snapshot.issues {
        let card = BoardIssue {
            id: issue.id.clone(),
            identifier: issue.identifier.clone(),
            title: issue.title.clone(),
        };
        match columns.iter_mut().find(|c| c.id == issue.state_id) {
            Some(column) => column.issues.push(card),
            None => other.push(card),
        }
    }
    if !other.is_empty() {
        columns.push(BoardColumn {
            id: String::new(),
            name: "Other".into(),
            kind: String::new(),
            issues: other,
        });
    }
    BoardLoad::Ready(BoardData {
        scope_label: scope_label(scope, snapshot),
        read_only: scope.read_only(),
        columns,
    })
}

/// `Linear · Priceless (PRICE) · team-wide` when the issues carry the team's
/// display name, else the id-based [`BoardScope::scope_label`]. The key is the
/// identifier prefix, shown only when every issue shares it.
pub(super) fn scope_label(scope: &BoardScope, snapshot: &BoardSnapshot) -> String {
    let Some(name) = snapshot.issues.iter().find_map(|i| i.team_name.clone()) else {
        return scope.scope_label();
    };
    let mut prefixes = snapshot
        .issues
        .iter()
        .map(|i| i.identifier.split_once('-').map(|(key, _)| key));
    let key = match prefixes.next().flatten() {
        Some(first) if !first.is_empty() && prefixes.all(|p| p == Some(first)) => Some(first),
        _ => None,
    };
    let team = match key {
        Some(key) if key != name => format!("{name} ({key})"),
        _ => name,
    };
    let where_ = match &scope.project {
        Some(project) => format!("project {project}"),
        None => "team-wide".to_string(),
    };
    format!("{} · {team} · {where_}", scope.backend.label())
}

pub(super) fn error_to_load(error: &BoardError) -> BoardLoad {
    match error {
        BoardError::Unauthorized(text) => BoardLoad::Unauthorized(text.clone()),
        BoardError::Unavailable(text) => BoardLoad::Unavailable(text.clone()),
        BoardError::Provider(text) | BoardError::Uncertain(text) => BoardLoad::Error(text.clone()),
    }
}

/// One-row result text for a finished move.
pub(super) fn describe_change(outcome: &ChangeOutcome) -> String {
    match outcome {
        ChangeOutcome::Refused(why) => format!("not moved: {why}"),
        ChangeOutcome::ReadFailed(error) => format!("not moved: cannot read live state: {error}"),
        ChangeOutcome::Conflict { .. } => {
            "not moved: the issue changed remotely; board refreshed".into()
        }
        ChangeOutcome::AuditFailed(error) => {
            format!("not moved: audit log unavailable ({error})")
        }
        ChangeOutcome::Applied {
            outcome_unrecorded, ..
        } => match outcome_unrecorded {
            None => "moved".into(),
            Some(error) => format!("moved; audit outcome not recorded ({error})"),
        },
        ChangeOutcome::Failed { error, .. } => format!("not moved: {error}"),
        ChangeOutcome::Uncertain { error, .. } => format!("move result unknown: {error}"),
    }
}

/// [`run_move_verdict`] without the verdict, for tests that only read the text.
#[cfg(test)]
pub(super) async fn run_move(
    request: &StatusChangeRequest,
    provider: &dyn BoardProvider,
    audit: &AuditLog,
    current_scope: &(dyn Fn() -> Result<BoardScope, crate::board::BoardUnavailable> + Sync),
) -> String {
    run_move_verdict(request, provider, audit, current_scope)
        .await
        .1
}

/// Run one confirmed move to completion: execute, then, if the result is
/// unknown, one read-only reconcile that records what really happened. Never
/// retries the write. The verdict tells the overlay whether to keep or undo
/// the optimistic card.
pub(super) async fn run_move_verdict(
    request: &StatusChangeRequest,
    provider: &dyn BoardProvider,
    audit: &AuditLog,
    current_scope: &(dyn Fn() -> Result<BoardScope, crate::board::BoardUnavailable> + Sync),
) -> (MoveVerdict, String) {
    // The synthetic "Other" column (empty id) holds issues in a state the board
    // has no column for. It is display-only: moving from it has no verifiable
    // prior state and moving to it names no state, so neither is ever sent.
    if request.expected_prior_state.is_empty() || request.requested_state.is_empty() {
        return (
            MoveVerdict::NotMoved,
            "not moved: the Other column is display-only".into(),
        );
    }
    // Confirmation::confirm refuses read-only scopes, so an ADO move can never
    // reach a provider or the audit log.
    let confirmation = match Confirmation::confirm(request) {
        Ok(confirmation) => confirmation,
        Err(why) => return (MoveVerdict::NotMoved, format!("not moved: {why}")),
    };
    let outcome = execute(request, confirmation, provider, audit, current_scope).await;
    let mut text = describe_change(&outcome);
    let mut verdict = match &outcome {
        ChangeOutcome::Applied { .. } => MoveVerdict::Moved,
        ChangeOutcome::Uncertain { .. } => MoveVerdict::Unknown,
        _ => MoveVerdict::NotMoved,
    };
    if let ChangeOutcome::Uncertain { pending, .. } = &outcome {
        let (result, audit_error) = reconcile(pending, provider, audit).await;
        verdict = match &result {
            ReconcileOutcome::Applied => MoveVerdict::Moved,
            _ => MoveVerdict::Unknown,
        };
        text = match result {
            ReconcileOutcome::Applied => "moved (confirmed by re-reading the issue)".into(),
            ReconcileOutcome::NotApplied => {
                "not applied yet (re-read shows no change; it may still land)".into()
            }
            ReconcileOutcome::Diverged { .. } => {
                "move result unknown: the issue is now in another state".into()
            }
            ReconcileOutcome::Unknown(error) => format!("move result unknown: {error}"),
        };
        if let Some(error) = audit_error {
            text.push_str(&format!("; audit outcome not recorded ({error})"));
        }
    }
    (verdict, text)
}

impl App {
    /// Fetch the board off the UI thread and deliver it on the next tick.
    fn spawn_board_load(&mut self, refresh: bool) {
        let Some(runtime) = self.board.runtime.as_ref() else {
            return;
        };
        let (scope, provider, cache, auth) = (
            runtime.scope.clone(),
            Arc::clone(&runtime.provider),
            Arc::clone(&runtime.cache),
            runtime.auth.clone(),
        );
        let generation = self.board.generation;
        let tx = self.board_sender();
        tokio::spawn(async move {
            let load = match cache.load(&scope, &auth, provider.as_ref(), refresh).await {
                Ok(snapshot) => to_board_load(&scope, &snapshot),
                Err(error) => error_to_load(&error),
            };
            let _ = tx.send(BoardEvent {
                generation,
                kind: BoardEventKind::Loaded { load, notice: None },
            });
        });
    }

    fn board_sender(&mut self) -> Sender<BoardEvent> {
        self.board.events.get_or_insert_with(channel).0.clone()
    }

    /// Apply finished background work and act on overlay requests. Cheap when
    /// the board was never opened.
    pub(crate) fn poll_board(&mut self) -> bool {
        if self.board_overlay.is_none() {
            // Closed: drop queued results and the runtime so a late result
            // never reopens the board or leaks into another world.
            self.board.runtime = None;
            self.board.events = None;
            return false;
        }
        let mut changed = false;
        while let Some(event) = self
            .board
            .events
            .as_ref()
            .and_then(|(_, rx)| rx.try_recv().ok())
        {
            if event.generation != self.board.generation {
                continue;
            }
            match event.kind {
                BoardEventKind::Ready(runtime) => {
                    self.board.runtime = Some(*runtime);
                    self.spawn_board_load(false);
                }
                BoardEventKind::MoveDone {
                    verdict,
                    text,
                    fresh,
                } => {
                    if let Some(cell) = self.board_overlay.as_ref() {
                        cell.borrow_mut().finish_move(verdict, &text, fresh);
                    }
                }
                BoardEventKind::Loaded { load, notice } => {
                    self.update_board_overlay(load);
                    if let (Some(notice), Some(cell)) = (notice, self.board_overlay.as_ref()) {
                        cell.borrow_mut().set_notice(notice);
                    }
                }
            }
            changed = true;
        }
        match self.take_board_action() {
            Some(BoardAction::Refresh) => {
                if self.board.runtime.is_some() {
                    self.update_board_overlay(BoardLoad::Loading);
                    self.spawn_board_load(true);
                    changed = true;
                }
            }
            Some(BoardAction::Confirmed(request)) => {
                // Without a runtime there is nothing to write through; never
                // leave the overlay on Loading.
                // The overlay already shows the card in its new status; the
                // result either keeps it or undoes it.
                if self.board.runtime.is_some() {
                    self.spawn_board_move(request);
                } else if let Some(cell) = self.board_overlay.as_ref() {
                    cell.borrow_mut().finish_move(
                        MoveVerdict::NotMoved,
                        "board is not ready; nothing was moved",
                        None,
                    );
                }
                changed = true;
            }
            Some(BoardAction::None | BoardAction::Close) | None => {}
        }
        changed
    }

    fn spawn_board_move(&mut self, request: MoveRequest) {
        let Some(runtime) = self.board.runtime.as_ref() else {
            return;
        };
        let change = StatusChangeRequest {
            scope: runtime.scope.clone(),
            issue_id: request.issue_id,
            expected_prior_state: request.from_column_id,
            requested_state: request.to_column_id,
        };
        let (provider, cache, auth, cwd, scope) = (
            Arc::clone(&runtime.provider),
            Arc::clone(&runtime.cache),
            runtime.auth.clone(),
            runtime.cwd.clone(),
            runtime.scope.clone(),
        );
        let generation = self.board.generation;
        let tx = self.board_sender();
        tokio::spawn(async move {
            // `execute` re-resolves the world twice: before the live read and
            // again right before dispatch. Resolving may run git, so the whole
            // move runs on a blocking thread, where the closure can call the
            // resolver directly and block_on drives the async provider calls.
            let (verdict, text) = match AuditLog::open_default() {
                Ok(audit) => {
                    let handle = tokio::runtime::Handle::current();
                    let (provider, cwd) = (Arc::clone(&provider), cwd.clone());
                    tokio::task::spawn_blocking(move || {
                        let current = || resolve_board(&cwd);
                        handle.block_on(run_move_verdict(
                            &change,
                            provider.as_ref(),
                            &audit,
                            &current,
                        ))
                    })
                    .await
                    .unwrap_or_else(|error| {
                        (
                            MoveVerdict::Unknown,
                            format!("board task failed ({error}); the move may have happened"),
                        )
                    })
                }
                Err(error) => (
                    MoveVerdict::NotMoved,
                    format!("not moved: audit log unavailable ({error})"),
                ),
            };
            // After a world change the open-time binding no longer describes
            // this repository: do not read through it again.
            let fresh = if text.contains(WORLD_CHANGED_MARK) {
                Some(BoardLoad::Unavailable(
                    "the repository's world changed; reopen /board to use its current binding"
                        .into(),
                ))
            } else {
                // A failed re-read is not shown as the board: the overlay
                // falls back to the verdict so the board stays usable.
                cache
                    .load(&scope, &auth, provider.as_ref(), true)
                    .await
                    .ok()
                    .map(|snapshot| to_board_load(&scope, &snapshot))
            };
            let _ = tx.send(BoardEvent {
                generation,
                kind: BoardEventKind::MoveDone {
                    verdict,
                    text,
                    fresh,
                },
            });
        });
    }
}

#[cfg(test)]
#[path = "board_tests.rs"]
mod board_tests;
