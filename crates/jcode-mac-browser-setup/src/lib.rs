use std::fs;
use std::io::{self, ErrorKind, Read};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const LABEL: &str = "dev.jcode.mac-browser-fleet";
pub const NATIVE_HOST_NAME: &str = "dev.jcode.mac_browser_fleet";

#[derive(Clone, Debug)]
pub struct InstallOptions {
    pub home: PathBuf,
    pub broker_path: PathBuf,
    pub homelab_host: String,
    pub chrome_extension_id: String,
    pub edge_extension_id: String,
    pub managed_cdp_chrome: Option<String>,
    pub managed_cdp_edge: Option<String>,
}

impl InstallOptions {
    pub fn fixture(home: PathBuf, broker_path: impl Into<PathBuf>) -> Self {
        Self {
            home,
            broker_path: broker_path.into(),
            homelab_host: "jcode-homelab".to_string(),
            chrome_extension_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            edge_extension_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            managed_cdp_chrome: None,
            managed_cdp_edge: None,
        }
    }

    pub fn launch_agent_path(&self) -> PathBuf {
        self.home
            .join("Library/LaunchAgents")
            .join(format!("{LABEL}.plist"))
    }

    pub fn chrome_native_host_path(&self) -> PathBuf {
        self.home
            .join("Library/Application Support/Google/Chrome/NativeMessagingHosts")
            .join(format!("{NATIVE_HOST_NAME}.json"))
    }

    pub fn edge_native_host_path(&self) -> PathBuf {
        self.home
            .join("Library/Application Support/Microsoft Edge/NativeMessagingHosts")
            .join(format!("{NATIVE_HOST_NAME}.json"))
    }

    pub fn peer_secret_path(&self) -> PathBuf {
        self.home
            .join("Library/Application Support/Jcode/MacBrowserFleet/peer.secret")
    }

    pub fn native_secret_path(&self) -> PathBuf {
        self.home
            .join("Library/Application Support/Jcode/MacBrowserFleet/native.secret")
    }

    pub fn policy_path(&self) -> PathBuf {
        self.home
            .join("Library/Application Support/Jcode/MacBrowserFleet/policy.toml")
    }

    pub fn ssh_include_path(&self) -> PathBuf {
        self.home.join(".ssh/jcode-mac-browser-fleet.conf")
    }

    pub fn socket_dir(&self) -> PathBuf {
        // Kept deliberately short: macOS `sockaddr_un.sun_path` allows only 104
        // bytes, and the Application Support directory alone consumes most of it.
        self.home.join(".jcode/mac-fleet")
    }

    pub fn socket_path(&self) -> PathBuf {
        self.socket_dir().join("broker.sock")
    }

    pub fn authority_socket_path(&self) -> PathBuf {
        self.socket_dir().join("authority.sock")
    }

    pub fn forwarded_socket_path(&self) -> PathBuf {
        PathBuf::from("~/.jcode/browser/mac-fleet.sock")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtifactKind {
    LaunchAgent,
    ChromeNativeHost,
    EdgeNativeHost,
    PeerSecret,
    NativeSecret,
    Policy,
    SshInclude,
}

#[derive(Clone, Debug, Default)]
pub struct InstallReport {
    pub installed: Vec<ArtifactKind>,
    pub refreshed: Vec<ArtifactKind>,
    pub backups: Vec<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtensionState {
    NeedsUserInstall,
    Installed,
}

#[derive(Clone, Debug, Default)]
pub struct ArtifactStatus {
    pub installed: bool,
}

#[derive(Clone, Debug)]
pub struct ExtensionStatus {
    pub chrome: ExtensionState,
    pub edge: ExtensionState,
}

#[derive(Clone, Debug)]
pub struct SetupStatus {
    pub launch_agent: ArtifactStatus,
    pub chrome_native_host: ArtifactStatus,
    pub edge_native_host: ArtifactStatus,
    pub policy: ArtifactStatus,
    pub ssh_include: ArtifactStatus,
    pub peer_secret_mode: Option<u32>,
    pub native_secret_mode: Option<u32>,
    pub extension_state: ExtensionStatus,
    pub tcp_listener_configured: bool,
}

pub fn install(opts: &InstallOptions) -> io::Result<InstallReport> {
    let mut report = InstallReport::default();
    write_artifact(
        opts.launch_agent_path(),
        render_launch_agent(opts),
        ArtifactKind::LaunchAgent,
        0o644,
        &mut report,
    )?;
    if extension_id_is_configured(&opts.chrome_extension_id) {
        write_artifact(
            opts.chrome_native_host_path(),
            render_native_host(opts, "chrome"),
            ArtifactKind::ChromeNativeHost,
            0o644,
            &mut report,
        )?;
    }
    if extension_id_is_configured(&opts.edge_extension_id) {
        write_artifact(
            opts.edge_native_host_path(),
            render_native_host(opts, "edge"),
            ArtifactKind::EdgeNativeHost,
            0o644,
            &mut report,
        )?;
    }
    write_secret(
        opts.peer_secret_path(),
        ArtifactKind::PeerSecret,
        "peer",
        &mut report,
    )?;
    write_secret(
        opts.native_secret_path(),
        ArtifactKind::NativeSecret,
        "native",
        &mut report,
    )?;
    write_artifact(
        opts.policy_path(),
        render_policy(),
        ArtifactKind::Policy,
        0o600,
        &mut report,
    )?;
    write_artifact(
        opts.ssh_include_path(),
        render_ssh_include(opts),
        ArtifactKind::SshInclude,
        0o600,
        &mut report,
    )?;
    Ok(report)
}

fn extension_id_is_configured(extension_id: &str) -> bool {
    extension_id != "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}

pub fn status(opts: &InstallOptions) -> io::Result<SetupStatus> {
    let ssh_text = read_optional(&opts.ssh_include_path())?.unwrap_or_default();
    let mode = match fs::metadata(opts.peer_secret_path()) {
        Ok(meta) => Some(meta.permissions().mode() & 0o777),
        Err(err) if err.kind() == ErrorKind::NotFound => None,
        Err(err) => return Err(err),
    };
    let native_mode = match fs::metadata(opts.native_secret_path()) {
        Ok(meta) => Some(meta.permissions().mode() & 0o777),
        Err(err) if err.kind() == ErrorKind::NotFound => None,
        Err(err) => return Err(err),
    };
    Ok(SetupStatus {
        launch_agent: ArtifactStatus {
            installed: opts.launch_agent_path().exists(),
        },
        chrome_native_host: ArtifactStatus {
            installed: opts.chrome_native_host_path().exists(),
        },
        edge_native_host: ArtifactStatus {
            installed: opts.edge_native_host_path().exists(),
        },
        policy: ArtifactStatus {
            installed: opts.policy_path().exists(),
        },
        ssh_include: ArtifactStatus {
            installed: opts.ssh_include_path().exists(),
        },
        peer_secret_mode: mode,
        native_secret_mode: native_mode,
        extension_state: ExtensionStatus {
            chrome: ExtensionState::NeedsUserInstall,
            edge: ExtensionState::NeedsUserInstall,
        },
        tcp_listener_configured: ssh_text.lines().any(|line| {
            let line = line.trim_start();
            line.starts_with("LocalForward ") || line.contains("0.0.0.0") || line.contains(":9222")
        }),
    })
}

pub fn remove(opts: &InstallOptions) -> io::Result<()> {
    for path in [
        opts.launch_agent_path(),
        opts.chrome_native_host_path(),
        opts.edge_native_host_path(),
        opts.peer_secret_path(),
        opts.native_secret_path(),
        opts.policy_path(),
        opts.ssh_include_path(),
    ] {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn write_secret(
    path: PathBuf,
    kind: ArtifactKind,
    label: &str,
    report: &mut InstallReport,
) -> io::Result<()> {
    if !path.exists() {
        let secret = format!("jcode-mac-browser-fleet-{label}-{}\n", secure_token());
        write_new(path, secret, 0o600)?;
        report.installed.push(kind);
    } else {
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        report.refreshed.push(kind);
    }
    Ok(())
}

fn write_artifact(
    path: PathBuf,
    content: String,
    kind: ArtifactKind,
    mode: u32,
    report: &mut InstallReport,
) -> io::Result<()> {
    match fs::read_to_string(&path) {
        Ok(existing) if existing == content => {
            fs::set_permissions(&path, fs::Permissions::from_mode(mode))?;
            report.refreshed.push(kind);
            Ok(())
        }
        Ok(_) => {
            let backup = backup_path(&path);
            fs::copy(&path, &backup)?;
            report.backups.push(backup);
            write_new(path, content, mode)?;
            report.refreshed.push(kind);
            Ok(())
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            write_new(path, content, mode)?;
            report.installed.push(kind);
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn write_new(path: PathBuf, content: String, mode: u32) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
}

fn backup_path(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap().to_string_lossy();
    path.with_file_name(format!("{name}.bak.{}", nonce()))
}

fn nonce() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn secure_token() -> String {
    let mut bytes = [0_u8; 32];
    if fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .is_ok()
    {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    } else {
        format!("{}{}", nonce(), nonce())
    }
}

fn read_optional(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

pub fn render_launch_agent(opts: &InstallOptions) -> String {
    let mut managed_cdp_arguments = String::new();
    if let Some(endpoint) = &opts.managed_cdp_chrome {
        managed_cdp_arguments.push_str(&format!(
            "    <string>--managed-cdp-chrome</string>\n    <string>{}</string>\n",
            xml_text(endpoint)
        ));
    }
    if let Some(endpoint) = &opts.managed_cdp_edge {
        managed_cdp_arguments.push_str(&format!(
            "    <string>--managed-cdp-edge</string>\n    <string>{}</string>\n",
            xml_text(endpoint)
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
    <string>broker</string>
	    <string>--socket</string>
	    <string>{}</string>
	    <string>--authority-socket</string>
	    <string>{}</string>
	    <string>--peer-secret</string>
	    <string>{}</string>
	    <string>--native-secret</string>
	    <string>{}</string>
    <string>--policy</string>
    <string>{}</string>
{}
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
</dict>
</plist>
"#,
        xml(&opts.broker_path),
        xml(&opts.socket_path()),
        xml(&opts.authority_socket_path()),
        xml(&opts.peer_secret_path()),
        xml(&opts.native_secret_path()),
        xml(&opts.policy_path()),
        managed_cdp_arguments
    )
}

pub fn render_native_host(opts: &InstallOptions, browser: &str) -> String {
    let (app, extension_id) = if browser == "edge" {
        ("com.microsoft.edge", &opts.edge_extension_id)
    } else {
        ("com.google.chrome", &opts.chrome_extension_id)
    };
    format!(
        r#"{{
  "name": "{NATIVE_HOST_NAME}",
  "description": "Jcode Mac browser fleet native messaging host for {app}",
  "path": "{}",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://{}/"
  ]
}}
"#,
        json(&opts.broker_path),
        json_text(extension_id)
    )
}

pub fn render_policy() -> String {
    r#"# Jcode Mac browser fleet policy defaults.
# Read-only inventory is allowed. Mutations require Mac approval or a Mac-issued lease.
read_only_inventory = true
mutation_default = "approval_required"
default_lease_minutes = 15
hard_denies = [
  "password_managers",
  "browser_settings",
  "extensions",
  "downloads",
  "payment_confirmation",
  "account_security",
  "authentication_recovery",
  "incognito",
  "browser_internal_urls",
]
"#
    .to_string()
}

pub fn render_ssh_include(opts: &InstallOptions) -> String {
    format!(
        r#"# Include from ~/.ssh/config with: Include ~/.ssh/jcode-mac-browser-fleet.conf
# Reverse Unix-socket forwarding only. No TCP listener is created.
Host {}
  ExitOnForwardFailure yes
  ServerAliveInterval 30
  ServerAliveCountMax 3
  StreamLocalBindUnlink yes
  RemoteForward {} {}
"#,
        opts.homelab_host,
        opts.forwarded_socket_path().display(),
        opts.socket_path().display()
    )
}

fn xml(path: &Path) -> String {
    xml_text(&path.to_string_lossy())
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn json(path: &Path) -> String {
    json_text(&path.to_string_lossy())
}

fn json_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("jcode-mac-browser-setup-{name}-{nonce}"));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn configure_fixture_extensions(opts: &mut InstallOptions) {
        opts.chrome_extension_id = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
        opts.edge_extension_id = "cccccccccccccccccccccccccccccccc".to_string();
    }

    #[test]
    fn install_status_remove_preserves_profiles_and_unrelated_ssh_config() {
        let root = tmp_root("lifecycle");
        let profile = root.join("Library/Application Support/Google/Chrome/Default/Preferences");
        fs::create_dir_all(profile.parent().unwrap()).unwrap();
        fs::write(&profile, "profile stays").unwrap();
        let ssh_config = root.join(".ssh/config");
        fs::create_dir_all(ssh_config.parent().unwrap()).unwrap();
        fs::write(&ssh_config, "Host github.com\n  User git\n").unwrap();

        let mut opts =
            InstallOptions::fixture(root.clone(), "/opt/jcode/bin/jcode-mac-browser-broker");
        configure_fixture_extensions(&mut opts);
        let report = install(&opts).unwrap();
        assert!(report.installed.contains(&ArtifactKind::LaunchAgent));
        assert!(report.installed.contains(&ArtifactKind::ChromeNativeHost));
        assert!(report.installed.contains(&ArtifactKind::EdgeNativeHost));
        assert!(report.installed.contains(&ArtifactKind::PeerSecret));
        assert!(report.installed.contains(&ArtifactKind::Policy));
        assert!(report.installed.contains(&ArtifactKind::SshInclude));

        let status = status(&opts).unwrap();
        assert!(status.launch_agent.installed);
        assert!(status.chrome_native_host.installed);
        assert!(status.edge_native_host.installed);
        assert_eq!(
            status.extension_state.chrome,
            ExtensionState::NeedsUserInstall
        );
        assert_eq!(
            status.extension_state.edge,
            ExtensionState::NeedsUserInstall
        );
        assert_eq!(status.peer_secret_mode, Some(0o600));
        assert_eq!(status.native_secret_mode, Some(0o600));
        assert!(!status.tcp_listener_configured);

        let plist = fs::read_to_string(opts.launch_agent_path()).unwrap();
        assert!(!plist.contains("<key>Sockets</key>"));
        assert!(plist.contains("<string>--socket</string>"));
        assert!(plist.contains("broker.sock"));
        assert!(plist.contains("<string>--native-secret</string>"));
        assert!(plist.contains("native.secret"));
        if Command::new("plutil")
            .arg("-lint")
            .arg(opts.launch_agent_path())
            .status()
            .is_ok()
        {
            assert!(Command::new("plutil")
                .arg("-lint")
                .arg(opts.launch_agent_path())
                .status()
                .unwrap()
                .success());
        }

        remove(&opts).unwrap();
        assert!(profile.exists());
        assert_eq!(fs::read_to_string(&profile).unwrap(), "profile stays");
        assert_eq!(
            fs::read_to_string(&ssh_config).unwrap(),
            "Host github.com\n  User git\n"
        );
        assert!(!opts.launch_agent_path().exists());
        assert!(!opts.chrome_native_host_path().exists());
        assert!(!opts.edge_native_host_path().exists());
        assert!(!opts.peer_secret_path().exists());
        assert!(!opts.native_secret_path().exists());
    }

    #[test]
    fn launch_agent_includes_only_configured_loopback_cdp_sources() {
        let mut opts = InstallOptions::fixture(
            PathBuf::from("/Users/test"),
            "/Users/test/.local/bin/jcode-mac-browser-fleet",
        );
        opts.managed_cdp_chrome = Some("http://127.0.0.1:9222?x=1&y=2".to_string());

        let plist = render_launch_agent(&opts);
        assert!(plist.contains("<string>--managed-cdp-chrome</string>"));
        assert!(plist.contains("http://127.0.0.1:9222?x=1&amp;y=2"));
        assert!(!plist.contains("--managed-cdp-edge"));
    }

    #[test]
    fn refresh_backs_up_operator_edited_files_and_keeps_secret_mode_0600() {
        let root = tmp_root("refresh");
        let mut opts = InstallOptions::fixture(root, "/opt/jcode/bin/jcode-mac-browser-broker");
        configure_fixture_extensions(&mut opts);
        install(&opts).unwrap();
        fs::write(opts.policy_path(), "# operator edit\n").unwrap();
        fs::write(opts.launch_agent_path(), "operator plist edit").unwrap();

        let report = install(&opts).unwrap();
        assert!(report
            .backups
            .iter()
            .any(|p| p.to_string_lossy().contains("policy.toml.bak")));
        assert!(report.backups.iter().any(|p| {
            p.to_string_lossy()
                .contains("dev.jcode.mac-browser-fleet.plist.bak")
        }));
        assert_eq!(
            fs::metadata(opts.peer_secret_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(opts.native_secret_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_ne!(
            fs::read_to_string(opts.peer_secret_path()).unwrap(),
            fs::read_to_string(opts.native_secret_path()).unwrap()
        );
    }

    #[test]
    fn rendered_manifests_and_ssh_include_are_safe_and_reverse_stream_local_only() {
        let root = tmp_root("render");
        let mut opts = InstallOptions::fixture(root, "/opt/jcode/bin/jcode-mac-browser-broker");
        configure_fixture_extensions(&mut opts);
        install(&opts).unwrap();

        let chrome = fs::read_to_string(opts.chrome_native_host_path()).unwrap();
        assert!(chrome.contains("com.google.chrome"));
        assert!(chrome.contains("chrome-extension://"));
        assert!(!chrome.contains("Default"));
        assert!(!chrome.contains("Profile"));

        let ssh = fs::read_to_string(opts.ssh_include_path()).unwrap();
        assert!(ssh.contains("StreamLocalBindUnlink yes"));
        assert!(ssh.contains("RemoteForward"));
        assert!(ssh.contains("~/.jcode/browser/mac-fleet.sock"));
        assert!(!ssh.contains("LocalForward"));
        assert!(!ssh.contains("0.0.0.0"));
        assert!(!ssh.contains(":9222"));
    }

    #[test]
    fn native_host_manifests_use_independent_browser_extension_ids() {
        let mut opts = InstallOptions::fixture(
            PathBuf::from("/Users/test"),
            "/Users/test/.local/bin/jcode-mac-browser-fleet",
        );
        opts.chrome_extension_id = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();
        opts.edge_extension_id = "cccccccccccccccccccccccccccccccc".to_string();

        let chrome = render_native_host(&opts, "chrome");
        let edge = render_native_host(&opts, "edge");

        assert!(chrome.contains("com.google.chrome"));
        assert!(chrome.contains("chrome-extension://bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb/"));
        assert!(!chrome.contains("cccccccccccccccccccccccccccccccc"));
        assert!(edge.contains("com.microsoft.edge"));
        assert!(edge.contains("chrome-extension://cccccccccccccccccccccccccccccccc/"));
        assert!(!edge.contains("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
    }

    #[test]
    fn native_host_manifest_path_remains_executable_without_encoded_arguments() {
        let opts = InstallOptions::fixture(
            PathBuf::from("/Users/test"),
            "/Users/test/.local/bin/jcode-mac-browser-fleet",
        );
        let chrome = render_native_host(&opts, "chrome");

        assert!(chrome.contains("\"path\": \"/Users/test/.local/bin/jcode-mac-browser-fleet\""));
        assert!(!chrome.contains("native-host"));
        assert!(!chrome.contains("--socket"));
        assert!(!chrome.contains("native.secret"));
    }

    #[test]
    fn install_can_configure_chrome_without_an_edge_extension_id() {
        let root = tmp_root("chrome-only");
        let mut opts =
            InstallOptions::fixture(root.clone(), root.join("bin/jcode-mac-browser-fleet"));
        opts.chrome_extension_id = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string();

        let report = install(&opts).unwrap();

        assert!(opts.chrome_native_host_path().is_file());
        assert!(!opts.edge_native_host_path().exists());
        assert!(report.installed.contains(&ArtifactKind::ChromeNativeHost));
        assert!(!report.installed.contains(&ArtifactKind::EdgeNativeHost));
    }

    #[test]
    fn install_can_configure_edge_without_a_chrome_extension_id() {
        let root = tmp_root("edge-only");
        let mut opts =
            InstallOptions::fixture(root.clone(), root.join("bin/jcode-mac-browser-fleet"));
        opts.edge_extension_id = "cccccccccccccccccccccccccccccccc".to_string();

        let report = install(&opts).unwrap();

        assert!(!opts.chrome_native_host_path().exists());
        assert!(opts.edge_native_host_path().is_file());
        assert!(!report.installed.contains(&ArtifactKind::ChromeNativeHost));
        assert!(report.installed.contains(&ArtifactKind::EdgeNativeHost));
    }

    #[test]
    fn rendered_socket_paths_stay_within_the_unix_domain_socket_limit() {
        // macOS `sockaddr_un.sun_path` holds at most 104 bytes including the
        // trailing NUL, so any rendered socket path must stay below that or the
        // broker fails to bind at runtime even though every file is installed.
        const MAX_UNIX_SOCKET_PATH_BYTES: usize = 103;
        let opts = InstallOptions::fixture(
            PathBuf::from("/Users/a-fairly-long-account-name"),
            "/Users/a-fairly-long-account-name/.local/bin/jcode-mac-browser-fleet",
        );

        for path in [opts.socket_path(), opts.authority_socket_path()] {
            let rendered = path.to_string_lossy();
            assert!(
                rendered.len() <= MAX_UNIX_SOCKET_PATH_BYTES,
                "socket path {rendered} is {} bytes, over the {MAX_UNIX_SOCKET_PATH_BYTES}-byte limit",
                rendered.len()
            );
        }
    }
}
