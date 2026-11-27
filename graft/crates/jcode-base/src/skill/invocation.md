# crates/jcode-base/src/skill/invocation.rs

- SkillInvocation · struct · L5-L8 — pub struct SkillInvocation<'a>
- resolve_invocation · function · L14-L44 — pub fn resolve_invocation<'a>(&self, input: &'a str) -> Option<SkillInvocation<'a>>
- tests · module · L48-L149 — mod tests
- registry · function · L53-L74 — fn registry(names: &[&str]) -> SkillRegistry
- matches_multi_word_skill_names · function · L77-L86 — fn matches_multi_word_skill_names()
- keeps_trailing_prompt_after_multi_word_name · function · L89-L102 — fn keeps_trailing_prompt_after_multi_word_name()
- prefers_longest_registered_match · function · L105-L114 — fn prefers_longest_registered_match()
- falls_back_to_single_token_when_no_registry_match_exists · function · L117-L126 — fn falls_back_to_single_token_when_no_registry_match_exists()
- does_not_bypass_terminal_file_drop_guard · function · L129-L136 — fn does_not_bypass_terminal_file_drop_guard()
- preserves_single_word_skill_invocations · function · L139-L148 — fn preserves_single_word_skill_invocations()
