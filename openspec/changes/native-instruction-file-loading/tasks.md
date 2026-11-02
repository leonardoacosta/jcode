## 1. Loader

- [x] 1.1 Add `InstructionSource { layer, path, status, chars }` list to `ContextInfo` in `crates/jcode-base/src/prompt.rs`
- [x] 1.2 Load `~/.agents/AGENTS.md` shared layer in both prompt paths (fresh and AGENTS.md snapshot)
- [x] 1.3 CLAUDE.md pairing plus canonical-path and content-hash dedupe
- [x] 1.4 Unit tests: shared present/missing, CLAUDE-only, identical, pointer, differing, `$HOME` cwd

## 2. Views

- [x] 2.1 `/context instructions` in `crates/jcode-tui` (truncate per TUI style rules)
- [x] 2.2 `jcode debug instructions [--dir]` CLI subcommand
- [x] 2.3 Tests for CLI success and invalid directory

## 3. Verify

- [ ] 3.1 Selfdev build; runtime check on an isolated socket that the shared layer appears in `/context instructions`
