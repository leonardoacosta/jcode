# crates/jcode-base/src/mobile_sync_thumbnail_cache_clone_tests.rs

- old_get · function · L4-L12 — fn old_get(cache: &mut ThumbnailCache, key: &[u8; 32]) -> Option<Option<WireImage>>
- old_insert · function · L14-L48 — fn old_insert(cache: &mut ThumbnailCache, key: [u8; 32], image: Option<WireImage>)
- assert_state · function · L50-L56 — fn assert_state(old: &ThumbnailCache, new: &ThumbnailCache)
- differential_exact_policy_and_outputs · function · L59-L101 — fn differential_exact_policy_and_outputs()
- benchmark_borrowed_candidates · function · L105-L135 — fn benchmark_borrowed_candidates()
- benchmark_unchanged_and_duplicate · function · L139-L170 — fn benchmark_unchanged_and_duplicate()
