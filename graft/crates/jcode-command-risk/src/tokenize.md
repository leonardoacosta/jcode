# crates/jcode-command-risk/src/tokenize.rs

- Token · struct · L10-L19 — pub struct Token
- word · function · L22-L29 — fn word(text: impl Into<String>) -> Self
- basename · function · L32-L38 — pub fn basename(&self) -> String
- is_flag · function · L40-L42 — pub fn is_flag(&self) -> bool
- is_recursive_flag · function · L45-L53 — pub fn is_recursive_flag(&self) -> bool
- SEGMENT_SEPARATORS · constant · L57-L57 — const SEGMENT_SEPARATORS: &[&str] = &["&&", "||", ";", "|", "\n", "(", ")"];
- split_segments · function · L63-L87 — pub fn split_segments(command: &str) -> Vec<Vec<Token>>
- tokenize · function · L95-L218 — pub fn tokenize(command: &str) -> Vec<Token>
- flush · function · L103-L118 — macro_rules! flush
- without_heredoc_bodies · function · L227-L256 — fn without_heredoc_bodies(command: &str) -> String
- embedded_substitutions · function · L266-L302 — pub fn embedded_substitutions(command: &str) -> Vec<String>
- scan_substitutions · function · L304-L349 — fn scan_substitutions(text: &str, shell_quotes: bool, found: &mut Vec<String>)
- matching_paren_body · function · L354-L393 — fn matching_paren_body(chars: &[char], start: usize) -> (String, usize)
- heredoc_delimiters · function · L397-L497 — fn heredoc_delimiters(line: &str) -> Vec<(String, bool, bool)>
- tokenize_tests · module · L501-L501 — mod tokenize_tests;
