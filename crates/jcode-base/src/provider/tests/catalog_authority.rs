//! `MultiProvider::model_list_is_authoritative` follows the active slot, so a
//! cold catalog on any dynamic-catalog route is reported (and can be refreshed)
//! instead of being mistaken for the account's full model list.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// A catalog-backed slot: its list is a placeholder until `loaded` is set.
#[derive(Clone)]
struct ColdCatalogRuntime {
    name: &'static str,
    loaded: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl Provider for ColdCatalogRuntime {
    async fn complete(
        &self,
        _: &[Message],
        _: &[ToolDefinition],
        _: &str,
        _: Option<&str>,
    ) -> Result<EventStream> {
        unreachable!("authority probe never completes a prompt")
    }
    fn name(&self) -> &str {
        self.name
    }
    fn model(&self) -> String {
        "placeholder".into()
    }
    fn available_models_for_switching(&self) -> Vec<String> {
        if self.loaded.load(Ordering::SeqCst) {
            vec!["a".into(), "b".into()]
        } else {
            vec!["placeholder".into()]
        }
    }
    fn model_list_is_authoritative(&self) -> bool {
        self.loaded.load(Ordering::SeqCst)
    }
    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(self.clone())
    }
}

fn multi_with(active: ActiveProvider, runtime: Arc<dyn Provider>) -> MultiProvider {
    let slot = |wanted: ActiveProvider| RwLock::new((active == wanted).then(|| runtime.clone()));
    MultiProvider {
        anthropic: slot(ActiveProvider::Claude),
        openai: slot(ActiveProvider::OpenAI),
        copilot_api: slot(ActiveProvider::Copilot),
        antigravity: slot(ActiveProvider::Antigravity),
        gemini: slot(ActiveProvider::Gemini),
        cursor: slot(ActiveProvider::Cursor),
        // Bedrock's slot holds the concrete provider type, so it cannot take a stub.
        bedrock: RwLock::new(None),
        openrouter: RwLock::new(None),
        openai_compatible_profiles: RwLock::new(std::collections::HashMap::new()),
        active_openai_compatible_profile: RwLock::new(None),
        session_id: RwLock::new(None),
        input_compression_profile: RwLock::new(None),
        active: RwLock::new(active),
        startup_notices: RwLock::new(Vec::new()),
        initial_provider: None,
        catalog_scope: next_catalog_scope(),
        routes_memo: std::sync::Mutex::new(None),
        post_auth_refreshes_pending: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    }
}

#[test]
fn cold_catalog_is_reported_for_every_dynamic_route_and_clears_when_loaded() {
    let routes = [
        ("openai", ActiveProvider::OpenAI),
        ("gemini", ActiveProvider::Gemini),
        ("anthropic", ActiveProvider::Claude),
        ("copilot", ActiveProvider::Copilot),
        ("antigravity", ActiveProvider::Antigravity),
        ("cursor", ActiveProvider::Cursor),
    ];
    for (name, active) in routes {
        let loaded = Arc::new(AtomicBool::new(false));
        let runtime: Arc<dyn Provider> = Arc::new(ColdCatalogRuntime {
            name,
            loaded: loaded.clone(),
        });
        let multi = multi_with(active, runtime);
        assert!(
            !multi.model_list_is_authoritative(),
            "{name}: a cold catalog must not look authoritative through MultiProvider"
        );
        loaded.store(true, Ordering::SeqCst);
        assert!(
            multi.model_list_is_authoritative(),
            "{name}: loaded catalog is authoritative"
        );
    }
}

#[test]
fn static_list_slot_stays_authoritative() {
    // The default trait answer (static lists) is `true` through the wrapper.
    let multi = test_multi_provider_with_cursor();
    assert!(multi.model_list_is_authoritative());
}
