## Why

Roadmap owner: [P02: Discoverable native tools, MCP and skills provenance](../../../docs/plans/PRODUCT_ROADMAP.md#p02-discoverable-native-tools-mcp-and-skills-provenance). Supporting: P01.

Forensics (60d) show 2269 codex tool timeouts and 348 apply_patch failures. Jcode's `bash` tool already promotes foreground timeouts to background tasks (`execute_foreground` and `execute_reload_persistable_foreground` in `crates/jcode-app-core/src/tool/bash.rs`, metadata `timeout_promoted`), but this behavior is untested on the reload path and not specified. Edit/apply_patch mismatch errors only report "not found" or a whitespace/indentation hint; they never show the nearest actual file text, so agents re-read whole files or retry blindly.

## What Changes

- Specify existing timeout auto-promotion as a contract (no kill, task id, output file, `bg` follow-up instructions) for both foreground paths; add a regression test for the reload-persistable path.
- Explicit `run_in_background=false` timeouts and the background-task timeout (which does kill) stay as-is.
- On `edit` `old_string` mismatch and `apply_patch` context/expected-lines mismatch, append the closest matching region of the current file (line-numbered, bounded) as a nearest-context hint.

## Capabilities

### New Capabilities
- `tool-failure-recovery`: timeout promotion and mismatch nearest-context hints.

## Impact

`crates/jcode-app-core/src/tool/{bash.rs,edit.rs,apply_patch.rs}` and their tests. No new deps, no config.

## Open decisions

1. **Similarity method** (default: line-level score using trimmed-line equality over a sliding window of the old text's line count, tie → earliest). Options: Levenshtein per line (better fuzzy, costlier on large files); `similar` crate if already in tree. Tradeoff: precision vs cost/deps.
2. **Hint size** (default: best window ±2 lines, max 20 lines, 2000 chars; skip files >2 MB). Larger hints cost tokens.
3. **Minimum score** (default: show hint only if ≥40% of lines match; otherwise keep current "use read tool" message) to avoid misleading hints.
