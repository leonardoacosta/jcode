# crates/jcode-render-core/src/wrap.rs

- WidthMeasure · interface · L11-L21 — pub trait WidthMeasure
- measure · function · L13-L13 — fn measure(&self, text: &str) -> usize;
- measure_char · function · L17-L20 — fn measure_char(&self, ch: char) -> usize
- ColumnWidth · struct · L25-L25 — pub struct ColumnWidth;
- measure · function · L28-L31 — fn measure(&self, text: &str) -> usize
- measure_char · function · L33-L36 — fn measure_char(&self, ch: char) -> usize
- wrap_line · function · L44-L134 — pub fn wrap_line<M: WidthMeasure>(
- wrap_lines · function · L137-L146 — pub fn wrap_lines<M: WidthMeasure>(
- push_token · function · L148-L167 — fn push_token(cur: &mut Vec<StyledSpan>, src: &StyledSpan, text: &str)
- tokenize · function · L170-L189 — fn tokenize(text: &str) -> Vec<&str>
