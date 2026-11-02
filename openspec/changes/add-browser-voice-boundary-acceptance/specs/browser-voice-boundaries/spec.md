## ADDED Requirements

### Requirement: Browser handoff stays in scope

A browser handoff SHALL act only on its assigned tab and SHALL return a typed result when it needs help or the bridge is unavailable.

#### Scenario: Help hand-back
- **WHEN** a handoff needs user login
- **THEN** it returns `requested_help = user_login` for that tab without acting further

#### Scenario: Out-of-scope tab
- **WHEN** a handoff attempts an action on a tab other than its `tab_id`
- **THEN** the action is refused with an error

#### Scenario: Bridge unavailable
- **WHEN** the browser bridge is not running
- **THEN** the tool returns an explicit unavailable error within its timeout

### Requirement: Voice capture is user-controlled

Voice input SHALL send a transcript only on explicit user send, and SHALL treat it as user message data.

#### Scenario: Cancel recording
- **WHEN** the user presses Esc while recording
- **THEN** no message is sent and the composer is unchanged

#### Scenario: Microphone missing
- **WHEN** no supported recorder is available
- **THEN** the existing microphone error is shown and no recording starts
