# crates/jcode-core/src/output_style.rs

- EMOJI_ENABLED · constant · L9-L9 — static EMOJI_ENABLED: AtomicBool = AtomicBool::new(true);
- terminal_print · function · L12-L17 — macro_rules! terminal_print
- terminal_println · function · L20-L26 — macro_rules! terminal_println
- terminal_eprint · function · L29-L34 — macro_rules! terminal_eprint
- terminal_eprintln · function · L37-L43 — macro_rules! terminal_eprintln
- set_emoji_enabled · function · L46-L48 — pub fn set_emoji_enabled(enabled: bool)
- emoji_enabled · function · L51-L53 — pub fn emoji_enabled() -> bool
- terminal_text · function · L56-L58 — pub fn terminal_text(text: &str) -> Cow<'_, str>
- terminal_text_with_emoji · function · L61-L67 — pub fn terminal_text_with_emoji(text: &str, enabled: bool) -> Cow<'_, str>
- replace_emoji_with_ascii · function · L70-L80 — pub fn replace_emoji_with_ascii(text: &str) -> String
- contains_emoji · function · L82-L84 — fn contains_emoji(text: &str) -> bool
- emoji_ascii_fallback · function · L86-L106 — fn emoji_ascii_fallback(grapheme: &str) -> &'static str
- grapheme_is_emoji · function · L108-L133 — fn grapheme_is_emoji(grapheme: &str) -> bool
- tests · module · L136-L157 — mod tests
- emoji_clusters_use_readable_ascii_fallbacks · function · L140-L147 — fn emoji_clusters_use_readable_ascii_fallbacks()
- non_emoji_unicode_is_preserved · function · L150-L156 — fn non_emoji_unicode_is_preserved()
