# crates/jcode-base/src/provider/registry.rs

- ProviderRegistry · struct · L9-L11 — pub(super) struct ProviderRegistry<'a>
- new · function · L14-L16 — pub(super) fn new(provider: &'a MultiProvider) -> Self
- real_openrouter · function · L18-L24 — pub(super) fn real_openrouter(&self) -> Option<Arc<dyn Provider>>
- compatible_profile · function · L26-L33 — pub(super) fn compatible_profile(&self, profile_id: &str) -> Option<Arc<dyn Provider>>
- install_compatible_profile · function · L35-L45 — pub(super) fn install_compatible_profile(
- active_compatible_profile_id · function · L47-L53 — pub(super) fn active_compatible_profile_id(&self) -> Option<String>
- set_active_compatible_profile · function · L55-L61 — pub(super) fn set_active_compatible_profile(&self, profile_id: impl Into<String>)
- clear_active_compatible_profile · function · L63-L69 — pub(super) fn clear_active_compatible_profile(&self)
- active_compatible_profile · function · L71-L74 — pub(super) fn active_compatible_profile(&self) -> Option<Arc<dyn Provider>>
- active_openrouter_execution · function · L77-L80 — pub(super) fn active_openrouter_execution(&self) -> Option<Arc<dyn Provider>>
