# crates/jcode-app-core/src/tool/selfdev/setup.rs

- SetupCheck · struct · L6-L12 — struct SetupCheck
- ok · function · L15-L22 — fn ok(name: &'static str, detail: impl Into<String>) -> Self
- missing · function · L24-L31 — fn missing(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self
- marker · function · L33-L35 — fn marker(&self) -> &'static str
- command_version · function · L39-L51 — fn command_version(program: &str, args: &[&str]) -> Option<String>
- do_setup · function · L59-L195 — pub(super) async fn do_setup(&self, ctx: &ToolContext) -> Result<ToolOutput>
- do_find_config · function · L200-L276 — pub(super) async fn do_find_config(&self, ctx: &ToolContext) -> Result<ToolOutput>
- do_reload_to_newer_build · function · L282-L313 — pub(super) async fn do_reload_to_newer_build(&self, _ctx: &ToolContext) -> Result<ToolOutput>
- selfdev_clone_dir · function · L316-L318 — fn selfdev_clone_dir() -> Result<std::path::PathBuf>
- clone_selfdev_source · function · L321-L353 — fn clone_selfdev_source() -> Result<std::path::PathBuf>
