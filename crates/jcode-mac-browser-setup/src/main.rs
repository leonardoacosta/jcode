use jcode_mac_browser_setup::{install, remove, status, InstallOptions};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(err) = run() {
        eprintln!("jcode-mac-browser-setup: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "status".to_string());
    let home = env::var_os("JCODE_MAC_BROWSER_FLEET_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(PathBuf::from))
        .ok_or("HOME or JCODE_MAC_BROWSER_FLEET_HOME must be set")?;
    let broker = env::var_os("JCODE_MAC_BROWSER_FLEET_BROKER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin/jcode-mac-browser-broker"));
    let mut opts = InstallOptions::fixture(home, broker);
    if let Ok(host) = env::var("JCODE_MAC_BROWSER_FLEET_HOMELAB_HOST") {
        if !host.trim().is_empty() {
            opts.homelab_host = host;
        }
    }
    if let Ok(user) = env::var("JCODE_MAC_BROWSER_FLEET_HOMELAB_USER") {
        if !user.trim().is_empty() {
            opts.homelab_user = user;
        }
    }
    if let Ok(extension_id) = env::var("JCODE_MAC_BROWSER_FLEET_EXTENSION_ID") {
        if !extension_id.trim().is_empty() {
            opts.chrome_extension_id = extension_id.clone();
            opts.edge_extension_id = extension_id;
        }
    }
    if let Ok(extension_id) = env::var("JCODE_MAC_BROWSER_FLEET_CHROME_EXTENSION_ID") {
        if !extension_id.trim().is_empty() {
            opts.chrome_extension_id = extension_id;
        }
    }
    if let Ok(extension_id) = env::var("JCODE_MAC_BROWSER_FLEET_EDGE_EXTENSION_ID") {
        if !extension_id.trim().is_empty() {
            opts.edge_extension_id = extension_id;
        }
    }
    opts.managed_cdp_chrome = non_empty_env("JCODE_MAC_BROWSER_FLEET_MANAGED_CDP_CHROME");
    opts.managed_cdp_edge = non_empty_env("JCODE_MAC_BROWSER_FLEET_MANAGED_CDP_EDGE");

    match command.as_str() {
        "install" => {
            let chrome_configured = extension_id_is_configured(&opts.chrome_extension_id);
            let edge_configured = extension_id_is_configured(&opts.edge_extension_id);
            if !chrome_configured && !edge_configured {
                return Err("set at least one browser-specific extension ID, or backward-compatible JCODE_MAC_BROWSER_FLEET_EXTENSION_ID, before install".into());
            }
            if chrome_configured {
                validate_extension_id(
                    "JCODE_MAC_BROWSER_FLEET_CHROME_EXTENSION_ID",
                    &opts.chrome_extension_id,
                )?;
            }
            if edge_configured {
                validate_extension_id(
                    "JCODE_MAC_BROWSER_FLEET_EDGE_EXTENSION_ID",
                    &opts.edge_extension_id,
                )?;
            }
            let report = install(&opts)?;
            println!(
                "installed={:?} refreshed={:?} backups={}",
                report.installed,
                report.refreshed,
                report.backups.len()
            );
        }
        "status" => {
            let status = status(&opts)?;
            println!(
                "launch_agent={} chrome_host={} edge_host={} policy={} ssh_include={} peer_secret_mode={:?} native_secret_mode={:?} tcp_listener_configured={}",
                status.launch_agent.installed,
                status.chrome_native_host.installed,
                status.edge_native_host.installed,
                status.policy.installed,
                status.ssh_include.installed,
                status.peer_secret_mode,
                status.native_secret_mode,
                status.tcp_listener_configured
            );
            println!(
                "extension_chrome={:?} extension_edge={:?}",
                status.extension_state.chrome, status.extension_state.edge
            );
        }
        "remove" | "uninstall" => {
            remove(&opts)?;
            println!("removed jcode-owned Mac browser fleet artifacts");
        }
        _ => {
            return Err(
                format!("unknown command {command:?}; use install, status, or remove").into(),
            );
        }
    }
    Ok(())
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

fn extension_id_is_configured(extension_id: &str) -> bool {
    extension_id != "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}

fn validate_extension_id(name: &str, extension_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    if extension_id == "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" {
        return Err(format!(
            "set {name}, or backward-compatible JCODE_MAC_BROWSER_FLEET_EXTENSION_ID, to the 32-character browser extension ID before install"
        )
        .into());
    }
    if extension_id.len() != 32 || !extension_id.bytes().all(|byte| matches!(byte, b'a'..=b'p')) {
        return Err(format!("{name} must be 32 lowercase letters from a through p").into());
    }
    Ok(())
}
