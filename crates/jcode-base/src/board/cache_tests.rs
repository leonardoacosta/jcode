use super::*;
use crate::board::test_scope;
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};

fn snap(tag: &str) -> BoardSnapshot {
    BoardSnapshot {
        states: vec![WorkflowState {
            id: tag.into(),
            name: tag.into(),
            position: 0.0,
        }],
        issues: vec![],
    }
}

fn key() -> CacheKey {
    CacheKey::new(&test_scope(BoardBackend::Linear), "auth-1")
}

#[test]
fn hit_only_for_exact_key_each_component_isolated() {
    let cache = BoardCache::new(Duration::from_secs(60));
    cache.put(key(), snap("base"));
    assert!(cache.get(&key()).is_some());

    let variants: Vec<(&str, CacheKey)> = vec![
        (
            "world",
            CacheKey {
                world: "other".into(),
                ..key()
            },
        ),
        (
            "backend",
            CacheKey {
                backend: BoardBackend::Ado,
                ..key()
            },
        ),
        (
            "auth",
            CacheKey {
                auth_identity: "auth-2".into(),
                ..key()
            },
        ),
        (
            "team",
            CacheKey {
                team: "team-2".into(),
                ..key()
            },
        ),
        (
            "project",
            CacheKey {
                project: Some("p".into()),
                ..key()
            },
        ),
    ];
    for (name, variant) in &variants {
        assert!(cache.get(variant).is_none(), "{name} leaked");
    }
    // A project-less entry is not a hit for a project key and vice versa.
    cache.put(variants[4].1.clone(), snap("proj"));
    assert_eq!(cache.get(&key()).unwrap().states[0].id, "base");
    assert_eq!(cache.get(&variants[4].1).unwrap().states[0].id, "proj");
}

#[test]
fn entries_expire_after_ttl() {
    let cache = BoardCache::new(Duration::from_millis(20));
    cache.put(key(), snap("a"));
    std::thread::sleep(Duration::from_millis(40));
    assert!(cache.get(&key()).is_none());
}

#[test]
fn auth_identity_is_secret_free_and_distinguishes_connections() {
    use crate::mcp::McpServerConfig;
    let cfg = |url: &str| -> McpServerConfig {
        serde_json::from_value(serde_json::json!({ "url": url })).unwrap()
    };
    let a = cfg("https://user:hunter2@mcp.example/mcp");
    let b = cfg("https://mcp.example/other");
    let ia = auth_identity("Linear", &a);
    assert!(!ia.contains("hunter2"));
    assert_ne!(ia, auth_identity("Linear", &b));
    assert_ne!(ia, auth_identity("Other", &a));
    assert_eq!(ia, auth_identity("Linear", &a));
}

#[derive(Default)]
struct Counting {
    reads: AtomicUsize,
    fail: Mutex<Option<BoardError>>,
}

#[async_trait]
impl BoardProvider for Counting {
    async fn workflow_states(&self, scope: &BoardScope) -> Result<Vec<WorkflowState>, BoardError> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if let Some(error) = self.fail.lock().unwrap().clone() {
            return Err(error);
        }
        Ok(snap(&scope.team).states)
    }
    async fn issues(&self, _: &BoardScope) -> Result<Vec<BoardIssue>, BoardError> {
        Ok(vec![])
    }
    async fn issue_state(&self, _: &BoardScope, _: &str) -> Result<String, BoardError> {
        unreachable!()
    }
    async fn set_issue_state(&self, _: &BoardScope, _: &str, _: &str) -> Result<(), BoardError> {
        unreachable!()
    }
}

#[tokio::test]
async fn load_caches_per_key_and_refresh_bypasses() {
    let cache = BoardCache::new(Duration::from_secs(60));
    let provider = Counting::default();
    let scope = test_scope(BoardBackend::Linear);
    cache.load(&scope, "a", &provider, false).await.unwrap();
    cache.load(&scope, "a", &provider, false).await.unwrap();
    assert_eq!(provider.reads.load(Ordering::SeqCst), 1);
    cache.load(&scope, "a", &provider, true).await.unwrap();
    assert_eq!(provider.reads.load(Ordering::SeqCst), 2);
    // Another auth identity or team re-reads instead of reusing.
    cache.load(&scope, "b", &provider, false).await.unwrap();
    let other_team = BoardScope {
        team: "team-2".into(),
        ..scope.clone()
    };
    let snapshot = cache
        .load(&other_team, "a", &provider, false)
        .await
        .unwrap();
    assert_eq!(snapshot.states[0].id, "team-2");
    assert_eq!(provider.reads.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn errors_are_not_masked_by_stale_or_foreign_data() {
    let cache = BoardCache::new(Duration::from_secs(60));
    let provider = Counting::default();
    let scope = test_scope(BoardBackend::Linear);
    let other = BoardScope {
        team: "team-2".into(),
        ..scope.clone()
    };
    cache.load(&other, "a", &provider, false).await.unwrap();
    cache.load(&scope, "a", &provider, false).await.unwrap();

    for error in [
        BoardError::Unauthorized("401".into()),
        BoardError::Provider("boom".into()),
        BoardError::Unavailable("down".into()),
    ] {
        *provider.fail.lock().unwrap() = Some(error.clone());
        // Forced refresh fails: same error back, same-key entry dropped.
        assert_eq!(
            cache.load(&scope, "a", &provider, true).await.unwrap_err(),
            error
        );
        assert!(cache.get(&CacheKey::new(&scope, "a")).is_none());
        // Without refresh there is nothing to serve, so it errors too.
        assert_eq!(
            cache.load(&scope, "a", &provider, false).await.unwrap_err(),
            error
        );
        // The other binding's entry is untouched but never offered here.
        assert!(cache.get(&CacheKey::new(&other, "a")).is_some());
        *provider.fail.lock().unwrap() = None;
        cache.load(&scope, "a", &provider, false).await.unwrap();
    }
}
