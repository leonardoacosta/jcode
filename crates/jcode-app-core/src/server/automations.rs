use super::{Server, runtime::ServerRuntime};
use crate::automations::{
    config,
    store::{Automation, Run, RunStatus, Store},
    web::{self, WebConfig},
};
use anyhow::{Context, Result, bail};
use chrono::Utc;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{
    net::TcpListener,
    sync::{Mutex, mpsc},
};
use tokio_util::sync::CancellationToken;

const POLL: Duration = Duration::from_secs(2);
const DEADLINE: Duration = Duration::from_secs(30 * 60);

pub(super) async fn start(server: &Server, runtime: &ServerRuntime, temporary: bool) {
    if temporary {
        return;
    }
    let socket = server.socket_path.clone();
    let provider = Arc::clone(&server.provider);
    let _ = runtime
        .spawn_scoped_task(move |cancel| async move { watcher(socket, provider, cancel).await })
        .await;
}

async fn watcher(
    socket: PathBuf,
    provider: Arc<dyn crate::provider::Provider>,
    cancel: CancellationToken,
) {
    let mut active: Option<(ConfigKey, CancellationToken, tokio::task::JoinHandle<()>)> = None;
    loop {
        if cancel.is_cancelled() {
            break;
        }
        let config_dir = match config::directory() {
            Ok(dir) => dir,
            Err(error) => {
                crate::logging::warn(&format!("Automation config path unavailable: {error}"));
                sleep(&cancel).await;
                continue;
            }
        };
        let loaded = config::load(&config_dir);
        let candidate = match loaded {
            Ok(Some(config)) if config.enabled && config.socket_path == socket => Some(config),
            Ok(_) => None,
            Err(error) => {
                crate::logging::warn(&format!("Automation config unavailable: {error}"));
                None
            }
        };
        if let Some(config) = candidate {
            let key = ConfigKey::from((&config, &config_dir));
            if active
                .as_ref()
                .is_some_and(|(old, _, task)| old == &key && task.is_finished())
            {
                stop_active(&mut active).await;
            }
            if active.as_ref().is_none_or(|(old, _, _)| old != &key) {
                stop_active(&mut active).await;
                let token = cancel.child_token();
                match run_instance(config_dir, config, Arc::clone(&provider), token.clone()).await {
                    Ok(task) => active = Some((key, token, task)),
                    Err(error) => {
                        crate::logging::warn(&format!("Automation runtime not started: {error}"))
                    }
                }
            }
        } else {
            stop_active(&mut active).await;
        }
        sleep(&cancel).await;
    }
    stop_active(&mut active).await;
}

#[derive(PartialEq, Eq)]
struct ConfigKey {
    state: PathBuf,
    port: u16,
    enabled: bool,
    tailnet: Option<String>,
    token: String,
    working_dir: PathBuf,
    provider: Option<String>,
    model: Option<String>,
}
impl From<(&config::Config, &PathBuf)> for ConfigKey {
    fn from((c, dir): (&config::Config, &PathBuf)) -> Self {
        Self {
            state: dir.join("state.json"),
            port: c.port,
            enabled: c.enabled,
            tailnet: c.tailnet_origin.clone(),
            token: c.control_token.clone(),
            working_dir: c.working_dir.clone(),
            provider: c.provider.clone(),
            model: c.model.clone(),
        }
    }
}

async fn stop_active(
    active: &mut Option<(ConfigKey, CancellationToken, tokio::task::JoinHandle<()>)>,
) {
    if let Some((_, token, task)) = active.take() {
        token.cancel();
        let _ = task.await;
    }
}

async fn run_instance(
    dir: PathBuf,
    cfg: config::Config,
    provider: Arc<dyn crate::provider::Provider>,
    cancel: CancellationToken,
) -> Result<tokio::task::JoinHandle<()>> {
    let store = Arc::new(Mutex::new(Store::open(dir.join("state.json"))?));
    store.lock().await.recover(Utc::now())?;
    Ok(tokio::spawn(async move {
        let api = listener_retry(cfg, Arc::clone(&store), cancel.child_token());
        let scheduler = recurrence(Arc::clone(&store), provider, cancel.child_token());
        tokio::pin!(api);
        tokio::pin!(scheduler);
        tokio::select! {
            _ = cancel.cancelled() => { let _ = scheduler.await; let _ = api.await; },
            result = &mut scheduler => { if let Err(error) = result { crate::logging::error(&format!("Automation scheduler stopped: {error}")); } let _ = api.await; },
            result = &mut api => {
                if let Err(error) = result { crate::logging::warn(&format!("Automation bulletin stopped: {error}")); }
                let _ = scheduler.await;
            }
        }
    }))
}

async fn listener_retry(
    cfg: config::Config,
    store: Arc<Mutex<Store>>,
    cancel: CancellationToken,
) -> Result<()> {
    loop {
        let listener = tokio::select! {
            _ = cancel.cancelled() => return Ok(()),
            bound = TcpListener::bind(("127.0.0.1", cfg.port)) => match bound {
                Ok(listener) => listener,
                Err(error) => { crate::logging::warn(&format!("Automation bulletin bind failed; retrying: {error}")); sleep(&cancel).await; continue; }
            }
        };
        let web_config = WebConfig {
            local_origin: cfg.local_origin(),
            tailnet_origin: cfg.tailnet_origin.clone(),
            control_token: cfg.control_token.clone(),
            default_working_dir: cfg.working_dir.clone(),
            default_provider: cfg.provider.clone(),
            default_model: cfg.model.clone(),
        };
        web::serve(
            listener,
            Arc::clone(&store),
            web_config,
            cancel.child_token(),
        )
        .await?;
    }
}

async fn recurrence(
    store: Arc<Mutex<Store>>,
    provider: Arc<dyn crate::provider::Provider>,
    cancel: CancellationToken,
) -> Result<()> {
    loop {
        if cancel.is_cancelled() {
            return Ok(());
        }
        let claim = store.lock().await.claim_due(Utc::now())?;
        if let Some((automation, run)) = claim {
            let result = execute(
                &automation,
                &run,
                Arc::clone(&store),
                Arc::clone(&provider),
                cancel.child_token(),
            )
            .await;
            let (status, output) = match result {
                Ok((output, status)) => (status, output),
                Err(error) => {
                    let diagnostic = safe_diagnostic(&error.to_string());
                    if diagnostic.contains("skill unavailable")
                        || diagnostic.contains("working directory is unavailable")
                    {
                        let mut db = store.lock().await;
                        let _ = db.pause(&automation.id);
                    }
                    (RunStatus::Failed, diagnostic)
                }
            };
            store
                .lock()
                .await
                .finish(&run.id, status, &output, Utc::now())?;
        }
        sleep(&cancel).await;
    }
}

async fn execute(
    automation: &Automation,
    run: &Run,
    store: Arc<Mutex<Store>>,
    provider_template: Arc<dyn crate::provider::Provider>,
    cancel: CancellationToken,
) -> Result<(String, RunStatus)> {
    use crate::{agent::Agent, protocol::ServerEvent, session::Session, tool::Registry};
    if !automation.working_dir.is_absolute() || !automation.working_dir.is_dir() {
        store.lock().await.pause(&automation.id)?;
        bail!("automation working directory is unavailable; paused automation");
    }
    let skills = crate::skill::SkillRegistry::load_for_working_dir(Some(&automation.working_dir))
        .context("load automation skills")?;
    let skill = skills
        .get(&automation.skill)
        .with_context(|| format!("skill unavailable: {}", automation.skill))?;
    let source = format!("{}#{}", skill.path.display(), sha256(&skill.content));
    let provider = provider_template.fork();
    if let Some(model) = automation.model.as_deref() {
        crate::provider::set_model_with_auth_refresh(provider.as_ref(), model)
            .context("select automation model")?;
    }
    let registry = Registry::new(Arc::clone(&provider)).await;
    registry
        .register_mcp_tools_for_dir(
            None,
            None,
            Some(run.id.clone()),
            Some(automation.working_dir.clone()),
        )
        .await;
    registry.unregister("swarm").await;
    let mut session = Session::create(None, Some(format!("Automation: {}", automation.skill)));
    session.working_dir = Some(automation.working_dir.to_string_lossy().into_owned());
    if let Some(model) = automation.model.clone() {
        session.model = Some(model);
    }
    if let Some(provider_name) = automation.provider.as_deref() {
        session.provider_key = Some(
            crate::provider::provider_from_model_key(provider_name)
                .map(jcode_provider_core::provider_key)
                .map(str::to_string)
                .with_context(|| format!("unknown automation provider: {provider_name}"))?,
        );
    }
    let mut agent = Agent::new_with_session(provider, registry, session, None);
    agent.set_working_dir_for_pending_context(Some(
        automation.working_dir.to_string_lossy().into_owned(),
    ));
    if !agent.set_remote_active_skill(Some(automation.skill.clone())) {
        bail!("automation skill missing from agent registry");
    }
    let session_id = agent.session_id().to_string();
    store
        .lock()
        .await
        .set_session_and_source(&run.id, Some(session_id), Some(source))?;
    let (tx, mut rx) = mpsc::unbounded_channel();
    let prompt = if automation.arguments.trim().is_empty() {
        skill.get_prompt()
    } else {
        format!(
            "{}\n\nAutomation arguments:\n{}",
            skill.get_prompt(),
            automation.arguments
        )
    };
    let start_message = agent.message_count();
    let terminal = {
        let signal = agent.graceful_shutdown_signal();
        let mut agent_run = Box::pin(agent.run_once_streaming_mpsc(&prompt, Vec::new(), None, tx));
        let deadline = tokio::time::sleep(DEADLINE);
        tokio::pin!(deadline);
        let mut terminal = None;
        loop {
            tokio::select! {
                _ = cancel.cancelled() => { signal.fire(); let _ = (&mut agent_run).await; terminal = Some(RunStatus::Interrupted); break; }
                _ = &mut deadline => { signal.fire(); let _ = (&mut agent_run).await; terminal = Some(RunStatus::Timeout); break; }
                event = rx.recv() => match event {
                    Some(ServerEvent::ToolStart { name, .. }) | Some(ServerEvent::ToolExec { name, .. }) if name == "ask_user_question" || name == "request_permission" => { signal.fire(); let _ = (&mut agent_run).await; terminal = Some(RunStatus::Blocked); break; }
                    Some(ServerEvent::ToolDone { name, error: Some(_), .. }) if name == "ask_user_question" || name == "request_permission" => { signal.fire(); let _ = (&mut agent_run).await; terminal = Some(RunStatus::Blocked); break; }
                    None => break,
                    _ => {}
                },
                result = &mut agent_run => { result.context("automation agent turn failed")?; break; }
            }
        }
        terminal
    };
    let output = agent
        .latest_assistant_text_after(start_message)
        .unwrap_or_default();
    agent.mark_closed();
    Ok((
        safe_diagnostic(&output),
        terminal.unwrap_or(RunStatus::Success),
    ))
}

fn redact(text: &str, secrets: &[String]) -> String {
    let mut value = text.to_string();
    for secret in secrets.iter().filter(|s| !s.is_empty()) {
        value = value.replace(secret, "[REDACTED]");
    }
    value
}

fn safe_diagnostic(text: &str) -> String {
    let mut secrets = [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENAI_ACCESS_TOKEN",
        "ANTHROPIC_AUTH_TOKEN",
        "OPENROUTER_API_KEY",
        "GITHUB_TOKEN",
        "COPILOT_GITHUB_TOKEN",
        "GH_TOKEN",
    ]
    .iter()
    .filter_map(|name| std::env::var(name).ok())
    .collect::<Vec<_>>();
    if let Ok(credentials) = crate::auth::claude::load_credentials() {
        secrets.extend([credentials.access_token, credentials.refresh_token]);
    }
    if let Ok(credentials) = crate::auth::codex::load_credentials() {
        secrets.extend([credentials.access_token, credentials.refresh_token]);
        secrets.extend(credentials.id_token);
    }
    redact(text, &secrets)
}
fn sha256(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
async fn sleep(cancel: &CancellationToken) {
    tokio::select! { _ = cancel.cancelled() => {}, _ = tokio::time::sleep(POLL) => {} }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redaction_removes_known_secrets() {
        assert_eq!(redact("token=abc", &["abc".into()]), "token=[REDACTED]");
    }
}
