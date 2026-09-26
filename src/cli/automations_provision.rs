use crate::automations::{
    config,
    provision::{self, CommandSpec, EndpointConfig, ServeStatus},
};
use anyhow::{Context, Result, bail, ensure};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const UNIT: &str = "jcode-automations.service";
const TAILNET_RECEIPT: &str = "provision-tailnet.json";
const SERVICE_RECEIPT: &str = "provision-service.json";

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    kind: String,
    https_port: u16,
    local_port: u16,
    host: String,
    unit: Option<String>,
    unit_contents: Option<String>,
}
fn read_receipt(directory: &Path, file: &str) -> Result<Option<Receipt>> {
    let p = directory.join(file);
    match std::fs::read(&p) {
        Ok(b) => Ok(Some(
            serde_json::from_slice(&b).context("invalid provisioning ownership receipt")?,
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn save_receipt(directory: &Path, file: &str, r: &Receipt) -> Result<()> {
    std::fs::create_dir_all(directory)?;
    crate::storage::write_json_secret(&directory.join(file), r)
        .context("persist ownership receipt before provisioning")
}
fn run(spec: &CommandSpec) -> Result<std::process::Output> {
    let o = provision::execute(spec).with_context(|| format!("run {}", spec.program.display()))?;
    ensure!(
        o.status.success(),
        "{} failed: {}",
        spec.program.display(),
        String::from_utf8_lossy(&o.stderr)
    );
    Ok(o)
}

pub(super) async fn tailnet(
    directory: &Path,
    mut settings: config::Config,
    https_port: u16,
    disable: bool,
) -> Result<()> {
    ensure!(https_port != 0, "HTTPS port must be nonzero");
    let receipt = read_receipt(directory, TAILNET_RECEIPT)?;
    if disable {
        let Some(r) = receipt.as_ref() else {
            println!("No owned Tailscale route. Nothing changed.");
            return Ok(());
        };
        ensure!(
            r.kind == "tailnet" && r.https_port == https_port,
            "ownership receipt does not match requested route; refusing removal"
        );
        let status = run(&CommandSpec::serve_status())?;
        match provision::inspect_serve_status(
            std::str::from_utf8(&status.stdout)?,
            r.https_port,
            r.local_port,
        ) {
            ServeStatus::Ready { dns_name } if dns_name == r.host => {
                run(&CommandSpec::remove(
                    &EndpointConfig::new(r.https_port, r.local_port).map_err(anyhow::Error::msg)?,
                ))?;
            }
            ServeStatus::Unavailable { .. } => println!("Owned route is already absent."),
            _ => bail!("Serve route differs from owned receipt or is ambiguous; preserving it"),
        }
        settings.tailnet_origin = None;
        settings.control_token = config::Config::new(
            settings.working_dir.clone(),
            settings.socket_path.clone(),
            settings.port,
        )
        .control_token;
        config::save(directory, &settings)?;
        std::fs::remove_file(directory.join(TAILNET_RECEIPT))
            .context("remove ownership receipt after route removal")?;
        println!(
            "Tailnet URL disabled; browser sessions revoked. Tailscale login, ACL and HTTPS policy unchanged."
        );
        return Ok(());
    }
    let identity = run(&CommandSpec::tailscale_status())?;
    let (running, host) = provision::tailscale_identity(
        std::str::from_utf8(&identity.stdout).context("Tailscale status was not UTF-8")?,
    )
    .map_err(anyhow::Error::msg)?;
    ensure!(
        running,
        "Tailscale is not running; start it and complete login manually before provisioning"
    );
    let port = https_port;
    let endpoint = EndpointConfig::new(port, settings.port).map_err(anyhow::Error::msg)?;
    if receipt.is_some() {
        bail!(
            "a tailnet ownership receipt already exists; disable the owned route before changing it"
        );
    }
    let before = run(&CommandSpec::serve_status())?;
    match provision::inspect_serve_status(std::str::from_utf8(&before.stdout)?, port, settings.port)
    {
        ServeStatus::Unavailable { .. } => {}
        ServeStatus::Ready { .. } => {
            bail!("matching Serve route already exists but is unowned; refusing to adopt")
        }
        ServeStatus::Blocked { reason } => {
            bail!("Serve configuration is ambiguous or conflicts: {reason}")
        }
    }
    let r = Receipt {
        kind: "tailnet".into(),
        https_port: port,
        local_port: settings.port,
        host: host.clone(),
        unit: None,
        unit_contents: None,
    };
    save_receipt(directory, TAILNET_RECEIPT, &r)?;
    let result = run(&CommandSpec::provision(&endpoint));
    let after = run(&CommandSpec::serve_status());
    let verified = after
        .as_ref()
        .ok()
        .and_then(|o| std::str::from_utf8(&o.stdout).ok())
        .map(|s| provision::inspect_serve_status(s, port, settings.port));
    if !matches!(verified,Some(ServeStatus::Ready{ref dns_name}) if dns_name==&host) {
        if let Err(e) = result {
            bail!(
                "Tailscale provisioning failed and resulting state is uncertain; ownership receipt preserved: {e}"
            );
        }
        bail!(
            "Tailscale result could not be verified; ownership receipt preserved. Inspect Serve configuration before retrying."
        );
    }
    result?;
    settings.tailnet_origin = Some(if port == 443 {
        format!("https://{}", host.trim_end_matches('.'))
    } else {
        format!("https://{}:{}", host.trim_end_matches('.'), port)
    });
    if let Err(e) = config::save(directory, &settings) {
        bail!(
            "route installed but config save failed; ownership receipt preserved for safe recovery: {e}"
        );
    }
    println!(
        "Private tailnet URL configured. ACL, Tailscale login and HTTPS policy were not changed. HTTPS availability depends on the tailnet's existing policy."
    );
    Ok(())
}

fn systemctl(args: &[&str]) -> Result<std::process::Output> {
    let o = Command::new("systemctl")
        .args(["--user"])
        .args(args)
        .output()
        .context("user systemd unavailable")?;
    ensure!(
        o.status.success(),
        "systemctl --user {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&o.stderr)
    );
    Ok(o)
}
#[cfg(target_os = "linux")]
fn live_socket(path: &Path) -> bool {
    std::os::unix::net::UnixStream::connect(path).is_ok()
}
#[cfg(target_os = "linux")]
pub(super) async fn service(
    directory: &Path,
    settings: &config::Config,
    uninstall: bool,
    enable_linger: bool,
) -> Result<()> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set")?;
    let launcher = home.join(".local/bin/jcode");
    ensure!(
        launcher.is_file(),
        "stable launcher does not exist: {}",
        launcher.display()
    );
    let unit_dir = home.join(".config/systemd/user");
    let unit_path = unit_dir.join(UNIT);
    let receipt = read_receipt(directory, SERVICE_RECEIPT)?;
    if uninstall {
        let Some(r) = receipt else {
            println!("No owned user service. Nothing changed.");
            return Ok(());
        };
        ensure!(
            r.kind == "service" && r.unit.as_deref() == Some(UNIT),
            "ownership receipt does not identify this service"
        );
        if unit_path.exists() {
            let current = std::fs::read_to_string(&unit_path)?;
            ensure!(
                r.unit_contents.as_deref() == Some(current.as_str()),
                "unit contents changed; refusing to stop or remove an unowned service"
            );
            systemctl(&["disable", "--now", UNIT])?;
            std::fs::remove_file(&unit_path)?;
            systemctl(&["daemon-reload"])?;
        }
        std::fs::remove_file(directory.join(SERVICE_RECEIPT))?;
        println!(
            "Owned user service stopped and removed. Automation configuration and history preserved."
        );
        return Ok(());
    }
    if let Some(r) = receipt {
        ensure!(
            r.kind == "service",
            "another provisioning ownership receipt exists"
        );
        bail!("owned service receipt already exists");
    }
    ensure!(
        !unit_path.exists(),
        "{} already exists and is not owned; refusing overwrite",
        unit_path.display()
    );
    if live_socket(&settings.socket_path) {
        bail!(
            "configured socket is live without an owned unit; refusing to compete with the unmanaged daemon. Stop it explicitly, then retry."
        );
    }
    systemctl(&["show", "--property=Version"])?;
    let uid = unsafe { libc::geteuid() }.to_string();
    let linger = Command::new("loginctl")
        .args(["show-user", &uid, "--property=Linger", "--value"])
        .output()
        .context("query user linger state")?;
    ensure!(
        linger.status.success(),
        "loginctl show-user failed: {}",
        String::from_utf8_lossy(&linger.stderr)
    );
    let needs_linger = String::from_utf8_lossy(&linger.stdout).trim() != "yes";
    let unit = provision::user_service_unit(
        &launcher,
        directory
            .parent()
            .context("automation directory has no parent")?,
        &settings.socket_path,
    )
    .map_err(anyhow::Error::msg)?;
    let r = Receipt {
        kind: "service".into(),
        https_port: 0,
        local_port: 0,
        host: String::new(),
        unit: Some(UNIT.into()),
        unit_contents: Some(unit.clone()),
    };
    save_receipt(directory, SERVICE_RECEIPT, &r)?;
    if let Err(e) =
        std::fs::create_dir_all(&unit_dir).and_then(|_| std::fs::write(&unit_path, &unit))
    {
        bail!("service unit write failed; receipt preserved: {e}");
    }
    if let Err(e) = systemctl(&["daemon-reload"])
        .and_then(|_| systemctl(&["enable", "--now", UNIT]).map(|_| ()))
    {
        bail!("service installation state uncertain; ownership receipt preserved: {e}");
    }
    if needs_linger {
        if enable_linger {
            let status = Command::new("loginctl")
                .args(["enable-linger", &uid])
                .status()
                .context("enable user linger")?;
            ensure!(status.success(), "loginctl enable-linger failed");
        } else {
            println!(
                "Service enabled for active user sessions. It will not run independently of login until explicitly enabled with --enable-linger."
            );
        }
    }
    println!("Installed {UNIT} using {}.", launcher.display());
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub(super) async fn service(
    _directory: &Path,
    _settings: &config::Config,
    _uninstall: bool,
    _enable_linger: bool,
) -> Result<()> {
    bail!("managed user service is supported only on Linux")
}
