# crates/jcode-app-core/examples/browser_size_probe.rs

- Counting · struct · L9-L9 — struct Counting;
- COUNT · constant · L10-L10 — static COUNT: AtomicBool = AtomicBool::new(false);
- ALLOCS · constant · L11-L11 — static ALLOCS: AtomicUsize = AtomicUsize::new(0);
- BYTES · constant · L12-L12 — static BYTES: AtomicUsize = AtomicUsize::new(0);
- alloc · function · L14-L20 — unsafe fn alloc(&self, layout: Layout) -> *mut u8
- dealloc · function · L21-L23 — unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
- realloc · function · L24-L30 — unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8
- ALLOCATOR · constant · L33-L33 — static ALLOCATOR: Counting = Counting;
- cpu_ns · function · L35-L45 — fn cpu_ns() -> u128
- sample · function · L46-L64 — fn sample(values: &[Value], iterations: usize, count: bool) -> (u128, usize, usize)
- main · function · L65-L124 — fn main()
