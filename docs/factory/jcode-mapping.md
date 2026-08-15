# Jcode mapping

> Status: observed implementation and proposed gaps

## Observed

- Agent turn runtime, tools, providers, recovery, streaming, and compaction.
- Semantic memory, explicit memory tools, session search, and consolidation.
- Swarm sessions, messaging, task graphs, deep-mode gates, and headless workers.
- Ambient scheduling, overnight work, command-center projections, and durable workflow state.
- Browser, MCP, communication, skills, and verification surfaces.

## Partial or proposed

- One provider-neutral task/run/artifact contract across all workers.
- Worktree, container, VM, and remote execution adapters.
- First-class artifact and evidence lineage.
- Unified gate, approval, merge, PR, deploy, and rollback lifecycle.
- Trajectory evaluation and regression corpus integration.
- Complete OpenSpec/Beads and command-center authority integration.

Primary paths include `crates/jcode-app-core/src/agent/`, `crates/jcode-plan/src/dag/`, `crates/jcode-command-center/`, `crates/jcode-base/src/memory.rs`, `crates/jcode-app-core/src/ambient/`, `crates/jcode-app-core/src/tool/`, and `skills/verification-before-completion/`.
