## ADDED Requirements

### Requirement: Preserve source and history before reconciliation
The executor SHALL preserve existing main history and recoverable copies of all local refs, dirty source, untracked content and stashes before integration.

#### Scenario: Dirty files are absent from Git bundles
- **WHEN** a source worktree contains uncommitted or untracked content
- **THEN** separate protected backups SHALL be verified by restoration before that worktree is modified

#### Scenario: Histories have no common ancestor
- **WHEN** graph inspection confirms unrelated v0.77 and v0.88 histories
- **THEN** reconciliation SHALL preserve original main lineage and explicitly review the merged v0.88 tree without resetting main or creating another integration branch

### Requirement: Account for every local work item
The executor SHALL inventory all local branches, patches and stashes and integrate completed user-authored custom work by default.

#### Scenario: Complete work conflicts with v0.88
- **WHEN** an otherwise complete custom change needs API adaptation
- **THEN** the executor SHALL resolve compatibility and verify original behavior rather than discard the feature solely because of conflicts

#### Scenario: Work is unfinished or duplicated
- **WHEN** an item is unfinished, already included or superseded
- **THEN** its disposition SHALL include recovery instructions or duplicate/supersession evidence and SHALL NOT silently lose unique behavior

### Requirement: Integrate sequentially on existing main
The executor SHALL use the existing main worktree and dependency-ordered verified commits, protecting concurrent changes.

#### Scenario: A batch fails or source drifts
- **WHEN** checks fail or another actor changes the recorded source state
- **THEN** the executor SHALL stop the affected operation, preserve diagnostics and unrelated edits, and refrain from promotion until resolved

### Requirement: Validate final source and runtime provenance
The executor SHALL validate the combined main and build from clean committed source with unchanged pre/post build identity before deployment.

#### Scenario: Previous individual tests passed
- **WHEN** previously tested branches are combined
- **THEN** applicable full suites and public integration checks SHALL run again on combined main, with unresolved release blockers preventing promotion

#### Scenario: Publisher rejects source identity
- **WHEN** a source or binary identity check fails
- **THEN** the executor SHALL rebuild or correct the source-specific supported workflow, not fabricate metadata to bypass the check

#### Scenario: Deployment fails
- **WHEN** the promoted daemon fails startup or required behavior
- **THEN** the executor SHALL restore the recorded previous deployment through supported promotion/reload and verify recovery

### Requirement: Remove only redundant inactive worktrees
The executor SHALL remove a worktree only after confirming its work and artifacts are accounted for, recoverable, and unused by active sessions.

#### Scenario: Deferred custom work remains
- **WHEN** a candidate worktree contains unfinished unique work
- **THEN** the executor SHALL retain it until verified recovery storage and remaining-task records exist, and SHALL preserve that recovery material after cleanup
