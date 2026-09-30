//! Explicit provisioning primitives. Inspection never mutates external state.
use serde_json::Value;
use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::Duration,
};
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const CAPTURE_LIMIT: u64 = 1_048_576;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointConfig {
    pub https_port: u16,
    pub local_port: u16,
}
impl EndpointConfig {
    pub fn new(https_port: u16, local_port: u16) -> Result<Self, String> {
        if https_port == 0 || local_port == 0 {
            return Err("ports must be between 1 and 65535".into());
        }
        Ok(Self {
            https_port,
            local_port,
        })
    }
    pub fn serve_args(&self) -> Vec<String> {
        vec![
            "serve".into(),
            "--bg".into(),
            format!("--https={}", self.https_port),
            format!("http://127.0.0.1:{}", self.local_port),
        ]
    }
    pub fn remove_args(&self) -> Vec<String> {
        vec![
            "serve".into(),
            "--bg".into(),
            format!("--https={}", self.https_port),
            format!("http://127.0.0.1:{}", self.local_port),
            "off".into(),
        ]
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServeStatus {
    Unavailable { reason: String },
    Ready { dns_name: String },
    Blocked { reason: String },
}
pub fn tailscale_identity(json: &str) -> Result<(bool, String), String> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| format!("invalid Tailscale status JSON: {e}"))?;
    let self_node = v.get("Self").ok_or("Tailscale status has no Self node")?;
    let dns = self_node
        .get("DNSName")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim_end_matches('.');
    if dns.is_empty() {
        return Err("Tailscale self node has no DNSName".into());
    }
    Ok((
        v.get("BackendState").and_then(Value::as_str) == Some("Running"),
        dns.to_owned(),
    ))
}
/// Inspect flat Tailscale ServeConfig JSON. Unknown fields/configuration fail closed.
pub fn inspect_serve_status(json: &str, port: u16, local_port: u16) -> ServeStatus {
    let v: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            return ServeStatus::Blocked {
                reason: format!("invalid Serve status JSON: {e}"),
            };
        }
    };
    let Some(root) = v.as_object() else {
        return ServeStatus::Blocked {
            reason: "unrecognized Serve status shape".into(),
        };
    };
    let wanted_proxy = format!("http://127.0.0.1:{local_port}");
    let empty = serde_json::Map::new();
    let Some(tcp) = root.get("TCP").map_or(Some(&empty), Value::as_object) else {
        return ServeStatus::Blocked {
            reason: "unknown TCP map".into(),
        };
    };
    let Some(web) = root.get("Web").map_or(Some(&empty), Value::as_object) else {
        return ServeStatus::Blocked {
            reason: "unknown Web map".into(),
        };
    };
    let Some(funnel) = root
        .get("AllowFunnel")
        .map_or(Some(&empty), Value::as_object)
    else {
        return ServeStatus::Blocked {
            reason: "unknown AllowFunnel map".into(),
        };
    };
    for key in ["Services", "Foreground"] {
        if root
            .get(key)
            .is_some_and(|v| v.as_object().is_none_or(|m| !m.is_empty()))
        {
            return ServeStatus::Blocked {
                reason: format!("nonempty or unknown {key} configuration"),
            };
        }
    }
    for key in root.keys() {
        if ![
            "TCP",
            "Web",
            "AllowFunnel",
            "Services",
            "Foreground",
            "ETag",
        ]
        .contains(&key.as_str())
        {
            return ServeStatus::Blocked {
                reason: format!("unknown ServeConfig field {key}"),
            };
        }
    }
    let wanted_port = port.to_string();
    if funnel.iter().any(|(hp, allowed)| {
        hp.rsplit_once(':')
            .is_none_or(|(_, p)| p == wanted_port && allowed.as_bool() != Some(false))
    }) {
        return ServeStatus::Blocked {
            reason: "Funnel enabled or unknown".into(),
        };
    }
    if tcp.keys().any(|k| k.parse::<u16>().is_err()) {
        return ServeStatus::Blocked {
            reason: "invalid TCP port key".into(),
        };
    }
    let tcp_entry = tcp.get(&wanted_port);
    let mut matching_web = web
        .iter()
        .filter(|(key, _)| key.rsplit_once(':').is_some_and(|(_, p)| p == wanted_port));
    let web_entry = matching_web.next();
    if tcp_entry.is_none() && web_entry.is_none() {
        return ServeStatus::Unavailable {
            reason: "no route at requested port".into(),
        };
    }
    if matching_web.next().is_some() {
        return ServeStatus::Blocked {
            reason: "multiple Web routes use requested port".into(),
        };
    }
    let Some(listener) = tcp_entry.and_then(Value::as_object) else {
        return ServeStatus::Blocked {
            reason: format!("port {port} has unknown TCP configuration"),
        };
    };
    if listener.get("HTTPS").and_then(Value::as_bool) != Some(true)
        || listener
            .get("HTTP")
            .is_some_and(|x| x.as_bool() != Some(false))
        || listener.get("TCPForward").is_some_and(|x| !x.is_null())
        || listener.get("TerminateTLS").is_some_and(|x| !x.is_null())
        || listener.get("ProxyProtocol").is_some_and(|x| !x.is_null())
        || listener.keys().any(|k| {
            ![
                "HTTPS",
                "HTTP",
                "TCPForward",
                "TerminateTLS",
                "ProxyProtocol",
            ]
            .contains(&k.as_str())
        })
    {
        return ServeStatus::Blocked {
            reason: format!("port {port} is not an exclusive HTTPS listener"),
        };
    }
    let Some((hostport, server)) = web_entry else {
        return ServeStatus::Blocked {
            reason: format!("HTTPS port {port} has no matching Web host-port"),
        };
    };
    let Some(server) = server.as_object() else {
        return ServeStatus::Blocked {
            reason: "unknown Web server config".into(),
        };
    };
    if server.keys().any(|k| k != "Handlers") {
        return ServeStatus::Blocked {
            reason: "unknown Web server fields".into(),
        };
    }
    let Some(handlers) = server.get("Handlers").and_then(Value::as_object) else {
        return ServeStatus::Blocked {
            reason: "unknown Web handlers map".into(),
        };
    };
    if handlers.len() != 1 {
        return ServeStatus::Blocked {
            reason: "requested endpoint has additional or missing handlers".into(),
        };
    }
    let Some(handler) = handlers.get("/").and_then(Value::as_object) else {
        return ServeStatus::Blocked {
            reason: "requested endpoint lacks root handler".into(),
        };
    };
    if handler.len() != 1
        || handler.get("Proxy").and_then(Value::as_str) != Some(wanted_proxy.as_str())
    {
        return ServeStatus::Blocked {
            reason: "requested endpoint is not the exact local proxy".into(),
        };
    }
    let Some((host, _)) = hostport.rsplit_once(':') else {
        return ServeStatus::Blocked {
            reason: "invalid Serve host-port".into(),
        };
    };
    if host.is_empty() || host.starts_with('[') {
        return ServeStatus::Blocked {
            reason: "invalid Serve host".into(),
        };
    }
    ServeStatus::Ready {
        dns_name: host.to_owned(),
    }
}

/// Remove only the endpoint owned by this provisioning receipt before config comparison.
pub fn serve_config_without_endpoint(json: &str, host: &str, port: u16) -> Result<Value, String> {
    let mut value: Value =
        serde_json::from_str(json).map_err(|e| format!("invalid Serve status JSON: {e}"))?;
    let root = value
        .as_object_mut()
        .ok_or("unrecognized Serve status shape")?;
    let host_port = format!("{host}:{port}");
    for (field, key) in [
        ("TCP", port.to_string()),
        ("Web", host_port.clone()),
        ("AllowFunnel", host_port),
    ] {
        if let Some(map) = root.get_mut(field) {
            let map = map
                .as_object_mut()
                .ok_or_else(|| format!("unknown {field} map"))?;
            map.remove(&key);
            if map.is_empty() {
                root.remove(field);
            }
        }
    }
    Ok(value)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub timeout: Duration,
}
impl CommandSpec {
    pub fn tailscale_status() -> Self {
        Self {
            program: "tailscale".into(),
            args: vec!["status".into(), "--json".into()],
            timeout: COMMAND_TIMEOUT,
        }
    }
    pub fn serve_status() -> Self {
        Self {
            program: "tailscale".into(),
            args: vec!["serve".into(), "status".into(), "--json".into()],
            timeout: COMMAND_TIMEOUT,
        }
    }
    pub fn provision(c: &EndpointConfig) -> Self {
        Self {
            program: "tailscale".into(),
            args: c.serve_args(),
            timeout: COMMAND_TIMEOUT,
        }
    }
    pub fn remove(c: &EndpointConfig) -> Self {
        Self {
            program: "tailscale".into(),
            args: c.remove_args(),
            timeout: COMMAND_TIMEOUT,
        }
    }
}
pub fn execute(spec: &CommandSpec) -> std::io::Result<Output> {
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let out_thread = std::thread::spawn(move || {
        let mut b = Vec::new();
        let mut buf = [0; 8192];
        loop {
            match out.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) if b.len() < CAPTURE_LIMIT as usize => {
                    let keep = n.min(CAPTURE_LIMIT as usize - b.len());
                    b.extend_from_slice(&buf[..keep]);
                }
                Ok(_) => {}
            }
        }
        b
    });
    let err_thread = std::thread::spawn(move || {
        let mut b = Vec::new();
        let mut buf = [0; 8192];
        loop {
            match err.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) if b.len() < CAPTURE_LIMIT as usize => {
                    let keep = n.min(CAPTURE_LIMIT as usize - b.len());
                    b.extend_from_slice(&buf[..keep]);
                }
                Ok(_) => {}
            }
        }
        b
    });
    let start = std::time::Instant::now();
    let mut status = None;
    loop {
        if status.is_none() {
            status = child.try_wait()?;
        }
        if let Some(status) =
            status.filter(|_| out_thread.is_finished() && err_thread.is_finished())
        {
            let mut stdout = out_thread.join().unwrap_or_default();
            let mut stderr = err_thread.join().unwrap_or_default();
            stdout.truncate(CAPTURE_LIMIT as usize);
            stderr.truncate(CAPTURE_LIMIT as usize);
            return Ok(Output {
                status,
                stdout,
                stderr,
            });
        }
        if start.elapsed() >= spec.timeout {
            #[cfg(unix)]
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            }
            let _ = child.kill();
            let _ = child.wait();
            let _ = out_thread.join();
            let _ = err_thread.join();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "provisioning command timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
pub fn user_service_unit(
    launcher: &Path,
    data_dir: &Path,
    socket: &Path,
) -> Result<String, String> {
    for path in [launcher, data_dir, socket] {
        if !path.is_absolute() {
            return Err(format!("path must be absolute: {}", path.display()));
        }
    }
    let launcher = systemd_escape(launcher)?;
    let data_dir_raw = data_dir.to_str().ok_or("path is not valid UTF-8")?;
    if data_dir_raw.contains(['\n', '\r']) {
        return Err("path contains a newline".into());
    }
    let data_dir = systemd_escape(data_dir)?;
    let socket = systemd_escape(socket)?;
    Ok(format!(
        "[Unit]\nDescription=Jcode user daemon\nAfter=default.target\n\n[Service]\nType=simple\nEnvironment=\"JCODE_HOME={}\"\nWorkingDirectory=\"{}\"\nExecStart=\"{}\" --no-update --socket \"{}\" serve\nRestart=on-failure\nRestartSec=3\n\n[Install]\nWantedBy=default.target\n",
        systemd_environment_escape(data_dir_raw),
        systemd_environment_escape(&data_dir),
        systemd_argument_escape(&launcher),
        systemd_argument_escape(&socket)
    ))
}
fn systemd_environment_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "%%")
        .replace('"', "\\\"")
}
fn systemd_argument_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "%%")
        .replace('$', "$$")
        .replace('"', "\\\"")
}
fn systemd_escape(path: &Path) -> Result<String, String> {
    let value = path.to_str().ok_or("path is not valid UTF-8")?;
    if value.contains(['\n', '\r']) {
        return Err("path contains a newline".into());
    }
    Ok(value.to_owned())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_routes_only_and_fail_closed() {
        let exact = r#"{"TCP":{"8443":{"HTTPS":true}},"Web":{"node.example.ts.net:8443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:3000"}}}},"AllowFunnel":{"node.example.ts.net:8443":false}}"#;
        assert_eq!(
            inspect_serve_status(exact, 8443, 3000),
            ServeStatus::Ready {
                dns_name: "node.example.ts.net".into()
            }
        );
        assert!(matches!(
            inspect_serve_status(
                r#"{"TCP":{"8443":{"HTTPS":true}},"Web":{"node:8443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:30000"}}}}}"#,
                8443,
                3000
            ),
            ServeStatus::Blocked { .. }
        ));
        assert!(matches!(
            inspect_serve_status(
                r#"{"TCP":{"8443":{"HTTPS":true}},"Web":{"node:8443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:3000"}}}},"AllowFunnel":{"node:8443":true}}"#,
                8443,
                3000
            ),
            ServeStatus::Blocked { .. }
        ));
        assert!(matches!(
            inspect_serve_status(
                r#"{"TCP":{"8443":{"HTTPS":true}},"Web":{"node:8443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:3000"},"/other":{"Proxy":"http://127.0.0.1:9000"}}}}}"#,
                8443,
                3000
            ),
            ServeStatus::Blocked { .. }
        ));
        assert!(matches!(
            inspect_serve_status(
                r#"{"TCP":{"8443":{"HTTPS":true}},"Web":{"node:8443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:3000"}}}},"Foreground":{"session":{"TCP":{"8444":{"HTTPS":true}}}}}"#,
                8443,
                3000
            ),
            ServeStatus::Blocked { .. }
        ));
        assert!(matches!(
            inspect_serve_status("{}", 8443, 3000),
            ServeStatus::Unavailable { .. }
        ));
    }

    #[test]
    fn endpoint_stripping_compares_unrelated_serve_routes() {
        let before = r#"{"TCP":{"443":{"HTTPS":true},"8443":{"HTTPS":true}},"Web":{"other.ts.net:443":{"Handlers":{}},"node.ts.net:8443":{"Handlers":{}}},"AllowFunnel":{"other.ts.net:443":false,"node.ts.net:8443":false}}"#;
        let after = r#"{"TCP":{"443":{"HTTPS":true}},"Web":{"other.ts.net:443":{"Handlers":{}}},"AllowFunnel":{"other.ts.net:443":false}}"#;
        assert_eq!(
            serve_config_without_endpoint(before, "node.ts.net", 8443).unwrap(),
            serve_config_without_endpoint(after, "node.ts.net", 8443).unwrap()
        );
        let changed = r#"{"TCP":{"443":{"HTTPS":true}},"Web":{"other.ts.net:443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:9"}}}},"AllowFunnel":{"other.ts.net:443":false}}"#;
        assert_ne!(
            serve_config_without_endpoint(before, "node.ts.net", 8443).unwrap(),
            serve_config_without_endpoint(changed, "node.ts.net", 8443).unwrap()
        );
    }
    #[test]
    fn environment_uses_systemd_value_quoting_not_exec_expansion_escaping() {
        let unit = user_service_unit(
            Path::new("/home/u/.local/bin/jcode"),
            Path::new("/home/u/.jcode $x%\""),
            Path::new("/run/user/1/jcode.sock"),
        )
        .unwrap();
        assert!(unit.contains("Environment=\"JCODE_HOME=/home/u/.jcode $x%%\\\"\""));
        assert!(unit.contains("WorkingDirectory=\"/home/u/.jcode $x%%\\\"\""));
        assert!(unit.contains("ExecStart=\"/home/u/.local/bin/jcode\""));
    }

    #[cfg(unix)]
    #[test]
    fn timeout_kills_descendants_holding_pipes() {
        use std::{fs, os::unix::fs::PermissionsExt};
        let dir = std::env::temp_dir().join(format!("jcode-timeout-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        for command in ["sleep 30 & wait", "sleep 30 & exit 0"] {
            let script = dir.join("test-command");
            fs::write(&script, format!("#!/bin/sh\n{command}\n")).unwrap();
            fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
            let start = std::time::Instant::now();
            let err = execute(&CommandSpec {
                program: script.clone(),
                args: vec![],
                timeout: Duration::from_millis(100),
            })
            .unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
            assert!(start.elapsed() < Duration::from_secs(2));
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn commands_and_unit_are_scoped() {
        let c = EndpointConfig::new(8443, 3000).unwrap();
        assert!(c.serve_args().contains(&"--bg".into()));
        assert_eq!(
            c.remove_args(),
            [
                "serve",
                "--bg",
                "--https=8443",
                "http://127.0.0.1:3000",
                "off"
            ]
        );
        assert!(EndpointConfig::new(0, 3000).is_err());
        let u = user_service_unit(
            Path::new("/home/u/.local/bin/jcode"),
            Path::new("/home/u/.jcode $x%\""),
            Path::new("/run/user/1/jcode.sock"),
        )
        .unwrap();
        assert!(u.contains("Environment=\"JCODE_HOME=/home/u/.jcode $x%%\\\"\""));
        assert!(u.contains("ExecStart=\"/home/u/.local/bin/jcode\""));
        assert!(
            user_service_unit(Path::new("jcode"), Path::new("/data"), Path::new("/sock")).is_err()
        );
    }
}
