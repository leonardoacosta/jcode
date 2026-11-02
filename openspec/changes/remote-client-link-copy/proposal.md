---
execution:
  version: 1
  depends_on: []
---
## Why

Copying or opening links from the TUI keeps failing when it runs on `homelab` under `herdr --remote` and the person viewing it is on Ghostty, Tern, or Moshi. Copies and opens default to the host. Native clipboard tools succeed on homelab's own display, so the TUI reports "Copied" while the client gets nothing. Fixes have only been made for one route at a time (`5e77f08d8`, `45c6ae220`, `70b8848c2`), so the problem keeps coming back.

## What Changes

- Remote detection: treat the session as remote when `HERDR_ENV`, `SSH_CONNECTION`, `SSH_TTY`, or any `MOSH_` variable is set, or when `JCODE_CLIPBOARD=osc52` is set. In a remote session, copying uses only OSC 52 and never `wl-copy`, `xclip`, or `arboard`.
- A link picker (`/links`) and a copy-last-link key that copy the full, unwrapped destination.
- A key and `/mouse` command that switch mouse capture on or off, so the client's native selection works.
- In a remote session, opening an HTTP(S) link always copies it through OSC 52 and shows the full URL in an unwrapped notice. Chrome on a Mac (`JCODE_LINK_OPEN_SSH_HOST`) remains an optional extra.
- A documented test matrix covering the client, transport, and OSC 52 probe, plus automated tests for the routing decision.

Roadmap: [P09 TUI/onboarding everyday UX](../../../docs/plans/PRODUCT_ROADMAP.md#p09-tuionboarding-everyday-ux), copy/selection and links gate.

## Non-scope

- Changes to Herdr, mosh, or the client apps themselves.
- Reading the clipboard over OSC 52 (paste).

## Touched surfaces

- `crates/jcode-tui/src/tui/app/helpers.rs` (`copy_to_clipboard`, remote detection)
- `crates/jcode-tui/src/tui/app/navigation.rs` (link open routing)
- `crates/jcode-tui/src/tui/app/commands.rs` and the keymap (`/links`, `/mouse`)
- `crates/jcode-tui/src/tui/mod.rs` (mouse capture state)
- `docs/remote-clipboard.md`
