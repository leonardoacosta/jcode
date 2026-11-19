## ADDED Requirements

### Requirement: Per-workspace hooks
A workspace rule SHALL accept a `hooks` table using the lifecycle hook schema, and its entries SHALL apply in addition to global hooks only for sessions resolved to that workspace.

#### Scenario: Workspace hook scoped
- **WHEN** the Brown workspace defines a `PreToolUse` hook and the session is in a personal repo
- **THEN** the Brown hook does not run

### Requirement: Workspace swarm model overrides
A workspace rule SHALL accept `swarm_model` and role-slot model overrides, applied with precedence explicit spawn model, then workspace override, then global `agents.swarm_model`.

#### Scenario: Workspace override used
- **WHEN** a coordinator in a Priceless workspace with `swarm_model` set spawns a worker without `model`
- **THEN** the worker uses the workspace `swarm_model`

#### Scenario: Explicit model wins
- **WHEN** the spawn passes an explicit `model`
- **THEN** the worker uses the explicit model

### Requirement: Workspace propagation to workers
Spawned swarm workers SHALL receive coordinator context as provenance, but SHALL verify the actual target repository/workspace before applying executable hooks, tracker hints, identity or model overrides. Verified worktrees of the same repository can retain the resolved identity. A different or unknown target SHALL NOT silently inherit another workspace's rules or authority.

#### Scenario: Verified same-repository worktree
- **WHEN** a worker's target is verified as a worktree of the same repository
- **THEN** it can inherit the matched context without treating the pathname alone as identity

#### Scenario: Worker in different cwd
- **WHEN** a worker is spawned with a `working_dir` outside any matched workspace
- **THEN** the worker exposes an unknown target or mismatch and does not apply the coordinator's workspace hooks, tracker or identity hints to that target
