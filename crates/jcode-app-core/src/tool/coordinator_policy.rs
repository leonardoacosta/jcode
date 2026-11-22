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

const DELEGATION_HINT: &str = "The overnight coordinator only coordinates: assign every edit, build, review and judgement to a swarm worker (swarm spawn/assign), and keep your own tools to swarm, todo, bg, read-only inspection and markdown notes.";

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
            // `--pre[-glob]` and `--hostname-bin` execute the named command.
            match args
                .iter()
                .find(|a| a.starts_with("--pre") || a.starts_with("--hostname-bin"))
            {
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

/// Keys in the effective git configuration (system, global, repository,
/// `include` files, `GIT_CONFIG_*` and `-c` environment) that make `status`,
/// `diff`, `log` or `show` execute a configured program:
/// - `filter.<name>.clean|smudge|process` run on worktree files for `status`
///   and worktree `diff`.
/// - `core.fsmonitor` runs a hook program (`true` starts git's own daemon).
/// - `gpg.program`, `gpg.<fmt>.program` verify signatures for `log` and `show`.
///
/// Other program-running keys were measured as unreachable from the
/// allowlisted commands in the bash tool: `core.pager` and `pager.*` need a
/// terminal and the tool sets `GIT_PAGER=cat` with a piped, tty-less stdout;
/// `credential.helper` only runs for network operations; `diff.*.textconv`,
/// `diff.*.command`, `diff.external` and `GIT_EXTERNAL_DIFF` are switched off by
/// the required `--no-textconv --no-ext-diff`. They are not denied, because
/// `core.pager = delta` and `diff.external = difft` are common and denying them
/// would disable git for the coordinator without closing a path.
const GIT_PROGRAM_KEYS: &str =
    r"^(filter\..+\.(clean|smudge|process)|core\.fsmonitor|gpg\.program|gpg\.[^.]+\.program)$";

#[cfg(test)]
thread_local! {
    /// Environment for the probe in tests, so a developer's own git
    /// configuration cannot change the outcome.
    static PROBE_ENV: std::cell::RefCell<Vec<(String, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn git_probe(dir: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    let mut command = std::process::Command::new("git");
    command.current_dir(dir).args(args);
    #[cfg(test)]
    PROBE_ENV.with(|env| {
        for (key, value) in env.borrow().iter() {
            command.env(key, value);
        }
    });
    command
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("could not run git to inspect its configuration: {error}"))
}

fn git_flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter().find_map(|a| {
        if a == flag {
            Some(String::new())
        } else {
            a.strip_prefix(flag)
                .and_then(|v| v.strip_prefix('='))
                .map(str::to_string)
        }
    })
}

/// Fail closed when running `sub` in `dir` could execute a configured program.
/// `git config --get-regexp` reads configuration only: it runs no filter,
/// fsmonitor, pager or hook (verified with marker scripts), and it follows
/// `include` files, so included keys are seen.
fn check_git_runs_no_program(
    sub: &str,
    rest: &[String],
    dir: &Path,
    no_optional_locks: bool,
) -> Result<(), String> {
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
                let is_gpg = key.starts_with("gpg.");
                if is_gpg && !matches!(sub, "log" | "show") {
                    continue;
                }
                let disabled = key == "core.fsmonitor"
                    && value.is_some_and(|v| {
                        matches!(
                            v.trim().to_ascii_lowercase().as_str(),
                            "false" | "no" | "off" | "0"
                        )
                    });
                if disabled {
                    continue;
                }
                return Err(format!(
                    "git config `{key}` makes git {sub} run a configured program"
                ));
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
    if !matches!(sub, "status" | "diff") {
        return Ok(());
    }

    // `status` and `diff` refresh the index, which runs `post-index-change`
    // unless optional locks are off. The hooks path honours `core.hooksPath`.
    if !no_optional_locks {
        let hook = git_probe(dir, &["rev-parse", "--git-path", "hooks/post-index-change"])?;
        let path = String::from_utf8_lossy(&hook.stdout).trim().to_string();
        if hook.status.success() && !path.is_empty() && dir.join(&path).is_file() {
            return Err(
                "git would run the post-index-change hook; add `--no-optional-locks` before the subcommand".into(),
            );
        }
    }

    // A submodule's own configuration and hooks are invisible to the probe
    // above, and its fsmonitor runs for `status` and `diff` unless submodules
    // are ignored (`=all` and `=dirty` were measured safe; `=untracked` and
    // `=none` still enter the submodule). Initialized submodules are those with
    // `submodule.*` configuration or a `modules` directory.
    let ignored = git_flag_value(rest, "--ignore-submodules")
        .is_some_and(|v| matches!(v.as_str(), "" | "all" | "dirty"));
    if !ignored {
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
            && std::fs::read_dir(dir.join(&modules))
                .is_ok_and(|mut entries| entries.next().is_some());
        if has_config || has_modules {
            return Err(format!(
                "this repository has initialized submodules whose own configuration git {sub} would trust; add `--ignore-submodules=all`"
            ));
        }
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
    // Configuration that makes an otherwise read-only subcommand run a program.
    // Checked before the subcommand's own rules so the denial names the key.
    check_git_runs_no_program(sub, rest, &dir, no_optional_locks)?;
    match sub.as_str() {
        "status" => Ok(()),
        // These render patches, so git runs a `diff.<driver>.textconv` program
        // (named by `.gitattributes` plus repo or global config) or
        // `diff.external` for them unless told not to. The policy layer can
        // only allow or deny, not rewrite, so the coordinator must say so:
        // both flags, spelled exactly, on every diff/log/show.
        "log" | "diff" | "show" => {
            if let Some(a) = rest.iter().find(|a| {
                a.starts_with("--output")
                    || is_abbrev_of(a, "--ext-diff")
                    || is_abbrev_of(a, "--textconv")
            }) {
                return Err(format!("git {sub} {a} is not read-only"));
            }
            for required in ["--no-textconv", "--no-ext-diff"] {
                if !rest.iter().any(|a| a == required) {
                    return Err(format!(
                        "git {sub} can run configured textconv or external diff programs; add `--no-textconv --no-ext-diff` (missing {required})"
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
