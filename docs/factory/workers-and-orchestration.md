# Workers and orchestration

> Status: external research mapped to Jcode

Use the simplest orchestration pattern that meets the task:

- single worker for bounded work;
- sequential workflow for fixed stages;
- parallel sectioning for independent analysis;
- orchestrator-workers when decomposition depends on the task;
- evaluator-optimizer for explicit feedback loops;
- swarm only when coordination produces more value than complexity.

**Observed Jcode:** native swarm coordination, messaging, task DAGs, deep-mode gates, and headless workers. **Proposed:** every worker receives a typed task contract and returns artifacts, evidence, confidence, and open questions.
