# crates/jcode-tui/tests/test_lock_order.rs

- test_body · function · L4-L12 — fn test_body<'a>(source: &'a str, name: &str) -> &'a str
- env_precedes_render · function · L14-L20 — fn env_precedes_render(body: &str, env_call: &str) -> bool
- inline_images_persistence_locks_env_before_render · function · L23-L32 — fn inline_images_persistence_locks_env_before_render()
- smoothness_benchmark_locks_env_before_render · function · L35-L44 — fn smoothness_benchmark_locks_env_before_render()
- ordering_check_rejects_both_original_inversions · function · L47-L62 — fn ordering_check_rejects_both_original_inversions()
