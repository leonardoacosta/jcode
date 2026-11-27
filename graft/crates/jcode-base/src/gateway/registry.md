# crates/jcode-base/src/gateway/registry.rs

- DeviceRegistry · struct · L12-L16 — pub struct DeviceRegistry
- load · function · L20-L32 — pub fn load() -> Self
- save · function · L35-L40 — pub fn save(&self) -> Result<()>
- generate_pairing_code · function · L43-L61 — pub fn generate_pairing_code(&mut self) -> String
- validate_code · function · L64-L77 — pub fn validate_code(&mut self, code: &str) -> bool
- pair_device · function · L80-L112 — pub fn pair_device(
- validate_token · function · L115-L121 — pub fn validate_token(&self, token: &str) -> Option<&PairedDevice>
- touch_device · function · L124-L134 — pub fn touch_device(&mut self, token: &str)
