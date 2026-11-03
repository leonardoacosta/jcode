//! Pre-send monetary contracts. Price estimates are never reservation authority.
//! The only supported adapter is an explicit offline synthetic fixture. It has
//! no endpoint, credentials, route registration, or real-money billing claim.

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::future::Future;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonetaryBudget {
    directory: PathBuf,
    task_id: String,
    original_limit_microusd: u64,
    ceiling_microusd: u64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MonetaryTotals {
    pub reserved_microusd: u64,
    pub actual_microusd: u64,
    pub unsettled_microusd: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Entry {
    Original { task_id: String, limit: u64 },
    Ceiling { limit: u64 },
    Reserve { ceiling: u64 },
    Actual { reservation: usize, charge: u64 },
}

struct State {
    ceiling: u64,
    reservations: Vec<(u64, Option<u64>)>,
}

fn safe_directory(path: &Path) -> Result<PathBuf> {
    ensure!(path.is_absolute(), "Monetary account path must be absolute");
    let mut parent = PathBuf::new();
    for part in path.components() {
        ensure!(
            !matches!(part, std::path::Component::ParentDir),
            "Unsafe monetary account path"
        );
        parent.push(part);
        let meta = std::fs::symlink_metadata(&parent)?;
        ensure!(
            meta.is_dir() && !meta.file_type().is_symlink(),
            "Unsafe monetary account directory"
        );
    }
    Ok(path.canonicalize()?)
}

fn open_journal(directory: &Path, create: bool) -> Result<File> {
    ensure!(
        safe_directory(directory)? == directory,
        "Monetary account path changed"
    );
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(create);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW).mode(0o600);
    }
    let file = options.open(directory.join("reservations.jsonl"))?;
    ensure!(file.metadata()?.is_file(), "Unsafe monetary journal");
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(file.metadata()?.nlink() == 1, "Aliased monetary journal");
    }
    file.lock()?;
    Ok(file)
}

fn append(file: &mut File, entry: &Entry) -> Result<()> {
    let mut bytes = serde_json::to_vec(entry)?;
    bytes.push(b'\n');
    file.seek(SeekFrom::End(0))?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

impl MonetaryBudget {
    pub fn create(
        directory: PathBuf,
        task_id: String,
        original_limit_microusd: u64,
    ) -> Result<Self> {
        ensure!(!task_id.is_empty(), "Missing monetary task identity");
        // Existing parents must be trusted and non-symlink. Only this private
        // account directory is created, never an alternate task/worktree root.
        safe_directory(
            directory
                .parent()
                .context("Missing monetary account parent")?,
        )?;
        std::fs::create_dir(&directory)?;
        let directory = safe_directory(&directory)?;
        let mut file = open_journal(&directory, true)?;
        append(
            &mut file,
            &Entry::Original {
                task_id: task_id.clone(),
                limit: original_limit_microusd,
            },
        )?;
        File::open(&directory)?.sync_all()?;
        Ok(Self {
            directory,
            task_id,
            original_limit_microusd,
            ceiling_microusd: original_limit_microusd,
        })
    }

    pub fn open(directory: PathBuf, task_id: &str, original_limit_microusd: u64) -> Result<Self> {
        let mut budget = Self {
            directory: safe_directory(&directory)?,
            task_id: task_id.into(),
            original_limit_microusd,
            ceiling_microusd: original_limit_microusd,
        };
        let mut file = open_journal(&budget.directory, false)?;
        budget.ceiling_microusd = budget.read(&mut file)?.ceiling;
        Ok(budget)
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }
    pub fn original_limit_microusd(&self) -> u64 {
        self.original_limit_microusd
    }
    pub fn ceiling_microusd(&self) -> u64 {
        self.ceiling_microusd
    }

    fn same_account(&self, other: &Self) -> bool {
        self.directory == other.directory
            && self.task_id == other.task_id
            && self.original_limit_microusd == other.original_limit_microusd
    }

    /// A total ceiling, not a fresh remaining allowance. Narrowing is durable.
    pub fn with_ceiling(mut self, total_ceiling_microusd: u64) -> Result<Self> {
        let mut file = open_journal(&self.directory, false)?;
        let state = self.read(&mut file)?;
        self.ceiling_microusd = state
            .ceiling
            .min(self.ceiling_microusd)
            .min(total_ceiling_microusd);
        append(
            &mut file,
            &Entry::Ceiling {
                limit: self.ceiling_microusd,
            },
        )?;
        Ok(self)
    }

    fn read(&self, file: &mut File) -> Result<State> {
        file.seek(SeekFrom::Start(0))?;
        let mut bytes = String::new();
        file.read_to_string(&mut bytes)?;
        ensure!(
            bytes.ends_with('\n'),
            "Incomplete monetary journal; reservation state unverifiable"
        );
        let mut lines = bytes.lines();
        let original: Entry = serde_json::from_str(
            lines
                .next()
                .context("Missing monetary original authorization")?,
        )?;
        match original {
            Entry::Original { task_id, limit } => ensure!(
                task_id == self.task_id && limit == self.original_limit_microusd,
                "Monetary original authorization mismatch"
            ),
            _ => bail!("Missing monetary original authorization"),
        }
        let mut state = State {
            ceiling: self.original_limit_microusd,
            reservations: vec![],
        };
        let mut reserved = 0u64;
        for line in lines {
            match serde_json::from_str::<Entry>(line)? {
                Entry::Ceiling { limit } => {
                    ensure!(limit <= state.ceiling, "Monetary ceiling widened");
                    state.ceiling = limit;
                }
                Entry::Reserve { ceiling } => {
                    reserved = reserved
                        .checked_add(ceiling)
                        .context("Monetary reservation overflow")?;
                    ensure!(
                        reserved <= state.ceiling,
                        "Monetary reservations exceeded ceiling"
                    );
                    state.reservations.push((ceiling, None));
                }
                Entry::Actual {
                    reservation,
                    charge,
                } => {
                    let (ceiling, actual) = state
                        .reservations
                        .get_mut(reservation)
                        .context("Unknown monetary reservation")?;
                    ensure!(
                        actual.is_none() && charge <= *ceiling,
                        "Invalid monetary settlement"
                    );
                    *actual = Some(charge);
                }
                Entry::Original { .. } => bail!("Repeated monetary original authorization"),
            }
        }
        Ok(state)
    }

    pub fn totals(&self) -> Result<MonetaryTotals> {
        let mut file = open_journal(&self.directory, false)?;
        let state = self.read(&mut file)?;
        let mut totals = MonetaryTotals::default();
        for (reserved, actual) in state.reservations {
            totals.reserved_microusd += reserved;
            totals.actual_microusd += actual.unwrap_or(0);
            if actual.is_none() {
                totals.unsettled_microusd += reserved;
            }
        }
        Ok(totals)
    }

    fn reserve(&self, capability: &MonetaryCapability) -> Result<Reservation> {
        ensure!(
            capability.matches_budget(self),
            "Monetary capability/account mismatch"
        );
        let mut ceiling = self.ceiling_microusd;
        if let Some(parent) = current() {
            ensure!(
                self.same_account(&parent),
                "Cannot replace enclosing monetary account"
            );
            ceiling = ceiling.min(parent.ceiling_microusd);
        }
        let mut file = open_journal(&self.directory, false)?;
        let state = self.read(&mut file)?;
        let reserved: u64 = state.reservations.iter().map(|(cost, _)| cost).sum();
        let next = reserved
            .checked_add(capability.max_charge_microusd())
            .context("Monetary reservation overflow")?;
        ensure!(
            next <= ceiling.min(state.ceiling),
            "Monetary budget exhausted"
        );
        let index = state.reservations.len();
        append(
            &mut file,
            &Entry::Reserve {
                ceiling: capability.max_charge_microusd(),
            },
        )?;
        Ok(Reservation {
            budget: self.clone(),
            index,
            ceiling: capability.max_charge_microusd(),
        })
    }
}

struct Reservation {
    budget: MonetaryBudget,
    index: usize,
    ceiling: u64,
}
impl Reservation {
    fn settle(self, actual: u64) -> Result<()> {
        ensure!(
            actual <= self.ceiling,
            "Actual charge exceeds authoritative reservation"
        );
        let mut file = open_journal(&self.budget.directory, false)?;
        let state = self.budget.read(&mut file)?;
        ensure!(
            state.reservations.get(self.index) == Some(&(self.ceiling, None)),
            "Reservation already settled or missing"
        );
        append(
            &mut file,
            &Entry::Actual {
                reservation: self.index,
                charge: actual,
            },
        )
    }
}

/// Not deserializable or user-constructible. Only an audited adapter may mint
/// this promise and must enforce its request bound before every transport send.
#[derive(Clone, Debug)]
pub struct MonetaryCapability {
    budget: MonetaryBudget,
}
impl MonetaryCapability {
    pub fn matches_budget(&self, budget: &MonetaryBudget) -> bool {
        self.budget.same_account(budget)
    }
    pub fn route(&self) -> &str {
        "offline-bounded"
    }
    pub fn model(&self) -> &str {
        "synthetic-fixed"
    }
    pub fn max_request_bytes(&self) -> usize {
        65_536
    }
    pub fn max_response_bytes(&self) -> usize {
        128
    }
    /// Synthetic units only, not an upstream or real-money pricing guarantee.
    pub fn max_charge_microusd(&self) -> u64 {
        1
    }
}

tokio::task_local! { static CURRENT: MonetaryBudget; }
fn current() -> Option<MonetaryBudget> {
    CURRENT.try_with(Clone::clone).ok()
}

pub async fn scope<F, T>(budget: MonetaryBudget, future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let budget = if let Some(parent) = current() {
        ensure!(
            budget.same_account(&parent),
            "Cannot replace enclosing monetary account"
        );
        budget.with_ceiling(parent.ceiling_microusd)?
    } else {
        budget
    };
    CURRENT.scope(budget, future).await
}

pub(crate) fn inherit<F: Future>(future: F) -> impl Future<Output = F::Output> {
    let budget = current();
    async move {
        if let Some(budget) = budget {
            CURRENT.scope(budget, future).await
        } else {
            future.await
        }
    }
}

/// Reject live adapters that have no audited monetary transport contract.
pub fn reject_unsupported() -> Result<()> {
    ensure!(
        current().is_none(),
        "budget-unverifiable: provider lacks authoritative pre-send monetary bound"
    );
    Ok(())
}

/// Explicit offline fixture, never registered as a selectable live provider.
/// Owns its account so bare task spawns cannot lose the reservation boundary.
#[derive(Clone)]
pub struct OfflineBoundedProvider {
    budget: MonetaryBudget,
}
impl OfflineBoundedProvider {
    pub fn new(budget: MonetaryBudget) -> Self {
        Self { budget }
    }
}

#[async_trait::async_trait]
impl crate::Provider for OfflineBoundedProvider {
    fn fork(&self) -> std::sync::Arc<dyn crate::Provider> {
        std::sync::Arc::new(self.clone())
    }
    fn name(&self) -> &str {
        "offline-bounded"
    }
    fn model(&self) -> String {
        "synthetic-fixed".into()
    }
    fn monetary_capability(&self) -> Option<MonetaryCapability> {
        Some(MonetaryCapability {
            budget: self.budget.clone(),
        })
    }
    async fn complete(
        &self,
        messages: &[jcode_message_types::Message],
        tools: &[jcode_message_types::ToolDefinition],
        system: &str,
        resume_session_id: Option<&str>,
    ) -> Result<crate::EventStream> {
        let capability = self.monetary_capability().unwrap();
        let request = serde_json::to_vec(&(
            capability.route(),
            capability.model(),
            messages,
            tools,
            system,
            resume_session_id,
        ))?;
        ensure!(
            request.len() <= capability.max_request_bytes(),
            "Offline request exceeds attested byte ceiling"
        );
        crate::provider_call_budget::record_bounded_call()?;
        let reservation = self.budget.reserve(&capability)?;
        let response = "offline bounded response";
        ensure!(
            response.len() <= capability.max_response_bytes(),
            "Offline response exceeds attested byte ceiling"
        );
        reservation.settle(1)?;
        Ok(Box::pin(futures::stream::iter(vec![
            Ok(jcode_message_types::StreamEvent::TextDelta(response.into())),
            Ok(jcode_message_types::StreamEvent::MessageEnd {
                stop_reason: Some("end_turn".into()),
            }),
        ])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Provider;
    use futures::StreamExt;

    fn directory() -> PathBuf {
        std::env::temp_dir().join(format!(
            "jcode-money-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn offline_completion_reserves_before_send_and_cannot_refund_or_widen() {
        let path = directory();
        let budget = MonetaryBudget::create(path.clone(), "task-a".into(), 2).unwrap();
        let provider = OfflineBoundedProvider::new(budget.clone());
        assert!(
            provider
                .monetary_capability()
                .unwrap()
                .matches_budget(&budget)
        );
        let mut stream = provider.complete(&[], &[], "", None).await.unwrap();
        assert!(matches!(
            stream.next().await.unwrap().unwrap(),
            jcode_message_types::StreamEvent::TextDelta(_)
        ));
        assert_eq!(budget.totals().unwrap().reserved_microusd, 1);
        assert_eq!(budget.totals().unwrap().actual_microusd, 1);
        let reopened = MonetaryBudget::open(path.clone(), "task-a", 2).unwrap();
        let narrowed = reopened.with_ceiling(1).unwrap();
        assert!(
            OfflineBoundedProvider::new(narrowed)
                .complete(&[], &[], "", None)
                .await
                .is_err()
        );
        let reopened = MonetaryBudget::open(path.clone(), "task-a", 2).unwrap();
        assert_eq!(reopened.ceiling_microusd(), 1);
        assert!(
            OfflineBoundedProvider::new(reopened)
                .complete(&[], &[], "", None)
                .await
                .is_err()
        );
        assert!(MonetaryBudget::open(path.clone(), "task-a", 99).is_err());
        assert!(MonetaryBudget::open(path.clone(), "other-task", 2).is_err());
        assert!(MonetaryBudget::create(path.clone(), "task-a".into(), 99).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[tokio::test]
    async fn retries_children_side_calls_and_bare_spawn_share_one_account() {
        let path = directory();
        let budget = MonetaryBudget::create(path.clone(), "task".into(), 3).unwrap();
        let provider = OfflineBoundedProvider::new(budget.clone());
        scope(budget.clone(), async {
            assert!(crate::provider_call_budget::record_call().is_err());
            assert!(crate::provider_call_budget::reject_unmetered("opaque").is_err());
            crate::provider_call_budget::spawn(async {
                assert!(crate::provider_call_budget::record_call().is_err());
                Ok::<_, anyhow::Error>(())
            })
            .await??;
            provider
                .complete(&[], &[], "foreground", None)
                .await?
                .collect::<Vec<_>>()
                .await;
            provider
                .fork()
                .complete(&[], &[], "compaction", None)
                .await?
                .collect::<Vec<_>>()
                .await;
            let child = provider.clone();
            tokio::spawn(async move { child.complete(&[], &[], "retry", None).await })
                .await??
                .collect::<Vec<_>>()
                .await;
            assert!(provider.complete(&[], &[], "followup", None).await.is_err());
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(
            budget.totals().unwrap(),
            MonetaryTotals {
                reserved_microusd: 3,
                actual_microusd: 3,
                unsettled_microusd: 0
            }
        );
        std::fs::remove_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn request_bounds_and_nested_account_substitution_fail_closed() {
        let path = directory();
        let other = directory();
        let budget = MonetaryBudget::create(path.clone(), "task".into(), 2).unwrap();
        let alternate = MonetaryBudget::create(other.clone(), "alternate".into(), 100).unwrap();
        let provider = OfflineBoundedProvider::new(budget.clone());
        assert!(
            provider
                .complete(&[], &[], &"x".repeat(65_537), None)
                .await
                .is_err()
        );
        assert_eq!(budget.totals().unwrap().reserved_microusd, 0);
        scope(budget.clone(), async {
            assert!(scope(alternate.clone(), async { Ok(()) }).await.is_err());
            assert!(
                OfflineBoundedProvider::new(alternate)
                    .complete(&[], &[], "", None)
                    .await
                    .is_err()
            );
            Ok(())
        })
        .await
        .unwrap();
        std::fs::remove_dir_all(path).unwrap();
        std::fs::remove_dir_all(other).unwrap();
    }

    #[test]
    fn crash_reservations_and_invalid_settlement_never_refund() {
        let path = directory();
        let budget = MonetaryBudget::create(path.clone(), "task".into(), 2).unwrap();
        let capability = OfflineBoundedProvider::new(budget.clone())
            .monetary_capability()
            .unwrap();
        drop(budget.reserve(&capability).unwrap()); // crash/cancel before settlement
        assert!(budget.reserve(&capability).unwrap().settle(2).is_err());
        assert_eq!(
            budget.totals().unwrap(),
            MonetaryTotals {
                reserved_microusd: 2,
                actual_microusd: 0,
                unsettled_microusd: 2
            }
        );
        let reopened = MonetaryBudget::open(path.clone(), "task", 2).unwrap();
        assert!(reopened.reserve(&capability).is_err());
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn monetary_budget_child_process() {
        let Some(path) = std::env::var_os("JCODE_TEST_MONEY_ACCOUNT") else {
            return;
        };
        let budget = MonetaryBudget::open(path.into(), "task", 8).unwrap();
        let capability = OfflineBoundedProvider::new(budget.clone())
            .monetary_capability()
            .unwrap();
        while let Ok(reservation) = budget.reserve(&capability) {
            reservation.settle(1).unwrap();
        }
    }

    #[test]
    fn concurrent_processes_never_overspend_original_account() {
        let path = directory();
        let budget = MonetaryBudget::create(path.clone(), "task".into(), 8).unwrap();
        let mut children: Vec<_> = (0..6)
            .map(|_| {
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "provider_monetary_budget::tests::monetary_budget_child_process",
                    ])
                    .env("JCODE_TEST_MONEY_ACCOUNT", &path)
                    .spawn()
                    .unwrap()
            })
            .collect();
        for child in &mut children {
            assert!(child.wait().unwrap().success());
        }
        assert_eq!(
            budget.totals().unwrap(),
            MonetaryTotals {
                reserved_microusd: 8,
                actual_microusd: 8,
                unsettled_microusd: 0
            }
        );
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn incomplete_corrupt_symlinked_or_overflow_journal_fails_closed() {
        for suffix in [
            "broken",
            "{}\n",
            "{\"kind\":\"Reserve\",\"ceiling\":18446744073709551615}\n{\"kind\":\"Reserve\",\"ceiling\":1}\n",
        ] {
            let path = directory();
            let budget = MonetaryBudget::create(path.clone(), "task".into(), u64::MAX).unwrap();
            let journal = path.join("reservations.jsonl");
            let mut file = OpenOptions::new().append(true).open(&journal).unwrap();
            file.write_all(suffix.as_bytes()).unwrap();
            let before = std::fs::read(&journal).unwrap();
            assert!(budget.totals().is_err());
            assert!(MonetaryBudget::open(path.clone(), "task", u64::MAX).is_err());
            assert_eq!(std::fs::read(&journal).unwrap(), before);
            std::fs::remove_dir_all(path).unwrap();
        }
        #[cfg(unix)]
        {
            let path = directory();
            MonetaryBudget::create(path.clone(), "task".into(), 2).unwrap();
            let alias = directory();
            std::os::unix::fs::symlink(&path, &alias).unwrap();
            assert!(MonetaryBudget::open(alias.clone(), "task", 2).is_err());
            std::fs::remove_file(alias).unwrap();
            std::fs::rename(path.join("reservations.jsonl"), path.join("original")).unwrap();
            std::os::unix::fs::symlink(path.join("original"), path.join("reservations.jsonl"))
                .unwrap();
            assert!(MonetaryBudget::open(path.clone(), "task", 2).is_err());
            std::fs::remove_dir_all(path).unwrap();
        }
    }
    #[tokio::test]
    async fn unsupported_shared_transport_fails_before_tcp_connect() {
        let path = directory();
        let budget = MonetaryBudget::create(path.clone(), "task".into(), 2).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let result = scope(
            budget.clone(),
            crate::transport::send_with_initial_response_timeout(
                reqwest::Client::new()
                    .post(format!("http://{address}/completions"))
                    .body("{}"),
                std::time::Duration::from_secs(1),
            ),
        )
        .await;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("budget-unverifiable")
        );
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), listener.accept())
                .await
                .is_err()
        );
        assert_eq!(budget.totals().unwrap().reserved_microusd, 0);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[tokio::test]
    async fn offline_adapter_preserves_existing_council_call_limit() {
        let path = directory();
        let budget = MonetaryBudget::create(path.clone(), "task".into(), 10).unwrap();
        let calls_path = directory();
        let calls =
            crate::provider_call_budget::ProviderCallBudget::create(calls_path.clone(), 1).unwrap();
        let provider = OfflineBoundedProvider::new(budget.clone());
        crate::provider_call_budget::scope(Some(calls.clone()), async {
            provider
                .complete(&[], &[], "", None)
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await;
            assert!(provider.complete(&[], &[], "", None).await.is_err());
        })
        .await;
        assert_eq!(calls.used().unwrap(), 1);
        assert_eq!(budget.totals().unwrap().reserved_microusd, 1);
        std::fs::remove_dir_all(path).unwrap();
        std::fs::remove_dir_all(calls_path).unwrap();
    }

    #[tokio::test]
    async fn aborted_operation_retains_synced_reservation() {
        let path = directory();
        let budget = MonetaryBudget::create(path.clone(), "task".into(), 1).unwrap();
        let copy = budget.clone();
        let (started, ready) = tokio::sync::oneshot::channel();
        let worker = tokio::spawn(async move {
            let capability = OfflineBoundedProvider::new(copy.clone())
                .monetary_capability()
                .unwrap();
            let _reservation = copy.reserve(&capability).unwrap();
            started.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        ready.await.unwrap();
        worker.abort();
        assert!(worker.await.unwrap_err().is_cancelled());
        let reopened = MonetaryBudget::open(path.clone(), "task", 1).unwrap();
        assert_eq!(
            reopened.totals().unwrap(),
            MonetaryTotals {
                reserved_microusd: 1,
                actual_microusd: 0,
                unsettled_microusd: 1
            }
        );
        assert!(
            OfflineBoundedProvider::new(reopened)
                .complete(&[], &[], "", None)
                .await
                .is_err()
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}
