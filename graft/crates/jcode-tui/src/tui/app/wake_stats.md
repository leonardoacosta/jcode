# crates/jcode-tui/src/tui/app/wake_stats.rs

- COUNTS · constant · L6-L6 — static COUNTS: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());
- note · function · L8-L16 — pub(crate) fn note(source: &str)
- note_debug · function · L20-L41 — pub(crate) fn note_debug<T: std::fmt::Debug>(prefix: &str, value: &T)
- Name · struct · L21-L21 — struct Name(String, bool);
- write_str · function · L23-L32 — fn write_str(&mut self, s: &str) -> std::fmt::Result
- snapshot · function · L43-L58 — pub(crate) fn snapshot() -> serde_json::Value
- tests · module · L61-L77 — mod tests
- debug_counter_stops_at_variant_name · function · L63-L76 — fn debug_counter_stops_at_variant_name()
