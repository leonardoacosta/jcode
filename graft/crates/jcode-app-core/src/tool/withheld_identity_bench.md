# crates/jcode-app-core/src/tool/withheld_identity_bench.rs

- identity · module · L6-L6 — mod identity;
- CountingAllocator · struct · L14-L14 — struct CountingAllocator;
- COUNT · constant · L15-L15 — static COUNT: AtomicBool = AtomicBool::new(false);
- CALLS · constant · L16-L16 — static CALLS: AtomicU64 = AtomicU64::new(0);
- BYTES · constant · L17-L17 — static BYTES: AtomicU64 = AtomicU64::new(0);
- ALLOCATOR · constant · L19-L19 — static ALLOCATOR: CountingAllocator = CountingAllocator;
- alloc · function · L21-L27 — unsafe fn alloc(&self, layout: Layout) -> *mut u8
- dealloc · function · L28-L30 — unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
- realloc · function · L31-L37 — unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8
- legacy · function · L39-L50 — fn legacy(tool: &str, input: &Value) -> u64
- pass · function · L51-L57 — fn pass(f: fn(&str, &Value) -> u64, value: &Value, iterations: usize) -> u128
- allocations · function · L58-L65 — fn allocations(f: fn(&str, &Value) -> u64, value: &Value) -> (u64, u64)
- main · function · L66-L142 — fn main()
