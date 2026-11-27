# crates/jcode-productivity-core/src/scan.rs

- CacheEntry · struct · L21-L25 — struct CacheEntry
- Cache · struct · L28-L33 — struct Cache
- CACHE_VERSION · constant · L35-L35 — const CACHE_VERSION: u32 = 1;
- cache_path · function · L37-L43 — fn cache_path() -> Result<PathBuf>
- sessions_dir · function · L45-L47 — fn sessions_dir() -> Result<PathBuf>
- ScanResult · struct · L50-L56 — pub struct ScanResult
- scan_all · function · L59-L158 — pub fn scan_all() -> Result<ScanResult>
- mtime_ns · function · L161-L164 — fn mtime_ns(meta: &std::fs::Metadata) -> i128
- mtime_ns · function · L167-L173 — fn mtime_ns(meta: &std::fs::Metadata) -> i128
- RawSession · struct · L184-L197 — struct RawSession
- RawMessage · struct · L200-L209 — struct RawMessage
- RawTokenUsage · struct · L212-L221 — struct RawTokenUsage
- RawBlock · enum · L225-L239 — enum RawBlock
- parse_session_file · function · L241-L245 — fn parse_session_file(path: &Path) -> Result<SessionSummary>
- summarize · function · L247-L329 — fn summarize(raw: RawSession) -> SessionSummary
- count_tool · function · L333-L347 — fn count_tool(tools: &mut BTreeMap<String, u32>, name: &str, input: &serde_json::Value)
- canonical_tool_name · function · L350-L359 — fn canonical_tool_name(name: &str) -> &str
- project_name · function · L361-L369 — fn project_name(working_dir: &str) -> String
