# Spec Delta

## ADDED Requirements

### Requirement: Evidence plateaus use preregistered cumulative observations

A personal-prompt campaign MUST bind every prior result root and its hash before inference and carry forward cumulative attempted, invalid, unsupported, baseline-sufficient, candidate-decision, and confirmed-lead counters without resetting them. It MUST distinguish historical counts from prospective plateau-eligible observations. Baseline sufficiency alone MUST NOT be called a plateau. Any plateau rule MUST be frozen before inference, retain the existing failure-signal, candidate, and confirmed-lead thresholds, and require complete valid observations over its preregistered independent fixtures. Reports MUST identify the rule, counters, evidence roots, and unmet remainder.

#### Scenario: Successor binds prior results
- **WHEN** a new campaign is registered from a completed predecessor
- **THEN** registration verifies and records all prior result roots and their hashes before any inference can run

#### Scenario: Baseline sufficiency is not a plateau
- **WHEN** one or more preregistered tiers are baseline-sufficient but the plateau rule's full evidence condition is unmet
- **THEN** the report retains baseline-sufficient dispositions and does not declare a plateau

#### Scenario: Invalid evidence cannot advance a plateau counter
- **WHEN** a required fixture, actor slot, assessment, or adapter observation is invalid, missing, or uncertain
- **THEN** it remains visible but does not increment valid plateau evidence or trigger a replacement slot

#### Scenario: Plateau threshold is reached
- **WHEN** all preregistered independent plateau screens are complete and valid and their frozen outcomes meet the stated plateau criterion
- **THEN** the campaign reports an evidence plateau separately from no-demonstrated-benefit, baseline sufficiency, unsupported families, or confirmed leads
