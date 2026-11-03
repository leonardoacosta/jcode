# worktree-policy Specification

## Purpose
Place and reuse explicitly authorized worktrees under validated repository and creator paths while preserving ownership, local work and manual cleanup control.

## Requirements

### Requirement: Global worktree policy controls placement
Jcode-owned worktree creation SHALL resolve destination and creation authority through one typed global worktree policy. The personal default SHALL follow `~/worktrees/<repo-key>/<creator>/<task>--<unique-id>`, and policy support SHALL NOT automatically create a worktree.

#### Scenario: Authorized destination
- **WHEN** a task has explicit worktree-creation authority and verified repository/creator identity
- **THEN** its canonical destination SHALL be within the effective allowed root and follow the selected layout

#### Scenario: No authority or unsafe path
- **WHEN** creation authority is missing, identity is unknown, or path traversal/symlink resolution escapes the allowed root
- **THEN** creation SHALL be refused without creating a branch or directory outside the policy

### Requirement: Reuse preserves existing work and owners
Worktree reuse SHALL verify repository, branch/base, active consumers and ownership. Conflicts SHALL be reported rather than repaired through reset, clean, stash, branch switching or deletion. Cleanup SHALL default to manual.

#### Scenario: Matching owned worktree
- **WHEN** an existing worktree has matching identity/base and a verified compatible owner state
- **THEN** it MAY be reused under the task's authority without creating another checkout

#### Scenario: Dirty or actively owned worktree
- **WHEN** a candidate worktree contains unrelated changes or has an incompatible live owner
- **THEN** Jcode SHALL preserve it and return a conflict requiring resolution rather than modifying it to fit the task

### Requirement: Project declarations cannot grant host execution
Repository-controlled worktree or setup declarations SHALL NOT authorize host commands, secret copying, broader roots or publication merely by existing in the checkout.

#### Scenario: Untrusted setup declaration
- **WHEN** a repository declares a host setup command without the required user-scoped trust and task authority
- **THEN** admission SHALL identify setup as blocked and SHALL NOT execute that command
