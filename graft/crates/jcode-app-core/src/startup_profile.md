# crates/jcode-app-core/src/startup_profile.rs

- PROFILE · constant · L4-L4 — static PROFILE: Mutex<Option<StartupProfile>> = Mutex::new(None);
- StartupProfile · struct · L6-L9 — pub struct StartupProfile
- new · function · L12-L18 — fn new() -> Self
- init · function · L21-L27 — pub fn init()
- mark · function · L29-L35 — pub fn mark(name: &str)
- report · function · L37-L77 — pub fn report() -> String
- report_to_log · function · L79-L84 — pub fn report_to_log()
