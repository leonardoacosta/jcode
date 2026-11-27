# crates/jcode-command-risk/src/gate_tests.rs

- ctx · function · L10-L16 — fn ctx() -> RiskContext
- none · function · L18-L20 — fn none() -> Justification
- saying · function · L22-L26 — fn saying(text: &str) -> Justification
- safe_commands_pass_straight_through · function · L29-L32 — fn safe_commands_pass_straight_through()
- in_project_cleanup_is_not_interrupted · function · L35-L38 — fn in_project_cleanup_is_not_interrupted()
- catastrophic_is_denied_even_with_a_perfect_justification · function · L41-L53 — fn catastrophic_is_denied_even_with_a_perfect_justification()
- first_attempt_at_a_risky_command_is_refused_with_a_reflection_prompt · function · L56-L67 — fn first_attempt_at_a_risky_command_is_refused_with_a_reflection_prompt()
- a_blind_retry_does_not_satisfy_the_gate · function · L70-L78 — fn a_blind_retry_does_not_satisfy_the_gate()
- empty_affirmations_are_rejected · function · L81-L93 — fn empty_affirmations_are_rejected()
- a_substantive_justification_allows_the_command · function · L96-L103 — fn a_substantive_justification_allows_the_command()
- justification_substance_check_is_a_low_but_real_bar · function · L106-L112 — fn justification_substance_check_is_a_low_but_real_bar()
- reflection_prompt_names_the_specific_path · function · L115-L124 — fn reflection_prompt_names_the_specific_path()
