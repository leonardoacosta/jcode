//! Hard tool allowlist for an overnight run's coordinator session.
//!
//! The coordinator coordinates; workers edit, build, review and judge. The
//! registry here is keyed by session id and consulted by `Registry::execute`
//! (so `batch` members are checked too). The policy ends lazily: each call
//! re-reads the run manifest and drops the restriction once the run is terminal.

use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};

/// session id -> run id
static COORDINATORS: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
/// Sessions already checked against persisted manifests and found unrestricted.
static NOT_COORDINATORS: LazyLock<RwLock<HashSet<String>>> =
    LazyLock::new(|| RwLock::new(HashSet::new()));

const DELEGATION_HINT: &str = "The overnight coordinator only coordinates: assign every edit, build, review and judgement to a swarm worker (swarm spawn/assign), and keep your own tools to swarm, todo, bg, read-only inspection and markdown notes. Read-only git needs `git --no-optional-locks status` or `git --no-optional-locks diff --no-textconv --no-ext-diff ...` (log and show take the same two flags, first after the subcommand).";

/// Restrict `session_id` to the coordinator allowlist until `run_id` is terminal.
pub fn install_for_overnight(session_id: &str, run_id: &str) {
    NOT_COORDINATORS
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .remove(session_id);
    COORDINATORS
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .insert(session_id.to_string(), run_id.to_string());
}

pub fn remove(session_id: &str) {
    COORDINATORS
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .remove(session_id);
}

/// True while `session_id` is an active overnight coordinator.
pub fn is_coordinator(session_id: &str) -> bool {
    active_restriction(session_id).is_some()
}

/// `Some(manifest)` when restricted; the manifest is `None` if it cannot be
/// read (fail closed, no event recorded).
fn active_restriction(session_id: &str) -> Option<Option<jcode_overnight_core::OvernightManifest>> {
    let run_id = COORDINATORS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(session_id)
        .cloned();
    if let Some(run_id) = run_id {
        return match crate::overnight::load_manifest(&run_id) {
            Ok(manifest) if manifest.status.is_terminal() => {
                remove(session_id);
                None
            }
            Ok(manifest) => Some(Some(manifest)),
            Err(_) => Some(None),
        };
    }
    if NOT_COORDINATORS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .contains(session_id)
    {
        return None;
    }
    // Session restored after a reload: the in-memory registry is empty but the
    // run may still be active on disk.
    let active = crate::overnight::session_manifests_newest_first(session_id)
        .ok()
        .and_then(|manifests| manifests.into_iter().next())
        .filter(|manifest| !manifest.status.is_terminal());
    match active {
        Some(manifest) => {
            COORDINATORS
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .insert(session_id.to_string(), manifest.run_id.clone());
            Some(Some(manifest))
        }
        None => {
            NOT_COORDINATORS
                .write()
                .unwrap_or_else(|e| e.into_inner())
                .insert(session_id.to_string());
            None
        }
    }
}

/// Registry hook: `Ok(())` to run, `Err(message)` is the denial result text.
/// `tool` is the canonical tool name; `is_mcp` is true for MCP-backed tools.
pub(crate) fn enforce(
    session_id: &str,
    tool: &str,
    input: &Value,
    is_mcp: bool,
    resolve: &dyn Fn(&Path) -> PathBuf,
) -> Result<(), String> {
    let Some(manifest) = active_restriction(session_id) else {
        return Ok(());
    };
    if is_mcp {
        return Ok(());
    }
    let Err(reason) = allows(tool, input, resolve) else {
        return Ok(());
    };
    // Run bookkeeping (task cards, issue drafts, validation output) lives in
    // the run directory and the prompts tell the coordinator to maintain it.
    if let Some(manifest) = manifest.as_ref()
        && matches!(tool, "edit" | "write")
        && input
            .get("file_path")
            .and_then(Value::as_str)
            .is_some_and(|path| inside_run_dir(&manifest.run_id, &resolve(Path::new(path))))
    {
        return Ok(());
    }
    if let Some(manifest) = manifest
        && let Err(error) = crate::overnight::record_event(
            &manifest,
            "coordinator_tool_denied",
            format!("Coordinator call to `{tool}` denied: {reason}"),
            json!({ "tool": tool, "reason": reason }),
            false,
        )
    {
        // The denial still stands; only its audit record is lost.
        crate::logging::warn(&format!(
            "Could not record coordinator_tool_denied for `{tool}` ({reason}) in run {}: {error:#}",
            manifest.run_id
        ));
    }
    Err(format!(
        "Tool '{tool}' denied for the overnight coordinator: {reason}. {DELEGATION_HINT}"
    ))
}

/// Collapse `.` and `..` lexically, then resolve symlinks on the longest
/// existing ancestor so a not-yet-created file is still placed correctly.
fn canonical_lossy(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut normal = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                normal.pop();
            }
            Component::CurDir => {}
            other => normal.push(other.as_os_str()),
        }
    }
    let mut existing = normal.clone();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    while !existing.exists() {
        match existing.file_name() {
            Some(name) => tail.push(name.to_os_string()),
            None => break,
        }
        if !existing.pop() {
            break;
        }
    }
    let mut out = std::fs::canonicalize(&existing).unwrap_or(existing);
    out.extend(tail.iter().rev());
    out
}

fn inside_run_dir(run_id: &str, path: &Path) -> bool {
    let Ok(run_dir) = crate::overnight::run_dir(run_id) else {
        return false;
    };
    canonical_lossy(path).starts_with(canonical_lossy(&run_dir))
}

/// Pure allowlist check for a coordinator tool call.
pub fn allows(tool: &str, input: &Value, resolve: &dyn Fn(&Path) -> PathBuf) -> Result<(), String> {
    match tool {
        "swarm"
        | "todo"
        | "schedule"
        | "bg"
        | "read"
        | "agentgrep"
        | "conversation_search"
        | "session_search"
        | "batch" => Ok(()),
        t if t == "mcp" || t.starts_with("mcp__") || t == "mcp_search" || t == "mcp_call" => Ok(()),
        "edit" | "write" => {
            let path = input
                .get("file_path")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("`{tool}` without file_path"))?;
            let resolved = resolve(Path::new(path));
            let real = std::fs::canonicalize(&resolved).unwrap_or(resolved);
            let is_md = |p: &Path| {
                p.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
            };
            if is_md(Path::new(path)) && is_md(&real) {
                Ok(())
            } else {
                Err(format!("`{tool}` is limited to *.md files, not {path}"))
            }
        }
        "bash" => {
            let command = input
                .get("command")
                .and_then(Value::as_str)
                .ok_or_else(|| "`bash` without command".to_string())?;
            read_only_bash(command, &resolve(Path::new(".")))
                .map_err(|why| format!("bash is read-only: {why}"))
        }
        other => Err(format!("`{other}` is not in the coordinator allowlist")),
    }
}

/// Accept only lines made of read-only commands joined by `&&`, `||`, `;`, `|`.
fn read_only_bash(line: &str, cwd: &Path) -> Result<(), String> {
    let segments = split_segments(line)?;
    if segments.is_empty() {
        return Err("empty command".into());
    }
    for words in segments {
        check_command(&words, cwd)?;
    }
    Ok(())
}

/// One shell word. `glob` marks an unquoted `*`, `?` or `[`: bash expands it to
/// file names, and a file named `-delete` or `--pre=x` becomes an option.
struct Word {
    text: String,
    glob: bool,
}

/// Quote-aware split into commands of unquoted words. Rejects redirection to
/// files, command/process substitution, heredocs and background operators.
fn split_segments(line: &str) -> Result<Vec<Vec<Word>>, String> {
    // Bash keeps `\r` inside a word and deletes `\<LF>` (a line continuation
    // that glues the next line onto the current word: `-ex\<LF>ec` is `-exec`).
    // The checks below compare exact option spellings, so neither may reach them.
    if line.contains('\r') {
        return Err("carriage return".into());
    }
    let chars: Vec<char> = line.chars().collect();
    let mut segments: Vec<Vec<Word>> = Vec::new();
    let mut words: Vec<Word> = Vec::new();
    let mut word = String::new();
    let mut glob = false;
    let mut in_word = false;
    let (mut single, mut double) = (false, false);
    let mut i = 0;

    macro_rules! end_word {
        () => {
            if in_word {
                words.push(Word {
                    text: std::mem::take(&mut word),
                    glob: std::mem::take(&mut glob),
                });
                in_word = false;
            }
        };
    }
    macro_rules! end_segment {
        () => {{
            end_word!();
            if words.is_empty() {
                return Err("empty command in pipeline".into());
            }
            segments.push(std::mem::take(&mut words));
        }};
    }

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if single {
            if c == '\'' {
                single = false;
            } else {
                word.push(c);
            }
            i += 1;
            continue;
        }
        if c == '`' {
            return Err("command substitution".into());
        }
        // Any expansion (`$VAR`, `${..}`, `$(..)`, `$((..))`, `$'..'`) can
        // produce arguments this parser never sees, such as `find . "$F"` with
        // F=-delete. Fail closed. Only single-quoted text and a `$` that bash
        // keeps literal (end of a double-quoted string, before a space) pass.
        if c == '$' {
            let literal = double && matches!(next, None | Some(' ') | Some('\t') | Some('"'));
            if !literal {
                return Err("shell expansion (`$`) can inject arguments".into());
            }
        }
        if double {
            match c {
                '"' => double = false,
                '\\' if next == Some('\n') => {
                    return Err("backslash-newline line continuation".into());
                }
                '\\' if next.is_some() => {
                    i += 1;
                    word.push(chars[i]);
                }
                _ => word.push(c),
            }
            i += 1;
            continue;
        }
        match c {
            '\'' => {
                single = true;
                in_word = true;
            }
            '"' => {
                double = true;
                in_word = true;
            }
            '\\' => {
                let Some(n) = next else {
                    return Err("trailing backslash".into());
                };
                if n == '\n' {
                    return Err("backslash-newline line continuation".into());
                }
                word.push(n);
                in_word = true;
                i += 1;
            }
            // Unquoted braces brace-expand (`{-delete,x}`) into extra argv words.
            '{' | '}' => return Err("brace expansion".into()),
            ' ' | '\t' => end_word!(),
            '\n' | ';' => end_segment!(),
            '&' => {
                if next == Some('&') {
                    end_segment!();
                    i += 1;
                } else if matches!(next, Some('>')) {
                    return Err("output redirection".into());
                } else {
                    return Err("background operator".into());
                }
            }
            '|' => {
                end_segment!();
                if next == Some('|') {
                    i += 1;
                }
            }
            '<' => {
                if matches!(next, Some('<') | Some('(')) {
                    return Err("heredoc or process substitution".into());
                }
                // Input redirection: skip the operator, its target is a word.
                end_word!();
            }
            '>' => {
                // Allow only fd duplication (`2>&1`) and `/dev/null` targets.
                let mut j = i + 1;
                if chars.get(j) == Some(&'>') {
                    j += 1;
                }
                let dup = chars.get(j) == Some(&'&');
                if dup {
                    j += 1;
                }
                while chars.get(j).is_some_and(|c| *c == ' ' || *c == '\t') {
                    j += 1;
                }
                let target: String = chars[j..]
                    .iter()
                    .take_while(|c| !c.is_whitespace() && !matches!(c, ';' | '&' | '|'))
                    .collect();
                let ok = if dup {
                    !target.is_empty() && target.chars().all(|c| c.is_ascii_digit())
                        || target == "-"
                } else {
                    target == "/dev/null"
                };
                if !ok {
                    return Err("output redirection".into());
                }
                // Drop a lone fd number preceding the operator (`2>`).
                if in_word && word.chars().all(|c| c.is_ascii_digit()) {
                    word.clear();
                    in_word = false;
                }
                end_word!();
                i = j + target.chars().count() - 1;
            }
            _ => {
                glob |= matches!(c, '*' | '?' | '[');
                word.push(c);
                in_word = true;
            }
        }
        i += 1;
    }
    if single || double {
        return Err("unterminated quote".into());
    }
    if in_word {
        words.push(Word { text: word, glob });
    }
    if !words.is_empty() {
        segments.push(words);
    }
    // A newline kept inside a quoted word means a continuation survived; no
    // allowed command needs one.
    if segments.iter().flatten().any(|w| w.text.contains('\n')) {
        return Err("newline inside a word".into());
    }
    Ok(segments)
}

fn check_command(words: &[Word], cwd: &Path) -> Result<(), String> {
    let name = words[0].text.as_str();
    // Commands with writing or executing options must not receive words that a
    // glob could turn into such an option; quote the pattern instead.
    if matches!(name, "find" | "rg" | "git") && words[1..].iter().any(|w| w.glob) {
        return Err(format!(
            "`{name}` with an unquoted glob could expand to an option; quote the pattern"
        ));
    }
    let args: Vec<String> = words[1..].iter().map(|w| w.text.clone()).collect();
    let args = args.as_slice();
    match name {
        "rg" => {
            // `--pre[-glob]` and `--hostname-bin` execute the named command and
            // `-z`/`--search-zip` (also inside a short cluster such as `-iz`) run
            // external decompressors from PATH. `--config-path` would load a
            // config file that can inject any of them.
            let short_cluster_with_z =
                |a: &str| a.starts_with('-') && !a.starts_with("--") && a[1..].contains('z');
            match args.iter().find(|a| {
                a.starts_with("--pre")
                    || a.starts_with("--hostname-bin")
                    || a.starts_with("--search-zip")
                    || a.starts_with("--config-path")
                    || short_cluster_with_z(a)
            }) {
                Some(a) => Err(format!("rg {a} runs a command")),
                None => Ok(()),
            }
        }
        "grep" | "ls" | "cat" | "head" | "tail" | "wc" | "jq" | "pwd" | "echo" => Ok(()),
        "find" => {
            const WRITERS: [&str; 9] = [
                "-exec", "-execdir", "-ok", "-okdir", "-delete", "-fprint", "-fprint0", "-fprintf",
                "-fls",
            ];
            match args.iter().find(|a| WRITERS.contains(&a.as_str())) {
                Some(a) => Err(format!("find {a} is not read-only")),
                None => Ok(()),
            }
        }
        "git" => check_git(args, cwd),
        other => Err(format!("`{other}` is not a read-only command")),
    }
}

/// Keys in the effective git configuration (repository and `include` files;
/// the coordinator's own environment drops system and global files) that make
/// `status`, `diff`, `log` or `show` execute a configured program:
/// - `filter.<name>.clean|smudge|process` run on worktree files for `status`
///   and worktree `diff`.
/// - `core.fsmonitor` runs a hook program (`true` starts git's own daemon).
/// - `hook.<name>.command` runs a configured hook, such as the
///   `post-index-change` that `status` and `diff` trigger.
/// - `gpg.program`, `gpg.<fmt>.program` verify signatures for `log` and `show`,
///   as do `log.showSignature` and `%G` placeholders in `format.pretty` or
///   `pretty.<name>`.
/// - `merge.<driver>.driver` runs for `--remerge-diff`, which `log.diffMerges`
///   can turn on for `log` and `show`.
/// - `diff.submodule = diff|log` enters submodules for `log`, `show` and `diff`.
///
/// Other program-running keys were measured as unreachable from the
/// allowlisted commands in the bash tool: `core.pager` and `pager.*` need a
/// terminal and the tool sets `GIT_PAGER=cat` with a piped, tty-less stdout;
/// `credential.helper` only runs for network operations; `diff.*.textconv`,
/// `diff.*.command`, `diff.external` and `GIT_EXTERNAL_DIFF` are switched off by
/// the required `--no-textconv --no-ext-diff`. They are not denied, because
/// `core.pager = delta` and `diff.external = difft` are common and denying them
/// would disable git for the coordinator without closing a path.
const GIT_PROGRAM_KEYS: &str = r"^(filter\..+\.(clean|smudge|process)|core\.fsmonitor|hook\..+\.command|gpg\.program|gpg\.[^.]+\.program|log\.showsignature|log\.diffmerges|merge\..+\.driver|diff\.submodule|format\.pretty|pretty\..+)$";

/// Environment changes for every coordinator git and rg, whether run by the
/// policy probe or by the bash tool (`None` removes the variable). The probe
/// reads configuration once and bash runs later, so the run must not be able to
/// differ from what was probed:
/// - System and global configuration are dropped, and command-line scope turns
///   the program hooks off: `core.fsmonitor=false`, `core.hooksPath` empty,
///   `protocol.allow=never`, and `diff.external=true` (a no-op program, since
///   an empty value makes `git diff` fail with "cannot run").
/// - Every inherited or session-supplied `GIT_*` variable (`GIT_DIR`,
///   `GIT_CONFIG_PARAMETERS`, `GIT_EXTERNAL_DIFF`, ...) and
///   `RIPGREP_CONFIG_PATH` is removed, so a session environment cannot make the
///   probe and the run judge different repositories or configuration.
pub(crate) fn hardened_env(
    session_names: impl IntoIterator<Item = String>,
) -> Vec<(String, Option<String>)> {
    #[cfg(unix)]
    const NULL: &str = "/dev/null";
    #[cfg(not(unix))]
    const NULL: &str = "NUL";
    let set: [(&str, &str); 14] = [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", NULL),
        ("GIT_CONFIG_COUNT", "4"),
        ("GIT_CONFIG_KEY_0", "core.fsmonitor"),
        ("GIT_CONFIG_VALUE_0", "false"),
        ("GIT_CONFIG_KEY_1", "core.hooksPath"),
        ("GIT_CONFIG_VALUE_1", NULL),
        ("GIT_CONFIG_KEY_2", "protocol.allow"),
        ("GIT_CONFIG_VALUE_2", "never"),
        ("GIT_CONFIG_KEY_3", "diff.external"),
        ("GIT_CONFIG_VALUE_3", "true"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("GIT_ASKPASS", "true"),
        ("GIT_PAGER", "cat"),
    ];
    let mut names: std::collections::BTreeSet<String> = std::env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .chain(session_names)
        .filter(|name| name.starts_with("GIT_") || name == "RIPGREP_CONFIG_PATH")
        .collect();
    names.insert("RIPGREP_CONFIG_PATH".to_string());
    let mut env: Vec<(String, Option<String>)> =
        names.into_iter().map(|name| (name, None)).collect();
    env.extend(
        set.iter()
            .map(|(key, value)| (key.to_string(), Some(value.to_string()))),
    );
    env
}

/// Apply `hardened_env` changes to a command.
pub(crate) fn apply_env(command: &mut std::process::Command, env: &[(String, Option<String>)]) {
    for (key, value) in env {
        match value {
            Some(value) => command.env(key, value),
            None => command.env_remove(key),
        };
    }
}

/// `Some(env)` to apply (after the session environment) to the child of a bash
/// call made by an active overnight coordinator, `None` for any other session.
pub(crate) fn coordinator_command_env(session_id: &str) -> Option<Vec<(String, Option<String>)>> {
    is_coordinator(session_id)
        .then(|| hardened_env(crate::session_env::session_env(session_id).into_keys()))
}

#[cfg(test)]
thread_local! {
    /// Extra environment for the probe in tests; the hardened environment is
    /// applied after it and wins.
    static PROBE_ENV: std::cell::RefCell<Vec<(String, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn git_probe(dir: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    let mut command = std::process::Command::new("git");
    command.current_dir(dir).args(args);
    // Names a test injects as if they came from the daemon's environment.
    #[cfg(test)]
    let injected: Vec<String> = PROBE_ENV.with(|env| {
        for (key, value) in env.borrow().iter() {
            command.env(key, value);
        }
        env.borrow().iter().map(|(key, _)| key.clone()).collect()
    });
    #[cfg(not(test))]
    let injected: Vec<String> = Vec::new();
    apply_env(&mut command, &hardened_env(injected));
    command
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("could not run git to inspect its configuration: {error}"))
}

/// Value of the last `--ignore-submodules[=<v>]` in `flags` (git: last wins).
fn ignore_submodules_value(flags: &[String]) -> Option<String> {
    flags
        .iter()
        .filter_map(|a| {
            if a == "--ignore-submodules" {
                Some(String::new())
            } else {
                a.strip_prefix("--ignore-submodules=").map(str::to_string)
            }
        })
        .next_back()
}

/// True when the repository has a gitlink (mode 160000: a committed nested
/// repository, with or without `.gitmodules`), `submodule.*` configuration or a
/// `modules` directory. git enters each of them for `status` and `diff`.
fn has_submodules(dir: &Path) -> Result<bool, String> {
    let staged = git_probe(dir, &["ls-files", "-s"])?;
    if staged
        .stdout
        .split(|b| *b == b'\n')
        .any(|line| line.starts_with(b"160000 "))
    {
        return Ok(true);
    }
    let has_config = git_probe(
        dir,
        &[
            "config",
            "-z",
            "--get-regexp",
            r"^submodule\..+\.(url|path|active)$",
        ],
    )?
    .status
    .success();
    let modules = git_probe(dir, &["rev-parse", "--git-path", "modules"])?;
    let modules = String::from_utf8_lossy(&modules.stdout).trim().to_string();
    let has_modules = !modules.is_empty()
        && std::fs::read_dir(dir.join(&modules)).is_ok_and(|mut entries| entries.next().is_some());
    Ok(has_config || has_modules)
}

fn config_flag_is_false(value: Option<&str>) -> bool {
    value.is_some_and(|v| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "false" | "no" | "off" | "0"
        )
    })
}

/// Fail closed when running `sub` in `dir` could execute a configured program.
/// `git config --get-regexp` reads configuration only: it runs no filter,
/// fsmonitor, pager or hook (verified with marker scripts), and it follows
/// `include` files, so included keys are seen. `ignored` is true when a leading
/// `--ignore-submodules=all|dirty` keeps git out of submodules.
fn check_git_runs_no_program(
    sub: &str,
    rest: &[String],
    dir: &Path,
    ignored: bool,
) -> Result<(), String> {
    let signed = matches!(sub, "log" | "show");
    let patches = matches!(sub, "log" | "show" | "diff");
    let mut config_enters_submodules = false;
    let out = git_probe(dir, &["config", "-z", "--get-regexp", GIT_PROGRAM_KEYS])?;
    match out.status.code() {
        // Exit 1: no matching key.
        Some(1) => {}
        Some(0) => {
            for entry in out.stdout.split(|b| *b == 0).filter(|e| !e.is_empty()) {
                let entry = String::from_utf8_lossy(entry);
                let (key, value) = entry
                    .split_once('\n')
                    .map_or((&*entry, None), |(k, v)| (k, Some(v)));
                let key = key.to_ascii_lowercase();
                let deny = |why: &str| Err(format!("git config `{key}` makes git {sub} {why}"));
                let has_signature_atom = value.is_some_and(|v| v.contains("%G"));
                match key.as_str() {
                    "core.fsmonitor" => {
                        if !config_flag_is_false(value) {
                            return deny("run a configured program");
                        }
                    }
                    "log.showsignature" => {
                        if signed && !config_flag_is_false(value) {
                            return deny("run gpg");
                        }
                    }
                    "log.diffmerges" => {
                        let remerge =
                            value.is_some_and(|v| v.trim().to_ascii_lowercase().starts_with('r'));
                        if signed && remerge {
                            return deny("run merge drivers");
                        }
                    }
                    "diff.submodule" => {
                        if patches && value.is_some_and(|v| matches!(v.trim(), "diff" | "log")) {
                            config_enters_submodules = true;
                        }
                    }
                    "format.pretty" => {
                        if signed && has_signature_atom {
                            return deny("run gpg");
                        }
                    }
                    k if k.starts_with("pretty.") => {
                        if signed && has_signature_atom {
                            return deny("run gpg");
                        }
                    }
                    k if k.starts_with("gpg.") || k.starts_with("merge.") => {
                        if signed {
                            return deny("run a configured program");
                        }
                    }
                    k if k.starts_with("hook.") => {
                        if matches!(sub, "status" | "diff") {
                            return deny("run a configured hook");
                        }
                    }
                    // `filter.*`, for worktree files.
                    _ => return deny("run a configured program"),
                }
            }
        }
        _ => {
            return Err(format!(
                "could not inspect git configuration ({})",
                String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .next()
                    .unwrap_or("git failed")
            ));
        }
    }

    // A submodule's own configuration and hooks are invisible to the probe
    // above, and its fsmonitor runs for `status` and `diff` unless submodules
    // are ignored (`=all` and `=dirty` were measured safe; `=untracked` and
    // `=none` still enter the submodule). `--submodule=diff|log` or
    // `diff.submodule` enters them for `log` and `show` as well.
    let submodule_option = rest.iter().any(|a| is_abbrev_of(a, "--submodule"));
    let enters = matches!(sub, "status" | "diff")
        || config_enters_submodules
        || (patches && submodule_option);
    if enters && !ignored && has_submodules(dir)? {
        return Err(format!(
            "this repository has submodules or nested repositories whose own configuration git {sub} would trust; add `--ignore-submodules=all` right after `{sub}`"
        ));
    }
    Ok(())
}

/// True when `arg` is `full` or a unique-looking long-option abbreviation of it
/// (git accepts `--ext-d` for `--ext-diff`). Four characters or more, so
/// `--exit-code` and `--stat` never match.
fn is_abbrev_of(arg: &str, full: &str) -> bool {
    let name = arg.split('=').next().unwrap_or(arg);
    name.len() >= 4 && full.starts_with(name)
}

/// Options that make `log`, `show` or `diff` write a file or run a program
/// whatever the configuration says. Checked against every word, pathspecs
/// included: an option that takes a value can consume a `--`, so a word after
/// one may still be an option, and a denial there costs nothing.
fn check_git_dangerous_options(sub: &str, rest: &[String]) -> Result<(), String> {
    let signed = matches!(sub, "log" | "show");
    for (i, a) in rest.iter().enumerate() {
        let next = rest.get(i + 1).map_or("", String::as_str);
        let remerge_merges = is_abbrev_of(a, "--diff-merges")
            && a.split_once('=')
                .map_or(next, |(_, value)| value)
                .to_ascii_lowercase()
                .starts_with('r');
        let bad = a.starts_with("--output")
            || is_abbrev_of(a, "--ext-diff")
            || is_abbrev_of(a, "--textconv")
            // `--remerge-diff` and `--diff-merges=remerge` run merge drivers.
            || is_abbrev_of(a, "--remerge-diff")
            || remerge_merges
            // gpg verifies signatures for the flag and for `%G` placeholders.
            || (signed && (is_abbrev_of(a, "--show-signature") || a.contains("%G")));
        if bad {
            return Err(format!("git {sub} {a} is not read-only"));
        }
    }
    Ok(())
}

fn check_git(args: &[String], cwd: &Path) -> Result<(), String> {
    let mut dir = cwd.to_path_buf();
    let mut no_optional_locks = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--no-pager" => i += 1,
            "--no-optional-locks" => {
                no_optional_locks = true;
                i += 1;
            }
            // `-C` is relative to the previous `-C`, like git itself.
            "-C" if i + 1 < args.len() => {
                dir = dir.join(&args[i + 1]);
                i += 2;
            }
            _ => break,
        }
    }
    let Some(sub) = args.get(i) else {
        return Err("git without subcommand".into());
    };
    let rest = &args[i + 1..];
    if !matches!(sub.as_str(), "status" | "log" | "diff" | "show" | "branch") {
        return Err(format!("git {sub} is not a read-only subcommand"));
    }
    // `status` and `diff` refresh the index, which writes `.git/index` and runs
    // `post-index-change` hooks (a hook file or `hook.<name>.command`) unless
    // optional locks are off. Required unconditionally: no probe can see a hook
    // that is installed after it ran.
    if matches!(sub.as_str(), "status" | "diff") && !no_optional_locks {
        return Err(format!(
            "git {sub} refreshes the index and may run hooks; use `git --no-optional-locks {sub}`"
        ));
    }
    // The opt-out flags count only as the run of flags right after the
    // subcommand. An option that takes a separate value (`-G`, `-S`,
    // `--word-diff-regex`, `--src-prefix`) swallows the next word, so a flag
    // placed after other options may never take effect (measured), and a word
    // after `--` is a pathspec.
    let leading = rest
        .iter()
        .take_while(|a| {
            matches!(a.as_str(), "--no-textconv" | "--no-ext-diff")
                || a.as_str() == "--ignore-submodules"
                || a.starts_with("--ignore-submodules=")
        })
        .count();
    let leading_flags = &rest[..leading];
    // `status` has no option that takes a separate value, so its flag may sit
    // anywhere before a bare `--` (git: last one wins).
    let ignore_scope = if sub == "status" {
        let end = rest.iter().position(|a| a == "--").unwrap_or(rest.len());
        &rest[..end]
    } else {
        leading_flags
    };
    let ignored = ignore_submodules_value(ignore_scope)
        .is_some_and(|v| matches!(v.as_str(), "" | "all" | "dirty"));
    // Configuration that makes an otherwise read-only subcommand run a program.
    // Checked before the subcommand's own rules so the denial names the key.
    check_git_runs_no_program(sub, rest, &dir, ignored)?;
    match sub.as_str() {
        "status" => Ok(()),
        // These render patches, so git runs a `diff.<driver>.textconv` program
        // (named by `.gitattributes` plus repo or global config) or
        // `diff.external` for them unless told not to. The policy layer can
        // only allow or deny, not rewrite, so the coordinator must say so:
        // both flags, spelled exactly, right after the subcommand.
        "log" | "diff" | "show" => {
            check_git_dangerous_options(sub, rest)?;
            for required in ["--no-textconv", "--no-ext-diff"] {
                if !leading_flags.iter().any(|a| a == required) {
                    return Err(format!(
                        "git {sub} can run configured textconv or external diff programs; put `--no-textconv --no-ext-diff` immediately after `{sub}` (missing {required})"
                    ));
                }
            }
            Ok(())
        }
        "branch" => {
            const FLAGS: [&str; 17] = [
                "-a",
                "--all",
                "-r",
                "--remotes",
                "-v",
                "-vv",
                "--verbose",
                "--list",
                "-l",
                "--show-current",
                "--no-color",
                "--color",
                "--contains",
                "--no-contains",
                "--merged",
                "--no-merged",
                "--points-at",
            ];
            let lists = rest.iter().any(|a| {
                matches!(
                    a.as_str(),
                    "--list"
                        | "-l"
                        | "--contains"
                        | "--no-contains"
                        | "--merged"
                        | "--no-merged"
                        | "--points-at"
                )
            });
            for a in rest {
                let ok = if a.starts_with('-') {
                    FLAGS.contains(&a.as_str())
                        || a.starts_with("--format=")
                        || a.starts_with("--sort=")
                } else {
                    lists
                };
                if !ok {
                    return Err(format!("git branch {a} may modify branches"));
                }
            }
            Ok(())
        }
        other => Err(format!("git {other} is not a read-only subcommand")),
    }
}

#[cfg(test)]
#[path = "coordinator_policy_tests.rs"]
pub(in crate::tool) mod tests;
