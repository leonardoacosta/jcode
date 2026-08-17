## ADDED Requirements

### Requirement: Native single-feature invocation
Jcode SHALL expose single-feature execution as a native skill named `apply` and SHALL preserve the selected approved feature argument.

#### Scenario: User invokes apply
- **WHEN** a user enters `/apply add-native-feature-workflow`
- **THEN** Jcode activates the native apply workflow for that feature
- **AND** does not activate a Codex- or Claude-owned implementation

### Requirement: Approved authoritative input
The workflow SHALL resolve one approved, current, implementation-ready feature from exactly one repository authority before mutation.

#### Scenario: Feature input is stale
- **WHEN** the authoritative feature, dependencies, repository revision, or verification contract changed after scheduling
- **THEN** apply rejects the stale schedule
- **AND** reports the fields that require reconstruction

### Requirement: Complete feature lifecycle
Apply SHALL execute implementation, verification, review, persistence, and truthful settlement as one lifecycle.

#### Scenario: Implementation succeeds but verification fails
- **WHEN** the implementation completes and any required verification fails
- **THEN** the feature is not reported complete
- **AND** the failure evidence and recovery obligation remain durable

### Requirement: Risk-selected review
Apply SHALL select review rigor from observable risk and SHALL require a different provider family only for high-risk and critical work.

#### Scenario: Feature is high risk
- **WHEN** the feature affects security, authentication, permissions, secrets, migrations, destructive operations, deployment infrastructure, public contracts, or equivalent scored risk
- **THEN** an independent reviewer from a different provider family performs adversarial review
- **AND** a separate verifier executes the acceptance contract

#### Scenario: Feature is normal risk
- **WHEN** the feature does not meet the high-risk threshold
- **THEN** required independent review may use the same provider family

### Requirement: Explicit execution path
Apply SHALL freeze either an Orca-supervised or Jcode-native execution path before mutation and SHALL never silently downgrade.

#### Scenario: Orca capability is unavailable but unnecessary
- **WHEN** the approved feature can satisfy all isolation, supervision, recovery, and verification needs through Jcode-native execution
- **THEN** preflight may select and report the Jcode-native path

#### Scenario: Required capability is unavailable
- **WHEN** the approved feature requires an Orca capability unavailable through the live runtime or Jcode adapter
- **THEN** apply pauses before mutation with an exact missing-capability result

### Requirement: Durable recovery
Apply SHALL resume from authoritative repository and runtime evidence and SHALL not treat conversation memory as execution state.

#### Scenario: Run is interrupted after mutation
- **WHEN** apply resumes after an interrupted attempt
- **THEN** it reconstructs state from the feature authority, frozen schedule, Git, Jcode checkpoints, runtime receipts, and fresh verification
- **AND** prevents duplicate mutation through attempt-scoped idempotency

### Requirement: Truthful closeout
Apply SHALL settle only from fresh, correlated evidence and SHALL preserve unresolved cleanup or recovery obligations.

#### Scenario: Worker completion lacks acceptance evidence
- **WHEN** an agent reports completion without the required verification receipts
- **THEN** the feature remains unverified
- **AND** Jcode does not advance its durable outcome to complete
