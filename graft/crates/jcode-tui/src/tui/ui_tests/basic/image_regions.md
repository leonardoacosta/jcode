# crates/jcode-tui/src/tui/ui_tests/basic/image_regions.rs

- IMG_REGION_WIDE_PNG_B64 · constant · L17-L17 — const IMG_REGION_WIDE_PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAACgAAAACCAIAAAClqwlqAAAAEklEQVR4nGMQUDAYEMQwUBYDACXEHgHR+y+wAAAAAElFTkSuQmCC";
- IMG_REGION_TALL_PNG_B64 · constant · L19-L19 — const IMG_REGION_TALL_PNG_B64: &str = "iVBORw0KGgoAAAANSUhEUgAAAAIAAADICAIAAABNp6ehAAAAGUlEQVR4nGMQUDAAIoZRapQapUapUYo+FAATfpYBTI1UPgAAAABJRU5ErkJggg==";
- anchored_tool_image_with · function · L21-L38 — fn anchored_tool_image_with(
- inline_image_id_for · function · L40-L42 — fn inline_image_id_for(png_b64: &str) -> u64
- test_compute_image_regions_mermaid_placeholder_owns_blank_run · function · L49-L68 — fn test_compute_image_regions_mermaid_placeholder_owns_blank_run()
- test_compute_image_regions_inline_image_uses_marker_geometry · function · L71-L91 — fn test_compute_image_regions_inline_image_uses_marker_geometry()
- test_compute_image_regions_inline_image_clamps_to_available_blanks · function · L94-L113 — fn test_compute_image_regions_inline_image_clamps_to_available_blanks()
- test_incremental_append_offsets_new_image_region_and_keeps_old · function · L122-L181 — fn test_incremental_append_offsets_new_image_region_and_keeps_old()
- test_prefix_reuse_mid_edit_reoffsets_region_below_boundary · function · L187-L250 — fn test_prefix_reuse_mid_edit_reoffsets_region_below_boundary()
- test_prepend_shifts_suffix_image_regions_and_matches_full_build · function · L257-L334 — fn test_prepend_shifts_suffix_image_regions_and_matches_full_build()
- test_prefix_reuse_edit_recomputes_region_height_not_stale · function · L344-L413 — fn test_prefix_reuse_edit_recomputes_region_height_not_stale()
