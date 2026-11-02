---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P01: Trusted core and acceptance/evidence baseline](../../../docs/plans/PRODUCT_ROADMAP.md#p01-trusted-core-and-acceptanceevidence-baseline)

## Why

AGENTS.md documents that a fresh build is inert until `~/.jcode/builds/shared-server/jcode` is repointed and the daemon restarted, so runtime checks can silently measure old code. The client knows its own `jcode_build_meta::version()` and the server's `remote_server_version` (`crates/jcode-tui/src/tui/app/state_ui.rs:1891-1911`), and publishing already rejects version reports without `git_hash` (`crates/jcode-build-support/src/lib.rs:301-305`). Nothing ties a claimed verification to the source revision and the binary that actually served it.

## What Changes

- Extend `jcode version --json` with the resolved serving daemon path, server git hash and dirty flag when connected.
- Add an evidence helper that captures source `HEAD`, worktree dirty state, client hash, serving server hash and a verdict.
- `/version` shows source, client and server hashes on one row each and flags mismatch.
- Verdicts: `matched`, `mismatched`, `unknown`; unknown is never reported as matched.

## Non-scope

Automatic daemon restart; CI integration; storing evidence in sessions or OpenSpec artifacts.

## Capabilities

### New Capabilities

- `runtime-acceptance-evidence`: see specs/runtime-acceptance-evidence/spec.md.

## Impact

`crates/jcode-build-support`, `crates/jcode-tui/src/tui/app/state_ui.rs`, protocol handshake for server hash. Old servers without a hash report `unknown`.

## Open decisions

1. Where evidence lives. Options: printed only; JSON file under scratch; session metadata. Recommended: printed plus `--json`, because it composes with existing workflows without new storage.
