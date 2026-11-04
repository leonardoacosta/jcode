use super::*;

/// A dispatch policy, not an OS sandbox. Reads can still access files or networks.
#[derive(Debug)]
pub struct PlanModeDenied(pub String);
impl std::fmt::Display for PlanModeDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plan mode denied: {}", self.0)
    }
}
impl std::error::Error for PlanModeDenied {}

static PLAN_TURNS: LazyLock<StdRwLock<HashMap<String, usize>>> =
    LazyLock::new(|| StdRwLock::new(HashMap::new()));

/// Owned by the turn future. Completion, error, panic and cancellation all drop it.
pub struct PlanTurnGuard(String);
impl PlanTurnGuard {
    pub fn new(session: &str) -> Self {
        *PLAN_TURNS
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .entry(session.into())
            .or_default() += 1;
        Self(session.into())
    }
}
impl Drop for PlanTurnGuard {
    fn drop(&mut self) {
        let mut turns = PLAN_TURNS.write().unwrap_or_else(|e| e.into_inner());
        if let Some(count) = turns.get_mut(&self.0) {
            *count -= 1;
            if *count == 0 {
                turns.remove(&self.0);
            }
        }
    }
}
pub fn plan_turn_active(session: &str) -> bool {
    PLAN_TURNS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .contains_key(session)
}

pub(crate) fn admit_plan_tool(name: &str, input: &Value, custom: bool) -> Result<()> {
    let action = input.get("action").and_then(Value::as_str).unwrap_or("");
    let allowed = !custom
        && match name {
            "read"
            | "agentgrep"
            | "conversation_search"
            | "session_search"
            | "webfetch"
            | "websearch"
            | "batch" => true,
            "bash" => {
                plan_bash_argv(input.get("command").and_then(Value::as_str).unwrap_or(""))?;
                input.get("run_in_background").is_none_or(|v| v == false)
            }
            // Only explicit observation actions. Unknown tools/actions fail closed.
            "browser" => {
                input.get("path").is_none_or(Value::is_null)
                    && matches!(
                        action,
                        "status"
                            | "list_tabs"
                            | "get_active_tab"
                            | "list_frames"
                            | "snapshot"
                            | "get_content"
                            | "interactables"
                    )
            }
            "memory" => matches!(action, "recall" | "search" | "list" | "related"),
            "skill" => matches!(action, "list" | "read"),
            _ => false,
        };
    if allowed {
        Ok(())
    } else {
        Err(PlanModeDenied(format!("tool '{name}' is not a read-only plan operation")).into())
    }
}

/// Deliberately small command language. Quoted literals work, shell syntax never does.
/// Unknown options are refused rather than guessed safe, including platform options.
pub(crate) fn plan_bash_argv(command: &str) -> Result<Vec<String>> {
    let deny = || {
        PlanModeDenied("bash allows only literal git status/diff/log, rg, ls, cat with known read-only options; no shell syntax".into())
    };
    if command
        .chars()
        .any(|c| c.is_control() || ";|&<>$`\\#!{}()~".contains(c))
    {
        return Err(deny().into());
    }
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    for c in command.chars() {
        if Some(c) == quote {
            quote = None;
        } else if quote.is_some() {
            word.push(c);
        } else if matches!(c, '\'' | '"') {
            quote = Some(c);
            started = true;
        } else if c == ' ' {
            if started {
                words.push(std::mem::take(&mut word));
                started = false;
            }
        } else {
            word.push(c);
            started = true;
        }
    }
    if quote.is_some() {
        return Err(deny().into());
    }
    if started {
        words.push(word);
    }
    let (program, args) = words.split_first().ok_or_else(deny)?;
    let (kind, args) = if program == "git" {
        let (sub, args) = args.split_first().ok_or_else(deny)?;
        if !matches!(sub.as_str(), "status" | "diff" | "log") {
            return Err(deny().into());
        }
        (sub.as_str(), args)
    } else if matches!(program.as_str(), "rg" | "ls" | "cat") {
        (program.as_str(), args)
    } else {
        return Err(deny().into());
    };
    let mut positional_only = false;
    for arg in args {
        if positional_only {
            continue;
        }
        if arg == "--" {
            positional_only = true;
            continue;
        }
        if !arg.starts_with('-') {
            continue;
        }
        let safe = match kind {
            "status" => matches!(
                arg.as_str(),
                "--short"
                    | "--branch"
                    | "--porcelain"
                    | "--porcelain=v1"
                    | "--porcelain=v2"
                    | "-s"
                    | "-b"
                    | "--untracked-files=no"
                    | "--untracked-files=normal"
                    | "--untracked-files=all"
            ),
            "diff" => matches!(
                arg.as_str(),
                "--stat"
                    | "--numstat"
                    | "--shortstat"
                    | "--name-only"
                    | "--name-status"
                    | "--cached"
                    | "--staged"
                    | "--check"
                    | "--no-color"
                    | "--no-ext-diff"
                    | "--no-textconv"
                    | "-p"
                    | "--patch"
            ),
            "log" => {
                matches!(
                    arg.as_str(),
                    "--oneline"
                        | "--stat"
                        | "--name-only"
                        | "--name-status"
                        | "--no-color"
                        | "--all"
                        | "--graph"
                        | "--no-decorate"
                ) || arg
                    .strip_prefix('-')
                    .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
            }
            "rg" => matches!(
                arg.as_str(),
                "-n" | "--line-number"
                    | "-i"
                    | "--ignore-case"
                    | "-l"
                    | "--files-with-matches"
                    | "--files"
                    | "--hidden"
                    | "--no-ignore"
                    | "-F"
                    | "--fixed-strings"
                    | "-w"
                    | "--word-regexp"
                    | "-c"
                    | "--count"
                    | "--no-heading"
                    | "--no-config"
            ),
            "ls" => matches!(
                arg.as_str(),
                "-l" | "-a"
                    | "-la"
                    | "-al"
                    | "-lh"
                    | "-lah"
                    | "-h"
                    | "-A"
                    | "-1"
                    | "--all"
                    | "--almost-all"
            ),
            "cat" => matches!(
                arg.as_str(),
                "-n" | "-b" | "-s" | "--number" | "--number-nonblank" | "--squeeze-blank"
            ),
            _ => false,
        };
        if !safe {
            return Err(deny().into());
        }
    }
    Ok(words)
}

pub(crate) async fn execute_plan_bash(input: &Value, ctx: &ToolContext) -> Result<ToolOutput> {
    let words = plan_bash_argv(input.get("command").and_then(Value::as_str).unwrap_or(""))?;
    // Use trusted platform binaries, not cwd/PATH wrappers, functions or aliases.
    #[cfg(unix)]
    let executable = ["/usr/bin", "/bin"]
        .iter()
        .map(|dir| std::path::Path::new(dir).join(&words[0]))
        .find(|path| path.is_file())
        .ok_or_else(|| PlanModeDenied(format!("trusted '{}' executable unavailable", words[0])))?;
    #[cfg(not(unix))]
    return Err(PlanModeDenied(
        "read-only bash execution is unavailable on this platform; use native read/agentgrep"
            .into(),
    )
    .into());
    #[cfg(unix)]
    {
        let mut command = tokio::process::Command::new(executable);
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("LC_ALL", "C")
            .kill_on_drop(true)
            .stdin(std::process::Stdio::null());
        if let Some(dir) = &ctx.working_dir {
            command.current_dir(dir);
        }
        if words[0] == "git" {
            command
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_ATTR_NOSYSTEM", "1")
                .env("GIT_OPTIONAL_LOCKS", "0")
                .env("GIT_TERMINAL_PROMPT", "0")
                .args([
                    "--no-pager",
                    "-c",
                    "core.fsmonitor=false",
                    "-c",
                    "core.hooksPath=/dev/null",
                    "-c",
                    "core.attributesFile=/dev/null",
                    "-c",
                    "submodule.recurse=false",
                    "-c",
                    "log.showSignature=false",
                ]);
            // Diff/status can invoke clean/process filters even without textconv.
            // Config enumeration itself does not run these executable values.
            let mut probe = tokio::process::Command::new(command.as_std().get_program());
            probe
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_PAGER", "cat")
                .args([
                    "--no-pager",
                    "config",
                    "--null",
                    "--name-only",
                    "--get-regexp",
                    "^filter\\.",
                ])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .kill_on_drop(true);
            if let Some(dir) = &ctx.working_dir {
                probe.current_dir(dir);
            }
            let config = tokio::time::timeout(std::time::Duration::from_secs(2), probe.status())
                .await
                .map_err(|_| {
                    PlanModeDenied("Git filter configuration inspection timed out".into())
                })??;
            if !config.success() && config.code() != Some(1) {
                return Err(PlanModeDenied(
                    "Git filter configuration could not be inspected".into(),
                )
                .into());
            }
            if config.success() {
                return Err(PlanModeDenied("Git repositories with configured filters require native read/agentgrep during plan mode".into()).into());
            }
            command.arg(&words[1]);
            if words[1] == "status" {
                command.arg("--ignore-submodules=all");
            }
            if matches!(words[1].as_str(), "diff" | "log") {
                command.args(["--no-ext-diff", "--no-textconv", "--ignore-submodules=all"]);
            }
            command.args(&words[2..]);
        } else {
            if words[0] == "rg" {
                command.arg("--no-config");
            }
            command.args(&words[1..]);
        }
        let started = std::time::Instant::now();
        let timeout = input
            .get("timeout")
            .and_then(Value::as_u64)
            .unwrap_or(120_000)
            .clamp(1, 120_000);
        use tokio::io::AsyncReadExt;
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = command.spawn()?;
        let mut stdout = child.stdout.take().unwrap().take(30_001);
        let mut stderr = child.stderr.take().unwrap().take(30_001);
        let capture = async {
            let mut out = Vec::new();
            let mut err = Vec::new();
            let stdout_first = tokio::select! {
                result = stdout.read_to_end(&mut out) => { result?; true },
                result = stderr.read_to_end(&mut err) => { result?; false },
            };
            if out.len() > 30_000 || err.len() > 30_000 {
                child.kill().await?;
            }
            if stdout_first {
                stderr.read_to_end(&mut err).await?;
            } else {
                stdout.read_to_end(&mut out).await?;
            }
            if out.len() > 30_000 || err.len() > 30_000 {
                let _ = child.kill().await;
            }
            let status = child.wait().await?;
            Ok::<_, std::io::Error>((out, err, status))
        };
        let (out, err, status) =
            tokio::time::timeout(std::time::Duration::from_millis(timeout), capture)
                .await
                .map_err(|_| PlanModeDenied("read-only command timed out".into()))??;
        let code = status.code().unwrap_or(-1);
        let text = format!(
            "{}{}\nExit code: {code}",
            String::from_utf8_lossy(&out),
            String::from_utf8_lossy(&err)
        );
        Ok(ToolOutput::new(crate::util::truncate_str(&text, 30_000)).with_metadata(serde_json::json!({"process":{"foreground":true,"completed":true,"exit_code":status.code()},"duration_ms":started.elapsed().as_millis().min(u64::MAX as u128) as u64})))
    }
}
