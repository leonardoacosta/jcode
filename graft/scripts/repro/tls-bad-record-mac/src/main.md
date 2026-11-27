# scripts/repro/tls-bad-record-mac/src/main.rs

- classify · function · L24-L63 — fn classify(error_str: &str) -> bool
- openai_classify_old · function · L68-L93 — fn openai_classify_old(error_str: &str) -> bool
- openai_classify_new · function · L96-L98 — fn openai_classify_new(error_str: &str) -> bool
- classify_chain · function · L101-L113 — fn classify_chain(err: &anyhow::Error) -> bool
- main · function · L116-L251 — async fn main() -> anyhow::Result<()>
- serde_json_body · function · L253-L260 — fn serde_json_body() -> Vec<u8>
- corrupt_proxy · function · L264-L324 — async fn corrupt_proxy(client: TcpStream, server_addr: std::net::SocketAddr) -> anyhow::Result<()>
