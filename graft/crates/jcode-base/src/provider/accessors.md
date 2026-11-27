# crates/jcode-base/src/provider/accessors.rs

- anthropic_provider · function · L4-L9 — pub(super) fn anthropic_provider(&self) -> Option<Arc<dyn Provider>>
- openai_provider · function · L11-L16 — pub(super) fn openai_provider(&self) -> Option<Arc<dyn Provider>>
- antigravity_provider · function · L18-L23 — pub(super) fn antigravity_provider(&self) -> Option<Arc<dyn Provider>>
- gemini_provider · function · L25-L30 — pub(super) fn gemini_provider(&self) -> Option<Arc<dyn Provider>>
- copilot_provider · function · L32-L37 — pub(super) fn copilot_provider(&self) -> Option<Arc<dyn Provider>>
- cursor_provider · function · L39-L44 — pub(super) fn cursor_provider(&self) -> Option<Arc<dyn Provider>>
- bedrock_provider · function · L46-L51 — pub(super) fn bedrock_provider(&self) -> Option<Arc<bedrock::BedrockProvider>>
- openrouter_provider · function · L53-L55 — pub(super) fn openrouter_provider(&self) -> Option<Arc<dyn Provider>>
- active_openrouter_execution_provider · function · L57-L59 — pub(super) fn active_openrouter_execution_provider(&self) -> Option<Arc<dyn Provider>>
- clear_active_openai_compatible_profile · function · L61-L63 — pub(super) fn clear_active_openai_compatible_profile(&self)
- has_claude_runtime · function · L65-L67 — pub(super) fn has_claude_runtime(&self) -> bool
- provider_slot_available · function · L69-L87 — pub(super) fn provider_slot_available(&self, provider: ActiveProvider) -> bool
- reconcile_auth_if_provider_missing · function · L89-L100 — pub(super) fn reconcile_auth_if_provider_missing(&self, provider: ActiveProvider) -> bool
