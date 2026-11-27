# crates/jcode-terminal-image/src/metadata.rs

- compact_path_label · function · L1-L9 — pub fn compact_path_label(path: &str) -> String
- compact_image_format · function · L11-L31 — pub fn compact_image_format(value: &str) -> String
- format_dimensions · function · L33-L35 — pub fn format_dimensions(width: u32, height: u32) -> String
- aspect_ratio · function · L37-L43 — pub fn aspect_ratio(width: u32, height: u32) -> Option<String>
- format_byte_count · function · L45-L73 — pub fn format_byte_count(bytes: u64) -> String
- KB · constant · L46-L46 — const KB: f64 = 1024.0;
- MB · constant · L47-L47 — const MB: f64 = KB * 1024.0;
- GB · constant · L48-L48 — const GB: f64 = MB * 1024.0;
- estimate_base64_decoded_len · function · L75-L100 — pub fn estimate_base64_decoded_len(data: &str) -> Option<u64>
- gcd · function · L102-L109 — fn gcd(mut a: u32, mut b: u32) -> u32
- tests · module · L112-L138 — mod tests
- format_byte_count_uses_compact_units · function · L116-L120 — fn format_byte_count_uses_compact_units()
- estimate_base64_decoded_len_handles_common_payloads · function · L123-L131 — fn estimate_base64_decoded_len_handles_common_payloads()
- aspect_ratio_reduces_dimensions · function · L134-L137 — fn aspect_ratio_reduces_dimensions()
