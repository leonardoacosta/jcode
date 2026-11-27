# crates/jcode-config-types/src/web_search.rs

- WebSearchEngine · enum · L8-L28 — pub enum WebSearchEngine
- as_str · function · L31-L40 — pub fn as_str(self) -> &'static str
- parse · function · L42-L52 — pub fn parse(value: &str) -> Option<Self>
- is_local · function · L55-L57 — pub fn is_local(self) -> bool
- CustomWebSearchMethod · enum · L62-L68 — pub enum CustomWebSearchMethod
- CustomWebSearchConfig · struct · L73-L89 — pub struct CustomWebSearchConfig
- default · function · L92-L107 — fn default() -> Self
- WebSearchConfig · struct · L113-L154 — pub struct WebSearchConfig
- default · function · L157-L174 — fn default() -> Self
- DEFAULT_NATIVE_WEB_SEARCH_MAX_USES · constant · L178-L178 — pub const DEFAULT_NATIVE_WEB_SEARCH_MAX_USES: u32 = 5;
- DEFAULT_ANTHROPIC_WEB_SEARCH_TOOL · constant · L180-L180 — pub const DEFAULT_ANTHROPIC_WEB_SEARCH_TOOL: &str = "web_search_20250305";
- native_enabled · function · L184-L186 — pub fn native_enabled(&self) -> bool
- tests · module · L190-L217 — mod tests
- kagi_config_is_opt_in_and_round_trips · function · L194-L216 — fn kagi_config_is_opt_in_and_round_trips()
