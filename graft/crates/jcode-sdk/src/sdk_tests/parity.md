# crates/jcode-sdk/src/sdk_tests/parity.rs

- Capability · struct · L20-L25 — struct Capability
- CAPABILITIES · constant · L28-L94 — const CAPABILITIES: &[Capability] = &[
- cap · function · L96-L98 — const fn cap(rust: &'static str, ts: &'static str) -> Capability
- the_rust_sdk_implements_every_shared_capability · function · L102-L115 — fn the_rust_sdk_implements_every_shared_capability()
- the_typescript_sdk_implements_every_shared_capability · function · L119-L136 — fn the_typescript_sdk_implements_every_shared_capability()
- neither_sdk_has_an_untriaged_public_capability · function · L144-L187 — fn neither_sdk_has_an_untriaged_public_capability()
- RUST_ONLY · constant · L190-L206 — const RUST_ONLY: &[&str] = &[
- TS_ONLY · constant · L213-L213 — const TS_ONLY: &[(&str, &str)] = &[("close", "Rust closes through Drop")];
- rust_client_source · function · L215-L226 — fn rust_client_source() -> String
- ts_client_source · function · L228-L232 — fn ts_client_source() -> Option<String>
- rust_public_methods · function · L235-L253 — fn rust_public_methods() -> Vec<String>
- ts_public_methods · function · L260-L286 — fn ts_public_methods() -> Option<Vec<String>>
