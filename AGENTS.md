# Repository Guidelines

## Repository Scope

- Jcode Desktop is in a separate repository.

## Development Workflow

- **Use the user's Git identity** - Create commits with the configured
  `user.name` and `user.email`. Do not override them with `Jcode`, `Jcode agent`,
  or a fabricated agent email. Preserve existing contributor attribution when
  integrating work. If no identity is configured, ask rather than inventing one.
- **Welcome pull requests from everyone** - Review contributions on their merits,
  regardless of whether the author is a maintainer, an existing contributor, a
  first-time contributor, or an agent. Good PRs can be merged directly after review
  and validation. Do not require a maintainer-authored rewrite merely because of
  who submitted the change. See `CONTRIBUTING.md` for the contribution policy.
- **Keep work scoped** - Work on your own branch and preserve unrelated work. When
  the user asks you to review or integrate a PR or branch, you may inspect, test,
  and integrate that contribution regardless of author status. Do not pull in
  unrelated branches or merge a PR without user authorization.
- **Validate and commit narrowly** - Validate changes before committing. In a dirty
  workspace, stage and commit only explicitly owned paths, never indiscriminately
  stage everything. Preserve unrelated or active-agent changes and coordinate or
  quiesce their owners before any commit that could include them. Never commit
  secrets. Report the resulting commit ID, or explain why committing was unsafe
  or impossible.

## Install Notes
- `~/.local/bin/jcode` is the launcher symlink used from `PATH`.
- `~/.jcode/builds/current/jcode` is the active local/source-build channel; self-dev builds and `scripts/install_release.sh` point the launcher here.
- `~/.jcode/builds/stable/jcode` is the stable release channel; `scripts/install.sh` installs this and points the launcher here.
- `~/.jcode/builds/versions/<version>/jcode` stores immutable binaries.
- `~/.jcode/builds/canary/jcode` still exists for canary/testing flows, but it is not the primary self-dev install path.
- On Windows, the equivalents are `%LOCALAPPDATA%\\jcode\\bin\\jcode.exe` for the launcher, `%LOCALAPPDATA%\\jcode\\builds\\stable\\jcode.exe` for stable, and `%LOCALAPPDATA%\\jcode\\builds\\versions\\<version>\\jcode.exe` for immutable installs; `scripts/install.ps1` currently installs the stable channel.
- Ensure `~/.local/bin` is **before** `~/.cargo/bin` in `PATH`.

## Verifying a change at runtime

`cargo build` alone proves nothing about behavior. `jcode run` and interactive
sessions are served by the long-lived daemon at
`~/.jcode/builds/shared-server/jcode`, which is a symlink into
`~/.jcode/builds/versions/<version>/`. Until that symlink is repointed and the
daemon restarted (`jcode self-dev --build`), a freshly built binary is inert and
every runtime check silently measures the old code.

To test a change without disturbing the shared daemon or the caller's session,
run your build against its own socket:

```bash
cargo build --profile selfdev
./target/selfdev/jcode run --no-update --socket /run/user/1000/jcode-mytest.sock '<prompt>'
```

Two things that waste time otherwise:

- `crate::logging::info` writes to a log file, not stderr, so instrumenting a
  code path with it produces no visible output under `--trace`. Use `eprintln!`
  for throwaway diagnostics and delete it before committing.
- Confirm which binary you are actually inspecting. `strings` on
  `builds/shared-server/jcode` reads a 70-byte symlink, not a program; resolve it
  with `readlink -f` first.

## Optional local agent memory and code navigation

- **Graft** is an optional local-first memory tool for reusable fixes, decisions,
  and project gotchas. When `graft` is installed, use `graft query` for a quick
  confidence-gated prior-solution check, `graft retrieve` for ranked hybrid
  recall, and `graft explore` only when connected context is useful. Treat
  retrieved memories as hints and verify them against current source. Do not
  install Graft, initialize profiles, or write memories unless asked.
- **Blink** (`ellipsis-dev/blink`) is an optional Jev-powered codebase path
  search, not a memory store. When `blink` is installed, use it to rank likely
  files for a natural-language location question; use `--recursive` or
  `--n_walkers N` only when a top-level scan is insufficient. Verify results
  by reading the files. It requires a configured TypeSafe API key and makes
  hosted Jev requests. Do not use it when credentials are unavailable, when
  network use is out of scope, or when ordinary `agentgrep` is enough.
