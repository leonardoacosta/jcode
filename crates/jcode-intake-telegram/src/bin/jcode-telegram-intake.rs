use jcode_intake_telegram::{RunnerConfig, TelegramClient, TelegramIntakeRunner, load_bot_token};

fn main() {
    if let Err(error) = run() {
        eprintln!("jcode Telegram intake failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let once = std::env::args()
        .skip(1)
        .any(|argument| argument == "--once");
    let config = RunnerConfig::from_env()?;
    let token = load_bot_token()?;
    let client = TelegramClient::new(token);
    let mut runner = TelegramIntakeRunner::from_config(&config, client, 30)?;
    if once {
        let outcome = runner.run_once()?;
        println!(
            "processed {} Telegram update(s); next offset {:?}",
            outcome.updates, outcome.next_offset
        );
        Ok(())
    } else {
        runner.run_continuous()?;
        Ok(())
    }
}
