use jcode_intake_slack::{RunnerConfig, SlackClient, SlackIntakeRunner, load_tokens};

fn main() {
    if let Err(error) = run() {
        eprintln!("jcode Slack intake failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = RunnerConfig::from_env()?;
    let (app_token, bot_token) = load_tokens()?;
    let client = SlackClient::new(app_token, bot_token);
    SlackIntakeRunner::open(config, client)?.run_continuous()?;
    Ok(())
}
