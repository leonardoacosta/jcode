# crates/jcode-session-types/tests/search_traversals.rs

- legacy · module · L3-L6 — mod legacy
- Allocator · struct · L10-L10 — struct Allocator;
- ALLOCATOR · constant · L13-L13 — static ALLOCATOR: Allocator = Allocator;
- alloc · function · L15-L22 — unsafe fn alloc(&self, l: Layout) -> *mut u8
- dealloc · function · L23-L25 — unsafe fn dealloc(&self, p: *mut u8, l: Layout)
- realloc · function · L26-L33 — unsafe fn realloc(&self, p: *mut u8, l: Layout, s: usize) -> *mut u8
- compare · function · L35-L58 — fn compare(text: &str, q: &SessionSearchQueryProfile)
- exact_legacy_custom_profiles · function · L60-L91 — fn exact_legacy_custom_profiles()
- cpu · function · L93-L103 — fn cpu() -> u128
- search_traversal_probe · function · L107-L229 — fn search_traversal_probe()
