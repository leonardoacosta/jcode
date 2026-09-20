## Context

Users frequently retype or search for commands they've run before. Standard zsh history completion (Tab on partial match) only finds exact prefix matches. `mrnugget/jev-shell-history` (63 stars, MIT) demonstrates Jev-powered fuzzy matching: send the buffer prefix + shell history to Jev, get back the best semantic match.

The evaluate tool provides the Jev API integration (Path B: Rust re-implementation). Path A (bundled Go binary) is independent and can proceed without the evaluate tool.

## Goals / Non-Goals

**Goals:**
- Jev-powered fuzzy history matching for shell commands
- `jcode shell-history` CLI subcommand (Path B) or bundled Go binary (Path A)
- zsh widget registration via `jcode init` — Tab key integration
- Silent fallback: Jev unavailable → standard zsh completion
- Privacy opt-out and history depth configuration

**Non-Goals:**
- Not a general-purpose shell autocomplete
- Not overriding standard completion when it has results
- Not implementing a full shell history search UI
- Not supporting fish, powershell, or other shells (zsh only initially, bash optional)

## Decisions

**Choice-based matching:** Jev receives the buffer prefix + deduplicated history as a Choice question. Jev picks the best match from the history entries. This is more reliable than Noul (which would need a question per history entry) and faster than Score (which would need to score all entries).

**Silent fallback:** Jev unavailable, timeout, no API key, confidence < 0.5, empty buffer, history ≤ 2 entries → return nothing, let standard zsh completion handle it. The user experiences this as "Tab didn't help for this command" — no error message, no disruption.

**Widget integration:** zsh widget (`_jev_history_complete`) bound to Tab key. Widget calls `jcode shell-history --buffer="$BUFFER"`. On result: replaces buffer with matched command. On no result: calls `zle expand-or-complete` (standard zsh completion). Standard completion with exact prefix matches wins — Jev only fires when standard has nothing.

**Privacy-first:** Shell history is sent to TypeSafe API. This is documented in the tool description. Opt-out via `shell_history.enabled false`. History depth limit via `shell_history.max_entries` (default 200). Users must be aware of the privacy implications.

**Two paths:** Path B (Rust re-implementation via evaluate tool) is preferred for consistency with Jcode architecture. Path A (bundled Go binary) is a lower-effort alternative that doesn't require the evaluate tool.

## Risks / Trade-offs

- **Risk:** Shell history may contain secrets (API keys, tokens, passwords). Mitigation: documented privacy note; opt-out; history depth limit. Users must make informed decisions.
- **Risk:** Jev may return an incorrect or dangerous match. Mitigation: confidence threshold (0.5); user reviews the command before pressing Enter; standard completion wins when it has exact matches.
- **Trade-off:** Path B requires the evaluate tool dependency. Path A requires a Go build step. Justification: Path B is more consistent with Jcode architecture; Path A is a fallback for environments without the evaluate tool.