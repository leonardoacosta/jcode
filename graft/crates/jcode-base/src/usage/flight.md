# crates/jcode-base/src/usage/flight.rs

- tests · module · L8-L8 — mod tests;
- Flights · type · L10-L10 — type Flights = HashMap<String, Weak<tokio::sync::Mutex<()>>>;
- FLIGHTS · constant · L11-L11 — static FLIGHTS: OnceLock<Mutex<Flights>> = OnceLock::new();
- flight · function · L13-L25 — fn flight(key: &str) -> Arc<tokio::sync::Mutex<()>>
- CancelledProbe · struct · L27-L27 — struct CancelledProbe(Option<String>);
- drop · function · L30-L40 — fn drop(&mut self)
- fetch · function · L43-L77 — pub(super) async fn fetch<F, Fut>(key: String, probe: F) -> Result<UsageData>
- RequestLog · struct · L82-L87 — pub(super) struct RequestLog
- new · function · L90-L100 — pub fn new(key: &str) -> Self
- received · function · L102-L113 — pub fn received(&mut self, status: u16, retry_after: Option<&str>)
- drop · function · L117-L126 — fn drop(&mut self)
