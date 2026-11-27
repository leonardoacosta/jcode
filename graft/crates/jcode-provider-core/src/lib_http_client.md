# crates/jcode-provider-core/src/lib_http_client.rs

- http_error_body · function · L8-L13 — pub async fn http_error_body(response: reqwest::Response, context: &str) -> String
- shared_http_client · function · L19-L56 — pub fn shared_http_client() -> reqwest::Client
- CLIENT · constant · L21-L21 — static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
- fresh_transport_client · function · L67-L79 — pub fn fresh_transport_client() -> reqwest::Client
