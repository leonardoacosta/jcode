## ADDED Requirements

### Requirement: Prompt budget check
The prompt budget check SHALL measure system prompt, instruction, and skill context with a named/versioned deterministic offline token approximation for a fixed fixture and SHALL fail when any section exceeds its recorded budget plus tolerance. `--check` SHALL be read-only. Measurements SHALL NOT imply exact provider tokens, behavioral quality or bill savings.

#### Scenario: Growth past budget fails
- **WHEN** the skills section measures 5% above its recorded budget with 2% tolerance
- **THEN** the check exits non-zero and names the skills section and delta

#### Scenario: Within budget passes
- **WHEN** all sections are at or below budget plus tolerance
- **THEN** the check exits zero

### Requirement: Explicit acceptance
Raising a budget SHALL require an explicit accept command with a non-empty reason, and SHALL record the change in the ledger.

Tightening after a reduction greater than 5% SHALL also occur only through that explicit acceptance command, never automatically during measurement or CI.

#### Scenario: Accept without reason
- **WHEN** a user runs accept with no reason
- **THEN** the budget is unchanged and the command fails

### Requirement: KEEP/REVERT ledger
Each accepted prompt change SHALL append a ledger entry with section deltas, optional prompt-experiment score delta, and a KEEP or REVERT decision; existing entries SHALL NOT be modified.

#### Scenario: Experiment result recorded
- **WHEN** a prompt change is evaluated by the prompt-experiment harness and accepted
- **THEN** the ledger entry includes the score delta and decision
