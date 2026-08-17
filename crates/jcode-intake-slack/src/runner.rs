use std::{
    fmt,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use chrono::Utc;
use jcode_intake_types::{
    Envelope, IntakeEvent, ProposalId, SqliteIntakeStore, SqliteStoreError, StoreError,
};
use serde_json::Value;

use crate::{ApiError, ParseOutcome, ParsedMessage, SlackTransport, parse};

pub const ALLOWED_WORKSPACE_ENV: &str = "JCODE_SLACK_ALLOWED_WORKSPACE";
pub const ALLOWED_SENDER_ENV: &str = "JCODE_SLACK_ALLOWED_SENDER";
pub const BOT_HANDLE_ENV: &str = "JCODE_SLACK_BOT_HANDLE";
pub const DATABASE_PATH_ENV: &str = "JCODE_DECISION_INBOX_DB";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerConfig {
    allowed_workspace: String,
    allowed_sender: String,
    operator_identity: String,
    bot_handle: String,
    database_path: PathBuf,
}

impl RunnerConfig {
    pub fn from_values(
        workspace: &str,
        sender: &str,
        bot_handle: &str,
        database_path: impl AsRef<Path>,
    ) -> Result<Self, RunnerError> {
        let workspace = canonical(workspace, "sl-team:", "Slack workspace id is required")?;
        let sender = canonical(sender, "sl:", "exactly one Slack sender id is required")?;
        let bot_handle = bot_handle.trim();
        if bot_handle.is_empty() {
            return Err(RunnerError::Configuration("Slack bot handle is required"));
        }
        Ok(Self {
            allowed_workspace: workspace,
            operator_identity: sender.clone(),
            allowed_sender: sender,
            bot_handle: bot_handle.to_owned(),
            database_path: database_path.as_ref().to_owned(),
        })
    }

    pub fn from_env() -> Result<Self, RunnerError> {
        let workspace = std::env::var(ALLOWED_WORKSPACE_ENV)
            .map_err(|_| RunnerError::Configuration("JCODE_SLACK_ALLOWED_WORKSPACE is required"))?;
        let sender = std::env::var(ALLOWED_SENDER_ENV)
            .map_err(|_| RunnerError::Configuration("JCODE_SLACK_ALLOWED_SENDER is required"))?;
        let handle = std::env::var(BOT_HANDLE_ENV).unwrap_or_else(|_| "<@JCODE>".to_owned());
        let path = std::env::var_os(DATABASE_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(default_database_path);
        Self::from_values(&workspace, &sender, &handle, path)
    }
}

fn canonical(value: &str, prefix: &str, error: &'static str) -> Result<String, RunnerError> {
    let value = value.trim();
    if value.is_empty() || value.contains(',') || value.split_whitespace().count() != 1 {
        return Err(RunnerError::Configuration(error));
    }
    Ok(if value.starts_with(prefix) {
        value.to_owned()
    } else {
        format!("{prefix}{value}")
    })
}

fn default_database_path() -> PathBuf {
    let home = std::env::var_os("JCODE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".jcode")))
        .unwrap_or_else(|| PathBuf::from(".jcode"));
    home.join("intake").join("decision-inbox.sqlite")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunOutcome {
    pub envelopes: usize,
    pub acknowledgements: usize,
}

pub struct SlackIntakeRunner<T> {
    config: RunnerConfig,
    transport: T,
    store: SqliteIntakeStore,
}

impl<T: SlackTransport> SlackIntakeRunner<T> {
    pub fn open(config: RunnerConfig, transport: T) -> Result<Self, RunnerError> {
        if let Some(parent) = config.database_path.parent() {
            std::fs::create_dir_all(parent).map_err(RunnerError::Io)?;
        }
        let store = SqliteIntakeStore::open(&config.database_path, None)?;
        Ok(Self {
            config,
            transport,
            store,
        })
    }

    pub fn run_once(&mut self) -> Result<RunOutcome, RunnerError> {
        let Some(envelope) = self.transport.next_envelope()? else {
            return Ok(RunOutcome {
                envelopes: 0,
                acknowledgements: 0,
            });
        };
        let envelope_id = envelope
            .get("envelope_id")
            .and_then(Value::as_str)
            .ok_or(ApiError::InvalidResponse)?
            .to_owned();
        let approval = approval_request(&envelope, &self.config);
        let handled = handle_durable(&envelope, &self.config, &mut self.store)?;
        self.transport.acknowledge(&envelope_id)?;

        if let Some((proposal, approver, conversation)) = approval {
            match self
                .store
                .approve(proposal, approver, Utc::now(), "slack".to_owned())
            {
                Ok(_) | Err(SqliteStoreError::Store(StoreError::ProposalAlreadyApproved(_))) => {}
                Err(error) => return Err(error.into()),
            }
            self.transport
                .send_message(&conversation, &format!("Approved proposal {}.", proposal.0))?;
        } else if let Some((conversation, text)) = handled.outbound {
            self.transport.send_message(&conversation, &text)?;
        }
        Ok(RunOutcome {
            envelopes: 1,
            acknowledgements: 1,
        })
    }

    pub fn run_continuous(&mut self) -> Result<(), RunnerError> {
        loop {
            match self.run_once() {
                Ok(_) => {}
                Err(RunnerError::Api(ApiError::Network)) => thread::sleep(Duration::from_secs(2)),
                Err(error) => return Err(error),
            }
        }
    }

    #[must_use]
    pub fn transport(&self) -> &T {
        &self.transport
    }
    #[must_use]
    pub fn store(&self) -> &SqliteIntakeStore {
        &self.store
    }
}

struct DurableHandled {
    outbound: Option<(String, String)>,
}

fn handle_durable(
    event: &Value,
    config: &RunnerConfig,
    store: &mut SqliteIntakeStore,
) -> Result<DurableHandled, SqliteStoreError> {
    match parse(event, &config.bot_handle) {
        ParseOutcome::Message(parsed) => {
            if parsed.is_group && !parsed.addresses_bot {
                return Ok(DurableHandled { outbound: None });
            }
            let authorized = parsed.workspace == config.allowed_workspace
                && parsed.sender == config.allowed_sender;
            let envelope = provider_neutral_envelope(
                &parsed,
                if authorized {
                    &config.operator_identity
                } else {
                    &parsed.sender
                },
            );
            let record = if authorized {
                store.receive(
                    envelope,
                    event.clone(),
                    Some(config.operator_identity.clone()),
                )?
            } else {
                store.receive_unauthorized(envelope, event.clone())?
            };
            let outbound = if authorized {
                store.events()?.iter().any(
                    |event| matches!(event, IntakeEvent::Redaction { record: id, .. } if *id == record),
                ).then(|| (
                    parsed.conversation.clone(),
                    "Note: credential-shaped content was redacted before storage.".to_owned(),
                ))
            } else {
                Some((
                    parsed.conversation.clone(),
                    format!(
                        "Sender {} is not authorized. Configure that identifier as the single Slack operator.",
                        parsed.sender
                    ),
                ))
            };
            Ok(DurableHandled { outbound })
        }
        ParseOutcome::Unhandled { variant } => {
            let envelope = Envelope {
                adapter: "slack".to_owned(),
                sender_identity: format!("unhandled:{variant}"),
                conversation: format!("unhandled:{variant}"),
                content: None,
                attachments: Vec::new(),
                received_at: Utc::now(),
            };
            store.receive_unauthorized(envelope, event.clone())?;
            Ok(DurableHandled { outbound: None })
        }
    }
}

fn provider_neutral_envelope(parsed: &ParsedMessage, identity: &str) -> Envelope {
    Envelope {
        adapter: "slack".to_owned(),
        sender_identity: identity.to_owned(),
        conversation: parsed.conversation.clone(),
        content: Some(parsed.text.clone()),
        attachments: Vec::new(),
        received_at: Utc::now(),
    }
}

fn approval_request(event: &Value, config: &RunnerConfig) -> Option<(ProposalId, String, String)> {
    let ParseOutcome::Message(message) = parse(event, &config.bot_handle) else {
        return None;
    };
    if message.workspace != config.allowed_workspace || message.sender != config.allowed_sender {
        return None;
    }
    let id = message
        .text
        .trim()
        .strip_prefix("approve ")?
        .trim()
        .parse()
        .ok()?;
    Some((
        ProposalId(id),
        config.operator_identity.clone(),
        message.conversation,
    ))
}

#[derive(Debug)]
pub enum RunnerError {
    Configuration(&'static str),
    Api(ApiError),
    Store(SqliteStoreError),
    Io(std::io::Error),
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => formatter.write_str(message),
            Self::Api(error) => error.fmt(formatter),
            Self::Store(error) => error.fmt(formatter),
            Self::Io(error) => write!(formatter, "failed to prepare Slack intake storage: {error}"),
        }
    }
}

impl std::error::Error for RunnerError {}
impl From<ApiError> for RunnerError {
    fn from(value: ApiError) -> Self {
        Self::Api(value)
    }
}
impl From<SqliteStoreError> for RunnerError {
    fn from(value: SqliteStoreError) -> Self {
        Self::Store(value)
    }
}
