## 1. Loader

- [ ] 1.1 Add `InstructionSource { layer, path, status, chars }` list to `ContextInfo` in `crates/jcode-base/src/prompt.rs`
- [ ] 1.2 Load `~/.agents/AGENTS.md` shared layer in both prompt paths (fresh and AGENTS.md snapshot)
- [ ] 1.3 CLAUDE.md pairing plus canonical-path and content-hash dedupe
- [ ] 1.4 Unit tests: shared present/missing, CLAUDE-only, identical, pointer, differing, `$HOME` cwd

## 2. Views

- [ ] 2.1 `/context instructions` in `crates/jcode-tui` (truncate per TUI style rules)
- [ ] 2.2 `jcode debug instructions [--dir]` CLI subcommand
- [ ] 2.3 Tests for CLI success and invalid directory

## 3. Verify

- [ ] 3.1 Selfdev build; runtime check on an isolated socket that the shared layer appears in `/context instructions`
