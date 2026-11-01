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
Spawned swarm workers SHALL receive the coordinator's resolved workspace and SHALL NOT re-resolve it from their own working directory.

#### Scenario: Worker in different cwd
- **WHEN** a worker is spawned with a `working_dir` outside any matched workspace
- **THEN** the worker uses the coordinator's resolved workspace context
