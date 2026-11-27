# crates/jcode-tui/src/tui/ui_smoothness.rs

- recorder · function · L20-L23 — fn recorder() -> &'static Mutex<AnchorStabilityRecorder>
- RECORDER · constant · L21-L21 — static RECORDER: OnceLock<Mutex<AnchorStabilityRecorder>> = OnceLock::new();
- hash_row · function · L27-L64 — fn hash_row(buffer: &Buffer, area: Rect, y: u16) -> u64
- PRIME · constant · L30-L30 — const PRIME: u64 = 0x0000_0100_0000_01b3;
- frame_from_buffer · function · L69-L92 — pub(crate) fn frame_from_buffer(
- observe_frame · function · L95-L109 — pub(super) fn observe_frame(
- report_json · function · L112-L117 — pub(crate) fn report_json() -> String
- reset · function · L120-L125 — pub(crate) fn reset()
