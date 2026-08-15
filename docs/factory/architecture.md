# Factory architecture

> Status: proposed model with observed Jcode foundations

```text
Agent kernel → worker runtime → workflow orchestrator → gates/evals → command center
```

The **agent kernel** owns turns, context, providers, tools, streaming, recovery, and compaction. The **worker runtime** adds a task contract, workspace, permissions, budgets, and an evidence bundle. The **orchestrator** manages initiatives, task graphs, dependencies, retries, and resumption. **Gates and evaluations** decide whether a transition is safe. The **command center** projects durable state for supervision.

**Observed Jcode evidence:** `crates/jcode-app-core/src/agent/`, `crates/jcode-plan/src/dag/`, `crates/jcode-command-center/`, swarm communication, memory, ambient scheduling, and verification skills.

**Proposed gap:** unify these surfaces into a provider-neutral task/run/artifact lifecycle across local and remote workers.
