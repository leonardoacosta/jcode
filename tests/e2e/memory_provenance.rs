use crate::test_support::*;
use jcode::memory::{MemoryCategory, MemoryEntry, MemoryManager, TrustLevel};

/// A recall block injected into a real agent turn must be attributable: the
/// memory-events log carries each item's id/source/trust/updated_at and the
/// number of entries dropped by the item cap.
#[tokio::test]
async fn injected_memory_logs_provenance_and_dropped_count() -> Result<()> {
    let env = setup_test_env()?;
    let home = std::path::PathBuf::from(std::env::var_os("JCODE_HOME").unwrap());

    let provider = MockProvider::new();
    provider.queue_response(vec![
        StreamEvent::TextDelta("ok".to_string()),
        StreamEvent::MessageEnd {
            stop_reason: Some("end_turn".to_string()),
        },
    ]);
    let provider: Arc<dyn Provider> = Arc::new(provider);
    let registry = Registry::new(provider.clone()).await;
    let mut agent = Agent::new(provider, registry);
    agent.set_memory_enabled(true);
    let session_id = agent.session_id().to_string();
    let project = home.join("project");
    std::fs::create_dir_all(&project)?;
    let project = project.to_string_lossy().into_owned();
    agent.set_working_dir(&project);

    let manager = MemoryManager::new();
    let mut entries = Vec::new();
    for i in 0..10 {
        let entry = MemoryEntry::new(MemoryCategory::Fact, format!("e2e provenance fact {i}"))
            .with_source("e2e-source")
            .with_trust(TrustLevel::High);
        manager.remember_global(entry.clone())?;
        entries.push(entry);
    }
    let prompt = jcode::memory::format_relevant_prompt(&entries, entries.len())
        .context("canonical memory prompt")?;
    jcode::memory::set_pending_memory_for_project(
        &session_id,
        prompt,
        entries.len(),
        entries.iter().map(|e| e.id.clone()).collect(),
        None,
        Some(&project),
    );
    assert!(jcode::memory::has_pending_memory(&session_id));

    agent.run_once("what do you remember?").await?;

    let logs = home.join("logs");
    let mut injected = None;
    for file in std::fs::read_dir(&logs)?.flatten() {
        let name = file.file_name().to_string_lossy().into_owned();
        if !(name.starts_with("memory-events-") && name.ends_with(".jsonl")) {
            continue;
        }
        for line in std::fs::read_to_string(file.path())?.lines() {
            let value: serde_json::Value = serde_json::from_str(line)?;
            if value["event"] == "memory_injected" {
                injected = Some(value["detail"].clone());
            }
        }
    }
    let detail = injected.context("no memory_injected event in the private JCODE_HOME log")?;
    assert_eq!(
        detail["dropped"], 2,
        "10 eligible entries, cap of 8: {detail}"
    );
    let items = detail["items"].as_array().context("items array")?;
    assert_eq!(items.len(), 8, "{detail}");
    for item in items {
        let id = item["id"].as_str().context("item id")?;
        assert!(entries.iter().any(|e| e.id == id), "unknown id {id}");
        assert_eq!(item["source"], "e2e-source");
        assert_eq!(item["trust"], "high");
        assert!(item["updated_at"].is_string(), "{item}");
        assert!(
            !item["content"].as_str().unwrap_or("").contains("[mem "),
            "short-id marker leaked into content: {item}"
        );
    }
    drop(env);
    Ok(())
}
