## ADDED Requirements

### Requirement: Independently deployable Command Center UI
The system SHALL build and deploy the Command Center UI without rebuilding or replacing the Jcode binary.

#### Scenario: UI-only upgrade
- **WHEN** an operator runs the root installer after changing only `apps/command-center`
- **THEN** the UI release is rebuilt, activated, and restarted
- **AND** the installed Jcode binary remains unchanged

### Requirement: Same-origin API bridge
The standalone service SHALL proxy Command Center API requests to Jcode while keeping Jcode authoritative for authentication and state.

#### Scenario: Browser API request
- **WHEN** the browser requests `/api/command-center/bootstrap` from the UI origin
- **THEN** the service forwards the request to the configured Jcode origin
- **AND** returns the upstream status, headers, and body without persisting domain data

#### Scenario: Jcode unavailable
- **WHEN** the upstream Jcode endpoint cannot be reached
- **THEN** the service returns a clear `502` response
- **AND** remains healthy enough to serve the UI and `/healthz`

### Requirement: Persistent systemd lifecycle
The installer SHALL create and enable a dedicated systemd user service that recovers from process failure and starts for the lingering user manager after reboot.

#### Scenario: Unexpected process exit
- **WHEN** the Node process exits unsuccessfully
- **THEN** systemd restarts it after the configured delay

#### Scenario: Reboot persistence
- **WHEN** the host reboots and the user manager starts
- **THEN** the enabled service starts without an interactive login when lingering is enabled

### Requirement: Safe repeatable installation
The installer SHALL activate releases atomically and preserve the last healthy release when build or activation fails.

#### Scenario: Repeat install
- **WHEN** the installer is run multiple times
- **THEN** each successful run creates one release and points `current` to it
- **AND** the unit remains enabled and active

#### Scenario: Failed activation
- **WHEN** the new service does not pass its health check
- **THEN** the installer restores the previous release and restarts it
- **AND** exits non-zero with actionable diagnostics

### Requirement: Loopback-first security
The service SHALL bind to loopback by default and SHALL NOT introduce a second credential store or domain database.

#### Scenario: Default installation
- **WHEN** no bind or upstream overrides are supplied
- **THEN** the UI listens on `127.0.0.1:43119`
- **AND** proxies to Jcode on `127.0.0.1:43118`

### Requirement: API-only Jcode daemon
The Jcode daemon SHALL expose the authenticated `/api/command-center/*` routes
without serving embedded or static UI assets.

#### Scenario: Legacy UI route rejection
- **WHEN** a client requests `/`, a client-side route, or an asset path from the daemon listener on `43118`
- **THEN** the daemon returns `404`
- **AND** the standalone UI service remains responsible for serving the React/Vite application

#### Scenario: Command Center API compatibility
- **WHEN** a browser or standalone UI proxy requests `/api/command-center/bootstrap` or another existing Command Center API route
- **THEN** the daemon preserves the existing authentication and response contract
