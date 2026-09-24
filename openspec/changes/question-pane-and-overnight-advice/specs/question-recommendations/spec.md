## ADDED Requirements

### Requirement: Truthfully labeled asynchronous advice
The system SHALL obtain optional advice from the configured Jev MCP contract with remote-evaluation permission, bounded relevant non-secret context, and stable option IDs. Manual interaction SHALL remain available while evaluating. Successful advice SHALL show Jev attribution and model confidence, not imply user consent.

#### Scenario: Current valid recommendation
- **WHEN** Jev returns a valid recommendation for the current revision
- **THEN** the pane labels it as Jev advice without preselecting an answer and an explicit user action can accept it

#### Scenario: Unavailable or abstaining evaluator
- **WHEN** permission is absent, MCP errors, evaluation exceeds 15 seconds, output is malformed or Jev abstains
- **THEN** the pane displays the corresponding status, permits manual answers, and schedules no automatic answer

#### Scenario: Stale result
- **WHEN** advice arrives after resolution or for a different request, revision or context hash
- **THEN** it is ignored and cannot change selection or resolve a question

#### Scenario: Provenance of explanations
- **WHEN** Jcode adds explanation text not present in evaluator output
- **THEN** that text is labeled as Jcode explanation rather than a Jev rationale
