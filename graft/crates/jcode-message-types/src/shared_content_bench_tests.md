# crates/jcode-message-types/src/shared_content_bench_tests.rs

- Original · struct · L7-L7 — struct Original(Arc<Vec<ContentBlock>>, #[serde(skip)] Arc<ContentMetrics>);
- BenchContent · interface · L9-L13 — trait BenchContent: Clone + for<'de> serde::Deserialize<'de>
- from_blocks · function · L10-L10 — fn from_blocks(blocks: Vec<ContentBlock>) -> Self;
- mutate · function · L11-L11 — fn mutate(&mut self);
- into_blocks · function · L12-L12 — fn into_blocks(self) -> Vec<ContentBlock>;
- from_blocks · function · L15-L17 — fn from_blocks(blocks: Vec<ContentBlock>) -> Self
- mutate · function · L18-L29 — fn mutate(&mut self)
- into_blocks · function · L30-L32 — fn into_blocks(self) -> Vec<ContentBlock>
- from_blocks · function · L35-L37 — fn from_blocks(blocks: Vec<ContentBlock>) -> Self
- mutate · function · L38-L40 — fn mutate(&mut self)
- into_blocks · function · L41-L43 — fn into_blocks(self) -> Vec<ContentBlock>
- measure · function · L45-L80 — fn measure<T: BenchContent>(blocks: &[ContentBlock], json: &[u8], path: &str) -> f64
- benchmark_representation_paths · function · L83-L124 — fn benchmark_representation_paths()
- detach_and_unique_mutation_release_cached_metadata · function · L127-L152 — fn detach_and_unique_mutation_release_cached_metadata()
- original_representation_wire_and_public_trait_parity · function · L155-L189 — fn original_representation_wire_and_public_trait_parity()
- assert_traits · function · L156-L156 — fn assert_traits<T: Send + Sync + Clone + Default + std::fmt::Debug>() {}
- consuming_unique_payload_releases_metrics_without_retaining_blocks · function · L192-L211 — fn consuming_unique_payload_releases_metrics_without_retaining_blocks()
