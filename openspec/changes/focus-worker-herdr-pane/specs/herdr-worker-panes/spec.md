## ADDED Requirements

### Requirement: Focus a worker's Herdr pane
Jcode SHALL record the Herdr pane of each visible worker and SHALL offer an action that focuses it.

#### Scenario: Pane known
- **WHEN** the user selects focus-pane for a worker launched in Herdr
- **THEN** Herdr SHALL focus that worker's pane

#### Scenario: No pane
- **WHEN** a worker is headless or its pane is unknown
- **THEN** the focus-pane action SHALL not be offered
