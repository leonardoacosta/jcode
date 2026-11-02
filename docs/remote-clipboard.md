# Remote clipboard and links

Use this when the TUI runs on one machine (for example `homelab` under
`herdr --remote`) and you view it from another (Ghostty, Tern, Moshi).

## How copying works

- **Remote session:** `HERDR_ENV`, `SSH_CONNECTION`, `SSH_TTY`,
  `MOSH_SERVER_PID` or `MOSH_CONNECTION` is set. The TUI copies only through
  OSC 52, which asks your terminal to set its own clipboard. It never writes
  the host's clipboard.
- **Override:** `JCODE_CLIPBOARD=osc52` forces OSC 52. `JCODE_CLIPBOARD=native`
  forces the host clipboard.
- **Opening a link remotely:** the URL is copied through OSC 52 and shown in
  the status line. If `JCODE_LINK_OPEN_SSH_HOST` (or
  `~/.jcode/link_open_ssh_host`) names a Mac, the TUI also opens it in Chrome there.

## Commands

- `/links` lists links in the transcript, newest first. `/links <n>` copies
  one exact URL, with no line breaks from wrapping.
- `/mouse` switches mouse capture off so your terminal's own drag selection
  works. Run it again to turn capture back on.

## Probe

Run this in the same pane, then paste on the client:

```bash
printf '\e]52;c;%s\a' "$(printf hi | base64)"
```

If `hi` doesn't paste, OSC 52 is dropped somewhere along the chain, so no TUI
setting can fix it.

## Matrix

Record results here after testing on real devices.

| Client | SSH | mosh |
|---|---|---|
| Ghostty (macOS) | unverified | unverified |
| Tern (iOS) | unverified | unverified |
| Moshi (iOS) | unverified | unverified |

Known limits: older mosh releases drop OSC 52, and some clients cap its size.
Herdr must forward OSC 52 to the attached client.
