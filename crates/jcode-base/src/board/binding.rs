//! Resolve the issue board of the active repository's verified world.
//!
//! The binding comes only from the user's global `[[workspaces.rules]]`
//! (`BoardBinding` on the rule, see `jcode_config_types`). A project checkout
//! cannot add one. Every refusal is a [`BoardUnavailable`]; callers must make
//! no provider call and must not retry with another rule, a last-used scope
//! or a user-global default.

use crate::workspace::{self, Identity, RepoFacts};
use jcode_config_types::{BoardBackend, BoardBinding, Tracker, WorkspaceRule, WorkspacesConfig};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// How a scope was obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeOrigin {
    /// A configured world binding (world + team, optionally project).
    Configured { world: String },
    /// Reachable on a configured account's server but not bound by any
    /// world rule. Read-only: never written.
    Discovered,
}

/// A verified world plus the explicit board binding it owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardScope {
    /// Where the scope came from. `Discovered` scopes are read-only.
    pub origin: ScopeOrigin,
    /// Workspace rule name (verified world). Empty for `Discovered`.
    pub world: String,
    pub backend: BoardBackend,
    /// Stable team id. Never a display name.
    pub team: String,
    /// Stable project id narrowing the team board, if configured.
    pub project: Option<String>,
    /// MCP server name that carries the backend's tools.
    pub server: String,
    /// Canonical repository root the world was verified from. `None` when the
    /// scope was resolved by name, not from a directory.
    pub canonical_root: Option<PathBuf>,
}

impl BoardScope {
    /// ADO boards and discovered (unconfigured) scopes are read-only.
    pub fn read_only(&self) -> bool {
        self.backend == BoardBackend::Ado || self.origin == ScopeOrigin::Discovered
    }

    pub fn is_discovered(&self) -> bool {
        self.origin == ScopeOrigin::Discovered
    }

    /// One-row scope label: `Linear · <team> · team-wide` or
    /// `Linear · <team> · project <id>`.
    pub fn scope_label(&self) -> String {
        let scope = match &self.project {
            Some(project) => format!("project {project}"),
            None => "team-wide".to_string(),
        };
        format!("{} · {} · {scope}", self.backend.label(), self.team)
    }
}

/// Why a directory has no board. Each variant carries a human reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardUnavailable {
    /// Global config did not load; rules cannot be trusted.
    InvalidConfig(String),
    /// Not a git repository, no `origin`, or no rule matches the origin.
    UnknownWorld,
    /// The rule's `user.email` check failed or could not run.
    IdentityUnverified {
        world: String,
    },
    /// Several rules match the origin and disagree.
    Ambiguous {
        worlds: Vec<String>,
    },
    NoBinding {
        world: String,
    },
    MissingTeam {
        world: String,
    },
    UnknownBackend {
        world: String,
        value: String,
    },
    /// `tracker` is set and names another backend.
    TrackerMismatch {
        world: String,
        tracker: String,
        backend: BoardBackend,
    },
    /// Brown worlds never query Linear.
    BrownRequiresAdo {
        world: String,
    },
    /// The repository lives on an Azure DevOps host, whatever the world is
    /// called: Linear is never queried for it.
    AzureHostRequiresAdo {
        world: String,
    },
    /// No workspace rule has this name (resolution by world name).
    UnknownWorldName {
        name: String,
    },
}

impl fmt::Display for BoardUnavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(why) => write!(f, "config is invalid, no board: {why}"),
            Self::UnknownWorld => write!(f, "this directory is not in a configured world"),
            Self::IdentityUnverified { world } => {
                write!(f, "git identity of world {world:?} is not verified")
            }
            Self::Ambiguous { worlds } => write!(
                f,
                "ambiguous world: {} rules match with different boards",
                worlds.len()
            ),
            Self::NoBinding { world } => {
                write!(f, "world {world:?} has no [board] binding")
            }
            Self::MissingTeam { world } => {
                write!(f, "world {world:?} board binding has no team id")
            }
            Self::UnknownBackend { world, value } => write!(
                f,
                "world {world:?} board backend {value:?} is unknown (expected linear or ado)"
            ),
            Self::TrackerMismatch {
                world,
                tracker,
                backend,
            } => write!(
                f,
                "world {world:?} tracker is {tracker:?} but its board backend is {}",
                backend.as_str()
            ),
            Self::BrownRequiresAdo { world } => {
                write!(f, "Brown boards require ADO (world {world:?})")
            }
            Self::AzureHostRequiresAdo { world } => write!(
                f,
                "Azure DevOps repositories require an ADO board (world {world:?})"
            ),
            Self::UnknownWorldName { name } => {
                write!(f, "no configured world named {name:?}")
            }
        }
    }
}

impl std::error::Error for BoardUnavailable {}

/// Resolve the board for the repository containing `dir`. Blocking: may run
/// git on a cache miss. Uses the strict global config, so a broken config
/// denies instead of falling back to defaults.
pub fn resolve_board(dir: &Path) -> Result<BoardScope, BoardUnavailable> {
    let config = crate::config::config_strict()
        .map_err(|error| BoardUnavailable::InvalidConfig(format!("{error:#}")))?;
    let facts = workspace::repo_facts(dir);
    resolve_board_from(&config.workspaces, facts.as_deref())
}

/// Pure resolution over already-loaded rules and repository facts.
pub fn resolve_board_from(
    rules: &WorkspacesConfig,
    facts: Option<&RepoFacts>,
) -> Result<BoardScope, BoardUnavailable> {
    let facts = facts.ok_or(BoardUnavailable::UnknownWorld)?;
    let origin = facts
        .origin
        .as_deref()
        .ok_or(BoardUnavailable::UnknownWorld)?;
    let matching = workspace::matching_rules(rules, origin);
    let Some(first) = matching.first().copied() else {
        return Err(BoardUnavailable::UnknownWorld);
    };
    if matching.iter().any(|other| !same_world(first, other)) {
        return Err(BoardUnavailable::Ambiguous {
            worlds: matching.iter().map(|r| r.name.trim().to_string()).collect(),
        });
    }
    // Every overlapping rule must pass the identity check, not just the first.
    for rule in &matching {
        if matches!(
            workspace::identity_of(rule, facts),
            Identity::Mismatch | Identity::Missing
        ) {
            return Err(BoardUnavailable::IdentityUnverified {
                world: rule.name.trim().to_string(),
            });
        }
    }
    scope_for(first, facts, origin)
}

fn expected_email(rule: &WorkspaceRule) -> Option<String> {
    rule.expected_email
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(str::to_ascii_lowercase)
}

/// `dev.azure.com`, `*.dev.azure.com` (ssh form) or `*.visualstudio.com`, from a sanitized `host/path` origin.
fn is_azure_devops_origin(origin: &str) -> bool {
    let host = origin.split('/').next().unwrap_or("").to_ascii_lowercase();
    host == "dev.azure.com"
        || host.ends_with(".dev.azure.com")
        || host.ends_with(".visualstudio.com")
}

/// Rules that match one origin must agree on world, tracker and board;
/// otherwise the choice would depend on config order.
fn same_world(a: &WorkspaceRule, b: &WorkspaceRule) -> bool {
    a.name.trim().eq_ignore_ascii_case(b.name.trim())
        && a.tracker.as_deref().map(str::trim) == b.tracker.as_deref().map(str::trim)
        && a.board == b.board
        && expected_email(a) == expected_email(b)
}

/// Config-only checks of a rule's board binding: no repository, identity or
/// directory involved. Returns the binding and its backend.
pub(super) fn validate_binding(
    rule: &WorkspaceRule,
) -> Result<(&BoardBinding, BoardBackend), BoardUnavailable> {
    validate_binding_for(rule, None)
}

/// [`validate_binding`] plus, when `origin` is known, the Azure host check
/// (kept between the Brown and tracker checks so refusal order is stable).
fn validate_binding_for<'a>(
    rule: &'a WorkspaceRule,
    origin: Option<&str>,
) -> Result<(&'a BoardBinding, BoardBackend), BoardUnavailable> {
    let world = rule.name.trim().to_string();
    let binding: &BoardBinding =
        rule.board
            .as_ref()
            .ok_or_else(|| BoardUnavailable::NoBinding {
                world: world.clone(),
            })?;
    let backend = binding
        .backend_kind()
        .ok_or_else(|| BoardUnavailable::UnknownBackend {
            world: world.clone(),
            value: binding.backend.trim().to_string(),
        })?;
    if binding.team_id().is_empty() {
        return Err(BoardUnavailable::MissingTeam { world });
    }
    let tracker = rule
        .tracker_kind()
        .map_err(|value| BoardUnavailable::InvalidConfig(format!("unknown tracker {value:?}")))?;
    let brown = world.eq_ignore_ascii_case("brown") || tracker == Some(Tracker::Ado);
    if brown && backend == BoardBackend::Linear {
        return Err(BoardUnavailable::BrownRequiresAdo { world });
    }
    if backend == BoardBackend::Linear && origin.is_some_and(is_azure_devops_origin) {
        return Err(BoardUnavailable::AzureHostRequiresAdo { world });
    }
    if let Some(tracker) = tracker
        && tracker != backend.tracker()
    {
        return Err(BoardUnavailable::TrackerMismatch {
            world,
            tracker: rule.tracker.clone().unwrap_or_default().trim().to_string(),
            backend,
        });
    }
    Ok((binding, backend))
}

fn scope_for(
    rule: &WorkspaceRule,
    facts: &RepoFacts,
    origin: &str,
) -> Result<BoardScope, BoardUnavailable> {
    let world = rule.name.trim().to_string();
    if matches!(
        workspace::identity_of(rule, facts),
        Identity::Mismatch | Identity::Missing
    ) {
        return Err(BoardUnavailable::IdentityUnverified { world });
    }
    let (binding, backend) = validate_binding_for(rule, Some(origin))?;
    Ok(BoardScope {
        origin: ScopeOrigin::Configured {
            world: world.clone(),
        },
        world,
        backend,
        team: binding.team_id().to_string(),
        project: binding.project_id().map(str::to_string),
        server: binding.server_name().to_string(),
        canonical_root: Some(facts.root.clone()),
    })
}

/// Resolve a board by name, with no directory: `world`, `world/team` or
/// `world/team/project`. Uses the strict global config. Blocking-free.
pub fn resolve_world(target: &str) -> Result<BoardScope, BoardUnavailable> {
    let config = crate::config::config_strict()
        .map_err(|error| BoardUnavailable::InvalidConfig(format!("{error:#}")))?;
    resolve_world_from(&config.workspaces, target)
}

/// Pure [`resolve_world`] over already-loaded rules. The world name is
/// case-insensitive. A team (and project, if the binding has one) equal to
/// the binding yields the configured scope; anything else yields a
/// read-only discovered scope on the world's server. No identity check: no
/// repository is involved, and the result never has a root.
pub fn resolve_world_from(
    rules: &WorkspacesConfig,
    target: &str,
) -> Result<BoardScope, BoardUnavailable> {
    let mut parts = target.trim().splitn(3, '/').map(str::trim);
    let name = parts.next().unwrap_or("");
    let team = parts.next().filter(|t| !t.is_empty());
    let project = parts.next().filter(|p| !p.is_empty());
    let matching: Vec<&WorkspaceRule> = rules
        .rules
        .iter()
        .filter(|rule| !name.is_empty() && rule.name.trim().eq_ignore_ascii_case(name))
        .collect();
    let Some(first) = matching.first().copied() else {
        return Err(BoardUnavailable::UnknownWorldName {
            name: name.to_string(),
        });
    };
    if matching.iter().any(|other| !same_world(first, other)) {
        return Err(BoardUnavailable::Ambiguous {
            worlds: matching.iter().map(|r| r.name.trim().to_string()).collect(),
        });
    }
    let (binding, backend) = validate_binding(first)?;
    let world = first.name.trim().to_string();
    let (team, project) = match team {
        None => (
            binding.team_id().to_string(),
            binding.project_id().map(str::to_string),
        ),
        Some(team) => (team.to_string(), project.map(str::to_string)),
    };
    let configured = team == binding.team_id() && project.as_deref() == binding.project_id();
    let (world, origin) = if configured {
        (world.clone(), ScopeOrigin::Configured { world })
    } else {
        (String::new(), ScopeOrigin::Discovered)
    };
    Ok(BoardScope {
        world,
        origin,
        backend,
        team,
        project,
        server: binding.server_name().to_string(),
        canonical_root: None,
    })
}

/// One board account: an MCP server that carries a backend's tools, and the
/// worlds whose bindings use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardAccount {
    pub server: String,
    pub backend: BoardBackend,
    /// Worlds bound to this server. Empty for servers listed only in
    /// `[board] accounts`.
    pub worlds: Vec<String>,
}

/// Accounts from the strict global config. See [`board_accounts_from`].
pub fn board_accounts() -> Result<Vec<BoardAccount>, BoardUnavailable> {
    let config = crate::config::config_strict()
        .map_err(|error| BoardUnavailable::InvalidConfig(format!("{error:#}")))?;
    Ok(board_accounts_from(
        &config.workspaces,
        &config.board.accounts,
    ))
}

/// Linear servers of valid world bindings in config order, then the extra
/// `accounts` entries, deduplicated case-sensitively by server name.
pub fn board_accounts_from(rules: &WorkspacesConfig, accounts: &[String]) -> Vec<BoardAccount> {
    let mut out: Vec<BoardAccount> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    for rule in &rules.rules {
        let Ok((binding, BoardBackend::Linear)) = validate_binding(rule) else {
            continue;
        };
        let server = binding.server_name().to_string();
        let world = rule.name.trim().to_string();
        let at = *index.entry(server.clone()).or_insert_with(|| {
            out.push(BoardAccount {
                server,
                backend: BoardBackend::Linear,
                worlds: Vec::new(),
            });
            out.len() - 1
        });
        if !out[at].worlds.contains(&world) {
            out[at].worlds.push(world);
        }
    }
    for server in accounts.iter().map(|s| s.trim()).filter(|s| !s.is_empty()) {
        if !index.contains_key(server) {
            index.insert(server.to_string(), out.len());
            out.push(BoardAccount {
                server: server.to_string(),
                backend: BoardBackend::Linear,
                worlds: Vec::new(),
            });
        }
    }
    out
}

#[cfg(test)]
#[path = "binding_tests.rs"]
mod tests;
