## Why

Deployment was mistaken for source consolidation. Local main remains on v0.77 while the deployed v0.88 source and custom work are split across two repositories. The goal is one local main containing completed user-authored work, with unfinished work recoverable rather than discarded.

## What Changes

- Preserve existing local main history and reconcile the v0.88 history into it without another integration branch.
- Inventory all local branches, working changes, untracked files and stashes in both repositories. Account for every item before cleanup.
- Integrate completed custom work by default, resolve compatibility against v0.88, and defer half-finished work with recovery instructions.
- Validate the combined main, build from its clean committed source, test in isolation, then deploy with rollback retained.
- Remove only worktrees proven redundant and unused by active processes.

## Capabilities

### New Capabilities
- `local-main-consolidation`: Recoverable reconciliation of split local source into a verified canonical main.

### Modified Capabilities
None prescribed. Existing custom capabilities must be inventoried and preserved or explicitly deferred, not silently redefined by this proposal.

## Impact

Existing main worktree, both local Git histories, completed custom source, tests, build provenance, and local deployment channels. No remote push, force reset, new integration branch, new feature development, or automatic completion of unfinished work.

## Approval status

History preservation, inclusion policy, integration sequence, and final validation/deployment/cleanup gates were approved in conversation on 2026-09-26. This consolidated written artifact awaits user review before execution through `apply`.
