//! Bounded observation only. No access to the application or its queues.

use crate::jev::{JevClient, JevShadowAdvice, JevShadowCase, JevShadowEvaluator};
use crate::todo::ContinuationEvidence;
use crate::todo::shadow::{ShadowRecord, ShadowVerdict};
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

pub(super) static OBSERVER: OnceLock<ShadowObserver> = OnceLock::new();

pub(super) struct ShadowObserver {
    evaluator: Mutex<Option<(String, Arc<JevShadowEvaluator>)>>,
    permit: Arc<tokio::sync::Semaphore>,
    sequence: AtomicU64,
}

impl Default for ShadowObserver {
    fn default() -> Self {
        Self {
            evaluator: Mutex::new(None),
            permit: Arc::new(tokio::sync::Semaphore::new(1)),
            sequence: AtomicU64::new(0),
        }
    }
}

struct PendingRecord<F: FnOnce(ShadowRecord)> {
    record: Option<ShadowRecord>,
    sink: Option<F>,
    observation: Option<(Arc<JevShadowEvaluator>, usize, Instant)>,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl<F: FnOnce(ShadowRecord)> Drop for PendingRecord<F> {
    fn drop(&mut self) {
        if let (Some(mut record), Some(sink)) = (self.record.take(), self.sink.take()) {
            if let Some((evaluator, before, started)) = &self.observation {
                record.calls = evaluator.requests_started().saturating_sub(*before);
                record.latency_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
            }
            sink(record);
        }
    }
}

impl ShadowObserver {
    pub(super) fn submit(
        &self,
        enabled: bool,
        config: &crate::config::JevConfig,
        evidence: ContinuationEvidence,
        deterministic_queued: bool,
        session: String,
        sink: impl FnOnce(ShadowRecord) + Send + 'static,
    ) -> Option<tokio::task::JoinHandle<()>> {
        if !enabled {
            return None;
        }
        let decision_id = format!(
            "poke-{}-{}",
            std::process::id(),
            self.sequence.fetch_add(1, Ordering::Relaxed)
        );
        let mut pending = PendingRecord {
            record: Some(ShadowRecord {
                decision_id: decision_id.clone(),
                deterministic_queued,
                evidence,
                verdict: ShadowVerdict::Uncertain,
                latency_ms: 0,
                calls: 0,
            }),
            sink: Some(sink),
            observation: None,
            permit: None,
        };
        pending.permit = Some(self.permit.clone().try_acquire_owned().ok()?);
        let route = serde_json::to_string(config).ok()?;
        let evaluator = {
            let mut current = self.evaluator.lock().ok()?;
            if current.is_none() {
                let client = JevClient::from_supervision_config(config).ok()?;
                *current = Some((
                    route.clone(),
                    Arc::new(JevShadowEvaluator::new(client).ok()?),
                ));
            }
            let (bound_route, evaluator) = current.as_ref()?;
            // Route changes require a process restart. Never reset the run cap.
            if *bound_route != route {
                return None;
            }
            evaluator.clone()
        };
        let runtime = tokio::runtime::Handle::try_current().ok()?;
        Some(runtime.spawn(async move {
            crate::logging::set_session(&session);
            let started = Instant::now();
            pending.observation = Some((evaluator.clone(), evaluator.requests_started(), started));
            let record = pending.record.as_mut().expect("pending record");
            let before = evaluator.requests_started();
            let case = JevShadowCase {
                case_id: decision_id.clone(), state_revision: Some(decision_id),
                scope_ref: Some("redacted-continuation-observation".into()),
                check_summary: (record.evidence.required_checks_available && record.evidence.required_checks_complete).then(|| serde_json::json!(record.evidence.required_checks)),
                evidence: serde_json::json!({"deterministic_queued":record.deterministic_queued,"evidence":record.evidence}),
            };
            let result = tokio::time::timeout(Duration::from_secs(2), evaluator.observe(case)).await;
            record.calls = evaluator.requests_started().saturating_sub(before);
            record.latency_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
            if let Ok(Ok(observation)) = result {
                record.verdict = match observation.advice {
                    JevShadowAdvice::Continue => ShadowVerdict::Continue,
                    JevShadowAdvice::Stop => ShadowVerdict::Stop,
                    JevShadowAdvice::Wait | JevShadowAdvice::Abstain => ShadowVerdict::Uncertain,
                };
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::ContinuationEvidence;
    use crate::todo::shadow::ShadowVerdict;

    struct LayaEnv(Option<std::ffi::OsString>);
    impl Drop for LayaEnv {
        fn drop(&mut self) {
            match self.0.take() {
                Some(value) => crate::env::set_var("JCODE_LAYA_BASE_URL", value),
                None => crate::env::remove_var("JCODE_LAYA_BASE_URL"),
            }
        }
    }

    fn check_evidence() -> ContinuationEvidence {
        let mut evidence = ContinuationEvidence {
            required_checks_available: true,
            required_checks_complete: true,
            ..Default::default()
        };
        evidence
            .required_checks
            .push(crate::todo::evidence::RequiredCheckEvidence {
                check_id: "check_1".into(),
                result: crate::todo::evidence::CheckResult::NotRun,
            });
        evidence
    }

    #[tokio::test]
    #[expect(
        clippy::await_holding_lock,
        reason = "Process-wide environment mutations must remain serialized through async loopback requests"
    )]
    async fn semantic_shadow_loopback_verdicts_failures_caps_and_route_changes() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let _lock = crate::storage::lock_test_env();
        let _env = LayaEnv(std::env::var_os("JCODE_LAYA_BASE_URL"));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        crate::env::set_var(
            "JCODE_LAYA_BASE_URL",
            format!("http://{}", listener.local_addr().unwrap()),
        );
        let server = tokio::spawn(async move {
            let mut index = 0;
            'connections: while index < 48 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut bytes = vec![];
                loop {
                    let mut chunk = [0; 4096];
                    let size =
                        tokio::time::timeout(Duration::from_secs(3), stream.read(&mut chunk))
                            .await
                            .unwrap()
                            .unwrap();
                    if size == 0 {
                        continue 'connections;
                    }
                    bytes.extend_from_slice(&chunk[..size]);
                    assert!(bytes.len() <= 80 * 1024, "bounded fixture request");
                    if let Some(header_end) = bytes.windows(4).position(|p| p == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..header_end]);
                        // Local health probes are not evaluator requests and must not
                        // consume a response control or require a request body.
                        if !headers.starts_with("POST ") {
                            stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await.unwrap();
                            continue 'connections;
                        }
                        let chunked = headers.lines().any(|line| {
                            line.split_once(':').is_some_and(|(key, value)| {
                                key.eq_ignore_ascii_case("transfer-encoding")
                                    && value.trim().eq_ignore_ascii_case("chunked")
                            })
                        });
                        if chunked {
                            if bytes[header_end + 4..].ends_with(b"\r\n0\r\n\r\n") {
                                break;
                            }
                            continue;
                        }
                        let length: usize = headers
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(str::trim)
                                    .map(str::parse)
                            })
                            .transpose()
                            .unwrap()
                            .unwrap_or(0);
                        if bytes.len() >= header_end + 4 + length {
                            break;
                        }
                    }
                }
                let text = String::from_utf8(bytes).unwrap();
                assert!(!text.contains("PRIVATE"));
                let (status, body) = match index {
                    0 => (
                        200,
                        r#"{"answers":{"advice":{"type":"choice","choice":"continue"}}}"#,
                    ),
                    1 => (
                        200,
                        r#"{"answers":{"advice":{"type":"choice","choice":"wait"}}}"#,
                    ),
                    2 => (200, "malformed"),
                    3 => (503, "PRIVATE_TRANSPORT_ERROR"),
                    4 => {
                        tokio::time::sleep(Duration::from_millis(2100)).await;
                        (200, "{}")
                    }
                    _ => (
                        200,
                        r#"{"answers":{"advice":{"type":"choice","choice":"stop"}}}"#,
                    ),
                };
                let response = format!(
                    "HTTP/1.1 {status} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
                index += 1;
            }
        });
        // A bodyless health probe and an empty connection must not shift cases.
        drop(tokio::net::TcpStream::connect(address).await.unwrap());
        let mut probe = tokio::net::TcpStream::connect(address).await.unwrap();
        probe
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        let mut reply = Vec::new();
        probe.read_to_end(&mut reply).await.unwrap();
        assert!(reply.starts_with(b"HTTP/1.1 404"));
        let config: crate::config::JevConfig = serde_json::from_value(
            serde_json::json!({"supervision":{"provider":"laya","model":"synthetic"}}),
        )
        .unwrap();
        let observer = ShadowObserver::default();
        let mut incomplete = check_evidence();
        incomplete.required_checks_complete = false;
        let (tx, rx) = tokio::sync::oneshot::channel();
        observer
            .submit(
                true,
                &config,
                incomplete,
                true,
                "synthetic".into(),
                move |record| {
                    tx.send(record).unwrap();
                },
            )
            .unwrap()
            .await
            .unwrap();
        let record = rx.await.unwrap();
        assert_eq!(record.calls, 0);
        assert_eq!(record.verdict, ShadowVerdict::Uncertain);
        for index in 0..49 {
            let (tx, rx) = tokio::sync::oneshot::channel();
            if let Some(task) = observer.submit(
                true,
                &config,
                check_evidence(),
                index % 2 == 0,
                "synthetic".into(),
                move |record| {
                    tx.send(record).unwrap();
                },
            ) {
                task.await.unwrap();
            }
            let record = rx.await.unwrap();
            assert_eq!(record.deterministic_queued, index % 2 == 0);
            assert_eq!(record.calls, usize::from(index < 48));
            assert_eq!(
                record.verdict,
                match index {
                    0 => ShadowVerdict::Continue,
                    1..=4 | 48 => ShadowVerdict::Uncertain,
                    _ => ShadowVerdict::Stop,
                }
            );
            assert!(record.latency_ms <= 3000);
        }
        server.await.unwrap();
        let mut changed = config.clone();
        changed.supervision.as_mut().unwrap().model = Some("changed".into());
        let (tx, rx) = tokio::sync::oneshot::channel();
        assert!(
            observer
                .submit(
                    true,
                    &changed,
                    check_evidence(),
                    true,
                    "synthetic".into(),
                    move |record| {
                        tx.send(record).unwrap();
                    }
                )
                .is_none()
        );
        assert_eq!(rx.await.unwrap().calls, 0);
    }

    #[tokio::test]
    async fn semantic_shadow_disabled_never_initializes_and_unconfigured_is_uncertain() {
        let observer = ShadowObserver::default();
        assert!(
            observer
                .submit(
                    false,
                    &crate::config::JevConfig::default(),
                    ContinuationEvidence::default(),
                    true,
                    "test".into(),
                    |_| panic!()
                )
                .is_none()
        );
        assert!(observer.evaluator.lock().unwrap().is_none());
        let (tx, rx) = tokio::sync::oneshot::channel();
        observer.submit(
            true,
            &crate::config::JevConfig::default(),
            ContinuationEvidence::default(),
            true,
            "test".into(),
            |record| {
                tx.send(record).unwrap();
            },
        );
        let record = rx.await.unwrap();
        assert_eq!(record.verdict, ShadowVerdict::Uncertain);
        assert_eq!(record.calls, 0);
        assert!(record.deterministic_queued);
    }

    #[tokio::test]
    #[expect(
        clippy::await_holding_lock,
        reason = "Process-wide environment mutations must remain serialized through async cancellation"
    )]
    async fn semantic_shadow_cancellation_records_started_call_and_releases_permit() {
        let _lock = crate::storage::lock_test_env();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let prior = std::env::var_os("JCODE_LAYA_BASE_URL");
        let _restore = LayaEnv(prior);
        crate::env::set_var("JCODE_LAYA_BASE_URL", format!("http://{address}"));
        let config: crate::config::JevConfig = serde_json::from_value(
            serde_json::json!({"supervision":{"provider":"laya","model":"synthetic"}}),
        )
        .unwrap();
        let observer = ShadowObserver::default();
        let evidence = check_evidence();
        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = observer
            .submit(
                true,
                &config,
                evidence,
                false,
                "synthetic".into(),
                |record| {
                    let _ = tx.send(record);
                },
            )
            .unwrap();
        let (_stream, _) = tokio::time::timeout(Duration::from_secs(3), listener.accept())
            .await
            .unwrap()
            .unwrap();
        let (busy_tx, busy_rx) = tokio::sync::oneshot::channel();
        assert!(
            observer
                .submit(
                    true,
                    &config,
                    check_evidence(),
                    true,
                    "other-session".into(),
                    |record| {
                        busy_tx.send(record).unwrap();
                    }
                )
                .is_none()
        );
        let busy = busy_rx.await.unwrap();
        assert_eq!(busy.calls, 0);
        assert_eq!(busy.verdict, ShadowVerdict::Uncertain);
        task.abort();
        let _ = task.await;
        let record = rx.await.unwrap();
        assert_eq!(record.calls, 1);
        assert_eq!(record.verdict, ShadowVerdict::Uncertain);
        assert!(!record.deterministic_queued);
        assert_eq!(observer.permit.available_permits(), 1);
    }
}
