# jcode-jevy-shell-history — Tasks

Dependency: `add-jevsdk-evaluate-tool` must be complete before Phase 1 begins (Path B: Rust re-implementation). Path A (bundled Go binary) can proceed independently.

Priority: BUNDLE — lowest effort, lowest risk.

## 1. Jev-powered history matching

- [x] 1.1 Implement shell history collection: Create `cli/shell_history.rs` that reads shell history file (configurable path), parses entries (strips zsh timestamps), deduplicates consecutive duplicates, limits to configurable depth (default 200). Verify zsh format parsing, dedup, depth limit, empty history handling.
- [x] 1.2 Implement Jev command matching: `match_command()` sends buffer prefix + history to Jev with Choice (cmd_0..cmd_N) and confidence Noul. Returns match or None if confidence <0.5. Timeout 3s→None. Skip if empty buffer or ≤2 history entries.

## 2. CLI integration

- [x] 2.1 Implement jcode shell-history subcommand: `jcode shell-history --buffer <text>` calls `match_command()`, prints result to stdout or nothing on no-match. Stderr for errors only (must not interfere with zsh widget). Exit 0 on success/no-match, non-zero on error.
- [x] 2.2 Add API key configuration: Reuse `TYPESAFE_API_KEY` or `OPENROUTER_API_KEY` from evaluate tool. No key→silently return no result (no Jev call). Debug log: "Jev shell history: no API key configured, skipping".

## 3. Shell integration

- [x] 3.1 Implement zsh widget registration: Add to `jcode init`: register `_jev_history_complete` zsh widget bound to Tab (and Alt+Tab for explicit Jev). Widget calls `jcode shell-history --buffer="$BUFFER"`, replaces buffer on result, falls back to `zle expand-or-complete` otherwise. Only triggers when standard completion has no results. Detect zsh at init time.
- [ ] 3.2 Implement bash support (optional): Register bash function via `bind -x` with same logic. Document as experimental. Verify same behavior as zsh widget.
- [ ] 3.3 Add user documentation and safety note: Document shell history sent to TypeSafe API. Privacy note in tool description. Opt-out: `shell_history.enabled false`. Depth limit: `shell_history.max_entries` (100).

## 4. Path A alternative — bundled Go binary

- [ ] 4.1 Bundle jev-shell-history Go binary (Path A): Download and bundle `mrnugget/jev-shell-history` binary with platform/build detection.
- [ ] 4.2 Auto-register shell widget via bundled binary (Path A): On `jcode init`, if bundled binary exists, register zsh widget calling it instead of `jcode shell-history`.

## Dependency graph

```
Phase 1: 1.1 → 1.2 → 2.1 → 2.2 → 3.1 → 3.2 → 3.3
Phase 4 (Path A): 4.1 → 4.2 (independent, parallel)
```

Phase 1+2 (core matching + CLI) must complete before Phase 3 (shell integration). Phase 4 (bundled binary) is independent.