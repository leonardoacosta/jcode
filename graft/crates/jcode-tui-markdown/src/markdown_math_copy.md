# crates/jcode-tui-markdown/src/markdown_math_copy.rs

- DISPLAY_LIMIT · constant · L10-L10 — const DISPLAY_LIMIT: usize = 4096;
- INLINE_LIMIT · constant · L11-L11 — const INLINE_LIMIT: usize = 8192;
- LINE_LIMIT · constant · L12-L12 — const LINE_LIMIT: usize = 16384;
- Bounded · struct · L14-L18 — struct Bounded<V>
- new · function · L21-L27 — fn new(limit: usize) -> Self
- insert · function · L29-L38 — fn insert(&mut self, key: u64, value: V)
- DISPLAY_SOURCES · constant · L41-L42 — static DISPLAY_SOURCES: LazyLock<Mutex<Bounded<String>>> =
- INLINE_SOURCES · constant · L43-L44 — static INLINE_SOURCES: LazyLock<Mutex<Bounded<String>>> =
- LINE_SPANS · constant · L45-L46 — static LINE_SPANS: LazyLock<Mutex<Bounded<Arc<Vec<InlineMathSpan>>>>> =
- hash_str · function · L48-L52 — fn hash_str(value: &str) -> u64
- display_key · function · L56-L64 — fn display_key<'a>(lines: impl IntoIterator<Item = &'a str>) -> u64
- register_display_math · function · L66-L71 — pub(crate) fn register_display_math(rendered_lines: &[String], source: &str)
- display_math_source · function · L74-L80 — pub(crate) fn display_math_source(content_lines: &[String]) -> Option<String>
- register_inline_math · function · L82-L89 — pub(crate) fn register_inline_math(rendered: &str, source: &str)
- InlineMathSpan · struct · L93-L98 — pub struct InlineMathSpan
- inline_math_spans · function · L103-L123 — pub fn inline_math_spans(line: &Line<'_>) -> Vec<InlineMathSpan>
- record_inline_math_for_line · function · L127-L144 — pub fn record_inline_math_for_line(line: &Line<'_>)
- inline_math_spans_for_plain_line · function · L147-L155 — pub fn inline_math_spans_for_plain_line(plain: &str) -> Option<Arc<Vec<InlineMathSpan>>>
