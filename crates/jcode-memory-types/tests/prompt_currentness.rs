use jcode_memory_types::{
    MemoryCategory, MemoryEntry, format_entries_for_prompt, format_relevant_display_prompt,
    format_relevant_prompt,
};

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
