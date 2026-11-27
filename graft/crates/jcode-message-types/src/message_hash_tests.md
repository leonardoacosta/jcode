# crates/jcode-message-types/src/message_hash_tests.rs

- CountingAllocator · struct · L10-L10 — struct CountingAllocator;
- ALLOCATOR · constant · L12-L12 — static ALLOCATOR: CountingAllocator = CountingAllocator;
- alloc · function · L14-L21 — unsafe fn alloc(&self, layout: Layout) -> *mut u8
- dealloc · function · L22-L24 — unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
- realloc · function · L25-L32 — unsafe fn realloc(&self, ptr: *mut u8, old: Layout, size: usize) -> *mut u8
- legacy · function · L35-L51 — fn legacy(message: &Message) -> u64
- cold · function · L53-L60 — fn cold(message: &Message) -> Message
- shapes · function · L62-L98 — fn shapes() -> Vec<Message>
- legacy_parity_and_cache_isolation · function · L101-L131 — fn legacy_parity_and_cache_isolation()
- controls_unicode_and_individual_metadata_changes · function · L134-L161 — fn controls_unicode_and_individual_metadata_changes()
- writer_preserves_serialization_errors · function · L166-L179 — fn writer_preserves_serialization_errors()
- Fails · struct · L167-L167 — struct Fails;
- serialize · function · L169-L171 — fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error>
- cpu_ns · function · L182-L190 — fn cpu_ns() -> u128
- Timespec · struct · L184-L187 — struct Timespec
- clock_gettime · function · L188-L189 — unsafe extern "C"
- benchmark_message_hash · function · L220-L268 — fn benchmark_message_hash()
