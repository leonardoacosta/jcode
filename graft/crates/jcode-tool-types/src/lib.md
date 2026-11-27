# crates/jcode-tool-types/src/lib.rs

- ToolOutput · struct · L2-L7 — pub struct ToolOutput
- ToolImage · struct · L10-L14 — pub struct ToolImage
- new · function · L17-L24 — pub fn new(output: impl Into<String>) -> Self
- with_title · function · L26-L29 — pub fn with_title(mut self, title: impl Into<String>) -> Self
- with_metadata · function · L31-L34 — pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self
- with_image · function · L36-L43 — pub fn with_image(mut self, media_type: impl Into<String>, data: impl Into<String>) -> Self
- with_labeled_image · function · L45-L57 — pub fn with_labeled_image(
- resolve_tool_name · function · L71-L116 — pub fn resolve_tool_name(name: &str) -> &str
- tests · module · L119-L154 — mod tests
- resolve_tool_name_strips_function_namespace_before_alias_resolution · function · L123-L127 — fn resolve_tool_name_strips_function_namespace_before_alias_resolution()
- resolve_tool_name_does_not_strip_unrecognized_namespaces · function · L130-L135 — fn resolve_tool_name_does_not_strip_unrecognized_namespaces()
- resolve_tool_name_maps_pascalcase_oauth_aliases · function · L138-L153 — fn resolve_tool_name_maps_pascalcase_oauth_aliases()
