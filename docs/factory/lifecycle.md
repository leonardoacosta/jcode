# Factory lifecycle

> Status: proposed model
> Authority: external agent-pattern research and Jcode workflow evidence

## Stages

1. **Intent:** capture the desired outcome, motivation, scope, and risk.
2. **Specification:** define requirements, acceptance criteria, exclusions, and evidence.
3. **Planning:** decompose work into tasks, dependencies, owners, and gates.
4. **Execution:** run bounded workers in an explicit workspace with approved tools.
5. **Artifacts:** preserve specs, plans, diffs, traces, test results, and decisions.
6. **Gates:** run deterministic checks and route failures or approvals.
7. **Evaluation:** assess outcome and trajectory, not only the final patch.
8. **Delivery:** merge, publish, deploy, or hand off with provenance.
9. **Learning:** classify failures and update specifications, skills, tools, or evals.

The outer sequence should be as deterministic as the task permits. Agents are most valuable inside stages where local decisions are difficult to predefine.

## Transition contract

Every transition should name its input artifact, output artifact, gate, owner, and failure route. A run is not complete merely because a model says it is done.
