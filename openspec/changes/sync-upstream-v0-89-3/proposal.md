## Why

Local Jcode reports v0.88.176 while upstream has released v0.89.3. Restore upstream ancestry without discarding local features or rewriting published fork history so future updates remain ordinary merges.

## What Changes

- Fetch and verify upstream v0.89.3 from https://github.com/1jehuang/jcode.git.
- Preserve a recovery ref and merge the pinned release into existing local main, without an additional integration branch.
- Resolve conflicts by preserving local customizations and incorporating compatible upstream changes.
- Configure an upstream remote tracking upstream master without changing main's origin/main tracking.
- Validate merged source and isolated runtime behavior before any local deployment.

## Capabilities

### New Capabilities
- `upstream-history-sync`: Recoverable, history-preserving upstream integration and future-update configuration.

### Modified Capabilities
None intentionally. Compatibility fixes must retain existing local behavior.

## Impact

Git ancestry, upstream remote configuration, conflicting source and tests, build provenance. No force-push, remote push, reset, rebase, unrelated cleanup, or automatic deployment.

## Approval status

User approved the history-preserving merge scope on 2026-09-30. Written artifacts await review before execution.
