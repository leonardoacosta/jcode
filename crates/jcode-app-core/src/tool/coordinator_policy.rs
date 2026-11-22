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
    if let Some(manifest) = manifest {
        let _ = crate::overnight::record_event(
            &manifest,
            "coordinator_tool_denied",
            format!("Coordinator call to `{tool}` denied: {reason}"),
            json!({ "tool": tool, "reason": reason }),
            false,
        );
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
            read_only_bash(command).map_err(|why| format!("bash is read-only: {why}"))
        }
        other => Err(format!("`{other}` is not in the coordinator allowlist")),
    }
}

/// Accept only lines made of read-only commands joined by `&&`, `||`, `;`, `|`.
fn read_only_bash(line: &str) -> Result<(), String> {
    let segments = split_segments(line)?;
    if segments.is_empty() {
        return Err("empty command".into());
    }
    for words in segments {
        check_command(&words)?;
    }
    Ok(())
}

/// Quote-aware split into commands of unquoted words. Rejects redirection to
/// files, command/process substitution, heredocs and background operators.
fn split_segments(line: &str) -> Result<Vec<Vec<String>>, String> {
    let chars: Vec<char> = line.chars().collect();
    let mut segments: Vec<Vec<String>> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut word = String::new();
    let mut in_word = false;
    let (mut single, mut double) = (false, false);
    let mut i = 0;

    macro_rules! end_word {
        () => {
            if in_word {
                words.push(std::mem::take(&mut word));
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
        if c == '`' || (c == '$' && next == Some('(')) {
            return Err("command substitution".into());
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
        words.push(word);
    }
    if !words.is_empty() {
        segments.push(words);
    }
    Ok(segments)
}

fn check_command(words: &[String]) -> Result<(), String> {
    let name = words[0].as_str();
    let args = &words[1..];
    match name {
        "rg" => {
            if args.iter().any(|a| a == "--pre" || a.starts_with("--pre=")) {
                return Err("rg --pre runs a command".into());
            }
            Ok(())
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
        "git" => check_git(args),
        other => Err(format!("`{other}` is not a read-only command")),
    }
}

fn check_git(args: &[String]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--no-pager" => i += 1,
            "-C" if i + 1 < args.len() => i += 2,
            _ => break,
        }
    }
    let Some(sub) = args.get(i) else {
        return Err("git without subcommand".into());
    };
    let rest = &args[i + 1..];
    match sub.as_str() {
        "status" | "log" | "diff" | "show" => {
            match rest
                .iter()
                .find(|a| a.starts_with("--output") || *a == "--ext-diff" || *a == "--textconv")
            {
                Some(a) => Err(format!("git {sub} {a} is not read-only")),
                None => Ok(()),
            }
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
