## 1. Implement (reference edits.json; adapt to current main)

- [ ] 1.1 `SessionSortMode` and `cmp_sessions`; `o`/`O` keys; relevance sort on search
- [ ] 1.2 Compact 3-line rows with display-cell truncation
- [ ] 1.3 Search match count
- [ ] 1.4 Adaptive pane split
- [ ] 1.5 Dynamic PgUp/PgDn step

## 2. Verify

- [ ] 2.1 `cargo test -p jcode-tui --lib session_picker` and `cargo test -p jcode-tui-session-picker`
- [ ] 2.2 debug_socket frames at 70, 100 and 140 columns
- [ ] 2.3 `openspec validate restore-resume-picker-redesign --strict`
