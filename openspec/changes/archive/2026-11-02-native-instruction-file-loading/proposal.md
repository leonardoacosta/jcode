## Why

Roadmap owner: [P07: Memory/context quality and privacy](../../../docs/plans/PRODUCT_ROADMAP.md#p07-memorycontext-quality-and-privacy).

Instruction loading lives in `crates/jcode-base/src/prompt.rs` (`load_agents_md_files_from_dirs`, `load_prompt_overlay_files_from_dir`). Today it loads project `AGENTS.md`, global `~/AGENTS.md` (deduped only by canonical path), and `prompt-overlay.md` files. It does not load `~/.agents/AGENTS.md`, so the user overlay must tell the model to read it manually each session (an extra tool call that may be skipped). Repos pair `CLAUDE.md` with `AGENTS.md` (often identical or a pointer), causing sprawl. Nothing shows which files actually composed the prompt; `ContextInfo` only tracks char counts shown in `/context`.

## What Changes

- Natively load `~/.agents/AGENTS.md` as a "Shared Instructions" layer, before global and project layers.
- Load `CLAUDE.md` only when no sibling `AGENTS.md` exists; when both exist, skip `CLAUDE.md` if it is a symlink, content-identical, or only references `AGENTS.md`; otherwise include it with a distinct header.
- Dedupe all instruction layers by canonical path and normalized content hash.
- Record each candidate file (layer, path, status: loaded / skipped-duplicate / skipped-missing / unreadable, chars) in `ContextInfo`.
- Add an effective-instructions view: `/context instructions` in the TUI and `jcode debug instructions [--dir]` in the CLI.
- The user's overlay pointer to `~/.agents/AGENTS.md` becomes unnecessary (user-owned file; not edited by this change).

## Capabilities

### New Capabilities
- `instruction-loading`: native shared-file loading, paired-file dedupe, effective-instructions view.

## Impact

`crates/jcode-base/src/prompt.rs`, `ContextInfo`, `/context` overlay in `crates/jcode-tui`, CLI dispatch. Prompt content changes once for users with new layers (prompt-cache miss on first session).

## Open decisions

1. **Paired, differing CLAUDE.md.** Recommended default: include both with distinct headers. Alternative: AGENTS.md wins and CLAUDE.md is ignored (smaller context, risks dropping Claude-only rules).
2. **Shared layer path.** Recommended default: fixed `~/.agents/AGENTS.md` plus a config list `instructions.extra_files`. Alternative: fixed path only (simpler, less flexible).
3. **Checkout-scoped files** (`~/.jcode/.agents/AGENTS.md`). Recommended default: not auto-loaded; available via `extra_files`. Alternative: auto-load `.agents/AGENTS.md` in every project root (broader behavior change).
