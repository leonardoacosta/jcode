//! Purge and retention tests. Every test runs against a private temporary
//! `JCODE_HOME`; none touches a real memory store.

use super::*;
use crate::memory::tests::with_temp_home;
use crate::memory_types::MemoryCategory;
use jcode_config_types::MemoryRetentionConfig;

fn entry(text: &str, days_old: i64, active: bool) -> MemoryEntry {
    let mut e = MemoryEntry::new(MemoryCategory::Fact, text);
    e.updated_at = Utc::now() - Duration::days(days_old);
    e.created_at = e.updated_at;
    e.active = active;
    if !active {
        e.superseded_by = Some("newer".into());
    }
    e
}

fn seed(manager: &MemoryManager) -> Vec<String> {
    let mut ids = Vec::new();
    for (text, age, active) in [
        ("active fresh", 1, true),
        ("active old", 400, true),
        ("inactive fresh", 2, false),
        ("inactive old", 120, false),
        ("inactive older", 365, false),
    ] {
        ids.push(
            manager
                .upsert_global_memory(entry(text, age, active))
                .unwrap(),
        );
    }
    ids
}

fn texts(manager: &MemoryManager) -> Vec<String> {
    let mut t: Vec<String> = manager
        .list_all()
        .unwrap()
        .into_iter()
        .map(|e| e.content)
        .collect();
    t.sort();
    t
}

fn filter(inactive: bool, days: Option<u64>) -> PurgeFilter {
    PurgeFilter {
        inactive_only: inactive,
        older_than_days: days,
        ..PurgeFilter::new(MemoryScope::All)
    }
}

#[test]
fn dry_run_lists_matches_and_deletes_nothing() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        let before = texts(&manager);
        let hits = manager.purge_candidates(&filter(true, Some(30))).unwrap();
        let mut names: Vec<_> = hits.iter().map(|h| h.preview.as_str()).collect();
        names.sort();
        assert_eq!(names, ["inactive old", "inactive older"]);
        assert_eq!(texts(&manager), before, "a dry run changes nothing");
        assert!(manager.purge_candidates(&filter(true, None)).unwrap().len() == 3);
    });
}

#[test]
fn apply_removes_only_matches_and_keeps_active_entries() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        let report = manager.purge_apply(&filter(true, Some(30))).unwrap();
        assert_eq!(report.removed, 2);
        assert_eq!(
            texts(&manager),
            ["active fresh", "active old", "inactive fresh"],
            "active entries, and inactive ones under the age, stay"
        );
    });
}

#[test]
fn age_alone_without_inactive_may_remove_active_entries_only_when_asked() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        let report = manager.purge_apply(&filter(false, Some(300))).unwrap();
        assert_eq!(report.removed, 2, "active old and inactive older");
        assert!(texts(&manager).contains(&"active fresh".to_string()));
    });
}

#[test]
fn a_purge_with_no_filter_is_refused() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        let error = manager.purge_apply(&filter(false, None)).unwrap_err();
        assert!(error.to_string().contains("without a filter"), "{error}");
        assert_eq!(texts(&manager).len(), 5);
    });
}

#[test]
fn scope_limits_the_purge() {
    with_temp_home(|_| {
        let manager = MemoryManager::new().with_project_dir("/retention-scope-test");
        manager
            .upsert_global_memory(entry("global inactive", 90, false))
            .unwrap();
        manager
            .upsert_project_memory(entry("project inactive", 90, false))
            .unwrap();
        let mut only_project = filter(true, Some(30));
        only_project.scope = MemoryScope::Project;
        let report = manager.purge_apply(&only_project).unwrap();
        assert_eq!(report.removed, 1);
        assert_eq!(texts(&manager), ["global inactive"]);
    });
}

#[test]
fn a_stale_snapshot_cannot_resurrect_a_purged_entry() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        // A recall or save loaded the graph before the purge ...
        let stale = manager.load_global_graph().unwrap();
        // ... the purge removes the old inactive entries ...
        assert_eq!(
            manager
                .purge_apply(&filter(true, Some(30)))
                .unwrap()
                .removed,
            2
        );
        // ... and the stale writer saves its older snapshot.
        manager.save_global_graph(&stale).unwrap();
        assert_eq!(
            texts(&manager),
            ["active fresh", "active old", "inactive fresh"],
            "purged ids stay gone after a stale save"
        );
        // The next recall excludes them too.
        let recalled = manager.search("inactive").unwrap();
        assert!(recalled.iter().all(|e| e.content == "inactive fresh"));
    });
}

#[test]
fn a_stale_upsert_cannot_resurrect_a_purged_entry() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        let ids = seed(&manager);
        let doomed = ids[3].clone(); // "inactive old"
        let stale_entry = manager
            .load_global_graph()
            .unwrap()
            .get_memory(&doomed)
            .cloned()
            .unwrap();
        manager.purge_apply(&filter(true, Some(30))).unwrap();
        // Someone re-saves the exact entry they read before the purge.
        manager.upsert_global_memory(stale_entry).unwrap();
        assert!(!texts(&manager).contains(&"inactive old".to_string()));
    });
}

#[test]
fn concurrent_saves_and_a_purge_never_bring_back_purged_ids() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        let snapshot = manager.load_global_graph().unwrap();
        let writers: Vec<_> = (0..4)
            .map(|_| {
                let snapshot = snapshot.clone();
                std::thread::spawn(move || {
                    let m = MemoryManager::new();
                    for _ in 0..10 {
                        let _ = m.save_global_graph(&snapshot);
                    }
                })
            })
            .collect();
        let purger = std::thread::spawn(|| {
            MemoryManager::new()
                .purge_apply(&filter(true, Some(30)))
                .unwrap()
                .removed
        });
        for w in writers {
            w.join().unwrap();
        }
        purger.join().unwrap();
        // Whatever the interleaving, a final stale save is dropped.
        manager.save_global_graph(&snapshot).unwrap();
        let left = texts(&manager);
        assert!(!left.contains(&"inactive old".to_string()), "{left:?}");
        assert!(!left.contains(&"inactive older".to_string()), "{left:?}");
        assert!(left.contains(&"active fresh".to_string()));
    });
}

#[test]
fn purge_report_never_echoes_more_than_a_bounded_one_line_preview() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        let long = format!("line one\nline two {}", "x".repeat(500));
        manager
            .upsert_global_memory(entry(&long, 90, false))
            .unwrap();
        let hit = &manager.purge_candidates(&filter(true, None)).unwrap()[0];
        assert!(!hit.preview.contains('\n'));
        assert!(
            hit.preview.chars().count() <= PREVIEW_CHARS + 1,
            "{}",
            hit.preview
        );
        assert!(hit.preview.ends_with('…'));
    });
}

#[test]
fn retention_is_off_unless_explicitly_enabled() {
    let now = Utc::now();
    let off = MemoryRetentionConfig::default();
    assert!(MemoryManager::retention_filter(&off, now).is_none());
    let enabled_only = MemoryRetentionConfig {
        enabled: true,
        ..off.clone()
    };
    assert!(MemoryManager::retention_filter(&enabled_only, now).is_none());
    let on = MemoryRetentionConfig {
        enabled: true,
        auto_purge: true,
        inactive_max_age_days: 45,
    };
    let f = MemoryManager::retention_filter(&on, now).unwrap();
    assert!(f.inactive_only, "auto purge never removes active entries");
    assert_eq!(f.older_than_days, Some(45));
}

#[test]
fn auto_purge_with_the_default_policy_deletes_nothing() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        manager.auto_purge(&MemoryRetentionConfig::default());
        assert_eq!(texts(&manager).len(), 5);
    });
}

#[test]
fn auto_purge_with_an_explicit_policy_removes_only_old_inactive_entries() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        manager.auto_purge(&MemoryRetentionConfig {
            enabled: true,
            auto_purge: true,
            inactive_max_age_days: 100,
        });
        // 100 days: "inactive old" (120) and "inactive older" (365) go; the
        // fresh inactive entry and both active entries stay.
        assert_eq!(
            texts(&manager),
            ["active fresh", "active old", "inactive fresh"]
        );
    });
}

#[test]
fn an_unreadable_tombstone_file_never_deletes_live_entries_and_purge_still_works() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        let path = manager.tombstone_path().unwrap();
        std::fs::write(&path, "{ this is not json").unwrap();
        // A save with an unreadable tombstone file keeps every entry: the
        // filter fails open for resurrection, never closed onto live data.
        let graph = manager.load_global_graph().unwrap();
        manager.save_global_graph(&graph).unwrap();
        assert_eq!(texts(&manager).len(), 5);
        // A purge refuses rather than guessing, and deletes nothing.
        let error = manager.purge_apply(&filter(true, Some(30))).unwrap_err();
        assert!(!error.to_string().is_empty());
        assert_eq!(texts(&manager).len(), 5, "no partial deletion");
    });
}

#[test]
fn a_purged_id_can_be_deliberately_re_added_with_a_new_id() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        manager.purge_apply(&filter(true, Some(30))).unwrap();
        // The same text under a fresh id is a new memory, not a resurrection.
        manager
            .remember_global(entry("inactive old", 0, true))
            .unwrap();
        assert!(texts(&manager).contains(&"inactive old".to_string()));
    });
}

fn goal_like(id: &str, text: &str, days_old: i64) -> MemoryEntry {
    let mut e = entry(text, days_old, true);
    e.id = id.to_string();
    e
}

#[test]
fn a_tombstone_in_one_graph_never_removes_the_same_id_in_another() {
    with_temp_home(|_| {
        let manager = MemoryManager::new().with_project_dir("/retention-collision");
        manager
            .upsert_project_memory(goal_like("goal:release", "project goal", 200))
            .unwrap();
        manager
            .upsert_global_memory(goal_like("goal:release", "global goal", 1))
            .unwrap();
        let mut only_project = filter(false, Some(30));
        only_project.scope = MemoryScope::Project;
        assert_eq!(manager.purge_apply(&only_project).unwrap().removed, 1);
        // The global graph is saved afterwards, as any writer would.
        let global = manager.load_global_graph().unwrap();
        manager.save_global_graph(&global).unwrap();
        manager
            .upsert_global_memory(goal_like("goal:other", "later write", 1))
            .unwrap();
        assert!(texts(&manager).contains(&"global goal".to_string()));
    });
}

#[test]
fn a_newer_write_of_a_purged_id_is_kept_but_the_purged_version_is_not() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        let old = goal_like("goal:x", "old goal text", 200);
        manager.upsert_global_memory(old.clone()).unwrap();
        manager.purge_apply(&filter(false, Some(30))).unwrap();
        // The identical old version is dropped.
        manager.upsert_global_memory(old).unwrap();
        assert!(!texts(&manager).contains(&"old goal text".to_string()));
        // A goal edited after the purge is a new version and survives.
        manager
            .upsert_global_memory(goal_like("goal:x", "edited goal text", 0))
            .unwrap();
        assert!(texts(&manager).contains(&"edited goal text".to_string()));
    });
}

#[test]
fn an_absurd_age_matches_nothing_and_never_panics() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        for days in [u64::MAX, i64::MAX as u64, 1 << 50, 200_000_000_000_000] {
            let report = manager.purge_apply(&filter(false, Some(days))).unwrap();
            assert_eq!(report.removed, 0, "days={days}");
        }
        assert_eq!(texts(&manager).len(), 5);
    });
}

#[test]
fn project_scope_without_a_project_directory_is_an_error_not_an_empty_result() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        let mut f = filter(true, Some(30));
        f.scope = MemoryScope::Project;
        assert!(manager.purge_candidates(&f).is_err());
        assert!(manager.purge_apply(&f).is_err());
    });
}

#[test]
fn the_preview_redacts_secrets_stored_before_the_gate_existed() {
    let shown = preview("deploy with OPENAI_API_KEY=sk-abcdefghijklmnopqrstuvwxyz123456 now");
    assert!(!shown.contains("abcdefghijklmnopqrstuvwxyz"), "{shown}");
}

#[test]
fn auto_purge_policy_targets_only_the_global_store() {
    let on = MemoryRetentionConfig {
        enabled: true,
        auto_purge: true,
        inactive_max_age_days: 45,
    };
    let f = MemoryManager::retention_filter(&on, Utc::now()).unwrap();
    assert_eq!(f.scope, MemoryScope::Global);
}

#[test]
fn a_corrupt_tombstone_file_keeps_entries_on_save_and_purge_names_the_file() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        std::fs::write(manager.tombstone_path().unwrap(), "{ nope").unwrap();
        let error = manager.purge_apply(&filter(true, Some(30))).unwrap_err();
        assert!(error.to_string().contains("purged-ids.json"), "{error}");
        assert_eq!(texts(&manager).len(), 5);
    });
}

#[test]
fn a_purge_is_idempotent_and_the_dry_run_leaves_every_file_byte_identical() {
    with_temp_home(|_| {
        let manager = MemoryManager::new();
        seed(&manager);
        let path = manager.global_memory_path().unwrap();
        let before = std::fs::read(&path).unwrap();
        let candidates = manager.purge_candidates(&filter(true, Some(30))).unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(!manager.tombstone_path().unwrap().exists());
    });
}
