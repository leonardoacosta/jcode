# crates/jcode-productivity-core/src/lib.rs

- aggregate · module · L9-L9 — mod aggregate;
- dashboard · module · L10-L10 — mod dashboard;
- markdown · module · L11-L11 — mod markdown;
- model · module · L12-L12 — mod model;
- scan · module · L13-L13 — mod scan;
- ProductivityOutput · struct · L21-L29 — pub struct ProductivityOutput
- generate · function · L33-L44 — pub fn generate() -> Result<ProductivityOutput>
- compute_report · function · L47-L50 — pub fn compute_report() -> Result<ProductivityReport>
- render_markdown · function · L53-L55 — pub fn render_markdown(report: &ProductivityReport) -> String
- render_png · function · L58-L60 — pub fn render_png(report: &ProductivityReport) -> Result<Vec<u8>>
- render_svg · function · L63-L65 — pub fn render_svg(report: &ProductivityReport) -> String
- save_png · function · L67-L74 — fn save_png(png: &[u8]) -> Result<PathBuf>
- report_from_summaries · function · L77-L79 — pub fn report_from_summaries(summaries: Vec<SessionSummary>) -> ProductivityReport
- tests · module · L82-L82 — mod tests;
