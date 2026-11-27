# crates/jcode-base/src/provider/stream_timeout.rs

- MAX_STREAM_IDLE_TIMEOUT_MULTIPLIER · constant · L16-L16 — pub const MAX_STREAM_IDLE_TIMEOUT_MULTIPLIER: u32 = 4;
- stream_idle_timeout · function · L23-L29 — pub fn stream_idle_timeout() -> Duration
- stream_idle_timeout_multiplier_for_effort · function · L35-L48 — pub fn stream_idle_timeout_multiplier_for_effort(effort: Option<&str>) -> u32
- stream_idle_timeout_for_effort · function · L51-L53 — pub fn stream_idle_timeout_for_effort(effort: Option<&str>) -> Duration
- max_stream_idle_timeout · function · L59-L61 — pub fn max_stream_idle_timeout() -> Duration
- tests · module · L64-L127 — mod tests
- multiplier_scales_with_reasoning_effort · function · L68-L112 — fn multiplier_scales_with_reasoning_effort()
- effort_scaling_never_shrinks_the_base_budget · function · L115-L126 — fn effort_scaling_never_shrinks_the_base_budget()
