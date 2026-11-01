## Context

Timeout promotion exists in `bash.rs` (`Err(_)` branch of `tokio::time::timeout` in `execute_foreground`; reload path emits `timeout_promoted`). Edit hints live in `flexible_match_hint` (`edit.rs`); apply_patch bails with `Failed to find context` / `Failed to find expected lines`.

## Decisions

- Add one shared pure fn `nearest_context(content, needle_lines) -> Option<String>` in `edit.rs` (pub(crate)), reused by `apply_patch.rs`. Runs only on the failure path, so cost is irrelevant for success.
- Keep existing exact hints (trimmed/indentation) first; nearest context is the fallback.
- Output format: `Closest match near line N:` followed by `N | text` lines, truncated per Open decision 2.

## Risks

- Misleading hints on low-similarity text → minimum score threshold.
- Token bloat → hard caps.
