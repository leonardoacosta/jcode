## 1. Policy and classification
- [x] 1.1 Add `[sessions.retention]` config with defaults (`enabled=false`, `auto_gc=false`, `debug_max_age_days=7`, `tiny_max_messages=3`, `tiny_max_age_days=14`)
- [x] 1.2 Implement pure `classify` in `crates/jcode-base/src/session.rs`, keep rules first
- [x] 1.3 Unit tests: saved+debug+tiny+old kept; old debug candidate; recent debug kept; unparseable kept; live kept

## 2. GC command
- [x] 2.1 Add `jcode sessions gc [--apply] [--json]`; dry-run by default
- [x] 2.2 Refuse `--apply` when `enabled=false` (non-zero exit naming key)
- [x] 2.3 Re-check `saved` from disk before each delete
- [x] 2.4 Integration test on temp dir: dry-run changes nothing; apply removes only candidates

## 3. Auto GC and search index
- [x] 3.1 Startup GC only when `enabled && auto_gc`
- [x] 3.2 Exclude `is_debug` sessions from recent search index unless `debug: true`
- [x] 3.3 Drop deleted ids from index; test in `session_search_tests.rs`

## 4. Verify
- [x] 4.1 `cargo test -p jcode-base -p jcode-app-core` passes
- [x] 4.2 Dry-run against the real store; confirm zero saved sessions listed
