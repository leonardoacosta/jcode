# crates/jcode-fuzzy/src/search_scores.rs

- MAX_WORDS · constant · L7-L7 — const MAX_WORDS: usize = 4096;
- MAX_BYTES · constant · L8-L8 — const MAX_BYTES: usize = 512 * 1024;
- TokenScores · struct · L10-L14 — struct TokenScores
- CachedWordScores · struct · L21-L25 — pub struct CachedWordScores
- prepare · function · L28-L39 — fn prepare(&mut self, token: &str)
- score · function · L42-L47 — pub fn score(&mut self, token: &str, word: &str) -> Option<i32>
- best_score · function · L51-L71 — pub fn best_score(&mut self, token: &str, text: &str, penalize_length: bool) -> Option<i32>
- score_word · function · L73-L97 — fn score_word(
- matches_text · function · L101-L108 — pub fn matches_text(&mut self, token: &str, text: &str) -> bool
- best_search_score · function · L111-L119 — pub fn best_search_score(&mut self, token: &str, text: &str) -> Option<i32>
- search_score · function · L121-L140 — fn search_score(&mut self, token: &str, word: &str) -> Option<i32>
- tests · module · L144-L246 — mod tests
- strict_search_caches_rejections_and_keeps_command_scores_independent · function · L148-L165 — fn strict_search_caches_rejections_and_keeps_command_scores_independent()
- cached_scores_and_boolean_matching_preserve_shared_matcher_semantics · function · L168-L212 — fn cached_scores_and_boolean_matching_preserve_shared_matcher_semantics()
- repeated_scores_reuse_entries_including_negative_results · function · L215-L226 — fn repeated_scores_reuse_entries_including_negative_results()
- cache_budget_and_long_words_never_change_eligible_scores · function · L229-L245 — fn cache_budget_and_long_words_never_change_eligible_scores()
