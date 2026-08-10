# Build, reload, and commit-deploy workflow

Jcode separates building a binary from switching running processes onto it.

## Existing actions

| Surface | Action | Behavior |
|---|---|---|
| TUI | `/reload` | Re-exec the client onto the newest binary already on disk. Does not build. |
| TUI | `/rebuild` | Pull, build, run release checks in the background, then reload the client when idle. |
| CLI | `jcode server reload` | Reload the shared server onto a newer installed binary. Live server sessions are handed over. |
| Script | `scripts/install_release.sh --fast` | Build, install the immutable version, update `stable`/`current`, and run `jcode server reload`. |

There is no separate top-level `jcode rebuild` CLI subcommand. `/rebuild` is an
interactive TUI action; the non-interactive equivalent is
`scripts/install_release.sh --fast`.

## Post-commit deployment

Install the hook once per clone:

```bash
scripts/install_deploy_hook.sh
```

The installer also enables this live-reload setting in
`$JCODE_HOME/config.toml` (normally `~/.jcode/config.toml`):

```toml
[display]
auto_client_reload = true
```

After installation, a commit touching buildable paths (`crates/`, `src/`, root
`build.rs`, Cargo manifests/lockfile, the Rust toolchain file, or `.cargo/`)
queues a background deployment. Documentation-only commits do not build.

The worker:

1. records the exact committed SHA;
2. builds that SHA in a clean detached worktree (never uncommitted worktree
   changes);
3. shares a Cargo target cache for incremental builds;
4. installs the immutable version and updates `stable`/`current`;
5. reloads the shared server;
6. causes opted-in active clients to re-exec immediately when idle, or queues
   the client reload until the current turn finishes.

Rapid commits are coalesced through a single request slot. A commit arriving
during a build is not lost: the worker drains the newest requested SHA before
exiting.

Deployment logs are written to `target/deploy.log`. To skip deployment for one
commit:

```bash
JCODE_NO_DEPLOY=1 git commit ...
```

To deploy the current committed SHA regardless of which paths its commit
touched (or to retry after a failed hook run):

```bash
JCODE_DEPLOY_FORCE=1 scripts/post_commit_deploy.sh
```

Self-dev/canary clients always auto-reload. Regular clients preserve the prior
manual `/reload` behavior unless `display.auto_client_reload` is enabled.
