# crates/jcode-applet-types/src/validate.rs

- Limits · struct · L10-L22 — pub struct Limits
- default · function · L25-L38 — fn default() -> Self
- ValidationError · enum · L42-L64 — pub enum ValidationError
- fmt · function · L67-L69 — fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
- validate_manifest · function · L75-L93 — pub fn validate_manifest(manifest: &Manifest) -> Result<(), ValidationError>
- validate_document · function · L96-L146 — pub fn validate_document(
- Walk · struct · L148-L155 — struct Walk<'a>
- require · function · L158-L164 — fn require(&self, capability: Capability) -> Result<(), ValidationError>
- text · function · L166-L172 — fn text(&mut self, text: &str) -> Result<(), ValidationError>
- action · function · L174-L211 — fn action(&self, action: &Action) -> Result<(), ValidationError>
- node · function · L213-L296 — fn node(&mut self, node: &Node, depth: usize) -> Result<(), ValidationError>
- image · function · L298-L344 — fn image(&self, source: &ImageSource) -> Result<(), ValidationError>
