# crates/jcode-app-core/src/tool/websearch_kagi.rs

- ENDPOINT · constant · L6-L6 — const ENDPOINT: &str = "https://kagi.com/api/v1/search";
- client · function · L9-L16 — fn client() -> Result<reqwest::Client>
- request · function · L18-L43 — fn request(
- search · function · L45-L58 — pub(super) async fn search(
- execute · function · L60-L80 — async fn execute(
- parse_results · function · L82-L111 — fn parse_results(body: &Value, count: usize) -> Result<Vec<SearchResult>>
- tests · module · L115-L115 — mod tests;
