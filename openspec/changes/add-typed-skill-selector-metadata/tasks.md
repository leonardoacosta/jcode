## 1. Implementation
- [ ] 1.1 Introduce a typed candidate struct (kind, name, description) in `state_ui_input_helpers.rs` and migrate `command_candidates`/`rank_suggestions`
- [ ] 1.2 Use `Skill.description` for local skill rows; sanitize control characters
- [ ] 1.3 Add optional descriptions to the remote skill payload; default to "unknown" for legacy peers
- [ ] 1.4 Render kind label with distinct style and ellipsis truncation by display cells

## 2. Verification
- [ ] 2.1 Unit tests: descriptions, legacy-peer unknown, collision precedence, control text, widths 0/1/20/40/80
- [ ] 2.2 `cargo test -p jcode-tui skill_startup ssh_remote`
- [ ] 2.3 Visual check with `debug_socket` tester frames, local and remote
- [ ] 2.4 `openspec validate add-typed-skill-selector-metadata --strict`
