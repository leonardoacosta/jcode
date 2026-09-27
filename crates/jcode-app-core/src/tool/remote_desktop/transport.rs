//! Bounded SSH transport for the finite remote desktop CLI surface.
//!
//! This module intentionally has no arbitrary command or flag input. The remote
//! command is assembled from fixed tokens and POSIX-quoted structured values.

use serde::Deserialize;
use serde_json::Value;
use std::io;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command as ProcessCommand;
use tokio::time::timeout;

const MAX_STDOUT: usize = 1024 * 1024;
const MAX_STDERR: usize = 16 * 1024;
const MAX_JSON_DEPTH: usize = 64;
const MAX_ARG_BYTES: usize = 16 * 1024;
const MAX_TARGET_BYTES: usize = 255;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operation {
    Status,
    ListApps,
    ListWindows,
    Snapshot,
    Find,
    Get,
    Is,
    Click,
    Type,
    Press,
}

impl Operation {
    fn tokens(self) -> &'static [&'static str] {
        match self {
            Self::Status => &["status"],
            Self::ListApps => &["list-apps"],
            Self::ListWindows => &["list-windows"],
            Self::Snapshot => &["snapshot"],
            Self::Find => &["find"],
            Self::Get => &["get"],
            Self::Is => &["is"],
            Self::Click => &["click"],
            Self::Type => &["type"],
            Self::Press => &["press"],
        }
    }

    fn mutates(self) -> bool {
        matches!(self, Self::Click | Self::Type | Self::Press)
    }
}

/// Operation-specific inputs. Flags are emitted only by this module, never by
/// callers, so excluded CLI capabilities cannot be smuggled through argv.
#[derive(Clone, Debug)]
pub enum Command {
    Status,
    ListApps,
    ListWindows,
    Snapshot {
        app: String,
    },
    Find {
        app: Option<String>,
        role: Option<String>,
        name: Option<String>,
        text: Option<String>,
    },
    Get {
        reference: String,
        property: String,
    },
    Is {
        reference: String,
        property: String,
    },
    Click {
        reference: String,
    },
    Type {
        reference: String,
        text: String,
    },
    Press {
        key: String,
    },
}

impl Command {
    fn operation(&self) -> Operation {
        match self {
            Self::Status => Operation::Status,
            Self::ListApps => Operation::ListApps,
            Self::ListWindows => Operation::ListWindows,
            Self::Snapshot { .. } => Operation::Snapshot,
            Self::Find { .. } => Operation::Find,
            Self::Get { .. } => Operation::Get,
            Self::Is { .. } => Operation::Is,
            Self::Click { .. } => Operation::Click,
            Self::Type { .. } => Operation::Type,
            Self::Press { .. } => Operation::Press,
        }
    }

    fn args(&self) -> Result<Vec<String>, TransportError> {
        let mut args = Vec::new();
        let add = |args: &mut Vec<String>, flag: &str, value: &str| -> Result<(), TransportError> {
            if !safe_text(value) || value.is_empty() || value.starts_with('-') {
                return Err(TransportError::InvalidInput("invalid option value"));
            }
            args.push(flag.to_owned());
            args.push(value.to_owned());
            Ok(())
        };
        match self {
            Self::Status | Self::ListApps | Self::ListWindows => {}
            Self::Snapshot { app } => {
                add(&mut args, "--app", app)?;
                args.push("--interactive-only".to_owned());
                args.push("--compact".to_owned());
            }
            Self::Find {
                app,
                role,
                name,
                text,
            } => {
                for (flag, value) in [
                    ("--app", app),
                    ("--role", role),
                    ("--name", name),
                    ("--text", text),
                ] {
                    if let Some(value) = value {
                        add(&mut args, flag, value)?;
                    }
                }
                if args.is_empty() {
                    return Err(TransportError::InvalidInput(
                        "find requires at least one selector",
                    ));
                }
            }
            Self::Get {
                reference,
                property,
            }
            | Self::Is {
                reference,
                property,
            } => {
                positional(&mut args, reference)?;
                let allowed: &[&str] = if matches!(self, Self::Get { .. }) {
                    &["text", "value", "title", "bounds", "role", "states"]
                } else {
                    &[
                        "visible", "enabled", "checked", "focused", "expanded", "selected",
                    ]
                };
                if !allowed.contains(&property.as_str()) {
                    return Err(TransportError::InvalidInput("unsupported property"));
                }
                add(&mut args, "--property", property)?;
            }
            Self::Click { reference } => positional(&mut args, reference)?,
            Self::Type { reference, text } => {
                positional(&mut args, reference)?;
                positional(&mut args, text)?;
            }
            Self::Press { key } => positional(&mut args, key)?,
        }
        Ok(args)
    }
}

fn positional(args: &mut Vec<String>, value: &str) -> Result<(), TransportError> {
    if !safe_text(value) || value.is_empty() || value.starts_with('-') {
        return Err(TransportError::InvalidInput("invalid positional argument"));
    }
    args.push(value.to_owned());
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Request {
    /// OpenSSH destination/alias, not an arbitrary option string.
    pub destination: String,
    /// Absolute remote executable path, constrained to a plain path.
    pub executable: String,
    operation: Command,
    timeout: Duration,
    /// Optional target-specific stdout limit, further capped by the hard maximum.
    max_output_bytes: Option<usize>,
}

impl Request {
    pub fn new(
        destination: String,
        executable: String,
        operation: Command,
        timeout: Duration,
        max_output_bytes: Option<usize>,
    ) -> Self {
        Self {
            destination,
            executable,
            operation,
            timeout,
            max_output_bytes,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Envelope {
    pub version: String,
    pub ok: bool,
    pub command: String,
    #[serde(default)]
    pub data: Option<Value>,
    #[serde(default)]
    pub error: Option<RemoteError>,
    #[serde(default)]
    pub disposition: Option<Disposition>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Disposition {
    pub delivery: Delivery,
    pub retry: Retry,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RemoteError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub suggestion: Option<String>,
    #[serde(default)]
    pub details: Option<Value>,
    #[serde(default)]
    pub disposition: Option<Disposition>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    NotDelivered,
    DeliveryUncertain,
    DeliveredUnverified,
    DeliveredVerified,
    Unknown,
    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Retry {
    Safe,
    Unsafe,
    Unknown,
    #[serde(other)]
    Other,
}

#[derive(Debug)]
pub enum TransportError {
    InvalidInput(&'static str),
    Io(io::Error),
    Timeout { mutating: bool },
    OutputLimit,
    Protocol(&'static str),
    SshFailed,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(reason) => write!(f, "invalid SSH request: {reason}"),
            Self::Io(_) => f.write_str("could not run SSH transport"),
            Self::Timeout { mutating: true } => f.write_str("SSH timed out; action delivery is uncertain. Observe the desktop before deciding whether to act again."),
            Self::Timeout { mutating: false } => f.write_str("SSH timed out"),
            Self::OutputLimit => f.write_str("SSH response exceeded its size limit"),
            Self::Protocol(reason) => write!(f, "invalid remote desktop response: {reason}"),
            Self::SshFailed => f.write_str("SSH failed; check connectivity, authentication, and configured host-key trust"),
        }
    }
}

impl std::error::Error for TransportError {}

impl From<io::Error> for TransportError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

fn safe_text(s: &str) -> bool {
    s.len() <= MAX_ARG_BYTES && !s.chars().any(|c| c == '\0' || c.is_control())
}

fn validate(request: &Request) -> Result<(), TransportError> {
    let dest = &request.destination;
    if dest.is_empty()
        || dest.len() > MAX_TARGET_BYTES
        || !dest
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-@:".contains(&b))
        || dest.starts_with('-')
    {
        return Err(TransportError::InvalidInput("invalid SSH destination"));
    }
    let path = &request.executable;
    if !path.starts_with('/')
        || path.len() > MAX_ARG_BYTES
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
    {
        return Err(TransportError::InvalidInput(
            "executable must be an absolute plain path",
        ));
    }
    request.operation.args()?;
    if request
        .max_output_bytes
        .is_some_and(|limit| limit == 0 || limit > MAX_STDOUT)
    {
        return Err(TransportError::InvalidInput(
            "output limit must be within the hard cap",
        ));
    }
    if request.timeout.is_zero() || request.timeout > Duration::from_secs(300) {
        return Err(TransportError::InvalidInput(
            "timeout must be between 1ms and 300s",
        ));
    }
    Ok(())
}

/// POSIX single-quote encoding. Rejects bytes that cannot safely be represented
/// in a process command (NUL and control characters are rejected separately).
pub fn quote_posix(value: &str) -> Result<String, TransportError> {
    if !safe_text(value) {
        return Err(TransportError::InvalidInput(
            "argument contains NUL/control bytes or is too long",
        ));
    }
    Ok(format!("'{}'", value.replace('\'', "'\\''")))
}

fn remote_command(request: &Request) -> Result<String, TransportError> {
    validate(request)?;
    let mut words = vec![quote_posix(&request.executable)?];
    words.extend(
        request
            .operation
            .operation()
            .tokens()
            .iter()
            .map(|token| quote_posix(token))
            .collect::<Result<Vec<_>, _>>()?,
    );
    words.extend(
        request
            .operation
            .args()?
            .iter()
            .map(|arg| quote_posix(arg))
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(words.join(" "))
}

fn json_depth(value: &Value, depth: usize) -> bool {
    if depth > MAX_JSON_DEPTH {
        return false;
    }
    match value {
        Value::Array(items) => items.iter().all(|v| json_depth(v, depth + 1)),
        Value::Object(items) => items.values().all(|v| json_depth(v, depth + 1)),
        _ => true,
    }
}

async fn read_capped<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> Result<Vec<u8>, TransportError> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = reader.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        if out.len().saturating_add(n) > max {
            return Err(TransportError::OutputLimit);
        }
        out.extend_from_slice(&chunk[..n]);
    }
    Ok(out)
}

/// Run exactly one bounded SSH process. No retry is performed, including for reads.
pub async fn execute(request: &Request) -> Result<Envelope, TransportError> {
    execute_with_program(request, "ssh").await
}

async fn execute_with_program(
    request: &Request,
    ssh_program: &str,
) -> Result<Envelope, TransportError> {
    execute_with_program_env(request, ssh_program, None).await
}

async fn execute_with_program_env(
    request: &Request,
    ssh_program: &str,
    marker: Option<&std::path::Path>,
) -> Result<Envelope, TransportError> {
    validate(request)?;
    let remote = remote_command(request)?;
    let mut child = ProcessCommand::new(ssh_program);
    if let Some(marker) = marker {
        child.env("SSH_PID_FILE", marker);
    }
    child
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("StrictHostKeyChecking=yes")
        .arg("-o")
        .arg("ConnectTimeout=10")
        .arg("--")
        .arg(&request.destination)
        .arg(remote)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = child.spawn()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| TransportError::Io(io::Error::other("missing stdout")))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| TransportError::Io(io::Error::other("missing stderr")))?;
    let stdout_limit = request.max_output_bytes.unwrap_or(MAX_STDOUT);
    let run = async {
        let (out, _err) = tokio::try_join!(
            read_capped(&mut stdout, stdout_limit),
            read_capped(&mut stderr, MAX_STDERR)
        )?;
        let status = child.wait().await?;
        if status.code().is_none() || status.code() == Some(255) {
            return Err(TransportError::SshFailed);
        }
        let value: Value = serde_json::from_slice(&out).map_err(|_| {
            if status.success() {
                TransportError::Protocol("malformed JSON")
            } else {
                TransportError::SshFailed
            }
        })?;
        if !json_depth(&value, 0) {
            return Err(TransportError::Protocol("JSON nesting limit exceeded"));
        }
        let envelope: Envelope = serde_json::from_value(value)
            .map_err(|_| TransportError::Protocol("unexpected envelope"))?;
        if envelope.version != "2.4" {
            return Err(TransportError::Protocol("unsupported protocol version"));
        }
        if envelope.command != request.operation.operation().tokens()[0] {
            return Err(TransportError::Protocol(
                "response command does not match request",
            ));
        }
        if envelope.ok && !status.success() {
            return Err(TransportError::Protocol(
                "success envelope with nonzero exit",
            ));
        }
        if envelope.ok == envelope.error.is_some() {
            return Err(TransportError::Protocol("inconsistent error envelope"));
        }
        if envelope.ok && envelope.data.is_none() {
            return Err(TransportError::Protocol("successful response has no data"));
        }
        if !envelope.ok
            && envelope.disposition.is_none()
            && envelope
                .error
                .as_ref()
                .and_then(|error| error.disposition.as_ref())
                .is_none()
        {
            return Err(TransportError::Protocol(
                "failure response lacks disposition",
            ));
        }
        Ok(envelope)
    };
    match timeout(request.timeout, run).await {
        Ok(result) => result,
        Err(_) => Err(TransportError::Timeout {
            mutating: request.operation.operation().mutates(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn req(operation: Command) -> Request {
        Request::new(
            "mac".into(),
            "/opt/agent-desktop".into(),
            operation,
            Duration::from_secs(5),
            None,
        )
    }

    #[test]
    fn posix_quote_round_trips_hostile_arguments() {
        let values = [
            "",
            "two words",
            "'quoted'",
            "$HOME",
            "`touch /tmp/nope`",
            "x; echo pwned",
            "雪☃",
        ];
        for value in values {
            let quoted = quote_posix(value).unwrap();
            let script = format!("set -- {quoted}; printf '%s' \"$1\"");
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(script)
                .output()
                .unwrap();
            assert_eq!(output.stdout, value.as_bytes());
        }
    }

    #[test]
    fn rejects_controls_and_command_injection_fields() {
        assert!(quote_posix("bad\0arg").is_err());
        assert!(quote_posix("bad\x1barg").is_err());
        assert!(
            validate(&Request {
                destination: "-oProxyCommand=evil".into(),
                ..req(Command::Status)
            })
            .is_err()
        );
        assert!(
            validate(&Request {
                executable: "/bin/x;touch".into(),
                ..req(Command::Status)
            })
            .is_err()
        );
        let excluded = [
            "--screenshot",
            "--clipboard",
            "--script",
            "--batch",
            "--raw-coordinates",
        ];
        let commands = [
            Command::Snapshot {
                app: "Finder".into(),
            },
            Command::Find {
                app: Some("Finder".into()),
                role: None,
                name: None,
                text: None,
            },
            Command::Get {
                reference: "ref-1".into(),
                property: "title".into(),
            },
            Command::Is {
                reference: "ref-1".into(),
                property: "enabled".into(),
            },
            Command::Click {
                reference: "ref-1".into(),
            },
            Command::Type {
                reference: "ref-1".into(),
                text: "hello".into(),
            },
            Command::Press {
                key: "Return".into(),
            },
        ];
        for command in commands {
            let argv = command.args().unwrap();
            assert!(!argv.iter().any(|arg| excluded.contains(&arg.as_str())));
            assert!(
                argv.iter()
                    .filter(|arg| arg.starts_with("--"))
                    .all(|arg| matches!(
                        arg.as_str(),
                        "--app"
                            | "--role"
                            | "--name"
                            | "--text"
                            | "--property"
                            | "--interactive-only"
                            | "--compact"
                    ))
            );
        }
        for option in excluded {
            assert!(positional(&mut Vec::new(), option).is_err());
        }
        assert!(
            Command::Snapshot {
                app: "--screenshot".into()
            }
            .args()
            .is_err()
        );
        assert!(
            Command::Get {
                reference: "ref-1".into(),
                property: "screenshot".into()
            }
            .args()
            .is_err()
        );
        assert!(
            Command::Is {
                reference: "ref-1".into(),
                property: "coordinates".into()
            }
            .args()
            .is_err()
        );
        assert_eq!(
            Command::Type {
                reference: "ref-1".into(),
                text: "hello".into()
            }
            .args()
            .unwrap(),
            ["ref-1", "hello"]
        );
    }

    #[tokio::test]
    async fn nonzero_cli_failure_preserves_delivery_disposition() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = dir.path().join("ssh");
        std::fs::write(&ssh, "#!/bin/sh\nprintf '%s' '{\"version\":\"2.4\",\"ok\":false,\"command\":\"click\",\"error\":{\"code\":\"ACTION_FAILED\",\"message\":\"fixture\",\"disposition\":{\"delivery\":\"delivery_uncertain\",\"retry\":\"unsafe\"}}}'\nexit 1\n").unwrap();
        std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
        let result = execute_with_program(
            &req(Command::Click {
                reference: "ref-1".into(),
            }),
            ssh.to_str().unwrap(),
        )
        .await
        .unwrap();
        assert!(!result.ok);
        let error = result.error.unwrap();
        assert_eq!(error.code, "ACTION_FAILED");
        assert_eq!(
            error.disposition.unwrap(),
            Disposition {
                delivery: Delivery::DeliveryUncertain,
                retry: Retry::Unsafe
            }
        );
    }

    #[tokio::test]
    #[ignore = "requires explicitly supplied trusted Mac target and executable"]
    async fn live_read_only_mac_transport() {
        let destination = std::env::var("JCODE_DESKTOP_TEST_HOST").expect("explicit host required");
        let executable =
            std::env::var("JCODE_DESKTOP_TEST_EXECUTABLE").expect("explicit executable required");
        let request = |operation| {
            Request::new(
                destination.clone(),
                executable.clone(),
                operation,
                Duration::from_secs(40),
                None,
            )
        };
        let status = execute(&request(Command::Status))
            .await
            .expect("status transport failed");
        assert!(status.ok);
        let windows = execute(&request(Command::ListWindows))
            .await
            .expect("window inventory transport failed");
        assert!(windows.ok);
        let data = windows.data.expect("inventory data missing");
        let windows = data.as_array().expect("inventory is not an array");
        let app = windows
            .iter()
            .find(|w| w["accessible"] == true && w["visible"] == true)
            .and_then(|w| w["app_name"].as_str())
            .expect("no accessible visible app available");
        let snapshot = execute(&request(Command::Snapshot { app: app.into() }))
            .await
            .expect("snapshot transport failed");
        assert!(snapshot.ok, "snapshot did not succeed");
        let data = snapshot.data.expect("snapshot data missing");
        assert_eq!(data["complete"], true);
        assert!(data["ref_count"].as_u64().is_some_and(|n| n > 0));
        let missing = execute(&request(Command::Snapshot {
            app: "JcodeNonexistentAcceptanceApp9f342".into(),
        }))
        .await
        .expect("remote failure envelope was discarded");
        assert!(!missing.ok);
        let error = missing.error.expect("error missing");
        assert!(error.disposition.is_some() || missing.disposition.is_some());
        println!(
            "Real SSH: status PASS, window inventory PASS, complete snapshot with references PASS, nonzero failure envelope preserved PASS"
        );
    }

    #[tokio::test]
    async fn configured_deadline_above_thirty_seconds_is_honored() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = dir.path().join("ssh");
        std::fs::write(&ssh, "#!/bin/sh\nsleep 31\nprintf '%s' '{\"version\":\"2.4\",\"ok\":true,\"command\":\"status\",\"data\":{}}'\n").unwrap();
        std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
        let request = Request {
            timeout: Duration::from_secs(40),
            ..req(Command::Status)
        };
        assert!(
            execute_with_program(&request, ssh.to_str().unwrap())
                .await
                .unwrap()
                .ok
        );
    }

    #[tokio::test]
    async fn failed_ssh_or_inconsistent_envelopes_never_succeed() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = dir.path().join("ssh");
        for (exit, body) in [
            (
                255,
                r#"{"version":"2.4","ok":true,"command":"status","data":{}}"#,
            ),
            (
                1,
                r#"{"version":"2.4","ok":true,"command":"status","data":{}}"#,
            ),
            (
                0,
                r#"{"version":"2.4","ok":false,"command":"status","disposition":{"delivery":"unknown","retry":"unknown"}}"#,
            ),
            (1, "invalid JSON"),
        ] {
            std::fs::write(
                &ssh,
                format!("#!/bin/sh\nprintf '%s' '{}'\nexit {exit}\n", body),
            )
            .unwrap();
            std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
            assert!(
                execute_with_program(&req(Command::Status), ssh.to_str().unwrap())
                    .await
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn timeout_kills_ssh_child_process() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = dir.path().join("ssh");
        let pid_file = dir.path().join("pid");
        std::fs::write(
            &ssh,
            "#!/bin/sh\necho $$ > \"$SSH_PID_FILE\"\nexec sleep 10\n",
        )
        .unwrap();
        std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
        let request = Request {
            timeout: Duration::from_millis(100),
            ..req(Command::Click {
                reference: "ref-1".into(),
            })
        };
        let result = execute_with_program_env(&request, ssh.to_str().unwrap(), Some(&pid_file))
            .await
            .unwrap_err();
        assert!(matches!(result, TransportError::Timeout { mutating: true }));
        let pid: u32 = std::fs::read_to_string(pid_file)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"));
        assert!(
            stat.is_err() || stat.unwrap().split_whitespace().nth(2) == Some("Z"),
            "timed-out SSH process is still running"
        );
    }

    #[tokio::test]
    async fn fake_ssh_receives_only_fixed_command_and_validates_envelope() {
        let dir = tempfile::tempdir().unwrap();
        let ssh = dir.path().join("ssh");
        std::fs::write(&ssh, "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$SSH_CAPTURE\"\nprintf '%s' '{\"version\":\"2.4\",\"ok\":true,\"command\":\"find\",\"data\":{\"items\":[]}}'\n").unwrap();
        std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o755)).unwrap();
        let capture = dir.path().join("args");
        // Set process environment only for this child invocation. Tests avoid mutating global env.
        let remote = remote_command(&req(Command::Find {
            app: None,
            role: None,
            name: Some("x';touch /tmp/nope;#".into()),
            text: None,
        }))
        .unwrap();
        let mut child = ProcessCommand::new(&ssh)
            .arg("-o")
            .arg("StrictHostKeyChecking=yes")
            .arg("--")
            .arg("mac")
            .arg(remote)
            .env("SSH_CAPTURE", &capture)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let result = timeout(Duration::from_secs(2), async {
            let bytes = read_capped(&mut tokio::io::BufReader::new(stdout), MAX_STDOUT)
                .await
                .unwrap();
            child.wait().await.unwrap();
            bytes
        })
        .await
        .unwrap();
        let envelope: Envelope = serde_json::from_slice(&result).unwrap();
        assert!(envelope.ok);
        let captured = std::fs::read_to_string(capture).unwrap();
        assert!(captured.contains("'x'\\'';touch /tmp/nope;#'"));
        assert!(!std::path::Path::new("/tmp/nope").exists());
    }

    #[test]
    fn validates_envelope_depth_and_mutation_disposition() {
        let deep = format!(
            "{}0{}",
            "[".repeat(MAX_JSON_DEPTH + 2),
            "]".repeat(MAX_JSON_DEPTH + 2)
        );
        let value: Value = serde_json::from_str(&deep).unwrap();
        assert!(!json_depth(&value, 0));
        let bad: Envelope = serde_json::from_str(r#"{"version":"2.4","ok":false,"command":"click","error":{"code":"INVALID_ARGS","message":"bad ref","disposition":{"delivery":"not_delivered","retry":"safe"}}}"#).unwrap();
        assert_eq!(
            bad.error.unwrap().disposition.unwrap().delivery,
            Delivery::NotDelivered
        );
    }

    #[test]
    fn operation_surface_is_finite_and_mutation_classified() {
        assert_eq!(Command::ListApps.operation().tokens(), &["list-apps"]);
        assert!(!Operation::Snapshot.mutates());
        assert!(Operation::Click.mutates());
        assert_eq!(
            Command::Snapshot {
                app: "Finder".into()
            }
            .args()
            .unwrap(),
            ["--app", "Finder", "--interactive-only", "--compact"]
        );
    }
}
