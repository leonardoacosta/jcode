# crates/jcode-session-types/src/prelowercased_tokenization_probe.rs

- thread_cpu_ns · function · L10-L20 — fn thread_cpu_ns() -> u128
- CountingAllocator · struct · L22-L22 — struct CountingAllocator;
- ALLOCATOR · constant · L27-L27 — static ALLOCATOR: CountingAllocator = CountingAllocator;
- alloc · function · L29-L36 — unsafe fn alloc(&self, layout: Layout) -> *mut u8
- dealloc · function · L37-L39 — unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
- realloc · function · L40-L47 — unsafe fn realloc(&self, ptr: *mut u8, old: Layout, size: usize) -> *mut u8
- optimized · function · L50-L52 — fn optimized(text: &str) -> Vec<String>
- tokenization_probe · function · L56-L123 — fn tokenization_probe()
