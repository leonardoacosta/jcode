# crates/jcode-message-types/src/content_capacity.rs

- ContentCapacity · struct · L12-L20 — pub struct ContentCapacity
- known_heap_bytes · function · L23-L25 — pub fn known_heap_bytes(&self) -> usize
- ContentCapacityAccounting · struct · L31-L35 — pub struct ContentCapacityAccounting<'a>
- new · function · L38-L40 — pub fn new() -> Self
- record_vec_allocation · function · L44-L46 — pub fn record_vec_allocation<T>(&mut self, values: &Vec<T>)
- record_shared · function · L48-L58 — pub fn record_shared(&mut self, content: &'a SharedContent)
- finish · function · L60-L62 — pub fn finish(self) -> ContentCapacity
- string · function · L64-L67 — fn string(&mut self, value: &String)
- optional_string · function · L69-L73 — fn optional_string(&mut self, value: &Option<String>)
- json · function · L75-L94 — fn json(&mut self, value: &serde_json::Value)
- record_block · function · L96-L168 — fn record_block(&mut self, block: &ContentBlock)
- tests · module · L172-L252 — mod tests
- reserved · function · L175-L179 — fn reserved(text: &str, capacity: usize) -> String
- counts_capacity_not_length_and_shared_backing_once · function · L182-L200 — fn counts_capacity_not_length_and_shared_backing_once()
- nested_json_keys_strings_and_array_spare_capacity_are_counted · function · L203-L227 — fn nested_json_keys_strings_and_array_spare_capacity_are_counted()
- distinct_empty_backings_and_detached_snapshots_are_not_deduplicated · function · L230-L251 — fn distinct_empty_backings_and_detached_snapshots_are_not_deduplicated()
