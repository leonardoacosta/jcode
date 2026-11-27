# crates/jcode-protocol/src/screen.rs

- SCREEN_MAX_BYTES · constant · L4-L4 — pub const SCREEN_MAX_BYTES: usize = 16 * 1024;
- SCREEN_MAX_WINDOWS · constant · L5-L5 — pub const SCREEN_MAX_WINDOWS: usize = 32;
- SCREEN_MAX_OUTPUTS · constant · L6-L6 — pub const SCREEN_MAX_OUTPUTS: usize = 16;
- SCREEN_LEASE_MS · constant · L7-L7 — pub const SCREEN_LEASE_MS: u64 = 30_000;
- SCREEN_MIN_UPDATE_MS · constant · L8-L8 — pub const SCREEN_MIN_UPDATE_MS: u64 = 2_000;
- ScreenContextAction · enum · L12-L16 — pub enum ScreenContextAction
- ScreenInteraction · enum · L20-L24 — pub enum ScreenInteraction
- ScreenSnapshot · struct · L28-L36 — pub struct ScreenSnapshot
- ScreenWindow · struct · L40-L49 — pub struct ScreenWindow
- ScreenOutput · struct · L53-L58 — pub struct ScreenOutput
- validate · function · L62-L104 — pub fn validate(&self) -> Result<(), &'static str>
