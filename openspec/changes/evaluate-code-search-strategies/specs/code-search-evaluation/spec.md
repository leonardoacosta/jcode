## ADDED Requirements

### Requirement: Fixed query set with ground truth
The evaluation SHALL use 18-22 queries over a pinned jcode commit, each with known answer paths and symbols, mixing exact-symbol, concept, caller, and cross-crate queries.

#### Scenario: Reproducible set
- **WHEN** the harness runs at the pinned commit
- **THEN** every query's answers exist at the listed paths

### Requirement: Comparable metrics per strategy
Each strategy (agentgrep, graft, ast-grep, search subagent, repo map) SHALL be measured on tool calls, tokens, recall@1, recall@5, and wall time per query.

#### Scenario: Missing tool
- **WHEN** a strategy is unavailable on the host
- **THEN** it is marked not run with the reason, not scored zero

### Requirement: Results and decisions
The change SHALL produce a results table and an adopt or reject decision with rationale for each strategy, including the Empryo-style repo map.

#### Scenario: Decision recorded
- **WHEN** results are complete
- **THEN** each strategy has exactly one of adopt or reject plus the metric evidence
