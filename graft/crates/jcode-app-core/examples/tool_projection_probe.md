# crates/jcode-app-core/examples/tool_projection_probe.rs

- fixtures · module · L6-L6 — mod fixtures;
- projection · module · L8-L8 — mod projection;
- Counting · struct · L13-L13 — struct Counting;
- ENABLE · constant · L14-L14 — static ENABLE: AtomicBool = AtomicBool::new(false);
- CALLS · constant · L15-L15 — static CALLS: AtomicUsize = AtomicUsize::new(0);
- BYTES · constant · L16-L16 — static BYTES: AtomicUsize = AtomicUsize::new(0);
- ALLOCATOR · constant · L18-L18 — static ALLOCATOR: Counting = Counting;
- alloc · function · L20-L26 — unsafe fn alloc(&self, layout: Layout) -> *mut u8
- realloc · function · L27-L33 — unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8
- dealloc · function · L34-L36 — unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
- baseline · function · L39-L60 — fn baseline(msg: &StoredMessage) -> Cow<'_, str>
- cpu_ns · function · L61-L71 — fn cpu_ns() -> u128
- execute · function · L72-L78 — fn execute(msg: &StoredMessage, new: bool)
- cases · function · L79-L140 — fn cases() -> Vec<(&'static str, StoredMessage)>
- main · function · L141-L185 — fn main()
- tool_serialization_matches_frozen_oracle · function · L187-L195 — fn tool_serialization_matches_frozen_oracle()
