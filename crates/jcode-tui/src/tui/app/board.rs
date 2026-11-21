//! `/board`: the issue board of the active repository's verified world.
//!
//! The core (`crate::board`) resolves the world, enforces the binding, reads
//! through an allowlisted Linear provider and performs the one allowed write.
//! This module only connects it to the overlay: it fetches off the UI thread,
//! delivers results over a channel polled on every tick, and turns a confirmed
//! move into `execute` with an audit record written before dispatch.

use super::App;
use crate::board::{
    AuditLog, BoardAccount, BoardBackend, BoardCache, BoardError, BoardProvider, BoardScope,
    BoardSnapshot, BoardUnavailable, ChangeOutcome, Confirmation, LinearMcpProvider,
    McpManagerCaller, ReconcileOutcome, ScopeOrigin, StatusChangeRequest, Team, board_accounts,
    execute, provider_for, reconcile, resolve_board, resolve_world,
};
use crate::tui::board_overlay::{
    BoardAction, BoardColumn, BoardData, BoardIssue, BoardLoad, MoveRequest, MoveVerdict,
};
use std::path::{Path, PathBuf};
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
pub(super) enum BoardRuntime {
    /// One world-bound (or named) board.
    One(OneBoard),
    /// Read-only view over every configured account.
    All(Arc<AllBoards>),
}

pub(super) struct OneBoard {
    scope: BoardScope,
    provider: Arc<dyn BoardProvider>,
    cache: Arc<BoardCache>,
    auth: String,
    /// Directory the world was verified from. `None` for a board opened by
    /// name, which re-verifies by name instead.
    cwd: Option<PathBuf>,
}

/// One account of the all-accounts view.
pub(super) struct AllAccount {
    server: String,
    auth: String,
    /// Why the account cannot be read (server missing, not global, ...).
    gate: Result<(), String>,
    /// Configured scopes of the worlds bound to this account.
    bindings: Vec<BoardScope>,
}

pub(super) struct AllBoards {
    accounts: Vec<AllAccount>,
    provider: Arc<LinearMcpProvider>,
    cache: BoardCache,
}

#[derive(Default)]
pub(super) struct BoardState {
    /// Bumped on every `/board`; events of older opens are dropped.
    generation: u64,
    runtime: Option<BoardRuntime>,
    events: Option<(Sender<BoardEvent>, Receiver<BoardEvent>)>,
}

/// What `/board ...` asks for.
#[derive(Debug, PartialEq, Eq)]
enum BoardCommand<'a> {
    /// Open the board: the cwd's world, or a named target.
    Open(Option<&'a str>),
    Account(AccountAction, Option<&'a str>),
    Usage,
}

const USAGE: &str = "Usage: /board [world[/team[/project]]] | /board login [account|world] | /board logout [account|world]";

fn parse_board_command(rest: &str) -> BoardCommand<'_> {
    let words: Vec<&str> = rest.split_whitespace().collect();
    match words.as_slice() {
        [] => BoardCommand::Open(None),
        ["login"] => BoardCommand::Account(AccountAction::Login, None),
        ["login", name] => BoardCommand::Account(AccountAction::Login, Some(name)),
        ["logout"] => BoardCommand::Account(AccountAction::Logout, None),
        ["logout", name] => BoardCommand::Account(AccountAction::Logout, Some(name)),
        [target] => BoardCommand::Open(Some(target)),
        _ => BoardCommand::Usage,
    }
}

pub(super) fn handle_board_command(app: &mut App, trimmed: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix("/board") else {
        return false;
    };
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return false;
    }
    let target = match parse_board_command(rest) {
        BoardCommand::Open(target) => target.map(str::to_string),
        BoardCommand::Account(action, name) => {
            board_account(app, action, name);
            return true;
        }
        BoardCommand::Usage => {
            app.push_display_message(super::DisplayMessage::error(USAGE));
            return true;
        }
    };
    // The session working directory is the only directory that counts: the
    // process cwd is never a substitute (a remote session's repository is
    // not this client's). Without one there is no world to verify, so the
    // board falls back to the read-only all-accounts view.
    let cwd = app.session.working_dir.clone().map(PathBuf::from);
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
        let built = tokio::task::spawn_blocking(move || prepare_runtime(cwd, target)).await;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountAction {
    Login,
    Logout,
}

/// `/board login|logout [name]`. `name` is an account (an MCP server) or a
/// world. Without a name, a world directory acts on its own world's account;
/// anywhere else login starts for every account without a token, and logout
/// asks for a name.
fn board_account(app: &mut App, action: AccountAction, name: Option<&str>) {
    let error = |app: &mut App, text: String| {
        app.push_display_message(super::DisplayMessage::error(text));
    };
    let cwd = app.session.working_dir.clone().map(PathBuf::from);
    // Runs on the UI thread: an explicit one-shot command, and resolving is a
    // few local git reads.
    let (server, world) = match name {
        Some(name) => match account_or_world(name) {
            Ok(found) => found,
            Err(why) => return error(app, format!("No board account: {why}.")),
        },
        None => match cwd.as_deref().map(resolve_board) {
            Some(Ok(scope)) => match linear_server(&scope) {
                Ok(server) => (server, Some(scope.world)),
                Err(why) => return error(app, why),
            },
            Some(Err(BoardUnavailable::UnknownWorld)) | None => {
                return outside_world_accounts(app, action, cwd.as_deref());
            }
            Some(Err(why)) => return error(app, format!("No board account: {why}.")),
        },
    };
    let config = crate::mcp::McpConfig::load_for_dir(cwd.as_deref());
    login_server(app, &config, action, &server, world.as_deref());
}

/// The Linear server of a scope; ADO boards have no `/board login`.
fn linear_server(scope: &BoardScope) -> Result<String, String> {
    if scope.backend != BoardBackend::Linear {
        return Err(format!(
            "The {} board has no /board login; it uses its own credentials.",
            scope.backend.label()
        ));
    }
    Ok(scope.server.clone())
}

/// An account (server) name, else a world name, as `(server, world)`.
fn account_or_world(name: &str) -> Result<(String, Option<String>), String> {
    let accounts = board_accounts().map_err(|why| why.to_string());
    if let Ok(accounts) = &accounts
        && let Some(account) = accounts.iter().find(|a| a.server == name)
    {
        return Ok((account.server.clone(), None));
    }
    match resolve_world(name) {
        Ok(scope) => linear_server(&scope).map(|server| (server, Some(scope.world))),
        Err(BoardUnavailable::UnknownWorldName { .. }) => Err(match accounts {
            Ok(_) => format!("'{name}' is neither a board account nor a configured world"),
            Err(why) => why,
        }),
        Err(why) => Err(why.to_string()),
    }
}

/// No verified world: every account without a token gets a login, one after
/// another. Logout has no safe default here, so it lists the accounts.
fn outside_world_accounts(app: &mut App, action: AccountAction, cwd: Option<&Path>) {
    let accounts = match board_accounts() {
        Ok(accounts) if !accounts.is_empty() => accounts,
        Ok(_) => {
            return app.push_display_message(super::DisplayMessage::error(
                "No board accounts: this directory is not in a configured world and no board account is configured.",
            ));
        }
        Err(why) => {
            return app.push_display_message(super::DisplayMessage::error(format!(
                "No board account: {why}."
            )));
        }
    };
    let names: Vec<&str> = accounts.iter().map(|a| a.server.as_str()).collect();
    if action == AccountAction::Logout {
        return app.push_display_message(super::DisplayMessage::error(format!(
            "Name the account to log out: /board logout <account>. Accounts: {}.",
            names.join(", ")
        )));
    }
    let config = crate::mcp::McpConfig::load_for_dir(cwd);
    let mut started = 0;
    for account in &accounts {
        let signed_in = config
            .servers
            .get(&account.server)
            .and_then(|s| s.url.as_deref().map(|url| (&account.server, url)))
            .is_some_and(|(server, url)| {
                crate::mcp::oauth::status(server, url).is_ok_and(|status| status.logged_in)
            });
        if signed_in {
            continue;
        }
        started += 1;
        login_server(app, &config, action, &account.server, None);
    }
    if started == 0 {
        app.push_display_message(super::DisplayMessage::system(format!(
            "All board accounts already have credentials: {}. Use /board login <account> to sign in again.",
            names.join(", ")
        )));
    }
}

/// Log one board account in or out. A loaded runtime holds a connection with
/// the old credentials, so it is dropped.
fn login_server(
    app: &mut App,
    config: &crate::mcp::McpConfig,
    action: AccountAction,
    name: &str,
    world: Option<&str>,
) {
    if let Err(load) = check_server(name, config) {
        let why = match load {
            BoardLoad::Unavailable(why) => why,
            other => format!("{other:?}"),
        };
        return app.push_display_message(super::DisplayMessage::error(format!(
            "No board account: {why}."
        )));
    }
    if !config.servers[name].is_remote() {
        return app.push_display_message(super::DisplayMessage::error(format!(
            "Board account '{name}' runs a local command with its own credentials; /board login applies to remote OAuth servers. Set its \"url\" to https://mcp.linear.app/mcp to sign in here."
        )));
    }
    if config.servers[name]
        .headers
        .keys()
        .any(|key| key.eq_ignore_ascii_case("authorization"))
    {
        return app.push_display_message(super::DisplayMessage::error(format!(
            "Board account '{name}' has a static Authorization header, which takes precedence over OAuth. Remove it to use /board login."
        )));
    }
    app.board.runtime = None;
    match action {
        AccountAction::Login => {
            if let Some(world) = world {
                app.push_display_message(super::DisplayMessage::system(format!(
                    "Board account for world '{world}': MCP server '{name}'. After signing in, run /board again."
                )));
            }
            super::commands_mcp::start_login(app, config, name);
        }
        AccountAction::Logout => super::commands_mcp::logout(app, config, name),
    }
}

/// The board reads and writes through this MCP server, so a repository must
/// not be able to supply it: project-local MCP config can define any command
/// under any name. Only a server from the user's global config is used.
pub(super) fn check_board_server(
    scope: &BoardScope,
    config: &crate::mcp::McpConfig,
) -> Result<(), BoardLoad> {
    if scope.backend != BoardBackend::Linear {
        return Ok(());
    }
    check_server(&scope.server, config)
}

fn check_server(server: &str, config: &crate::mcp::McpConfig) -> Result<(), BoardLoad> {
    // The board's manager has no project directory, so a server limited to
    // `workspaces` would always read as out of scope. Say so instead.
    if config
        .servers
        .get(server)
        .is_some_and(|server| !server.workspaces.is_empty())
    {
        return Err(BoardLoad::Unavailable(format!(
            "MCP server '{server}' is limited to specific workspaces; the board cannot use a workspace-scoped server"
        )));
    }
    match config.server_scope(server) {
        Some("global") => Ok(()),
        Some(other) => Err(BoardLoad::Unavailable(format!(
            "MCP server '{server}' comes from {other} config; the board only uses a server from your global config"
        ))),
        None => Err(BoardLoad::Unavailable(format!(
            "MCP server '{server}' is not configured"
        ))),
    }
}

/// Resolve what to show and build the provider. Every refusal is returned as
/// the state to show; nothing is queried and no other binding is tried.
///
/// A `target` is resolved by name and never from `cwd`. Otherwise the cwd's
/// world is used; a directory in no world (or no directory at all) gets the
/// read-only all-accounts view. Any other refusal (identity, ambiguity,
/// Brown/Azure) is shown as is.
fn prepare_runtime(
    cwd: Option<PathBuf>,
    target: Option<String>,
) -> Result<BoardRuntime, BoardLoad> {
    let unavailable = |why: BoardUnavailable| BoardLoad::Unavailable(why.to_string());
    if let Some(target) = target {
        let scope = resolve_world(&target).map_err(unavailable)?;
        return one_board(scope, None);
    }
    match cwd.as_deref().map(resolve_board) {
        Some(Ok(scope)) => one_board(scope, cwd),
        Some(Err(BoardUnavailable::UnknownWorld)) | None => all_boards(cwd.as_deref()),
        Some(Err(why)) => Err(unavailable(why)),
    }
}

fn one_board(scope: BoardScope, cwd: Option<PathBuf>) -> Result<BoardRuntime, BoardLoad> {
    // One config load feeds both the provider and the cache identity, so the
    // identity always describes the server actually used.
    let config = crate::mcp::McpConfig::load_for_dir(cwd.as_deref());
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
    Ok(BoardRuntime::One(OneBoard {
        scope,
        provider,
        cache: Arc::new(BoardCache::new(CACHE_TTL)),
        auth,
        cwd,
    }))
}

fn all_boards(cwd: Option<&Path>) -> Result<BoardRuntime, BoardLoad> {
    let accounts = board_accounts().map_err(|why| BoardLoad::Unavailable(why.to_string()))?;
    if accounts.is_empty() {
        return Err(BoardLoad::Unavailable(
            "this directory is not in a configured world and no board account is configured".into(),
        ));
    }
    let config = crate::mcp::McpConfig::load_for_dir(cwd);
    let accounts = accounts
        .into_iter()
        .map(|account: BoardAccount| AllAccount {
            auth: match config.servers.get(&account.server) {
                Some(server) => crate::board::auth_identity(&account.server, server),
                None => format!("unconfigured:{}", account.server),
            },
            gate: check_server(&account.server, &config).map_err(|load| match load {
                BoardLoad::Unavailable(why) => why,
                other => format!("{other:?}"),
            }),
            bindings: account
                .worlds
                .iter()
                .filter_map(|world| resolve_world(world).ok())
                .filter(|scope| scope.backend == BoardBackend::Linear)
                .collect(),
            server: account.server,
        })
        .collect();
    let manager = Arc::new(crate::mcp::McpManager::with_config(config));
    let provider = Arc::new(LinearMcpProvider::new(Arc::new(McpManagerCaller::new(
        manager,
    ))));
    Ok(BoardRuntime::All(Arc::new(AllBoards {
        accounts,
        provider,
        cache: BoardCache::new(CACHE_TTL),
    })))
}

const ALL_LABEL: &str = "All accounts · read-only";
/// Column order of the all-accounts view. Other kinds (triage, duplicate,
/// unknown) land in a trailing "Other" column.
const ALL_KINDS: [(&str, &str); 5] = [
    ("backlog", "Backlog"),
    ("unstarted", "Unstarted"),
    ("started", "Started"),
    ("completed", "Completed"),
    ("canceled", "Canceled"),
];

/// One readable team of the all-accounts view.
struct Part {
    snapshot: Arc<BoardSnapshot>,
    /// Team key (`PRICE`), empty when the provider gave none.
    key: String,
}

/// Read every account's teams. A failing account adds a notice and never
/// hides the others. Returns the board and the notice to show over it.
async fn load_all(all: &AllBoards, refresh: bool) -> (BoardLoad, Option<String>) {
    let per_account = futures::future::join_all(
        all.accounts
            .iter()
            .map(|account| load_account(all, account, refresh)),
    )
    .await;
    let mut parts = Vec::new();
    let mut notices = Vec::new();
    for (account_parts, account_notices) in per_account {
        parts.push(account_parts);
        notices.extend(account_notices);
    }
    let multi = all.accounts.len() > 1;
    let mut columns: Vec<BoardColumn> = ALL_KINDS
        .iter()
        .map(|(kind, name)| BoardColumn {
            id: (*kind).into(),
            name: (*name).into(),
            kind: (*kind).into(),
            issues: Vec::new(),
        })
        .collect();
    let mut other = Vec::new();
    let mut total = 0;
    for (account, account_parts) in all.accounts.iter().zip(&parts) {
        for part in account_parts {
            for issue in &part.snapshot.issues {
                total += 1;
                let kind = part
                    .snapshot
                    .states
                    .iter()
                    .find(|s| s.id == issue.state_id)
                    .map(|s| s.kind.as_str())
                    .unwrap_or("");
                let card = BoardIssue {
                    id: issue.id.clone(),
                    // Normally `KEY-12` already; label the card when it is not.
                    identifier: match part.key.as_str() {
                        key if key.is_empty()
                            || issue.identifier.starts_with(&format!("{key}-")) =>
                        {
                            issue.identifier.clone()
                        }
                        key => format!("{key} {}", issue.identifier),
                    },
                    title: if multi {
                        format!("[{}] {}", account.server, issue.title)
                    } else {
                        issue.title.clone()
                    },
                };
                match columns.iter_mut().find(|c| c.kind == kind) {
                    Some(column) => column.issues.push(card),
                    None => other.push(card),
                }
            }
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
    let notice = (!notices.is_empty()).then(|| notices.join("; "));
    if total > 0 {
        let load = BoardLoad::Ready(BoardData {
            scope_label: ALL_LABEL.into(),
            read_only: true,
            columns,
        });
        return (load, notice);
    }
    let load = match notice {
        None => BoardLoad::Empty,
        Some(text) if notices.iter().all(|n| n.contains("run /board login")) => {
            BoardLoad::Unauthorized(text)
        }
        Some(text) => BoardLoad::Error(text),
    };
    (load, None)
}

async fn load_account(
    all: &AllBoards,
    account: &AllAccount,
    refresh: bool,
) -> (Vec<Part>, Vec<String>) {
    let server = account.server.as_str();
    let mut notices = Vec::new();
    if let Err(why) = &account.gate {
        notices.push(format!("{server}: {why}"));
        return (Vec::new(), notices);
    }
    let teams: Vec<Team> = match all.provider.teams(server).await {
        Ok(teams) => teams,
        Err(error) => {
            notices.push(account_notice(server, &error));
            return (Vec::new(), notices);
        }
    };
    let loads = teams.iter().map(|team| async move {
        // A team a world binds keeps its configured scope; every other team
        // is discovered, so nothing shows twice and nothing unbound is writable.
        // A project-bound binding would hide the rest of the team, so only a
        // team-wide binding is reused.
        let scope = match account
            .bindings
            .iter()
            .find(|b| b.team == team.id && b.project.is_none())
        {
            Some(binding) => binding.clone(),
            None => BoardScope {
                origin: ScopeOrigin::Discovered,
                world: String::new(),
                backend: BoardBackend::Linear,
                team: team.id.clone(),
                project: None,
                server: server.to_string(),
                canonical_root: None,
            },
        };
        let loaded = all
            .cache
            .load(&scope, &account.auth, all.provider.as_ref(), refresh)
            .await;
        (team, loaded)
    });
    let mut parts = Vec::new();
    for (team, loaded) in futures::future::join_all(loads).await {
        match loaded {
            Ok(snapshot) => parts.push(Part {
                snapshot,
                key: team.key.clone(),
            }),
            Err(error) => {
                let name = if team.key.is_empty() {
                    &team.name
                } else {
                    &team.key
                };
                notices.push(match error {
                    BoardError::Unauthorized(_) => account_notice(server, &error),
                    other => format!("{server}/{name}: {other}"),
                });
            }
        }
    }
    (parts, notices)
}

fn account_notice(server: &str, error: &BoardError) -> String {
    match error {
        BoardError::Unauthorized(_) => format!("{server}: run /board login {server}"),
        other => format!("{server}: {other}"),
    }
}

/// Re-verify the scope right before a write. A scope with a repository root
/// is verified from the directory; one opened by name has no root and is
/// re-resolved by `world[/team[/project]]`.
fn current_scope(
    cwd: Option<&Path>,
    expected: &BoardScope,
) -> Result<BoardScope, BoardUnavailable> {
    match (&expected.canonical_root, cwd) {
        (Some(_), Some(cwd)) => resolve_board(cwd),
        (Some(_), None) => Err(BoardUnavailable::UnknownWorld),
        (None, _) => {
            let mut target = expected.world.clone();
            target.push('/');
            target.push_str(&expected.team);
            if let Some(project) = &expected.project {
                target.push('/');
                target.push_str(project);
            }
            resolve_world(&target)
        }
    }
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
        return format!("{}{}", scope.scope_label(), account_suffix(scope));
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
    format!(
        "{} · {team} · {where_}{}",
        scope.backend.label(),
        account_suffix(scope)
    )
}

/// ` · via <server>` when the board is bound to a non-default MCP server, so
/// the user can see which account the board reads through.
fn account_suffix(scope: &BoardScope) -> String {
    if scope.server == jcode_config_types::DEFAULT_BOARD_SERVER {
        String::new()
    } else {
        format!(" · via {}", scope.server)
    }
}

pub(super) fn error_to_load(error: &BoardError) -> BoardLoad {
    match error {
        // The hint leads: the row truncates, and the raw error can be long.
        BoardError::Unauthorized(text) => {
            BoardLoad::Unauthorized(format!("run /board login · {text}"))
        }
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
        let generation = self.board.generation;
        let tx = self.board_sender();
        match self.board.runtime.as_ref() {
            Some(BoardRuntime::One(runtime)) => {
                let (scope, provider, cache, auth) = (
                    runtime.scope.clone(),
                    Arc::clone(&runtime.provider),
                    Arc::clone(&runtime.cache),
                    runtime.auth.clone(),
                );
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
            Some(BoardRuntime::All(all)) => {
                let all = Arc::clone(all);
                tokio::spawn(async move {
                    let (load, notice) = load_all(&all, refresh).await;
                    let _ = tx.send(BoardEvent {
                        generation,
                        kind: BoardEventKind::Loaded { load, notice },
                    });
                });
            }
            None => {}
        }
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
                if matches!(self.board.runtime, Some(BoardRuntime::One(_))) {
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
        // The all-accounts view is read-only and never offers a confirmation.
        let Some(BoardRuntime::One(runtime)) = self.board.runtime.as_ref() else {
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
                    let (provider, cwd, expected) =
                        (Arc::clone(&provider), cwd.clone(), change.scope.clone());
                    tokio::task::spawn_blocking(move || {
                        let current = || current_scope(cwd.as_deref(), &expected);
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
