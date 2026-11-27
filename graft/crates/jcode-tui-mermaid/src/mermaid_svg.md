# crates/jcode-tui-mermaid/src/mermaid_svg.rs

- estimate_diagram_size · function · L11-L37 — pub(super) fn estimate_diagram_size(content: &str) -> (usize, usize)
- calculate_render_size · function · L40-L67 — pub(super) fn calculate_render_size(
- normalize_render_target_width · function · L69-L80 — pub(super) fn normalize_render_target_width(width: f64) -> u32
- extract_xml_attribute · function · L86-L91 — pub(super) fn extract_xml_attribute<'a>(tag: &'a str, attr: &str) -> Option<&'a str>
- parse_svg_length · function · L97-L109 — pub(super) fn parse_svg_length(value: &str) -> Option<f32>
- parse_svg_viewbox_size · function · L115-L127 — pub(super) fn parse_svg_viewbox_size(tag: &str) -> Option<(f32, f32)>
- parse_svg_explicit_size · function · L133-L137 — pub(super) fn parse_svg_explicit_size(tag: &str) -> Option<(f32, f32)>
- format_svg_length · function · L143-L152 — fn format_svg_length(value: f32) -> String
- set_xml_attribute · function · L158-L178 — pub(super) fn set_xml_attribute(tag: &str, attr: &str, value: &str) -> String
- retarget_svg_for_png · function · L184-L216 — pub(super) fn retarget_svg_for_png(svg: &str, target_width: f64, target_height: f64) -> String
- primary_font_family · function · L219-L226 — fn primary_font_family(fonts: &str) -> String
- primary_font_family_in_db · function · L233-L259 — fn primary_font_family_in_db(fonts: &str, db: &usvg::fontdb::Database) -> String
- GENERIC · constant · L234-L241 — const GENERIC: [&str; 6] = [
- parse_hex_color_for_png · function · L262-L295 — fn parse_hex_color_for_png(input: &str) -> Option<resvg::tiny_skia::Color>
- write_output_png_cached_fonts · function · L298-L329 — pub(super) fn write_output_png_cached_fonts(
