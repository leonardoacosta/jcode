use crate::automations::config;
use anyhow::{Context, Result, ensure};
use clap::Subcommand;
#[path = "automations_provision.rs"]
mod provision_commands;
use provision_commands::{service, tailnet};

#[derive(Subcommand, Debug)]
pub(crate) enum AutomationCommand {
    /// Configure the daemon's bulletin. Does not change services or network policy.
    Init {
        #[arg(long)]
        port: Option<u16>,
    },
    /// Print a short-lived browser pairing URL. Treat this URL as a credential.
    Open {
        #[arg(long)]
        tailnet: bool,
    },
    /// Show configured URLs and live listener readiness without secrets.
    Status,
    /// Provision private HTTPS through Tailscale Serve, without changing ACLs or login.
    Tailnet {
        #[arg(long, default_value_t = 8443)]
        https_port: u16,
        #[arg(long)]
        disable: bool,
        #[arg(long)]
        confirm: bool,
    },
    /// Install or remove the managed Linux user service. Never stops unrelated daemons.
    Service {
        #[arg(long)]
        uninstall: bool,
        #[arg(long)]
        enable_linger: bool,
        #[arg(long)]
        confirm: bool,
    },
}

pub(crate) async fn run(action: AutomationCommand, model: Option<&str>) -> Result<()> {
    let directory = config::directory()?;
    if let AutomationCommand::Init { port } = action {
        ensure!(
            config::load(&directory)?.is_none(),
            "automations already configured; configuration was not replaced"
        );
        let listener =
            std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port.unwrap_or(0)))
                .context("reserve local bulletin port")?;
        let mut settings = config::Config::new(
            std::env::current_dir()?,
            crate::server::socket_path(),
            listener.local_addr()?.port(),
        );
        settings.model = model.map(str::to_owned);
        config::save(&directory, &settings)?;
        println!(
            "Configured {}. The configured daemon activates this automatically when running. No service or Tailscale route changed.",
            settings.local_origin()
        );
        return Ok(());
    }
    let settings = config::load(&directory)?
        .context("automations not configured; run `jcode automations init` first")?;
    match action {
        AutomationCommand::Init { .. } => unreachable!(),
        AutomationCommand::Open { tailnet } => {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::none())
                .build()?;
            let response = client
                .post(format!("{}/api/bootstrap", settings.local_origin()))
                .header(reqwest::header::ORIGIN, settings.local_origin())
                .bearer_auth(&settings.control_token)
                .json(&serde_json::json!({"target": if tailnet { "tailnet" } else { "local" }}))
                .send()
                .await
                .context("bulletin unavailable; start or restart its configured daemon")?;
            ensure!(
                response.status().is_success(),
                "bulletin refused browser pairing ({})",
                response.status()
            );
            let result: serde_json::Value = response.json().await?;
            let url = result
                .get("url")
                .and_then(serde_json::Value::as_str)
                .context("invalid pairing response")?;
            println!("{url}");
            eprintln!("Private single-use pairing link, valid for five minutes. Do not share it.");
        }
        AutomationCommand::Status => {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .redirect(reqwest::redirect::Policy::none())
                .build()?;
            let response = client
                .get(format!("{}/", settings.local_origin()))
                .send()
                .await;
            let listener_reachable = response.is_ok();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "enabled": settings.enabled,
                    "local_url": settings.local_origin(),
                    "tailnet_url": settings.tailnet_origin,
                    "listener_reachable": listener_reachable,
                    "socket": settings.socket_path,
                    "note": "A reachable listener is not proof of authenticated bulletin or tailnet readiness."
                }))?
            );
        }
        AutomationCommand::Tailnet {
            https_port,
            disable,
            confirm,
        } => {
            ensure!(
                confirm,
                "Tailscale Serve configuration changes require --confirm. No login or access-policy changes will be made."
            );
            tailnet(&directory, settings, https_port, disable).await?;
        }
        AutomationCommand::Service {
            uninstall,
            enable_linger,
            confirm,
        } => {
            ensure!(
                confirm,
                "Managed user-service changes require --confirm. Enabling linger additionally requires --enable-linger."
            );
            service(&directory, &settings, uninstall, enable_linger).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    #[derive(Parser)]
    struct TestArgs {
        #[command(subcommand)]
        action: AutomationCommand,
    }
    #[test]
    fn provisioning_is_never_implicitly_confirmed() {
        let parsed = TestArgs::try_parse_from(["test", "tailnet"]).unwrap();
        assert!(matches!(
            parsed.action,
            AutomationCommand::Tailnet {
                confirm: false,
                https_port: 8443,
                disable: false
            }
        ));
        assert!(TestArgs::try_parse_from(["test", "service", "--enable-linger"]).is_ok());
    }
}
