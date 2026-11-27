# crates/jcode-voice/src/resample.rs

- Resampler · struct · L5-L13 — pub(super) struct Resampler
- new · function · L15-L30 — pub fn new(rate: u32) -> Result<Self, VoiceError>
- push · function · L31-L46 — pub fn push(&mut self, sample: f32, out: &mut Vec<i16>)
- emit · function · L47-L73 — fn emit(&mut self, out: &mut Vec<i16>)
- finish · function · L75-L79 — pub fn finish(&mut self, out: &mut Vec<i16>)
- tests · module · L83-L124 — mod tests
- tone · function · L85-L96 — fn tone(rate: u32, hz: f64) -> Vec<i16>
- rms · function · L97-L99 — fn rms(x: &[i16]) -> f64
- duration_and_antialias_at_native_rates · function · L101-L111 — fn duration_and_antialias_at_native_rates()
- finite_bounded_and_chunk_independent · function · L113-L123 — fn finite_bounded_and_chunk_independent()
