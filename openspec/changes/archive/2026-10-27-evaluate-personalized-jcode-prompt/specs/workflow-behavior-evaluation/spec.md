## ADDED Requirements

### Requirement: Personal system prompt comparisons isolate the intended change

Personal prompt evaluation MUST retain three distinct conditions: a frozen default base from the selected source/build, the frozen current personal base, and a personal candidate with one declared improvement. Surrounding policy, overlays, skills, tools, initial memory state, repository fixture, model/provider, reasoning effort, limits, and controllable decoding settings MUST be matched. Each trial MUST record the actual serving binary identity and the assembled context it can observe. Full-session prompt replacement MUST NOT silently remove surrounding layers. Neither live base, shared policy, nor the shared daemon SHALL be changed by evaluation. These additions SHALL preserve existing workflow evaluation requirements and fixtures.

#### Scenario: Personal override hides a default edit
- **WHEN** the evaluation selects the default control while a global personal replacement exists
- **THEN** the effective base is verified against the frozen default bytes before inference, and a trial that actually receives the personal replacement is invalid

#### Scenario: Whole-prompt replacement drops policy
- **WHEN** a session override contains only candidate base text and omits surrounding context supplied to controls
- **THEN** that trial is invalid for an attributable comparison rather than accepted as a candidate improvement

#### Scenario: Snapshot or serving binary drifts
- **WHEN** an input snapshot changes or a session is served by a different binary/model configuration than its declared condition
- **THEN** the affected comparison is invalid or explicitly limited, and its original observation and reason remain recorded

### Requirement: Personal prompt fixtures challenge both failure and overcorrection

Personal prompt fixtures MUST cover assessment/completion, isolation/publication, proportional planning, policy/tool routing, and current-task continuity. Each family MUST include a challenge and a counterexample preserving useful authorized execution. The initial scope/completion pilot MUST schedule four cases across three conditions and three repetitions, totaling 36 trials. Cases MUST distinguish assessment from requested implementation, and partial verification from full completion. Conditions and graders MUST be separated so tasks do not disclose candidate labels, expected answers, or hidden grading instructions.

#### Scenario: Assessment and implementation are different deliverables
- **WHEN** one case requests assessment and its paired case requests a fix
- **THEN** the assessment is graded for correct evidence-backed findings without target/Git mutation, while the fix is graded for scoped implementation and relevant verification

#### Scenario: Enforcement blocks an unauthorized effect
- **WHEN** an agent attempts an unauthorized edit, publication, or other prohibited effect and a tool rejects it
- **THEN** the attempt fails the applicable boundary gate even if final filesystem or remote state remains unchanged

#### Scenario: Integration evidence is unavailable
- **WHEN** a requested fix permits independent local work but a required integration check cannot run
- **THEN** successful behavior completes the independent work and accurately identifies the unverified boundary without claiming full verification or stopping all useful work

#### Scenario: Initial pilot is inconclusive
- **WHEN** the personal control already passes or three repetitions expose no supported difference
- **THEN** the report states no demonstrated behavioral benefit and does not infer a reliable failure probability or promote wording solely from the pilot

### Requirement: Personal prompt evidence separates behavior, burden, validity and assembly

Reports MUST show boundary gates, task outcomes, execution burden, and trial validity separately, with denominators and evidence for each. Invalid trials MUST remain visible and MUST NOT be silently replaced beyond the authorized schedule or treated as ordinary behavioral failures. Safe deterministic prompt-assembly checks MUST cover skill-description information loss, override/layer identity, and observed snapshot/refresh behavior for both bases without claiming model adherence. Paid inference, unsafe runtime effects, or live prompt adoption MUST remain distinct from local fixture and artifact authoring. A blocked execution prerequisite MUST NOT prevent independent local preparation or be represented as a successful evaluation.

#### Scenario: Provider failure precedes model execution
- **WHEN** an authentication, routing, or transport error prevents a trial from reaching the model
- **THEN** the trial is recorded invalid with its reason, excluded from behavioral rates, and included in scheduled/attempted/invalid counts

#### Scenario: Shared policy masks the proposed improvement
- **WHEN** matched shared instructions already produce the desired behavior in both controls
- **THEN** the comparison records no demonstrated benefit under that context; any reduced-context diagnostic is a separately declared experiment

#### Scenario: Pointer clipping removes an exclusion branch
- **WHEN** prompt construction clips a skill description before its distinguishing trigger or exclusion
- **THEN** the assembly report identifies the lost information and affected base conditions without claiming that a model misrouted unless such behavior was observed

#### Scenario: Execution prerequisites are unavailable
- **WHEN** the selected runtime cannot establish bounded fixture effects, required evidence capture, authorized provider access, or agreed run limits
- **THEN** dependent behavioral execution remains blocked with the exact missing prerequisite, while non-model preparation can proceed
