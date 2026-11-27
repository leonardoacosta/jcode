# crates/jcode-provider-copilot-runtime/src/routing.rs

- SONNET5_EFFORTS · constant · L6-L6 — pub(crate) const SONNET5_EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];
- copilot_model_efforts · function · L10-L22 — pub(crate) fn copilot_model_efforts(
- copilot_model_uses_responses_api · function · L24-L26 — pub(crate) fn copilot_model_uses_responses_api(model: &str) -> bool
- copilot_api_path · function · L28-L34 — pub(crate) fn copilot_api_path(uses_responses_api: bool) -> &'static str
- is_retryable_error · function · L36-L48 — pub(crate) fn is_retryable_error(error_str: &str) -> bool
