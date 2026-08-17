use std::{
    fmt,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use chrono::Utc;
use jcode_intake_types::{ProposalId, SqliteIntakeStore, SqliteStoreError, StoreError};
use serde_json::Value;

use crate::{
    Allowlist, ApiError, Handled, TelegramAdapter, TelegramClient,
    mapping::{ParseOutcome, parse},
};

pub const ALLOWED_SENDER_ENV: &str = "JCODE_TELEGRAM_ALLOWED_SENDER";
pub const BOT_HANDLE_ENV: &str = "JCODE_TELEGRAM_BOT_HANDLE";
pub const DATABASE_PATH_ENV: &str = "JCODE_TELEGRAM_INTAKE_DB";

pub trait TelegramTransport {
    fn get_updates(
        &mut self,
        offset: Option<i64>,
        timeout_seconds: u32,
    ) -> Result<Vec<Value>, ApiError>;
    fn send_message(&mut self, conversation: &str, text: &str) -> Result<Value, ApiError>;
}

impl TelegramTransport for TelegramClient {
    fn get_updates(
        &mut self,
        offset: Option<i64>,
        timeout_seconds: u32,
    ) -> Result<Vec<Value>, ApiError> {
        TelegramClient::get_updates(self, offset, timeout_seconds)
    }

    fn send_message(&mut self, conversation: &str, text: &str) -> Result<Value, ApiError> {
        TelegramClient::send_message(self, conversation, text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerConfig {
    allowed_sender: String,
    operator_identity: String,
    bot_handle: String,
    database_path: PathBuf,
}

impl RunnerConfig {
    pub fn from_values(
        allowed_sender: &str,
        bot_handle: &str,
        database_path: impl AsRef<Path>,
    ) -> Result<Self, RunnerError> {
        let allowed_sender = allowed_sender.trim();
        if allowed_sender.is_empty()
            || allowed_sender.contains(',')
            || allowed_sender.split_whitespace().count() != 1
        {
            return Err(RunnerError::Configuration(
                "exactly one Telegram sender id is required",
            ));
        }
        let numeric = allowed_sender.strip_prefix("tg:").unwrap_or(allowed_sender);
        if numeric.parse::<i64>().is_err() {
            return Err(RunnerError::Configuration(
                "Telegram sender id must be an integer",
            ));
        }
        let bot_handle = bot_handle.trim();
        if !bot_handle.starts_with('@') || bot_handle.len() == 1 {
            return Err(RunnerError::Configuration(
                "Telegram bot handle must start with @",
            ));
        }
        let allowed_sender = format!("tg:{numeric}");
        Ok(Self {
            operator_identity: allowed_sender.clone(),
            allowed_sender,
            bot_handle: bot_handle.to_owned(),
            database_path: database_path.as_ref().to_owned(),
        })
    }

    pub fn from_env() -> Result<Self, RunnerError> {
        let sender = std::env::var(ALLOWED_SENDER_ENV)
            .map_err(|_| RunnerError::Configuration("JCODE_TELEGRAM_ALLOWED_SENDER is required"))?;
        let handle = std::env::var(BOT_HANDLE_ENV).unwrap_or_else(|_| "@jcode_bot".to_owned());
        let path = std::env::var_os(DATABASE_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(default_database_path);
        Self::from_values(&sender, &handle, path)
    }

    #[must_use]
    pub fn allowed_sender(&self) -> &str {
        &self.allowed_sender
    }
    #[must_use]
    pub fn operator_identity(&self) -> &str {
        &self.operator_identity
    }
    #[must_use]
    pub fn bot_handle(&self) -> &str {
        &self.bot_handle
    }
    #[must_use]
    pub fn database_path(&self) -> PathBuf {
        self.database_path.clone()
    }
}

fn default_database_path() -> PathBuf {
    let home = std::env::var_os("JCODE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".jcode")))
        .unwrap_or_else(|| PathBuf::from(".jcode"));
    home.join("intake").join("telegram.sqlite")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunOutcome {
    pub updates: usize,
    pub next_offset: Option<i64>,
}

pub struct TelegramIntakeRunner<T> {
    transport: T,
    adapter: TelegramAdapter,
    store: SqliteIntakeStore,
    next_offset: Option<i64>,
    poll_timeout_seconds: u32,
}

impl<T: TelegramTransport> TelegramIntakeRunner<T> {
    pub fn open(
        transport: T,
        adapter: TelegramAdapter,
        database_path: impl AsRef<Path>,
        poll_timeout_seconds: u32,
    ) -> Result<Self, RunnerError> {
        if let Some(parent) = database_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(RunnerError::Io)?;
        }
        let store = SqliteIntakeStore::open(database_path, None)?;
        let next_offset = store.polling_offset("telegram")?;
        Ok(Self {
            transport,
            adapter,
            store,
            next_offset,
            poll_timeout_seconds,
        })
    }

    pub fn run_once(&mut self) -> Result<RunOutcome, RunnerError> {
        let updates = self
            .transport
            .get_updates(self.next_offset, self.poll_timeout_seconds)?;
        for update in &updates {
            let approval = approval_request(update, &self.adapter);
            let handled = self.adapter.handle_durable(update, &mut self.store)?;
            if matches!(handled, Handled::Recorded(_))
                && let Some((proposal, approver, conversation)) = approval
            {
                match self
                    .store
                    .approve(proposal, approver, Utc::now(), "telegram".to_owned())
                {
                    Ok(_)
                    | Err(SqliteStoreError::Store(StoreError::ProposalAlreadyApproved(_))) => {}
                    Err(error) => return Err(error.into()),
                }
                self.adapter
                    .deliver(conversation, format!("Approved proposal {}.", proposal.0));
            }
            if let Some(id) = update.get("update_id").and_then(Value::as_i64) {
                self.next_offset = Some(
                    self.next_offset
                        .map_or(id + 1, |current| current.max(id + 1)),
                );
                self.store
                    .set_polling_offset("telegram", self.next_offset.expect("offset was set"))?;
            }
        }
        for outbound in self.adapter.take_outbound() {
            self.transport
                .send_message(&outbound.conversation, &outbound.text)?;
        }
        Ok(RunOutcome {
            updates: updates.len(),
            next_offset: self.next_offset,
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

impl TelegramIntakeRunner<TelegramClient> {
    pub fn from_config(
        config: &RunnerConfig,
        client: TelegramClient,
        poll_timeout_seconds: u32,
    ) -> Result<Self, RunnerError> {
        let mut allowlist = Allowlist::new(config.operator_identity());
        allowlist.allow(config.allowed_sender());
        Self::open(
            client,
            TelegramAdapter::new(allowlist, config.bot_handle()),
            config.database_path(),
            poll_timeout_seconds,
        )
    }
}

fn approval_request(
    update: &Value,
    adapter: &TelegramAdapter,
) -> Option<(ProposalId, String, String)> {
    let ParseOutcome::Message(message) = parse(update, "") else {
        return None;
    };
    let approver = adapter.operator_for_sender(&message.sender)?.to_owned();
    let id = message
        .text
        .trim()
        .strip_prefix("approve ")?
        .trim()
        .parse()
        .ok()?;
    Some((ProposalId(id), approver, message.conversation))
}

#[derive(Debug)]
pub enum RunnerError {
    Configuration(&'static str),
    Api(ApiError),
    Store(SqliteStoreError),
    Io(std::io::Error),
}

impl fmt::Display for RunnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => f.write_str(message),
            Self::Api(error) => error.fmt(f),
            Self::Store(error) => error.fmt(f),
            Self::Io(error) => write!(f, "failed to prepare Telegram intake storage: {error}"),
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

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, path::PathBuf};

    use jcode_intake_types::{ProposalState, SqliteIntakeStore};
    use serde_json::{Value, json};
    use tempfile::tempdir;

    use super::*;
    use crate::{Allowlist, ApiError, TelegramAdapter};

    #[derive(Default)]
    struct FakeTransport {
        updates: VecDeque<Vec<Value>>,
        polls: Vec<(Option<i64>, u32)>,
        sent: Vec<(String, String)>,
    }

    impl TelegramTransport for FakeTransport {
        fn get_updates(
            &mut self,
            offset: Option<i64>,
            timeout_seconds: u32,
        ) -> Result<Vec<Value>, ApiError> {
            self.polls.push((offset, timeout_seconds));
            Ok(self.updates.pop_front().unwrap_or_default())
        }

        fn send_message(&mut self, conversation: &str, text: &str) -> Result<Value, ApiError> {
            self.sent.push((conversation.to_owned(), text.to_owned()));
            Ok(json!({"message_id": 1}))
        }
    }

    fn update(update_id: i64, text: &str) -> Value {
        json!({
            "update_id": update_id,
            "message": {
                "chat": {"id": 555, "type": "private"},
                "from": {"id": 7},
                "text": text,
            }
        })
    }

    fn runner(path: PathBuf, transport: FakeTransport) -> TelegramIntakeRunner<FakeTransport> {
        let mut allowlist = Allowlist::new("operator");
        allowlist.allow("tg:7");
        TelegramIntakeRunner::open(
            transport,
            TelegramAdapter::new(allowlist, "@jcode_bot"),
            path,
            20,
        )
        .expect("runner opens durable store")
    }

    #[test]
    fn config_requires_exactly_one_sender_and_builds_canonical_identifiers() {
        let config = RunnerConfig::from_values("7", "@jcode_bot", "/state/intake.sqlite")
            .expect("valid config");
        assert_eq!(config.allowed_sender(), "tg:7");
        assert_eq!(config.operator_identity(), "tg:7");
        assert_eq!(config.bot_handle(), "@jcode_bot");
        assert_eq!(
            config.database_path(),
            PathBuf::from("/state/intake.sqlite")
        );

        assert!(RunnerConfig::from_values("7,8", "@jcode_bot", "/tmp/db").is_err());
        assert!(RunnerConfig::from_values("", "@jcode_bot", "/tmp/db").is_err());
    }

    #[test]
    fn run_once_polls_records_durably_advances_offset_and_sends_outbound() {
        let temp = tempdir().unwrap();
        let db = temp.path().join("intake.sqlite");
        let mut transport = FakeTransport::default();
        transport.updates.push_back(vec![
            update(40, "implement the runner"),
            update(41, "token 123456789:AAHfSHFyTvJmL5RkQxWnPzZbCdEfGhIjKlM"),
        ]);
        let mut runner = runner(db.clone(), transport);

        let outcome = runner.run_once().expect("poll succeeds");
        assert_eq!(
            outcome,
            RunOutcome {
                updates: 2,
                next_offset: Some(42)
            }
        );
        assert_eq!(runner.transport().polls, vec![(None, 20)]);
        assert_eq!(
            runner.transport().sent.len(),
            1,
            "redaction notice delivered"
        );
        assert_eq!(runner.transport().sent[0].0, "tg:555");

        drop(runner);
        let reopened = SqliteIntakeStore::open(&db, None).unwrap();
        assert_eq!(reopened.records().unwrap().len(), 2);
        assert_eq!(reopened.proposals().unwrap().len(), 1);
        assert!(reopened.tracked_work().unwrap().is_empty());
    }

    #[test]
    fn approval_message_approves_existing_proposal_without_identity_mapping() {
        let temp = tempdir().unwrap();
        let db = temp.path().join("intake.sqlite");
        let mut transport = FakeTransport::default();
        transport
            .updates
            .push_back(vec![update(1, "implement telegram intake")]);
        transport.updates.push_back(vec![update(2, "approve 1")]);
        let mut runner = runner(db, transport);

        runner.run_once().unwrap();
        runner.run_once().unwrap();

        let proposals = runner.store().proposals().unwrap();
        assert_eq!(proposals[0].state, ProposalState::Approved);
        assert_eq!(proposals[0].approved_by.as_deref(), Some("operator"));
        assert_eq!(proposals[0].approved_channel.as_deref(), Some("telegram"));
        assert_eq!(runner.store().tracked_work().unwrap().len(), 1);
        assert!(
            runner
                .transport()
                .sent
                .iter()
                .any(|(_, text)| text.contains("Approved proposal 1"))
        );
    }

    #[test]
    fn reopening_runner_restores_the_persisted_polling_offset() {
        let temp = tempdir().unwrap();
        let db = temp.path().join("intake.sqlite");
        let mut transport = FakeTransport::default();
        transport.updates.push_back(vec![update(41, "status")]);
        let mut first = runner(db.clone(), transport);

        first.run_once().unwrap();
        drop(first);

        let mut reopened = runner(db, FakeTransport::default());
        reopened.run_once().unwrap();

        assert_eq!(reopened.transport().polls, vec![(Some(42), 20)]);
    }

    #[test]
    fn replaying_an_approved_command_is_idempotent() {
        let temp = tempdir().unwrap();
        let db = temp.path().join("intake.sqlite");
        let mut transport = FakeTransport::default();
        transport
            .updates
            .push_back(vec![update(1, "implement telegram intake")]);
        transport.updates.push_back(vec![update(2, "approve 1")]);
        let mut first = runner(db.clone(), transport);

        first.run_once().unwrap();
        first.run_once().unwrap();
        drop(first);

        let mut replay = FakeTransport::default();
        replay.updates.push_back(vec![update(2, "approve 1")]);
        let mut reopened = runner(db, replay);

        reopened
            .run_once()
            .expect("approval replay remains successful");
        assert_eq!(reopened.store().tracked_work().unwrap().len(), 1);
    }
}
