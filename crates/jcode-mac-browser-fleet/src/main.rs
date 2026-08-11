use std::fs;
use std::path::PathBuf;

use jcode_mac_browser_fleet::{Broker, BrokerConfig, BrowserKind, ManagedCdpSource};

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
    let mut managed_cdp = Vec::new();
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("missing value for broker argument")?;
        match flag.to_string_lossy().as_ref() {
            "--socket" => socket = Some(PathBuf::from(value)),
            "--peer-secret" => peer_secret = Some(PathBuf::from(value)),
            "--policy" => policy = Some(PathBuf::from(value)),
            "--managed-cdp-chrome" => managed_cdp.push((BrowserKind::Chrome, value)),
            "--managed-cdp-edge" => managed_cdp.push((BrowserKind::Edge, value)),
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

    let mut broker = Broker::bind(BrokerConfig {
        socket_path,
        secret,
        max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
        max_in_flight: DEFAULT_MAX_IN_FLIGHT,
    })
    .await?;
    for (browser, endpoint) in managed_cdp {
        let source =
            ManagedCdpSource::new(endpoint.to_string_lossy(), browser, 256, 4 * 1024 * 1024)?;
        broker.apply_inventory(source.discover().await?)?;
    }
    broker.serve().await?;
    Ok(())
}
