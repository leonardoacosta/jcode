use std::fs;
use std::path::PathBuf;

use jcode_mac_browser_fleet::{
    AuthorityAction, AuthorityEnvelope, Broker, BrokerConfig, BrowserKind, Capability,
    ManagedCdpSource, NativeHostBridgeConfig, TargetRef, serve_native_host,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

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
    let Some(command) = args.next() else {
        return run_native_host(Vec::new()).await;
    };
    if command == std::ffi::OsStr::new("authority") {
        return run_authority(args.collect()).await;
    }
    if command == std::ffi::OsStr::new("native-host") {
        return run_native_host(args.collect()).await;
    }
    if command != std::ffi::OsStr::new("broker") {
        return Err(
            "usage: jcode-mac-browser-fleet [native-host --socket PATH --peer-secret PATH] | broker --socket PATH --authority-socket PATH --peer-secret PATH --policy PATH | authority grant|revoke|emergency-stop|release-emergency-stop|status"
                .into(),
        );
    }

    let mut socket = None;
    let mut peer_secret = None;
    let mut authority_socket = None;
    let mut policy = None;
    let mut managed_cdp = Vec::new();
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("missing value for broker argument")?;
        match flag.to_string_lossy().as_ref() {
            "--socket" => socket = Some(PathBuf::from(value)),
            "--authority-socket" => authority_socket = Some(PathBuf::from(value)),
            "--peer-secret" => peer_secret = Some(PathBuf::from(value)),
            "--policy" => policy = Some(PathBuf::from(value)),
            "--managed-cdp-chrome" => managed_cdp.push((BrowserKind::Chrome, value)),
            "--managed-cdp-edge" => managed_cdp.push((BrowserKind::Edge, value)),
            _ => return Err(format!("unknown broker argument: {}", flag.to_string_lossy()).into()),
        }
    }

    let socket_path = socket.ok_or("--socket is required")?;
    let authority_socket_path = authority_socket;
    let secret_path = peer_secret.ok_or("--peer-secret is required")?;
    let policy_path = policy.ok_or("--policy is required")?;
    let secret = fs::read_to_string(secret_path)?.trim().to_string();
    if secret.is_empty() {
        return Err("peer secret is empty".into());
    }
    fs::metadata(policy_path)?;

    let mut broker = Broker::bind(BrokerConfig {
        socket_path,
        authority_socket_path,
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

async fn run_native_host(args: Vec<std::ffi::OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = None;
    let mut peer_secret = None;
    let mut max_payload_bytes = DEFAULT_MAX_PAYLOAD_BYTES;
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or("missing value for native-host argument")?;
        match flag.to_string_lossy().as_ref() {
            "--socket" => socket = Some(PathBuf::from(value)),
            "--peer-secret" => peer_secret = Some(PathBuf::from(value)),
            "--max-payload-bytes" => {
                max_payload_bytes = value.to_string_lossy().parse::<usize>()?
            }
            _ => {
                return Err(
                    format!("unknown native-host argument: {}", flag.to_string_lossy()).into(),
                );
            }
        }
    }
    let home = std::env::var_os("HOME").ok_or("HOME is required for native-host defaults")?;
    let home = PathBuf::from(home);
    let socket_path = socket.unwrap_or_else(|| {
        home.join("Library/Application Support/Jcode/MacBrowserFleet/jcode-mac-browser-fleet.sock")
    });
    let secret_path = peer_secret.unwrap_or_else(|| {
        home.join("Library/Application Support/Jcode/MacBrowserFleet/peer.secret")
    });
    let secret = fs::read_to_string(secret_path)?.trim().to_string();
    if secret.is_empty() {
        return Err("peer secret is empty".into());
    }
    let config = NativeHostBridgeConfig {
        socket_path,
        secret,
        max_payload_bytes,
    };
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    serve_native_host(config, &mut stdin, &mut stdout).await?;
    Ok(())
}

async fn run_authority(args: Vec<std::ffi::OsString>) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args.into_iter();
    let subcommand = args
        .next()
        .ok_or("usage: jcode-mac-browser-fleet authority grant|revoke|emergency-stop|release-emergency-stop|status --socket PATH ...")?
        .to_string_lossy()
        .to_string();
    let mut socket = None;
    let mut target = None;
    let mut capabilities = Vec::new();
    let mut duration_seconds = None;
    let mut lease_id = None;
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("missing value for authority argument")?;
        match flag.to_string_lossy().as_ref() {
            "--socket" => socket = Some(PathBuf::from(value)),
            "--target" | "--target-json" => {
                target = Some(serde_json::from_str::<TargetRef>(&value.to_string_lossy())?)
            }
            "--capability" => capabilities.push(parse_capability(&value.to_string_lossy())?),
            "--duration-seconds" => {
                duration_seconds = Some(value.to_string_lossy().parse::<u64>()?)
            }
            "--lease-id" => lease_id = Some(value.to_string_lossy().to_string()),
            _ => {
                return Err(
                    format!("unknown authority argument: {}", flag.to_string_lossy()).into(),
                );
            }
        }
    }
    let socket = socket.ok_or("--socket is required")?;
    let action = match subcommand.as_str() {
        "grant" => AuthorityAction::GrantLease,
        "revoke" => AuthorityAction::RevokeLease,
        "emergency-stop" => AuthorityAction::EmergencyStop,
        "release-emergency-stop" => AuthorityAction::ReleaseEmergencyStop,
        "status" => AuthorityAction::Status,
        _ => return Err(format!("unknown authority subcommand: {subcommand}").into()),
    };
    let request = AuthorityEnvelope {
        version: 1,
        action,
        lease_id,
        target,
        capabilities,
        duration_seconds,
    };
    let mut stream = UnixStream::connect(socket).await?;
    let mut encoded = serde_json::to_vec(&request)?;
    encoded.push(b'\n');
    stream.write_all(&encoded).await?;
    let mut reader = BufReader::new(stream);
    let mut line = Vec::new();
    reader.read_until(b'\n', &mut line).await?;
    print!("{}", String::from_utf8(line)?);
    Ok(())
}

fn parse_capability(value: &str) -> Result<Capability, Box<dyn std::error::Error>> {
    match value {
        "navigate" => Ok(Capability::Navigate),
        "click" => Ok(Capability::Click),
        "type" => Ok(Capability::Type),
        "press" => Ok(Capability::Press),
        _ => Err(format!("unsupported capability: {value}").into()),
    }
}
