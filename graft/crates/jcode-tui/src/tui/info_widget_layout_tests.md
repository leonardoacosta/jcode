# crates/jcode-tui/src/tui/info_widget_layout_tests.rs

- contended_data · function · L21-L118 — fn contended_data() -> InfoWidgetData
- rects_overlap · function · L120-L126 — fn rects_overlap(a: &Rect, b: &Rect) -> bool
- assert_placements_sane · function · L130-L156 — fn assert_placements_sane(label: &str, area: Rect, placements: &[WidgetPlacement])
- margins_for · function · L158-L170 — fn margins_for(width: u16, rows: usize, centered: bool) -> Margins
- degenerate_sizes_with_full_contention_never_panic_or_escape · function · L176-L217 — fn degenerate_sizes_with_full_contention_never_panic_or_escape()
- overview_suppresses_mergeable_widgets_under_contention · function · L222-L241 — fn overview_suppresses_mergeable_widgets_under_contention()
- high_usage_is_not_duplicated_when_overview_can_fit · function · L244-L300 — fn high_usage_is_not_duplicated_when_overview_can_fit()
- high_usage_stays_visible_when_overview_cannot_fit · function · L303-L320 — fn high_usage_stays_visible_when_overview_cannot_fit()
- swarm_status_widget_is_never_available · function · L324-L366 — fn swarm_status_widget_is_never_available()
- margin_profile_longer_than_area_cannot_place_below_viewport · function · L372-L388 — fn margin_profile_longer_than_area_cannot_place_below_viewport()
- stale_anchor_above_shifted_area_is_rehomed_not_drawn_out_of_bounds · function · L394-L413 — fn stale_anchor_above_shifted_area_is_rehomed_not_drawn_out_of_bounds()
- resident_overview_grows_in_place_when_its_content_grows · function · L423-L476 — fn resident_overview_grows_in_place_when_its_content_grows()
- covered_resident_rehomes_immediately_when_content_is_settled · function · L486-L536 — fn covered_resident_rehomes_immediately_when_content_is_settled()
