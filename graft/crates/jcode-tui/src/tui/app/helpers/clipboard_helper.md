# crates/jcode-tui/src/tui/app/helpers/clipboard_helper.rs

- copy_via_clipboard_helper · function · L17-L62 — pub(crate) fn copy_via_clipboard_helper(program: &str, args: &[&str], text: &str) -> bool
- tests · module · L69-L103 — mod tests
- helper_that_exits_successfully_counts_as_a_copy · function · L73-L75 — fn helper_that_exits_successfully_counts_as_a_copy()
- helper_that_exits_nonzero_falls_through · function · L78-L80 — fn helper_that_exits_nonzero_falls_through()
- missing_helper_binary_falls_through · function · L83-L89 — fn missing_helper_binary_falls_through()
- long_lived_helper_counts_as_a_copy_without_blocking · function · L94-L102 — fn long_lived_helper_counts_as_a_copy_without_blocking()
- ordering_tests · module · L112-L208 — mod ordering_tests
- first_helper_that_wins · function · L119-L130 — fn first_helper_that_wins() -> Option<&'static str>
- StubPath · struct · L134-L137 — struct StubPath
- with · function · L140-L154 — fn with(stubs: &[(&str, &str)]) -> Self
- drop · function · L158-L163 — fn drop(&mut self)
- wayland_still_wins_when_wl_copy_works · function · L169-L173 — fn wayland_still_wins_when_wl_copy_works()
- xclip_takes_over_when_wl_copy_fails · function · L178-L186 — fn xclip_takes_over_when_wl_copy_fails()
- xsel_takes_over_when_wl_copy_and_xclip_are_missing · function · L189-L193 — fn xsel_takes_over_when_wl_copy_and_xclip_are_missing()
- no_working_helper_reports_failure_so_later_fallbacks_run · function · L199-L207 — fn no_working_helper_reports_failure_so_later_fallbacks_run()
