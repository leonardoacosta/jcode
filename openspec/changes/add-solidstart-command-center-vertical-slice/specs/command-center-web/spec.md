## ADDED Requirements

### Requirement: Daemon-hosted SolidStart application
Jcode SHALL provide the command center as a Jcode-managed SolidStart application without introducing an independently authoritative workflow service or database.

#### Scenario: Command center starts with the daemon
- **WHEN** the experimental command-center configuration is enabled
- **THEN** the Jcode-managed service serves the application, command API, snapshots, and event stream through one authentication boundary

#### Scenario: Command center is disabled
- **WHEN** the command-center configuration is disabled
- **THEN** no command-center HTTP listener is created and all existing TUI, daemon, initiative, schedule, and session behavior remains available

#### Scenario: SSR integration requires a child process
- **WHEN** SolidStart SSR cannot run in-process with the Rust daemon
- **THEN** any daemon-supervised child process has no independent persistence, no externally exposed listener, no domain authority, and terminates with the daemon-managed lifecycle

### Requirement: Initiative route set
The application SHALL provide stable routes for initiative discovery, initiative detail, and a selected linked run.

#### Scenario: Browse initiatives
- **WHEN** a user opens `/initiatives`
- **THEN** the application lists accessible resumable and historical initiatives with status, current milestone, progress evidence, blocker state, and freshness metadata

#### Scenario: Open initiative detail
- **WHEN** a user opens `/initiatives/:initiativeId`
- **THEN** the application renders the selected initiative's durable pane and either the most relevant linked execution pane or an explicit no-run state

#### Scenario: Open a specific linked run
- **WHEN** a user opens `/initiatives/:initiativeId/runs/:runId`
- **THEN** the application verifies the relationship and renders that run or a typed mismatch/not-found state without substituting an unrelated run

### Requirement: Split durable and live workspace
The initiative detail route SHALL keep durable intent visible beside linked live execution at supported desktop and embedded-surface widths.

#### Scenario: Initiative and runtime are available
- **WHEN** an initiative has a reachable linked run
- **THEN** the durable pane shows outcome, status, milestone, steps, success criteria, blockers, next actions, child references, schedules, and checkpoint history while the execution pane shows run health, normalized Orca graph, workers/sessions, gates, attention items, and event timeline

#### Scenario: Initiative has no linked run
- **WHEN** an initiative has no current run relationship
- **THEN** the durable pane remains fully usable and the execution pane explains that no run is linked without inventing or inferring one

#### Scenario: Embedded-width layout
- **WHEN** the command center is rendered at the minimum width defined for a future desktop surface
- **THEN** all authoritative information and actions remain reachable without depending on a standalone browser tab, browser chrome, or a full-window-only interaction

### Requirement: Initiative management actions
The vertical slice SHALL allow authorized users to update the current milestone and step state, checkpoint progress, and manage blockers or next actions through typed Jcode commands.

#### Scenario: Checkpoint initiative progress
- **WHEN** a user submits a checkpoint summary with current milestone, blockers, and next actions
- **THEN** Jcode persists one timestamped initiative update and the page reconciles to the returned authoritative initiative revision

#### Scenario: Update conflicts with newer state
- **WHEN** a command was based on an initiative revision older than the current server revision
- **THEN** the command is rejected as stale and the UI offers reconciliation without overwriting newer state

#### Scenario: Unauthorized initiative mutation
- **WHEN** a browser session lacks permission for an initiative command
- **THEN** the action is unavailable or rejected and the current initiative state remains visible and unchanged

### Requirement: Linked schedule projection
The initiative route SHALL show linked schedule timing and health without implementing the complete global schedule-management milestone.

#### Scenario: Schedule is healthy
- **WHEN** an initiative has a linked enabled schedule
- **THEN** the durable pane shows cadence, timezone, next fire, last result, retry state, and a link to the resulting run when present

#### Scenario: Schedule is missed or repeatedly failing
- **WHEN** the scheduling engine reports a missed wake, stale claim, or repeated failure
- **THEN** the route surfaces the condition as an attention item with evidence and safe recovery actions supported by the server

#### Scenario: No schedule is linked
- **WHEN** an initiative has no linked schedule
- **THEN** the page renders an explicit empty state and does not infer a schedule from ambient queue proximity

### Requirement: Reactive event application
The Solid client SHALL apply ordered Jcode events without full-page refresh while preserving local layout and selection state.

#### Scenario: Initiative event arrives
- **WHEN** a valid next-sequence initiative, schedule, run, approval, or normalized Orca event arrives
- **THEN** the affected projection updates and unrelated pane sizes, filters, selections, drafts, and scroll positions remain stable

#### Scenario: Event stream disconnects
- **WHEN** the browser loses the event stream
- **THEN** the page displays a persistent disconnected or stale indicator, retains clearly labeled last-known data, and attempts bounded reconnection

#### Scenario: Reconnection requires snapshot replacement
- **WHEN** the server cannot replay from the client's sequence
- **THEN** the page fetches and atomically installs a fresh snapshot before presenting itself as live

### Requirement: Explicit interface states
Every independently loaded command-center region SHALL represent loading, empty, unavailable, stale, error, and data states without hiding failures behind transient notifications.

#### Scenario: Orca is unavailable while Jcode is healthy
- **WHEN** the initiative snapshot loads but the Orca adapter is unavailable
- **THEN** the durable pane renders normally and the execution pane renders an unavailable state with last-observed evidence and disabled unsafe actions

#### Scenario: Command fails after user action
- **WHEN** a typed command returns a recoverable failure
- **THEN** the initiating control displays the failure with inspect, retry, reconcile, or dismiss actions appropriate to the error

#### Scenario: Initial snapshot fails
- **WHEN** the route cannot obtain an authoritative snapshot
- **THEN** it renders a route-level error and retry path and does not display cached data as current

### Requirement: Conservative optimistic behavior
The web client SHALL use optimistic updates only for reversible client-local interactions and SHALL keep authoritative mutations pending until server acknowledgment.

#### Scenario: User resizes the split panes
- **WHEN** a user changes pane sizes
- **THEN** the layout responds immediately because pane geometry is client-owned state

#### Scenario: User triggers a run action
- **WHEN** a user requests start, retry, cancel, milestone, or checkpoint mutation
- **THEN** the control shows a pending state until Jcode acknowledges the command and the UI does not claim the domain transition succeeded early

### Requirement: Terminal Control Room remains lightweight
This child change SHALL NOT duplicate the SolidStart command-center interactions in the terminal Context Control Room.

#### Scenario: User opens the current Control Room
- **WHEN** the user invokes the existing terminal Control Room shortcut
- **THEN** the current context-inspection behavior remains available without new initiative editing, graph manipulation, schedule administration, or approval workflows from this child change

### Requirement: Accessible and keyboard-operable command center
The command-center route SHALL be operable using keyboard navigation and SHALL expose meaningful semantic structure and status announcements.

#### Scenario: Navigate split workspace by keyboard
- **WHEN** a keyboard user moves among initiative controls, execution nodes, attention items, and the event timeline
- **THEN** focus order is predictable, focus is visible, and no action requires pointer-only interaction

#### Scenario: Live status changes
- **WHEN** a command completes, connection state changes, or an attention item appears
- **THEN** the application announces the change through an appropriate non-disruptive accessible status region

### Requirement: End-to-end vertical-slice acceptance
The feature SHALL include a representative acceptance workflow using a real managed Jcode daemon and Orca runtime rather than only mocked frontend fixtures.

#### Scenario: Manage and observe an initiative
- **WHEN** an authenticated user opens an existing initiative, updates or checkpoints its current milestone, observes a linked execution event, disconnects and reconnects, and resumes the initiative
- **THEN** the workflow preserves authoritative state, reconciles event sequence correctly, and displays the linked Jcode and Orca identities throughout

#### Scenario: Runtime dependency is unavailable
- **WHEN** the same workflow runs while Orca is intentionally unavailable
- **THEN** initiative management remains functional, runtime state is explicitly degraded, and no unsafe runtime command is reported as successful
