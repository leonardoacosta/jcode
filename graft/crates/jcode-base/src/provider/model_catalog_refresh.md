# crates/jcode-base/src/provider/model_catalog_refresh.rs

- note_openai_model_catalog_refresh_attempt · function · L4-L6 — pub fn note_openai_model_catalog_refresh_attempt()
- note_openai_model_catalog_refresh_attempt_for_scope · function · L8-L10 — pub(super) fn note_openai_model_catalog_refresh_attempt_for_scope(scope: &str)
- anthropic_model_catalog_refresh_throttled · function · L12-L14 — fn anthropic_model_catalog_refresh_throttled(scope: &str) -> bool
- should_refresh_openai_model_catalog · function · L16-L18 — pub fn should_refresh_openai_model_catalog() -> bool
- should_refresh_openai_model_catalog_for_scope · function · L20-L28 — pub fn should_refresh_openai_model_catalog_for_scope(scope: &str) -> bool
- should_refresh_anthropic_model_catalog · function · L30-L33 — pub fn should_refresh_anthropic_model_catalog() -> bool
- should_refresh_anthropic_model_catalog_for_scope · function · L35-L46 — pub fn should_refresh_anthropic_model_catalog_for_scope(scope: &str) -> bool
- begin_openai_model_catalog_refresh · function · L48-L50 — pub fn begin_openai_model_catalog_refresh() -> bool
- begin_openai_model_catalog_refresh_for_scope · function · L52-L54 — pub fn begin_openai_model_catalog_refresh_for_scope(scope: &str) -> bool
- begin_anthropic_model_catalog_refresh · function · L56-L59 — pub fn begin_anthropic_model_catalog_refresh() -> Option<String>
- begin_anthropic_model_catalog_refresh_for_scope · function · L61-L68 — pub fn begin_anthropic_model_catalog_refresh_for_scope(scope: &str) -> bool
- begin_forced_anthropic_model_catalog_refresh_for_scope · function · L70-L72 — pub fn begin_forced_anthropic_model_catalog_refresh_for_scope(scope: &str) -> bool
- finish_openai_model_catalog_refresh · function · L74-L76 — pub fn finish_openai_model_catalog_refresh()
- finish_openai_model_catalog_refresh_for_scope · function · L78-L80 — pub fn finish_openai_model_catalog_refresh_for_scope(scope: &str)
- finish_anthropic_model_catalog_refresh_for_scope · function · L82-L84 — pub fn finish_anthropic_model_catalog_refresh_for_scope(scope: &str)
- anthropic_model_cache_is_fresh · function · L86-L88 — pub(super) fn anthropic_model_cache_is_fresh(scope: &str) -> bool
