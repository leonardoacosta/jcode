---
execution:
  version: 1
  depends_on: ["add-jevsdk-evaluate-tool"]
---

# jcode-jevy-shell-history

## Summary

Add a Jev-driven shell history completion feature. When the user presses Tab in zsh with a partial command, Jev selects the best matching command from the shell history. The feature can be implemented either by bundling the `mrnugget/jev-shell-history` Go binary or by re-implementing the logic in Rust as a `jcode shell-history` subcommand.

This is a **BUNDLE** priority — lowest effort, lowest risk, immediate quality-of-life improvement.

## Motivation

Shell history completion is a solved problem for exact prefix matching (Ctrl+R). But fuzzy matching — "that docker command I ran last week to clean up volumes" — requires the user to remember enough of the command to find it. Jev can select the best match from history given a natural-language prefix or partial command in ~300ms for ~$0.0001, providing smart Tab completion with no setup beyond registering a zsh widget.

## Actors

- **Jcode user** (human): types partial command in zsh, presses Tab
- **Jev shell-history tool**: sends prefix + recent history → Jev returns best match
- **zsh**: widget registered by `jcode init` that intercepts Tab

## Scope

- Zsh widget registered on `jcode init` (or manual setup)
- On Tab: capture current command line buffer + recent shell history
- Send to Jev: Choice question over recent commands matching the prefix context
- Insert selected command into the shell buffer
- Configurable history depth (default: last 200 commands)
- Configurable debounce (default: immediate on Tab, no streaming)

## Non-scope

- Not supporting bash, fish, or other shells (initial scope: zsh only)
- Not supporting streaming (word-by-word as user types) — Tab-triggered only
- Not sending commands to remote servers — Jev request contains history text, handled per TypeSafe privacy policy
- Not a shell built-in — stays as a Jcode subcommand or bundled binary

## Behavior

### Happy path

1. User types: `docker cl` → presses Tab
2. Widget captures: buffer = "docker cl", history = last 200 commands
3. Jev request:
   ```
   state: {buffer: "docker cl", history: ["docker compose up -d", "docker ps", "docker clean volumes --force", ...]}
   questions: {
     match: {type: "choice", criteria: {
       "cmd_0": "docker compose up -d",
       "cmd_1": "docker ps",
       "cmd_2": "docker clean volumes --force"
     }}
   }
   ```
4. Jev returns: match = "cmd_2" (docker clean volumes --force), confidence = 0.87
5. Widget replaces buffer with: `docker clean volumes --force`
6. User reviews and presses Enter to execute

### Edge cases

- **No good match**: Jev confidence < 0.5 → do nothing, let zsh fallback kick in
- **Exact prefix match exists**: standard zsh completion wins (Jev only triggers when standard completion has no results)
- **Empty history**: return "no history available"
- **Jev unavailable**: fall back to standard zsh completion silently
- **Command buffer is empty**: do nothing
- **History contains secrets**: user must be aware history text is sent to TypeSafe API

## Design

### Integration paths

**Path A: Bundle Go binary**
- Copy `mrnugget/jev-shell-history` into bundled servers
- Register zsh widget on `jcode init`
- Widget calls `jev-shell-history` binary, which handles Jev API call

**Path B: Rust re-implementation**
- New `jcode shell-history` subcommand
- Uses `evaluate` tool for Jev call
- zsh widget calls `jcode shell-history` instead of external binary
- Fewer dependencies, consistent with Jcode architecture

### zsh widget registration

```zsh
# Placed in ~/.zshrc or registered by jcode init
_jev_history_complete() {
  local buffer="$BUFFER"
  if [[ -z "$buffer" ]]; then
    zle expand-or-complete
    return
  fi
  local result=$(jcode shell-history --buffer="$buffer" 2>/dev/null)
  if [[ -n "$result" ]]; then
    BUFFER="$result"
    CURSOR=${#BUFFER}
  else
    zle expand-or-complete
  fi
}
zle -N _jev_history_complete
bindkey '^I' _jev_history_complete  # Tab
```

## Dependencies

- **Depends on**: `add-jevsdk-evaluate-tool` (for Path B — Rust re-implementation)
- **Prior art**: `https://github.com/mrnugget/jev-shell-history` (63 stars, MIT)
- **Priority**: BUNDLE — lowest effort, lowest risk

## Touched capabilities

| Capability | Effect |
|-----------|--------|
| New: `cli/shell_history.rs` (Path B) | `jcode shell-history` subcommand |
| `jcode init` | Register zsh widget |
| Bundled servers | Optional: bundle jev-shell-history Go binary |

## Acceptance

1. User types partial command → Tab → Jev returns best match from history
2. No good match → standard zsh completion runs instead
3. Jev unavailable → Tab works as normal zsh completion
4. Exact prefix match in history → standard completion wins (not overridden)
5. Empty buffer → no Jev call, standard completion
6. Widget registered automatically by `jcode init` (no manual zshrc editing)
7. History depth respects configured limit (default 200)