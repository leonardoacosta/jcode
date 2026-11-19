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
    assert!(
        store
            .format_for_prompt(1)
            .unwrap()
            .contains("current value")
    );
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

#[test]
fn prompt_includes_short_memory_id_and_forgets_resolve_it() {
    let entry = MemoryEntry::new(MemoryCategory::Fact, "uses tabs");
    let short = jcode_memory_types::short_memory_id(&entry.id);
    // The whole random u64 tail, not a truncation of it.
    assert!(short.len() >= 8 && short.bytes().all(|b| b.is_ascii_digit()));
    assert!(entry.id.ends_with(&format!("_{short}")));
    let prompt = format_relevant_prompt(std::slice::from_ref(&entry), 5).unwrap();
    assert!(
        prompt.contains(&format!("1. [mem {short}] uses tabs")),
        "{prompt}"
    );
    assert!(
        !prompt.contains(&entry.id),
        "full id must not be exposed: {prompt}"
    );
}

#[test]
fn item_cap_truncates_and_reports_dropped_count() {
    let entries: Vec<_> = (0..12)
        .map(|i| MemoryEntry::new(MemoryCategory::Fact, format!("distinct fact {i}")))
        .collect();
    let prompt = format_relevant_prompt(&entries, 100).unwrap();
    assert_eq!(
        prompt.matches("[mem ").count(),
        jcode_memory_types::MAX_INJECTED_ITEMS
    );
    assert!(prompt.contains("(4 more memories omitted)"), "{prompt}");
    // Not truncation when the caller asks for fewer or nothing.
    assert!(format_relevant_prompt(&entries, 0).is_none());
}

#[test]
fn char_cap_truncates_and_reports_dropped_count() {
    let big = "x".repeat(jcode_memory_types::MAX_INJECTED_CHARS - 10);
    let a = MemoryEntry::new(MemoryCategory::Fact, big);
    let b = MemoryEntry::new(MemoryCategory::Fact, "this one does not fit");
    let prompt = format_relevant_prompt(&[a, b], 5).unwrap();
    assert_eq!(prompt.matches("[mem ").count(), 1);
    assert!(!prompt.contains("does not fit"));
    assert!(prompt.contains("(1 more memories omitted)"), "{prompt}");
}

#[test]
fn old_injected_item_and_new_item_round_trip() {
    let old = r#"{"section":"Facts","content":"uses tabs"}"#;
    let item: jcode_memory_types::InjectedMemoryItem = serde_json::from_str(old).unwrap();
    assert!(
        item.id.is_none()
            && item.source.is_none()
            && item.trust.is_none()
            && item.updated_at.is_none()
    );
    let mut entry = MemoryEntry::new(MemoryCategory::Fact, "uses tabs");
    entry.source = Some("user".into());
    let item = jcode_memory_types::InjectedMemoryItem {
        section: "Facts".into(),
        content: "uses tabs".into(),
        id: Some(entry.id.clone()),
        source: entry.source.clone(),
        trust: Some(entry.trust),
        updated_at: Some(entry.updated_at),
    };
    let back: jcode_memory_types::InjectedMemoryItem =
        serde_json::from_str(&serde_json::to_string(&item).unwrap()).unwrap();
    assert_eq!(back.id, Some(entry.id));
    assert_eq!(back.source.as_deref(), Some("user"));
    assert_eq!(back.updated_at, Some(entry.updated_at));
}

#[test]
fn short_id_keeps_the_whole_random_tail_and_never_truncates_other_shapes() {
    assert_eq!(
        jcode_memory_types::short_memory_id("mem_1700000000000_18446744073709551615"),
        "18446744073709551615"
    );
    for odd in ["goal:ship-mobile-mvp", "mem-ephemeral", "abc", ""] {
        assert_eq!(jcode_memory_types::short_memory_id(odd), odd);
    }
}

#[test]
fn recall_tool_formatter_is_not_lowered_by_the_injection_cap() {
    // `memory recall mode=recent` defaults to limit 10: the 8-item injection cap
    // must not apply to it (and it carries no ids or omission trailer).
    let entries: Vec<_> = (0..12)
        .map(|i| MemoryEntry::new(MemoryCategory::Fact, format!("tool fact {i}")))
        .collect();
    let listed = format_entries_for_prompt(&entries, 10).unwrap();
    assert_eq!(listed.matches("tool fact").count(), 10, "{listed}");
    assert!(!listed.contains("[mem "), "{listed}");
    assert!(!listed.contains("omitted"), "{listed}");
    // ...and not the char cap either.
    let big = "y".repeat(jcode_memory_types::MAX_INJECTED_CHARS);
    let two = [
        MemoryEntry::new(MemoryCategory::Fact, big),
        MemoryEntry::new(MemoryCategory::Fact, "second"),
    ];
    let listed = format_entries_for_prompt(&two, 10).unwrap();
    assert!(
        listed.contains("second") && listed.contains("yyyy"),
        "{listed}"
    );
}

#[test]
fn structured_items_match_the_rendered_block_and_count_cap_drops() {
    let mut entries: Vec<_> = (0..10)
        .map(|i| {
            let mut e = MemoryEntry::new(MemoryCategory::Fact, format!("structured fact {i}"));
            e.source = Some("user".into());
            e.trust = jcode_memory_types::TrustLevel::High;
            e
        })
        .collect();
    // Content that imitates the omission trailer must not change the count.
    entries[0].content = "(99 more memories omitted)".into();
    let f = jcode_memory_types::format_relevant_prompt_with_items(&entries, 100).unwrap();
    assert_eq!(f.items.len(), jcode_memory_types::MAX_INJECTED_ITEMS);
    assert_eq!(f.dropped, 2);
    for item in &f.items {
        let id = item.id.as_deref().unwrap();
        let entry = entries.iter().find(|e| e.id == id).unwrap();
        assert_eq!(item.source.as_deref(), Some("user"));
        assert_eq!(item.trust, Some(jcode_memory_types::TrustLevel::High));
        assert_eq!(item.updated_at, Some(entry.updated_at));
        assert!(f.text.contains(&format!(
            "[mem {}]",
            jcode_memory_types::short_memory_id(id)
        )));
    }
}

#[test]
fn char_cap_is_reported_in_structured_dropped_count() {
    let big = "x".repeat(jcode_memory_types::MAX_INJECTED_CHARS - 10);
    let entries = [
        MemoryEntry::new(MemoryCategory::Fact, big),
        MemoryEntry::new(MemoryCategory::Fact, "does not fit"),
        MemoryEntry::new(MemoryCategory::Fact, "also does not fit"),
    ];
    let f = jcode_memory_types::format_relevant_prompt_with_items(&entries, 5).unwrap();
    assert_eq!((f.items.len(), f.dropped), (1, 2));
}
