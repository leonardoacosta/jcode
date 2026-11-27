# crates/jcode-base/src/provider/anthropic.rs

- CACHE_TTL_1H · constant · L24-L24 — static CACHE_TTL_1H: AtomicU8 = AtomicU8::new(0);
- set_cache_ttl_1h · function · L27-L29 — pub fn set_cache_ttl_1h(enabled: bool)
- is_cache_ttl_1h · function · L32-L38 — pub fn is_cache_ttl_1h() -> bool
- CLAUDE_CLI_USER_AGENT · constant · L54-L54 — pub const CLAUDE_CLI_USER_AGENT: &str = concat!("jcode/", env!("CARGO_PKG_VERSION"));
- OAUTH_BETA_HEADERS · constant · L56-L56 — pub const OAUTH_BETA_HEADERS: &str = ANTHROPIC_OAUTH_BETA_HEADERS;
- effectively_1m · function · L59-L61 — pub fn effectively_1m(model: &str) -> bool
- new_oauth_request_id · function · L63-L65 — pub fn new_oauth_request_id() -> String
- apply_oauth_attribution_headers · function · L72-L77 — pub fn apply_oauth_attribution_headers(
- AVAILABLE_MODELS · constant · L80-L80 — pub const AVAILABLE_MODELS: &[&str] = jcode_provider_core::ALL_CLAUDE_MODELS;
- load_anthropic_api_key · function · L82-L130 — pub fn load_anthropic_api_key() -> Result<String>
- has_anthropic_api_key · function · L132-L134 — pub fn has_anthropic_api_key() -> bool
