# crates/jcode-app-core/src/tool/webfetch_corpus_tests.rs

- corpus_dir · function · L19-L23 — fn corpus_dir() -> Option<PathBuf>
- Page · struct · L25-L28 — struct Page
- load_corpus · function · L30-L55 — fn load_corpus() -> Vec<Page>
- is_html · function · L57-L60 — fn is_html(html: &str) -> bool
- punctuation_noise_ratio · function · L64-L73 — fn punctuation_noise_ratio(text: &str) -> f64
- corpus_extraction_invariants · function · L76-L156 — fn corpus_extraction_invariants()
- count_escaped_occurrences · function · L162-L181 — fn count_escaped_occurrences(html: &str, needle: &str) -> usize
- strip_tags · function · L185-L197 — fn strip_tags(html: &str) -> String
- count_in_code_blocks · function · L201-L222 — fn count_in_code_blocks(html: &str, needle: &str) -> usize
- find_residual_tag · function · L230-L252 — fn find_residual_tag(html: &str, out: &str) -> Option<(usize, &'static str)>
- TAGS · constant · L231-L233 — const TAGS: [&str; 10] = [
- snippet · function · L254-L266 — fn snippet(text: &str, idx: usize) -> String
- corpus_report · function · L272-L305 — fn corpus_report()
- corpus_script_matches_test_expectations · function · L308-L320 — fn corpus_script_matches_test_expectations()
- dump_outputs · function · L326-L342 — fn dump_outputs()
