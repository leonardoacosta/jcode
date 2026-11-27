# crates/jcode-tui-style/examples/light_bench.rs

- LIGHT_BG · constant · L8-L8 — const LIGHT_BG: (u8, u8, u8) = (255, 255, 255);
- build · function · L10-L16 — fn build(pairs: &[(Role, &str)]) -> Palette
- flipped_defaults · function · L20-L28 — fn flipped_defaults() -> Palette
- rgb_to_hsl · function · L30-L54 — fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32)
- hsl_to_rgb · function · L56-L76 — fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8)
- hand_tuned · function · L84-L109 — fn hand_tuned() -> Palette
- report · function · L111-L131 — fn report(name: &str, palette: &Palette) -> HarmonyReport
- objective · function · L135-L143 — fn objective(palette: &Palette) -> (u8, u32)
- HUE_SLACK · constant · L147-L147 — const HUE_SLACK: f32 = 18.0;
- tune · function · L155-L217 — fn tune(start: &Palette) -> Palette
- main · function · L219-L237 — fn main()
