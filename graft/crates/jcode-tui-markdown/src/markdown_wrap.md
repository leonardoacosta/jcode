# crates/jcode-tui-markdown/src/markdown_wrap.rs

- wrap_line · function · L5-L228 — pub fn wrap_line(
- StyledPiece · struct · L231-L234 — struct StyledPiece
- WrapToken · struct · L237-L242 — struct WrapToken
- word_chars · function · L247-L251 — fn word_chars(&self) -> impl Iterator<Item = (char, Style)> + '_
- tokenize_wrap_words · function · L260-L308 — fn tokenize_wrap_words(spans: &[Span<'static>]) -> Vec<WrapToken>
- append_token_spaces · function · L313-L333 — fn append_token_spaces(
- wrap_line_balanced · function · L335-L379 — fn wrap_line_balanced(line: &Line<'static>, width: usize) -> Option<Vec<Line<'static>>>
- tokenize_balanced_wrap · function · L381-L431 — fn tokenize_balanced_wrap(line: &Line<'static>) -> Option<Vec<WrapToken>>
- push_merged_span · function · L436-L444 — fn push_merged_span(spans: &mut Vec<Span<'static>>, text: &str, style: Style)
- push_piece_char · function · L446-L457 — fn push_piece_char(pieces: &mut Vec<StyledPiece>, ch: char, style: Style)
- balanced_wrap_breaks · function · L459-L509 — fn balanced_wrap_breaks(tokens: &[WrapToken], width: usize) -> Option<(Vec<usize>, usize)>
- line_width_for_break · function · L511-L523 — fn line_width_for_break(tokens: &[WrapToken], start: usize, end: usize) -> usize
- build_balanced_line_spans · function · L525-L538 — fn build_balanced_line_spans(tokens: &[WrapToken]) -> Vec<Span<'static>>
- wrap_lines · function · L540-L549 — pub fn wrap_lines(
- progress_bar · function · L551-L558 — pub fn progress_bar(progress: f32, width: usize) -> String
- progress_line · function · L560-L570 — pub fn progress_line(label: &str, progress: f32, width: usize) -> Line<'static>
