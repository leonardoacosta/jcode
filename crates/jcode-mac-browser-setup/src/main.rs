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
    let opts = InstallOptions::fixture(home, broker);

    match command.as_str() {
        "install" => {
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
            println!("launch_agent={} chrome_host={} edge_host={} policy={} ssh_include={} peer_secret_mode={:?} tcp_listener_configured={}",
                status.launch_agent.installed,
                status.chrome_native_host.installed,
                status.edge_native_host.installed,
                status.policy.installed,
                status.ssh_include.installed,
                status.peer_secret_mode,
                status.tcp_listener_configured);
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
            )
        }
    }
    Ok(())
}
