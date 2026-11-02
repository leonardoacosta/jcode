## ADDED Requirements

### Requirement: Compaction fact-retention check

The evaluation SHALL check each fixture's must-retain facts against the produced summary and report retained and missing facts with pre/post token counts.

#### Scenario: All facts retained
- **WHEN** the summary contains every must-retain fact for a fixture
- **THEN** the fixture verdict is `passed` with token counts reported

#### Scenario: Fact dropped
- **WHEN** a must-retain fact is absent from the summary
- **THEN** the fixture verdict is `failed` and the missing fact is named

### Requirement: Truthful verdicts without a provider

Live mode SHALL report `inconclusive`, not `passed`, when no provider summary is produced.

#### Scenario: Provider unavailable
- **WHEN** live mode runs and the provider errors or is not configured
- **THEN** the fixture verdict is `inconclusive` with the error reason

#### Scenario: Provider available
- **WHEN** live mode produces a summary
- **THEN** the report names the model and source revision used
