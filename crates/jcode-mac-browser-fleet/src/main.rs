use std::fs;
use std::path::PathBuf;

use jcode_mac_browser_fleet::{Broker, BrokerConfig};

const DEFAULT_MAX_PAYLOAD_BYTES: usize = 1024 * 1024;
const DEFAULT_MAX_IN_FLIGHT: usize = 32;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("jcode Mac browser fleet broker failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("broker")) {
        return Err(
            "usage: jcode-mac-browser-fleet broker --socket PATH --peer-secret PATH --policy PATH"
                .into(),
        );
    }

    let mut socket = None;
    let mut peer_secret = None;
    let mut policy = None;
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("missing value for broker argument")?;
        match flag.to_string_lossy().as_ref() {
            "--socket" => socket = Some(PathBuf::from(value)),
            "--peer-secret" => peer_secret = Some(PathBuf::from(value)),
            "--policy" => policy = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown broker argument: {}", flag.to_string_lossy()).into()),
        }
    }

    let socket_path = socket.ok_or("--socket is required")?;
    let secret_path = peer_secret.ok_or("--peer-secret is required")?;
    let policy_path = policy.ok_or("--policy is required")?;
    let secret = fs::read_to_string(secret_path)?.trim().to_string();
    if secret.is_empty() {
        return Err("peer secret is empty".into());
    }
    fs::metadata(policy_path)?;

    Broker::bind(BrokerConfig {
        socket_path,
        secret,
        max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
        max_in_flight: DEFAULT_MAX_IN_FLIGHT,
    })
    .await?
    .serve()
    .await?;
    Ok(())
}
