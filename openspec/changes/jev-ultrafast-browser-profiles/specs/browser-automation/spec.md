## ADDED Requirements

### Requirement: Upstream-backed browser automation
Jcode SHALL route browser actions through `browser-use/jev-ultrafast` and SHALL NOT silently fall back to the removed native Firefox bridge.

#### Scenario: Runtime available
- **WHEN** the browser tool executes a supported action with a valid profile
- **THEN** it performs the action through the configured upstream runtime and returns its result

#### Scenario: Runtime missing or incompatible
- **WHEN** the configured upstream runtime cannot be found or does not support the required operation
- **THEN** the tool returns an actionable diagnostic, makes no browser mutation, and does not invoke a legacy bridge

### Requirement: Labeled local profiles
Jcode SHALL provide labeled local profiles: fresh managed temporary or persistent state, and references to pre-existing externally owned profiles. The agent MAY create profiles and SHALL be able to select profiles and change the default; user confirmation is required for deletion.

#### Scenario: Profile creation and listing
- **WHEN** the user or agent creates a valid unique profile and lists profiles
- **THEN** the profile appears with stable metadata and no secrets or browser-state contents exposed

#### Scenario: Invalid or duplicate name
- **WHEN** a profile name is invalid or already exists
- **THEN** creation fails without changing existing profiles or files

#### Scenario: Persistence and isolation
- **WHEN** two managed persistent profiles are used before and after a Jcode restart
- **THEN** each resumes its own browser state and actions in one profile do not access the other's state

#### Scenario: No explicit default
- **WHEN** a browser action omits profile selection and no default is configured
- **THEN** it fails with instructions to select or configure a profile, without creating one implicitly

### Requirement: Profile selection and session binding
Browser sessions SHALL bind to the selected profile at session creation; the user or agent may change the default, which SHALL affect only new sessions.

#### Scenario: Change default with active session
- **WHEN** the user or agent changes the default while a browser session is active
- **THEN** the active session remains attached to its original profile and new sessions use the new default

#### Scenario: Agent changes default
- **WHEN** the agent selects an existing profile as the default
- **THEN** the default changes for new sessions and no profile data is created or deleted

#### Scenario: Explicit profile selection
- **WHEN** an action names an existing profile
- **THEN** the action uses that profile; unknown profile names fail without falling back to the default

### Requirement: Safe profile deletion
Explicit deletion of managed persistent data SHALL require user confirmation and SHALL refuse active profiles. Attached data SHALL never be deleted. Temporary cleanup SHALL follow the session-owned lifecycle.

#### Scenario: Unconfirmed or agent-requested deletion
- **WHEN** browser automation requests profile deletion without a user confirmation path
- **THEN** deletion is unavailable and profile data remains unchanged

#### Scenario: Active profile
- **WHEN** the user confirms deletion of a profile with an active session
- **THEN** deletion is refused with guidance to close the session first

#### Scenario: Confirmed inactive profile
- **WHEN** the user explicitly confirms deletion of an inactive profile
- **THEN** Jcode removes only that inactive managed persistent profile's data and reports cleanup failure without masking remaining data

### Requirement: Non-destructive migration
Upgrading SHALL stop using legacy bridge internals without modifying or deleting existing browser bridge files or browser data.

#### Scenario: Upgrade with legacy installation
- **WHEN** Jcode starts after the new integration is installed and legacy bridge files exist
- **THEN** Jcode does not execute, rewrite, migrate, or delete those files or silently import browser state

#### Scenario: Legacy runtime unavailable
- **WHEN** only the removed legacy bridge is installed
- **THEN** browser actions report upstream setup instructions and do not use the legacy bridge

### Requirement: Fresh and temporary profile lifecycle
Fresh profiles SHALL start empty. Creation SHALL select temporary or persistent storage. Temporary profiles SHALL belong to one Jcode session and SHALL NOT become a durable global default.

#### Scenario: Fresh profile
- **WHEN** the agent creates a fresh temporary or persistent profile
- **THEN** it receives a unique label/identity with no imported cookies or storage

#### Scenario: Owner closes
- **WHEN** a temporary profile's owner session closes and browser handles are released
- **THEN** Jcode cleans up only that temporary profile's owned state

#### Scenario: Crash or cleanup failure
- **WHEN** recovery finds temporary state whose inactivity cannot be established or whose cleanup fails
- **THEN** state is retained with a pending-cleanup error and unrelated profiles remain untouched

### Requirement: Labeled attachment and session authorization
The agent SHALL be able to register and select labeled existing profiles without copying state. Selection SHALL authorize use of their signed-in sessions without another login-consent prompt. Consequential actions SHALL retain their normal authorization requirements.

#### Scenario: Select signed-in profile
- **WHEN** an existing labeled profile with a signed-in session is selected
- **THEN** automation uses that profile's existing session without requesting login consent again

#### Scenario: Detach external profile
- **WHEN** an attached profile registration is removed or its Jcode session closes
- **THEN** Jcode leaves its external browser data intact

#### Scenario: Locked or inaccessible profile
- **WHEN** a selected external profile cannot be safely connected because of locks or debugging permissions
- **THEN** Jcode reports the constraint without removing locks, changing permissions, killing unrelated browsers, or substituting another profile

#### Scenario: Duplicate backing identity
- **WHEN** two labels resolve to the same browser profile
- **THEN** Jcode rejects duplicate registration rather than treating them as independent isolated profiles

### Requirement: Scoped runtime connection
Automation SHALL remain bound to its selected profile and SHALL fail closed if its scoped endpoint disappears.

#### Scenario: Endpoint loss or wrong-profile target
- **WHEN** the selected endpoint dies or an action references another profile's tab/session
- **THEN** the call fails without discovering or acting on a different browser
