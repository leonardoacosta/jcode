# crates/jcode-provider-openai/src/request_normalization_tests.rs

- legacy_reorder_outputs · function · L15-L148 — fn legacy_reorder_outputs<F: FnMut(OpenAiRequestLogLevel, &str)>(
- fixtures · function · L150-L164 — fn fixtures() -> Vec<Value>
- compare · function · L166-L179 — fn compare(input: Vec<Value>)
- legacy_oracle_order_duplicates_orphans_and_native_calls · function · L182-L204 — fn legacy_oracle_order_duplicates_orphans_and_native_calls()
- final_emission_moves_selected_output_storage · function · L207-L230 — fn final_emission_moves_selected_output_storage()
- CountingAllocator · struct · L234-L234 — struct CountingAllocator;
- ALLOCATOR · constant · L239-L239 — static ALLOCATOR: CountingAllocator = CountingAllocator;
- alloc · function · L241-L248 — unsafe fn alloc(&self, layout: Layout) -> *mut u8
- dealloc · function · L249-L251 — unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
- realloc · function · L252-L259 — unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8
- normalization_release_benchmark · function · L264-L311 — fn normalization_release_benchmark()
