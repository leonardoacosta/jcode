//! Workspace detection from the git `origin`, plus the cached Git identity check.
//!
//! Rules are `[[workspaces.rules]]` in the user's global config (see
//! `jcode_config_types::WorkspacesConfig`). A rule's `origin` glob is matched
//! against the credential-free `host/path` of the working directory's `origin`
//! remote; the first match wins, no match means "unknown" and nothing is
//! injected.
//!
//! Security properties this module owns:
//! - Credentials (`user:token@`) are removed before any matching, logging,
//!   display or prompt injection. The raw URL is never stored.
//! - Resolution always probes the *target* directory. A spawned worker's
//!   coordinator context is provenance only; it never grants the worker another
//!   workspace's hooks, hints, identity or model overrides.
//! - Git config is read, never written.

use crate::config::Config;
use jcode_config_types::{HooksConfig, Tracker, WorkspaceRule, WorkspacesConfig};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime};

/// Cap on the rule-derived hint lines injected into the prompt.
const HINT_LIMIT: usize = 400;
const NAME_LIMIT: usize = 64;
const PROVENANCE_CAP: usize = 4096;

static GIT_SPAWNS: AtomicUsize = AtomicUsize::new(0);

/// Number of `git` processes this module has started (for cache tests).
pub fn git_spawn_count() -> usize {
    GIT_SPAWNS.load(Ordering::Relaxed)
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ---------------------------------------------------------------- origin

/// `host/path` of a git remote URL with userinfo, port, query, fragment,
/// trailing `/` and `.git` removed. `None` for local paths and anything that
/// has no host.
///
/// Handles `https://user:tok@host/a/b.git`, `ssh://git@host:22/a/b`, and the
/// scp form `git@host:a/b`.
pub fn sanitize_origin(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let (host, path) = if let Some((_, rest)) = raw.split_once("://") {
        // Userinfo is everything up to the last `@` before the query or
        // fragment, so a password holding a raw `/` still cannot reach the
        // host or path. (A literal `@` in a path therefore reads as userinfo:
        // such an origin may fail to match, which is the safe direction.)
        let pre_query = &rest[..rest.find(['?', '#']).unwrap_or(rest.len())];
        let after_userinfo = pre_query.rsplit_once('@').map_or(pre_query, |(_, r)| r);
        let end = after_userinfo.find('/').unwrap_or(after_userinfo.len());
        let (hostport, path) = after_userinfo.split_at(end);
        (host_of(hostport)?, path.to_string())
    } else {
        let first_slash = raw.find('/').unwrap_or(raw.len());
        let head = &raw[..first_slash];
        let start = head.rfind('@').map_or(0, |at| at + 1);
        let rest = &raw[start..];
        let (host, path) = if rest.starts_with('[') {
            let close = rest.find(']')?;
            (&rest[..=close], rest[close + 1..].strip_prefix(':')?)
        } else {
            rest.split_once(':')?
        };
        // `C:\repo` is a Windows drive, not an scp host.
        if host.is_empty()
            || host.contains(['/', '\\'])
            || (host.len() == 1 && path.starts_with('\\'))
        {
            return None;
        }
        (host.to_string(), path.to_string())
    };
    let path = path.split(['?', '#']).next().unwrap_or("");
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    Some(if path.is_empty() {
        host
    } else {
        format!("{host}/{path}")
    })
}

/// Host of `host[:port]` or `[ipv6][:port]`. `None` for an empty host or a
/// port that is not all digits (a malformed URL: reject rather than guess).
fn host_of(hostport: &str) -> Option<String> {
    let (host, tail) = if hostport.starts_with('[') {
        let close = hostport.find(']')?;
        hostport.split_at(close + 1)
    } else {
        hostport.split_at(hostport.find(':').unwrap_or(hostport.len()))
    };
    let port_ok = tail.is_empty()
        || tail
            .strip_prefix(':')
            .is_some_and(|port| port.bytes().all(|b| b.is_ascii_digit()));
    (port_ok && !host.is_empty()).then(|| host.to_string())
}

/// Compile a case-insensitive origin glob: `*` and `?` stay inside one path
/// segment, `**` crosses segments (`**/` also matches nothing).
fn glob_regex(glob: &str) -> Option<Regex> {
    let mut re = String::from("(?is)^");
    let mut chars = glob.trim().chars().peekable();
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
            other => re.push_str(&regex::escape(&other.to_string())),
        }
    }
    re.push('$');
    Regex::new(&re).ok()
}

/// Compiled globs by source text. Rules are tiny and few; the bound only
/// guards against a pathological config churn.
static GLOBS: LazyLock<Mutex<HashMap<String, Option<Regex>>>> = LazyLock::new(Default::default);
const GLOB_CACHE_CAP: usize = 256;

fn glob_matches(glob: &str, origin: &str) -> bool {
    let mut cache = lock(&GLOBS);
    if cache.len() >= GLOB_CACHE_CAP && !cache.contains_key(glob) {
        cache.clear();
    }
    cache
        .entry(glob.to_string())
        .or_insert_with(|| glob_regex(glob))
        .as_ref()
        .is_some_and(|re| re.is_match(origin))
}

/// First rule (config order) whose `origin` glob matches `origin`.
pub fn match_rule<'a>(rules: &'a WorkspacesConfig, origin: &str) -> Option<&'a WorkspaceRule> {
    rules.rules.iter().find(|rule| {
        !rule.name.trim().is_empty()
            && !rule.origin.trim().is_empty()
            && glob_matches(&rule.origin, origin)
    })
}

/// Every rule (config order) whose `origin` glob matches `origin`. Callers
/// that must not guess between overlapping rules (issue boards) use this
/// instead of [`match_rule`]'s first-match-wins.
pub fn matching_rules<'a>(rules: &'a WorkspacesConfig, origin: &str) -> Vec<&'a WorkspaceRule> {
    rules
        .rules
        .iter()
        .filter(|rule| {
            !rule.name.trim().is_empty()
                && !rule.origin.trim().is_empty()
                && glob_matches(&rule.origin, origin)
        })
        .collect()
}

// ------------------------------------------------------------ repo facts

/// What the checkout says about itself. `origin` is already credential-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoFacts {
    pub root: PathBuf,
    /// Shared across worktrees: two directories are the same repository
    /// exactly when their `common_dir` is equal.
    pub common_dir: PathBuf,
    pub origin: Option<String>,
    /// Effective `git config user.email`.
    pub email: Option<String>,
}

struct RepoPaths {
    root: PathBuf,
    /// This checkout's own git dir (`<root>/.git`, or `<common>/worktrees/<n>`).
    git_dir: PathBuf,
    common_dir: PathBuf,
}

/// Locate the repository without spawning git: walk up to `.git` (a directory,
/// or a `gitdir:` file for worktrees and submodules).
fn find_repo(dir: &Path) -> Option<RepoPaths> {
    let start = std::fs::canonicalize(dir).ok()?;
    for ancestor in start.ancestors() {
        let dot_git = ancestor.join(".git");
        let Ok(meta) = std::fs::symlink_metadata(&dot_git) else {
            continue;
        };
        let git_dir = if meta.is_dir() {
            dot_git
        } else {
            let text = std::fs::read_to_string(&dot_git).ok()?;
            let target = text.lines().find_map(|l| l.strip_prefix("gitdir:"))?.trim();
            let target = Path::new(target);
            let target = if target.is_absolute() {
                target.to_path_buf()
            } else {
                ancestor.join(target)
            };
            std::fs::canonicalize(target).ok()?
        };
        let common_dir = match std::fs::read_to_string(git_dir.join("commondir")) {
            Ok(rel) => std::fs::canonicalize(git_dir.join(rel.trim())).ok()?,
            Err(_) => git_dir.clone(),
        };
        return Some(RepoPaths {
            root: ancestor.to_path_buf(),
            git_dir,
            common_dir,
        });
    }
    None
}

type Stamp = Vec<Option<(SystemTime, u64)>>;

/// How deep `include`/`includeIf` chains are followed for the stamp. Git
/// itself allows 10; identity splits use one or two.
const INCLUDE_DEPTH: usize = 3;

/// Files whose change can alter origin or effective `user.email`: the repo,
/// worktree, system, global and XDG configs plus every file they `include`.
fn config_stamp(paths: &RepoPaths) -> Stamp {
    let mut files = vec![
        paths.common_dir.join("config"),
        paths.git_dir.join("config.worktree"),
    ];
    match std::env::var_os("GIT_CONFIG_SYSTEM") {
        Some(system) => files.push(PathBuf::from(system)),
        None => files.push(PathBuf::from("/etc/gitconfig")),
    }
    let home = dirs::home_dir();
    if let Some(global) = std::env::var_os("GIT_CONFIG_GLOBAL") {
        files.push(PathBuf::from(global));
    } else if let Some(home) = &home {
        files.push(home.join(".gitconfig"));
        files.push(
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"))
                .join("git/config"),
        );
    }
    let mut seen = HashSet::new();
    let mut stamp = Stamp::new();
    let mut frontier = files;
    for _ in 0..=INCLUDE_DEPTH {
        let mut next = Vec::new();
        for path in frontier {
            if !seen.insert(path.clone()) {
                continue;
            }
            stamp.push(file_stamp(&path));
            next.extend(included_paths(&path, home.as_deref()));
        }
        frontier = next;
    }
    stamp
}

fn file_stamp(path: &Path) -> Option<(SystemTime, u64)> {
    std::fs::metadata(path).ok().map(|meta| {
        (
            meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            meta.len(),
        )
    })
}

/// Targets of `path = ...` lines (`[include]`, `[includeIf "..."]`) in a git
/// config. Matching every `path` key, in any section, over-approximates:
/// an extra file in the stamp only costs a stat.
fn included_paths(config: &Path, home: Option<&Path>) -> Vec<PathBuf> {
    let Ok(text) = std::fs::read_to_string(config) else {
        return Vec::new();
    };
    let base = config.parent().unwrap_or(Path::new("."));
    text.lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            if !key.trim().eq_ignore_ascii_case("path") {
                return None;
            }
            let value = value.trim().trim_matches('"');
            let path = match value.strip_prefix("~/") {
                Some(rest) => home?.join(rest),
                None => PathBuf::from(value),
            };
            Some(if path.is_absolute() {
                path
            } else {
                base.join(path)
            })
        })
        .collect()
}

/// Backstop for what the stamp cannot see (`GIT_CONFIG_COUNT` env, glob
/// includes, clock-skewed mtimes): re-probe at least this often.
const CACHE_TTL: Duration = Duration::from_secs(60);

/// Longest a single `git` call may run before it is killed.
const GIT_TIMEOUT: Duration = Duration::from_secs(2);

/// Run `git` with a hard time limit; a hung filesystem or credential helper
/// must not wedge the caller. The child is killed on timeout.
fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    git_output_with(root, args, GIT_TIMEOUT)
}

fn git_output_with(root: &Path, args: &[&str], timeout: Duration) -> Option<String> {
    use std::io::Read;
    GIT_SPAWNS.fetch_add(1, Ordering::Relaxed);
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        let _ = tx.send(buf);
    });
    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    let buf = rx.recv_timeout(Duration::from_secs(1)).ok()?;
    let text = String::from_utf8_lossy(&buf).trim().to_string();
    (status.success() && !text.is_empty()).then_some(text)
}

fn probe(paths: &RepoPaths) -> RepoFacts {
    RepoFacts {
        root: paths.root.clone(),
        common_dir: paths.common_dir.clone(),
        origin: git_output(&paths.root, &["remote", "get-url", "origin"])
            .as_deref()
            .and_then(sanitize_origin),
        email: git_output(&paths.root, &["config", "--get", "user.email"]),
    }
}

struct Entry {
    stamp: Stamp,
    probed_at: Instant,
    facts: Arc<RepoFacts>,
}

static CACHE: LazyLock<Mutex<HashMap<PathBuf, Entry>>> = LazyLock::new(Default::default);
static INFLIGHT: LazyLock<Mutex<HashSet<PathBuf>>> = LazyLock::new(Default::default);

struct InflightGuard(PathBuf);

impl Drop for InflightGuard {
    fn drop(&mut self) {
        lock(&INFLIGHT).remove(&self.0);
    }
}

fn cached(paths: &RepoPaths, stamp: &Stamp) -> Option<Arc<RepoFacts>> {
    lock(&CACHE)
        .get(&paths.root)
        .filter(|entry| &entry.stamp == stamp && entry.probed_at.elapsed() < CACHE_TTL)
        .map(|entry| Arc::clone(&entry.facts))
}

fn refresh(paths: &RepoPaths) -> Arc<RepoFacts> {
    // Stamp first: a config edit during the probe makes the next check re-probe.
    let stamp = config_stamp(paths);
    let facts = Arc::new(probe(paths));
    lock(&CACHE).insert(
        paths.root.clone(),
        Entry {
            stamp,
            probed_at: Instant::now(),
            facts: Arc::clone(&facts),
        },
    );
    facts
}

/// Facts for the repository containing `dir`. Spawns git only when the repo's
/// or the user's git config changed since the last probe.
pub fn repo_facts(dir: &Path) -> Option<Arc<RepoFacts>> {
    let paths = find_repo(dir)?;
    let stamp = config_stamp(&paths);
    Some(cached(&paths, &stamp).unwrap_or_else(|| refresh(&paths)))
}

/// Like [`repo_facts`] but never runs git on the caller's thread (render
/// paths). A miss starts a background probe and returns the stale entry, or
/// `None` the first time.
pub fn repo_facts_nonblocking(dir: &Path) -> Option<Arc<RepoFacts>> {
    let paths = find_repo(dir)?;
    let stamp = config_stamp(&paths);
    if let Some(facts) = cached(&paths, &stamp) {
        return Some(facts);
    }
    let stale = lock(&CACHE)
        .get(&paths.root)
        .map(|entry| Arc::clone(&entry.facts));
    if lock(&INFLIGHT).insert(paths.root.clone()) {
        std::thread::spawn(move || {
            // Released on drop so a panicking probe cannot freeze refreshes.
            let _guard = InflightGuard(paths.root.clone());
            refresh(&paths);
        });
    }
    stale
}

// ------------------------------------------------------------ resolution

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Identity {
    /// The rule has no `expected_email`.
    NotChecked,
    Match,
    Mismatch,
    /// `user.email` is unset. Never invented.
    Missing,
}

/// A working directory resolved to a configured workspace.
pub struct Resolved {
    pub rule: &'static WorkspaceRule,
    pub facts: Arc<RepoFacts>,
}

impl Resolved {
    pub fn name(&self) -> &str {
        self.rule.name.trim()
    }

    pub fn identity(&self) -> Identity {
        identity_of(self.rule, &self.facts)
    }
}

pub fn identity_of(rule: &WorkspaceRule, facts: &RepoFacts) -> Identity {
    let Some(expected) = rule
        .expected_email
        .as_deref()
        .map(str::trim)
        .filter(|e| !e.is_empty())
    else {
        return Identity::NotChecked;
    };
    match facts.email.as_deref().map(str::trim) {
        None | Some("") => Identity::Missing,
        Some(actual) if actual.eq_ignore_ascii_case(expected) => Identity::Match,
        Some(_) => Identity::Mismatch,
    }
}

fn resolve_with(dir: &Path, block: bool) -> Option<Resolved> {
    let rules = &crate::config::config().workspaces;
    if rules.is_empty() {
        return None;
    }
    let facts = if block {
        repo_facts(dir)
    } else {
        repo_facts_nonblocking(dir)
    }?;
    let rule = match_rule(rules, facts.origin.as_deref()?)?;
    Some(Resolved { rule, facts })
}

/// Workspace of `dir`. Blocking: may run git on a cache miss.
pub fn resolve(dir: &Path) -> Option<Resolved> {
    resolve_with(dir, true)
}

/// Header row text (`ws: brown · repo: tracker:ado`, plus ` !id` on identity mismatch or ` ?id`
/// when `user.email` is unset). `None` for unknown workspaces. With
/// `block == false` git never runs on the calling thread.
pub fn badge(dir: &Path, block: bool) -> Option<String> {
    let resolved = resolve_with(dir, block)?;
    let mut text = format!("ws: {}", truncate_chars(resolved.name(), NAME_LIMIT));
    if let Some(repo) = resolved.facts.root.file_name() {
        let repo = truncate_chars(&repo.to_string_lossy(), NAME_LIMIT);
        text.push_str(&format!(" · {repo}:"));
    }
    match resolved.rule.tracker_kind() {
        Ok(Some(Tracker::Linear)) => text.push_str(" tracker:linear"),
        Ok(Some(Tracker::Ado)) => text.push_str(" tracker:ado"),
        _ => {}
    }
    match resolved.identity() {
        Identity::Mismatch => text.push_str(" !id"),
        Identity::Missing => text.push_str(" ?id"),
        Identity::Match | Identity::NotChecked => {}
    }
    Some(text)
}

/// Badges refreshed at most this often per directory.
const BADGE_TTL: Duration = Duration::from_secs(2);
const BADGE_CACHE_CAP: usize = 64;

type BadgeEntry = (Instant, Option<String>);
static BADGES: LazyLock<Mutex<HashMap<PathBuf, BadgeEntry>>> = LazyLock::new(Default::default);

/// [`badge`] for the render thread: never runs git and, because it is called
/// every frame, touches the filesystem and compiles nothing more often than
/// every [`BADGE_TTL`].
pub fn badge_cached(dir: &Path) -> Option<String> {
    if let Some((at, text)) = lock(&BADGES).get(dir)
        && at.elapsed() < BADGE_TTL
    {
        return text.clone();
    }
    let text = badge(dir, false);
    let mut cache = lock(&BADGES);
    if cache.len() >= BADGE_CACHE_CAP {
        cache.clear();
    }
    cache.insert(dir.to_path_buf(), (Instant::now(), text.clone()));
    text
}

fn truncate_chars(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max {
        return flat;
    }
    let mut cut: String = flat.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

// ----------------------------------------------------------------- hooks

/// Rule warnings plus the lifecycle-hook validation global `[hooks]` gets, so
/// a workspace gate with a bad matcher is reported instead of silently dropped.
pub fn config_warnings(rules: &WorkspacesConfig) -> Vec<String> {
    let mut out = rules.warnings();
    for rule in &rules.rules {
        out.extend(
            crate::hooks::lifecycle_config_warnings(&rule.hooks)
                .into_iter()
                .map(|warning| format!("workspace rule \"{}\": {warning}", rule.name.trim())),
        );
    }
    out
}

/// Global lifecycle hooks plus the matched workspace's, for `event` in `cwd`.
/// `None` when this call has no workspace entries (use the global path, which
/// also owns broken-config denial) or hooks are suppressed.
pub fn lifecycle_hooks_for(cwd: Option<&str>, event: &str) -> Option<HooksConfig> {
    if std::env::var_os("JCODE_HOOKS_DISABLED").is_some() {
        return None;
    }
    let config: &'static Config = crate::config::config_strict().ok()?;
    if !config.workspaces.has_lifecycle_entries(event) {
        return None;
    }
    let resolved = resolve(Path::new(cwd?))?;
    if resolved.rule.hooks.lifecycle_entries(event).is_empty() {
        return None;
    }
    Some(config.hooks.with_workspace_hooks(&resolved.rule.hooks))
}

// ------------------------------------------------------------ swarm model

/// A workspace-level model for a spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOverride {
    pub model: String,
    pub workspace: String,
    /// The role slot that supplied the model, `None` for `swarm_model`.
    pub role: Option<String>,
}

/// The workspace override for a worker whose *target* directory is
/// `target_dir`. The coordinator's own workspace is irrelevant: only the rule
/// the target resolves to can apply.
pub fn spawn_model_override(
    target_dir: Option<&Path>,
    role: Option<&str>,
) -> Option<ModelOverride> {
    let resolved = resolve(target_dir?)?;
    let (model, role) = resolved.rule.spawn_model(role)?;
    Some(ModelOverride {
        model,
        workspace: resolved.name().to_string(),
        role,
    })
}

// ------------------------------------------------------------ provenance

/// What the coordinator resolved to when it spawned a worker. Provenance only:
/// the worker's own target is always re-resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    pub workspace: String,
    pub common_dir: PathBuf,
}

static PROVENANCE: LazyLock<Mutex<HashMap<String, Provenance>>> = LazyLock::new(Default::default);

/// Remember the coordinator's workspace for a freshly spawned worker session.
/// Process-local: a daemon restart loses it, which only drops the mismatch
/// note, never changes what applies to the worker.
pub fn record_provenance(worker_session_id: &str, coordinator_dir: Option<&Path>) {
    let Some(resolved) = coordinator_dir.and_then(resolve) else {
        return;
    };
    let mut map = lock(&PROVENANCE);
    if map.len() >= PROVENANCE_CAP {
        map.clear();
    }
    map.insert(
        worker_session_id.to_string(),
        Provenance {
            workspace: resolved.name().to_string(),
            common_dir: resolved.facts.common_dir.clone(),
        },
    );
}

pub fn provenance(session_id: &str) -> Option<Provenance> {
    lock(&PROVENANCE).get(session_id).cloned()
}

/// How a worker's actual target relates to its coordinator's provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetCheck {
    /// Same repository (shared git common dir): a verified worktree.
    SameRepository,
    /// A different checkout that resolves, by its own origin, to the same rule.
    SameWorkspace,
    /// Another workspace rule applies to the target.
    DifferentWorkspace(String),
    /// The target is outside any matched workspace.
    Unknown,
}

pub fn check_target(provenance: &Provenance, target: Option<&Resolved>) -> TargetCheck {
    match target {
        Some(t) if t.facts.common_dir == provenance.common_dir => TargetCheck::SameRepository,
        Some(t) if t.name() == provenance.workspace => TargetCheck::SameWorkspace,
        Some(t) => TargetCheck::DifferentWorkspace(t.name().to_string()),
        None => TargetCheck::Unknown,
    }
}

// ---------------------------------------------------------------- prompt

/// Bounded system-prompt block for the workspace of `dir`, `None` when the
/// workspace is unknown and there is no provenance mismatch to report.
///
/// Hints are phrased as configured, not authoritative.
pub fn prompt_block(dir: &Path, provenance: Option<&Provenance>) -> Option<String> {
    let target = resolve(dir);
    let mut lines: Vec<String> = Vec::new();

    if let Some(resolved) = &target {
        let rule = resolved.rule;
        lines.push(format!(
            "Workspace (configured from git origin rules, not authoritative): {}",
            truncate_chars(resolved.name(), NAME_LIMIT)
        ));
        if rule.hints_valid() {
            let mut hints = Vec::new();
            if let Ok(Some(tracker)) = rule.tracker_kind() {
                hints.push(match tracker {
                    Tracker::None => "Tracker: none configured.".to_string(),
                    other => format!("Tracker: {} is the configured tracker.", other.label()),
                });
            }
            if let Some(template) = rule
                .branch_template
                .as_deref()
                .map(str::trim)
                .filter(|t| !t.is_empty())
            {
                hints.push(format!(
                    "Branch template: {}",
                    truncate_chars(template, 120)
                ));
            }
            let skills: Vec<&str> = rule
                .skills
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !skills.is_empty() {
                hints.push(format!("Skills: {}", skills.join(", ")));
            }
            lines.push(truncate_block(&hints.join("\n"), HINT_LIMIT));
        }
        match resolved.identity() {
            Identity::Mismatch => lines.push(
                "Git identity: effective user.email differs from this workspace's expected_email. \
                 Check `git config user.email` before committing; jcode does not change it."
                    .to_string(),
            ),
            Identity::Missing => lines.push(
                "Git identity: user.email is not set. Ask the user before committing; do not invent one."
                    .to_string(),
            ),
            Identity::Match | Identity::NotChecked => {}
        }
    }

    if let Some(prov) = provenance {
        let note = match check_target(prov, target.as_ref()) {
            TargetCheck::SameRepository | TargetCheck::SameWorkspace => None,
            TargetCheck::DifferentWorkspace(name) => Some(format!(
                "Spawned from workspace {} but this target resolves to {}; only {}'s rules apply.",
                truncate_chars(&prov.workspace, NAME_LIMIT),
                truncate_chars(&name, NAME_LIMIT),
                truncate_chars(&name, NAME_LIMIT),
            )),
            TargetCheck::Unknown => Some(format!(
                "Spawned from workspace {} but this target is outside any matched workspace; \
                 that workspace's hooks, tracker, identity and model rules do not apply here.",
                truncate_chars(&prov.workspace, NAME_LIMIT),
            )),
        };
        lines.extend(note);
    }

    (!lines.is_empty()).then(|| format!("# Workspace\n{}", lines.join("\n")))
}

/// Cap a multi-line block, keeping whole lines where possible.
fn truncate_block(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(max.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
