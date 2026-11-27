# crates/jcode-app-core/examples/search_projection_probe.rs

- fixtures · module · L15-L15 — mod fixtures;
- oracle · module · L17-L17 — mod oracle;
- projection · module · L19-L19 — mod projection;
- Counting · struct · L23-L23 — struct Counting;
- ENABLE · constant · L24-L24 — static ENABLE: AtomicBool = AtomicBool::new(false);
- CALLS · constant · L25-L25 — static CALLS: AtomicUsize = AtomicUsize::new(0);
- BYTES · constant · L26-L26 — static BYTES: AtomicUsize = AtomicUsize::new(0);
- ALLOCATOR · constant · L28-L28 — static ALLOCATOR: Counting = Counting;
- alloc · function · L30-L36 — unsafe fn alloc(&self, layout: Layout) -> *mut u8
- realloc · function · L37-L43 — unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8
- dealloc · function · L44-L46 — unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
- cpu_ns · function · L48-L58 — fn cpu_ns() -> u128
- execute · function · L59-L65 — fn execute(msg: &StoredMessage, tools: bool, new: bool)
- main · function · L66-L113 — fn main()
