# crates/jcode-app-core/src/tool/agentgrep/args.rs

- ResolvedSearchScope · struct · L3-L6 — struct ResolvedSearchScope
- resolved_search_scope · function · L8-L43 — fn resolved_search_scope(
- build_grep_args · function · L45-L69 — pub(super) fn build_grep_args(params: &AgentGrepInput, ctx: &ToolContext) -> Result<GrepArgs>
- build_find_args · function · L71-L103 — pub(super) fn build_find_args(params: &AgentGrepInput, ctx: &ToolContext) -> Result<FindArgs>
- build_outline_args · function · L105-L135 — pub(super) fn build_outline_args(
- build_smart_args_and_query · function · L137-L174 — pub(super) fn build_smart_args_and_query(
- trace_or_smart_terms_owned · function · L176-L205 — pub(super) fn trace_or_smart_terms_owned(params: &AgentGrepInput) -> Result<Vec<String>>
- outline_file_arg · function · L207-L221 — fn outline_file_arg(params: &AgentGrepInput) -> Result<String>
- parse_full_region_mode · function · L223-L232 — fn parse_full_region_mode(value: Option<&str>) -> Result<FullRegionMode>
- resolved_root_string · function · L234-L236 — fn resolved_root_string(ctx: &ToolContext, path: Option<&str>) -> Option<String>
- resolve_search_root · function · L238-L242 — pub(super) fn resolve_search_root(ctx: &ToolContext, path: Option<&str>) -> Result<PathBuf>
- summarize_agentgrep_request · function · L244-L278 — pub(super) fn summarize_agentgrep_request(
