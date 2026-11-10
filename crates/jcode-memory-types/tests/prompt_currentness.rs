use jcode_memory_types::{
    MemoryCategory, MemoryEntry, format_entries_for_prompt, format_relevant_display_prompt,
    format_relevant_prompt,
};

#[test]
fn store_filters_superseded_before_top_k_limit() {
    let mut stale = MemoryEntry::new(MemoryCategory::Fact, "obsolete value");
    stale.superseded_by = Some("replacement".into());
    stale.access_count = 100;
    let current = MemoryEntry::new(MemoryCategory::Fact, "current value");
    let mut store = jcode_memory_types::MemoryStore::new();
    store.add(stale);
    store.add(current);
    assert_eq!(store.get_relevant(1)[0].content, "current value");
    assert!(store.format_for_prompt(1).unwrap().contains("current value"));
}

#[test]
fn prompt_formatters_exclude_active_but_superseded_legacy_entries() {
    let mut superseded = MemoryEntry::new(MemoryCategory::Fact, "obsolete value");
    superseded.superseded_by = Some("replacement".into());
    assert!(
        superseded.active,
        "legacy inconsistent state is intentional"
    );
    let mut inactive = MemoryEntry::new(MemoryCategory::Fact, "inactive value");
    inactive.active = false;
    let current = MemoryEntry::new(MemoryCategory::Fact, "current value");
    let entries = [superseded, inactive, current];
    for format in [
        format_entries_for_prompt,
        format_relevant_prompt,
        format_relevant_display_prompt,
    ] {
        let prompt = format(&entries, 1)
            .expect("current item must not lose the item budget to a stale item");
        assert!(prompt.contains("current value"), "{prompt}");
        assert!(!prompt.contains("obsolete value"), "{prompt}");
        assert!(!prompt.contains("inactive value"), "{prompt}");
        assert!(format(&entries, 0).is_none());
        assert!(format(&entries[..2], 10).is_none());
    }
}
