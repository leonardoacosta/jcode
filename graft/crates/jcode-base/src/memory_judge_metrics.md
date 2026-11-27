# crates/jcode-base/src/memory_judge_metrics.rs

- JudgeDecision · enum · L38-L67 — pub enum JudgeDecision
- label · function · L71-L81 — pub fn label(self) -> &'static str
- is_no_llm · function · L85-L87 — pub fn is_no_llm(self) -> bool
- is_degradation · function · L92-L100 — pub fn is_degradation(self) -> bool
- ALL · constant · L104-L112 — pub const ALL: [JudgeDecision; 7] = [
- from_rerank_outcome · function · L117-L125 — pub fn from_rerank_outcome(outcome: crate::memory_rerank::RerankOutcome) -> Self
- COUNTS · constant · L129-L137 — static COUNTS: [AtomicU64; 7] = [
- decision_index · function · L139-L149 — fn decision_index(d: JudgeDecision) -> usize
- record · function · L154-L177 — pub fn record(decision: JudgeDecision, session_id: &str, candidate_count: usize)
- count · function · L180-L182 — pub fn count(decision: JudgeDecision) -> u64
- reset · function · L185-L189 — pub fn reset()
- JudgeMetricsSnapshot · struct · L193-L210 — pub struct JudgeMetricsSnapshot
- snapshot · function · L213-L249 — pub fn snapshot() -> JudgeMetricsSnapshot
- tests · module · L252-L312 — mod tests
- all_array_matches_enum_and_indices · function · L256-L266 — fn all_array_matches_enum_and_indices()
- classification_is_consistent · function · L269-L293 — fn classification_is_consistent()
- snapshot_computes_rates · function · L296-L311 — fn snapshot_computes_rates()
