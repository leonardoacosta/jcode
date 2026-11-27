# crates/jcode-tui-render/tests/swarm_gallery_fuzz_audit.rs

- member · function · L12-L32 — fn member(id: &str, status: &str, role: Option<&str>, body: &[&str]) -> GalleryMember
- plain · function · L34-L36 — fn plain(line: &Line<'_>) -> String
- hints · function · L38-L49 — fn hints() -> Vec<SwarmStripHint>
- member_sets · function · L51-L97 — fn member_sets() -> Vec<Vec<GalleryMember>>
- gallery_never_panics_and_stays_width_bounded · function · L100-L121 — fn gallery_never_panics_and_stays_width_bounded()
- panel_never_panics_across_degenerate_inputs · function · L124-L136 — fn panel_never_panics_across_degenerate_inputs()
- panel_lines_respect_width_bound · function · L139-L167 — fn panel_lines_respect_width_bound()
- strip_never_panics_and_stays_width_bounded · function · L170-L200 — fn strip_never_panics_and_stays_width_bounded()
- dock_never_panics_and_respects_width_and_height_bounds · function · L203-L240 — fn dock_never_panics_and_respects_width_and_height_bounds()
