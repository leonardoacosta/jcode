# workflow-evidence-and-control Specification

## Purpose
Make the four workflow verbs choose proportionate evidence and causal execution responses without duplicating native plans, adding universal ceremony or changing authorization boundaries.

## Requirements

### Requirement: Decision-relevant claims select proportionate evidence

Explore and Feature MUST identify the decision-relevant claim or question, an observable condition that would falsify it and the minimum safe evidence needed. They MUST distinguish confirmed, contradicted, unknown and unavailable evidence, and expand investigation only for a concrete remaining gap. Static code, runtime observation and shipped-artifact evidence MUST NOT be treated as interchangeable when the claim requires a different observable surface. Ordinary source analysis MUST NOT require reverse-engineering tooling.

#### Scenario: Source inspection cannot establish runtime behavior
- **WHEN** a conclusion depends on actual packaged/runtime behavior that available source evidence cannot establish
- **THEN** the workflow names the required observation, obtains it only within authorized capabilities or reports the conclusion unverified, and does not claim source inspection proves it

#### Scenario: Small source-only question
- **WHEN** an ordinary repository question can be answered by existing code and tests
- **THEN** the workflow uses that evidence without requiring a reverse-engineering service, extra audit framework or unrelated runtime setup

#### Scenario: No safe observable evidence
- **WHEN** the falsifying observation would require unavailable access or an unapproved consequential action
- **THEN** the workflow states the evidence gap and blocked dependent conclusion while continuing useful authorized work, rather than asserting the claim or bypassing consent

### Requirement: Feature checks executable task completeness in native artifacts

Feature MUST check that each task conveys the selected scope, relevant touched surfaces, observable result and verification recipe with an expected result. Recovery/rollback information MUST be included where the task creates a meaningful state or operational risk, not as universal boilerplate. This check MUST reuse the native proposal/spec/design/tasks contract without adding a second task ledger or mandatory extra artifact, and MUST NOT fabricate exact paths or commands that are not yet known.

#### Scenario: Missing verification outcome
- **WHEN** a draft task says to verify behavior without naming an observable outcome or usable verification recipe
- **THEN** Feature corrects the task or records the concrete blocker before presenting it as executable

#### Scenario: Mechanical bounded task
- **WHEN** a small task has clear scope, observable outcome and a relevant existing check with no meaningful rollback requirement
- **THEN** Feature accepts that concise task without demanding a new document, artificial two-minute decomposition or generic rollback section

### Requirement: Execution responds to the cause of a stalled attempt

Apply and Apply-All MUST distinguish missing context, correctness/scope concerns, oversized tasks, insufficient reasoning, invalid plans and temporary external waits when responding to worker reports. They MUST choose a causal next action such as supplying evidence, narrowing scope, escalating reasoning, correcting the authorized plan or reporting the blocker. A failed attempt that produces no relevant new evidence or progress MUST NOT be repeated unchanged. Repeating a safe observation after a justified external wait or because new evidence is expected MUST remain possible without an arbitrary retry-count rule.

#### Scenario: Worker needs context
- **WHEN** a worker cannot proceed because a required file, decision or constraint was absent from its brief
- **THEN** the coordinator supplies the missing verified context or names its resolver instead of merely telling the same worker to try harder

#### Scenario: Unproductive unchanged retry
- **WHEN** an attempt fails again with the same relevant inputs, evidence and action
- **THEN** the coordinator changes the action or scope, seeks new evidence, escalates appropriately or stops the blocked work with a reason

#### Scenario: Justified wait is not a stall
- **WHEN** a safe retry observes a still-running external check and the next observation is justified by its progress/terminal signal
- **THEN** the workflow may wait and observe again without treating the unchanged command alone as an invalid retry or creating an artificial failure quota

### Requirement: Delegation and task review remain selective and attributable

Delegated work MUST carry the selected task scope, relevant files/resources, execution base, constraints, acceptance and evidence needed to act without importing the coordinator's whole transcript. Review MUST cover the actual task's full attributable change range, not assume its final commit contains all work. Extra per-task reviewers or fresh implementers MAY be used when a concrete correctness, security, integration, uncertainty or isolation risk warrants them; they MUST NOT become a universal additional gate. Existing required final independent review and safe nonconflicting parallelism MUST remain intact.

#### Scenario: Multi-commit delegated task
- **WHEN** a worker completes one task in several commits
- **THEN** the review covers its recorded execution base through the returned head, or the equivalent attributable dirty diff, rather than only the last commit

#### Scenario: Low-risk task
- **WHEN** a bounded mechanical task is adequately covered by its check and existing final review
- **THEN** the workflow does not require extra task reviewers or fresh-agent dispatch merely to satisfy the new lens

#### Scenario: Independent Apply-All tasks
- **WHEN** selected ready tasks have verified prerequisites and no shared mutable-resource conflict
- **THEN** existing native parallel execution remains available and a borrowed blanket no-parallel-implementers rule does not serialize them

### Requirement: Harness plans do not replace workflow authority or permission

The four verbs MUST retain native OpenSpec artifacts and `tasks.md` as durable workflow/task authority. Harness-native plans, todos, checkpoints and model handoffs MAY assist execution but MUST NOT create a competing lifecycle, imply completed acceptance or grant permission. A read-only planning request MUST NOT invoke a mode known to begin edits. Harness behavior MUST be described only to the extent supported by current verified documentation or observed runtime evidence; unknown equivalence MUST remain unknown.

#### Scenario: OMP prewalk during planning-only work
- **WHEN** the user authorizes only research or a draft plan
- **THEN** the workflow does not use OMP prewalk as a read-only gate because its documented normal-session handoff can occur after edits begin

#### Scenario: Harness-native plan used for execution
- **WHEN** Jcode's native execution plan or an OMP plan draft is used alongside an OpenSpec change
- **THEN** it is treated as an execution/session view or temporary draft, decisions and tasks are reconciled into native OpenSpec before relying on them, and no tracker or external-action authority is inferred

#### Scenario: Harness semantics not established
- **WHEN** documentation or runtime evidence does not establish a claimed native plan-mode guarantee
- **THEN** the workflow reports that limit and uses the known supported workflow instead of inventing cross-harness equivalence
