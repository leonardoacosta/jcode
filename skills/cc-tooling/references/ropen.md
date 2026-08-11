# Opening Files in a Browser (*open family)

**Ownership:** The whole `*open` family — `ropen`/`sopen`/`gopen`/`mopen` (VIEW, portal-aware)
and `copen`/`vopen`/`zopen` (EDIT, portal-unaware) — is owned and chezmoi-deployed by `if`
(`~/dev/personal/installfest`), not cc. Source of truth: `if`'s `docs/open-family.md`. cc only calls these
commands by name; it does not build, vendor, or symlink them onto `$PATH`.

**What cc workflows actually use:**
- `ropen <file>` — the one most cc commands/skills call. Fire-and-forget: registers a mount and
  dispatches a Mac browser open (falling back through Atlas -> the live-mount server), exiting
  in ~30ms. Use whenever Leo needs to *see* a file (HTML review pages, visual explainers,
  rendered Markdown) from this headless Arch box.
- `mopen <file>` — same idea, but posts a clickable Nexus desktop notification instead of
  auto-opening. No live-reload watcher (a notification is one-shot).
- `gate.sh` Stage C auto-rewrites `xdg-open` -> `ropen` by literal command-name substitution
  (no path involved), so `xdg-open file.html` works too.

**Don't:**
- Wrap `ropen <file>` in `Bash({run_in_background: true})` — it already exits in ~30ms.
  Backgrounding it adds noise without value.
- Write HTML to `/tmp/` and expect `ropen` to serve it — it serves from the file's git root, so
  files outside a repo won't have a stable URL. Keep generated viewers in the project's `docs/`.
- Assume cc owns the CLI binaries, the systemd unit (`ropen.service`), or the live-mount
  server — all three live in `if` now.

For server lifecycle, the Atlas portal fallback contract, env vars (`ATLAS_BASE_URL`,
`ROPEN_PORT`, etc.), or the VIEW/EDIT resolution order, read `if`'s `docs/open-family.md`
directly rather than re-deriving it here.
