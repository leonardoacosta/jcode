//! Non-refundable inference permits shared by every process serving a council.
//! A create-new file is the atomic claim, so crashes, retries and seat resumes
//! cannot refund or reset a permit. Missing/corrupt state fails closed.

use anyhow::{Context, Result, bail};
use std::future::Future;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug)]
pub struct ProviderCallBudget {
    directory: PathBuf,
    limit: u32,
}

impl ProviderCallBudget {
    pub fn create(directory: PathBuf, limit: u32) -> Result<Self> {
        std::fs::create_dir_all(&directory)?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join("limit"))?;
        write!(file, "{limit}")?;
        Ok(Self { directory, limit })
    }

    pub fn open(directory: PathBuf) -> Result<Self> {
        let limit = std::fs::read_to_string(directory.join("limit"))
            .context("missing council provider-call budget")?
            .trim()
            .parse()
            .context("invalid council provider-call budget")?;
        Ok(Self { directory, limit })
    }

    pub fn used(&self) -> Result<u32> {
        let mut count = 0;
        for entry in std::fs::read_dir(&self.directory)? {
            if entry?.file_name().to_string_lossy().starts_with("call-") {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn acquire(&self) -> Result<()> {
        for index in 0..self.limit {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.directory.join(format!("call-{index}")))
            {
                Ok(_) => return Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(error).context("cannot claim council provider-call permit");
                }
            }
        }
        bail!(
            "council provider-call budget exhausted (max_calls={})",
            self.limit
        )
    }
}

#[derive(Clone)]
struct Scope {
    budget: ProviderCallBudget,
    prepaid: Arc<AtomicBool>,
}

tokio::task_local! { static CURRENT: Option<Scope>; }

fn current() -> Option<Scope> {
    CURRENT.try_with(Clone::clone).ok().flatten()
}

/// Retain the scope explicitly when spawning tool/provider worker tasks.
/// Tokio task locals are otherwise not inherited by a new task.
pub fn inherit<F: Future>(future: F) -> impl Future<Output = F::Output> {
    crate::provider_monetary_budget::inherit(CURRENT.scope(current(), future))
}

pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(inherit(future))
}

pub async fn scope<F: Future>(budget: Option<ProviderCallBudget>, future: F) -> F::Output {
    let context = budget
        .map(|budget| Scope {
            budget,
            prepaid: Arc::new(AtomicBool::new(false)),
        })
        .or_else(current);
    CURRENT.scope(context, future).await
}

/// Charge each Agent completion, including injected providers. The first
/// transport send spends this reservation, retries acquire additional permits.
/// Reservations are deliberately not refunded on preparation/cancellation errors.
pub async fn completion<F, T>(future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    if let Some(context) = current() {
        context.budget.acquire()?;
        let context = Scope {
            budget: context.budget,
            prepaid: Arc::new(AtomicBool::new(true)),
        };
        CURRENT.scope(Some(context), future).await
    } else {
        future.await
    }
}

/// Call immediately before every inference send, inside retry loops. Requests
/// outside an Agent completion (for example compaction) acquire their own permit.
pub fn record_call() -> Result<()> {
    crate::provider_monetary_budget::reject_unsupported()?;
    record_bounded_call()
}

pub(crate) fn record_bounded_call() -> Result<()> {
    if let Some(context) = current()
        && !context.prepaid.swap(false, Ordering::SeqCst)
    {
        context.budget.acquire()?;
    }
    Ok(())
}

/// Opaque CLI/browser/SDK transports cannot promise a bound on hidden retries.
pub fn reject_unmetered(provider: &str) -> Result<()> {
    crate::provider_monetary_budget::reject_unsupported()?;
    if current().is_some() {
        bail!(
            "{provider} cannot enforce the council provider-call budget; select a metered API provider"
        )
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "jcode-call-budget-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[tokio::test]
    async fn completion_retry_and_followup_share_nonrefundable_budget() {
        let directory = directory();
        let budget = ProviderCallBudget::create(directory.clone(), 3).unwrap();
        scope(Some(budget.clone()), async {
            completion(async {
                tokio::spawn(inherit(async {
                    record_call().unwrap(); // prepaid first send
                    record_call().unwrap(); // actual provider retry
                }))
                .await
                .unwrap();
                Ok(())
            })
            .await
            .unwrap();
            completion(async {
                record_call()?;
                Ok(())
            })
            .await
            .unwrap(); // tool follow-up
            assert!(
                completion(async { Ok(()) })
                    .await
                    .unwrap_err()
                    .to_string()
                    .contains("budget exhausted")
            );
            assert!(reject_unmetered("opaque").is_err());
        })
        .await;
        assert_eq!(budget.used().unwrap(), 3);
        assert!(
            ProviderCallBudget::open(directory.clone())
                .unwrap()
                .acquire()
                .is_err()
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn budget_child_process() {
        let Some(directory) = std::env::var_os("JCODE_TEST_CALL_BUDGET") else {
            return;
        };
        let budget = ProviderCallBudget::open(directory.into()).unwrap();
        while budget.acquire().is_ok() {}
    }

    #[test]
    fn concurrent_processes_cannot_overspend_or_reset() {
        let directory = directory();
        let budget = ProviderCallBudget::create(directory.clone(), 8).unwrap();
        let mut children: Vec<_> = (0..6)
            .map(|_| {
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "provider_call_budget::tests::budget_child_process",
                    ])
                    .env("JCODE_TEST_CALL_BUDGET", &directory)
                    .spawn()
                    .unwrap()
            })
            .collect();
        for child in &mut children {
            assert!(child.wait().unwrap().success());
        }
        assert_eq!(budget.used().unwrap(), 8);
        assert!(ProviderCallBudget::create(directory.clone(), 100).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn missing_or_corrupt_limit_fails_closed() {
        let directory = directory();
        assert!(ProviderCallBudget::open(directory.clone()).is_err());
        std::fs::write(directory.join("limit"), "invalid").unwrap();
        assert!(ProviderCallBudget::open(directory.clone()).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[tokio::test]
    async fn failed_or_cancelled_completion_does_not_refund() {
        let directory = directory();
        let budget = ProviderCallBudget::create(directory.clone(), 2).unwrap();
        scope(Some(budget.clone()), async {
            assert!(
                completion::<_, ()>(async { anyhow::bail!("preparation failed") })
                    .await
                    .is_err()
            );
            let (started, ready) = tokio::sync::oneshot::channel();
            let task = spawn(completion(async {
                let _ = started.send(());
                std::future::pending::<Result<()>>().await
            }));
            ready.await.unwrap();
            task.abort();
            assert!(task.await.unwrap_err().is_cancelled());
            assert!(completion(async { Ok(()) }).await.is_err());
        })
        .await;
        assert_eq!(budget.used().unwrap(), 2);
        assert!(reject_unmetered("ordinary unbudgeted call").is_ok());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
