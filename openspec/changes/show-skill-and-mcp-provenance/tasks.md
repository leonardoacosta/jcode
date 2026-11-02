## 1. Implementation
- [ ] 1.1 Add provenance fields to `Skill` populated by each loader root in `skill.rs`; track shadowed roots on override
- [ ] 1.2 Add `/skill info <name>` detail output and compact root label in selector rows
- [ ] 1.3 Show MCP tool server/scope from MCP manager data; "unknown" fallback
- [ ] 1.4 Extend remote skill payload additively

## 2. Verification
- [ ] 2.1 Tests: override shadowing, symlink, not-found, control text in paths, MCP unknown scope, widths 0/1/20/40/80
- [ ] 2.2 `cargo test -p jcode-base skill` and `cargo test -p jcode-tui`
- [ ] 2.3 `openspec validate show-skill-and-mcp-provenance --strict`
