# crates/jcode-tui/src/tui/ui/profile.rs

- RenderProfile · struct · L5-L11 — struct RenderProfile
- PROFILE_STATE · constant · L13-L13 — static PROFILE_STATE: OnceLock<Mutex<RenderProfile>> = OnceLock::new();
- profile_state · function · L15-L17 — fn profile_state() -> &'static Mutex<RenderProfile>
- profile_enabled · function · L19-L22 — pub(super) fn profile_enabled() -> bool
- ENABLED · constant · L20-L20 — static ENABLED: OnceLock<bool> = OnceLock::new();
- record_profile · function · L24-L54 — pub(super) fn record_profile(prepare: Duration, draw: Duration, total: Duration)
