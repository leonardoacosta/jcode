## 1. Timeout promotion contract

- [ ] 1.1 Add a test in `bash_tests.rs` covering timeout promotion on the reload-persistable foreground path (process alive after timeout, `timeout_promoted` metadata, `bg` instructions in output)
- [ ] 1.2 Confirm a foreground promotion's message names the task id and output file; verify via existing tests at `bash_tests.rs:314,344,463`

## 2. Nearest-context hints

- [ ] 2.1 Implement `nearest_context` in `edit.rs` with caps and threshold; unit-test best window, ties, below-threshold, huge file skip
- [ ] 2.2 Use it as fallback in `flexible_match_hint`; test edit mismatch output includes line-numbered closest region
- [ ] 2.3 Append it to apply_patch `Failed to find context` and `Failed to find expected lines` errors; test in `apply_patch_tests.rs`
- [ ] 2.4 Run `cargo test -p jcode-app-core tool::` and confirm pass
