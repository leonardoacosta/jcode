## ADDED Requirements

### Requirement: Focus a worker's Herdr pane
Jcode SHALL record the Herdr pane of each visible worker and SHALL offer an action that focuses it.

Pane focus/restore SHALL verify current pane-to-session/repository identity. Restore SHALL respect durable execution revocation; stale pane IDs SHALL NOT launch old work or focus an unrelated pane.

#### Scenario: Pane known
- **WHEN** the user selects focus-pane for a worker launched in Herdr
- **THEN** Herdr SHALL focus that worker's pane

#### Scenario: No pane
- **WHEN** a worker is headless or its pane is unknown
- **THEN** the focus-pane action SHALL not be offered
