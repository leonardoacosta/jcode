# crates/jcode-tui/src/tui/app/tests/onboarding_golden.rs

- render_onboarding_text · function · L22-L48 — fn render_onboarding_text(app: &App, width: u16, height: u16) -> String
- render_onboarding_buffer · function · L51-L68 — fn render_onboarding_buffer(
- svg_escape · function · L70-L76 — fn svg_escape(value: &str) -> String
- svg_color · function · L78-L106 — fn svg_color(color: ratatui::style::Color, fallback: &str) -> String
- onboarding_buffer_svg · function · L111-L171 — fn onboarding_buffer_svg(buffer: &ratatui::buffer::Buffer) -> String
- CELL_W · constant · L115-L115 — const CELL_W: u32 = 12;
- CELL_H · constant · L116-L116 — const CELL_H: u32 = 24;
- FONT_SIZE · constant · L117-L117 — const FONT_SIZE: u32 = 18;
- DEFAULT_BG · constant · L118-L118 — const DEFAULT_BG: &str = "#0b0f14";
- DEFAULT_FG · constant · L119-L119 — const DEFAULT_FG: &str = "#d8dee9";
- write_onboarding_svg · function · L173-L182 — fn write_onboarding_svg(
- write_full_frame_svg · function · L186-L203 — fn write_full_frame_svg(
- app_in_phase · function · L207-L213 — fn app_in_phase(phase: OnboardingPhase) -> App
- dump · function · L215-L219 — fn dump(title: &str, text: &str)
- onboarding_golden_walks_every_phase · function · L222-L393 — fn onboarding_golden_walks_every_phase()
- onboarding_golden_walks_failure_and_async_states · function · L407-L505 — fn onboarding_golden_walks_failure_and_async_states()
- onboarding_golden_telemetry_settings_page · function · L511-L548 — fn onboarding_golden_telemetry_settings_page()
- onboarding_import_happy_path_images · function · L558-L739 — fn onboarding_import_happy_path_images()
