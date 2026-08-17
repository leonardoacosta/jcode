# Orca Command Center tasks 4.5c-d implementation contract

Date: 2026-08-17 UTC
Status: decision-complete implementation artifact
Scope: crash-reconcilable start, exact-Dispatch retry, exact-Dispatch cancel, and cleanup accounting for the approved Orca `1.4.176` compatibility profile

## 1. Required outcome

Implement the three existing public Jcode commands without adding another public Orca mutation:

- `start_initiative_run` creates one Jcode attempt, one Orca Run namespace, one Orca Task, and one supervised Dispatch.
- `retry_linked_run` creates one new Jcode attempt and one new Dispatch for the exact prior Dispatch through `worker-start --retry-of`.
- `cancel_linked_run` fences the exact selected Dispatch through `worker-stop`, then uses `worker-abandon` only when process termination cannot be proven.

An Orca Run is never treated as terminal outcome authority. The Dispatch is the executable attempt. Every Orca mutation request and response must be durably recorded before the next composition step.

This work depends on task 4.5a's `SqliteOrcaOperationStore` and task 4.5b's pinned `1.4.176` command and response fixtures. Runtime capabilities remain disabled until all three pieces and task 4.5e acceptance are present.

## 2. Evidence that determines the implementation

The repository contract is in:

- `openspec/changes/add-solidstart-command-center-vertical-slice/tasks.md`, tasks 4.5c-d
- `openspec/changes/add-solidstart-command-center-vertical-slice/design.md`, decision 6
- `openspec/changes/add-solidstart-command-center-vertical-slice/specs/command-center-protocol/spec.md`, closed runtime command capability set
- `docs/audits/ORCA_COMMAND_CENTER_LIFECYCLE_CAPABILITY_AUDIT_2026-08-17.md`

The installed `1.4.176` command registry reports schema version `1` and exposes these required commands:

```text
orchestration run-create
orchestration run-use
orchestration run-current
orchestration run-list
orchestration run-show
orchestration task-create
orchestration task-list
orchestration worker-start
orchestration worker-show
orchestration worker-stop
orchestration worker-abandon
orchestration worker-release
orchestration worker-list
```

The installed implementation adds one critical requirement that is not visible in the current `OrcaCliAdapter`:

1. The Orca CLI automatically creates a random mutation request ID for every orchestration mutation.
2. `--retry-request <id>` overrides that generated ID.
3. The runtime stores mutation receipts under that ID and rejects reuse with different method or parameter hashes.
4. A pending `worker-start` receipt stores the accepted Dispatch ID before worker effects begin.
5. A pending Run or Task mutation after runtime restart can report `operation_unknown` without the created Run or Task ID.

Therefore Jcode must generate and persist a stable Orca request ID **before the first invocation of every mutation step**, and must pass that ID through `--retry-request` on the first call and every replay. Calling without `--retry-request` is unsafe because a Jcode crash can lose the CLI-generated ID.

## 3. Implementation boundaries

Prefer a new isolated module:

```text
crates/jcode-app-core/src/command_center/orca_lifecycle.rs
```

Keep `crates/jcode-app-core/src/command_center.rs` edits limited to:

- declaring the module
- constructing the lifecycle adapter with the operation store and compatibility profile
- changing `OrcaCommandRunner` to return process output instead of rejecting nonzero exit codes
- delegating the three `OrcaAdapter` mutation methods

Required contract edits remain in:

```text
crates/jcode-command-center/src/lib.rs
crates/jcode-command-center/src/orca_operation_store.rs  # only if 4.5a types need alignment
```

Required fixtures and tests belong in new paths rather than the existing large adapter test block:

```text
crates/jcode-app-core/src/command_center/orca_lifecycle_tests.rs
crates/jcode-app-core/tests/fixtures/orca-1.4.176/
```

Do not place the orchestration state machine directly into the existing `command_center.rs` file.

## 4. Closed placement decision

The first compatibility profile supports only an exact existing Orca-managed worktree and a configured agent launch.

Supported placement:

```text
--worktree id:<worktree-id> --agent <configured-agent>
```

Optional `--model` and `--effort` are allowed only when the pinned runtime capability and fixture set proves them.

Rejected in this slice:

- `--worktree current`, because it is not stable across daemon restart or remote selection
- `new-child` and `new-top-level`, because they add worktree creation policy and cleanup beyond this task
- an inferred terminal chosen from `terminal list`
- an inferred agent
- remote `--on` unless an exact saved environment and its server identity are persisted in the operation payload

A configured existing terminal may be supported later. The initial profile should always use `--agent`, which creates a distinct agent terminal for each attempt. Retry reconstructs the same exact worktree and agent/model/effort choices, but creates a distinct terminal and Dispatch.

## 5. Canonical identity preflight

Before recording an executable start, resolve and persist these identities:

1. `worktree current --json`
2. `repo list --json`
3. `project setups --json`
4. `project list --json`

Resolution rules:

- Run the CLI in `OrcaCliAdapter.working_dir` by setting `Command::current_dir`.
- Canonicalize `working_dir` and require it to be inside the returned current worktree path.
- Take `repoId`, `hostId`, and worktree `id` from `worktree current`.
- Require exactly one ready project host setup where `repoId` and `hostId` match.
- Require exactly one project whose `id` matches the setup's `projectId` and whose `sourceRepoIds` contains the repository ID.
- Persist project ID, repository ID, host setup ID, host ID, worktree ID, worktree path fingerprint, and runtime ID.
- Recheck the same identities before each mutation step. Any drift moves the operation to `RecoveryRequired` and invokes no mutation.

The existing adapter currently places a repository ID in `OrcaProjectId`. That must not be reused for lifecycle mutation identity.

## 6. Coordinator terminal and serialization

`run-create`, `run-use`, `task-create`, and `worker-start` require a live coordinator terminal with stable pane identity. The adapter must not guess one.

Add explicit adapter configuration:

```rust
pub struct OrcaCoordinatorBinding {
    pub terminal: OrcaTerminalId,
}
```

Resolve it from an explicit daemon configuration value, with `ORCA_TERMINAL_HANDLE` accepted only when it was captured for the Command Center daemon and successfully verified through `terminal show --json`.

If no verified terminal is available, advertise no start or retry capability.

One terminal can be bound to only one Run at a time. Add an adapter-wide `tokio::sync::Mutex<()>` around the following sequence:

1. observe or establish the target Run binding
2. create or observe the Task
3. start or observe the Dispatch

Retry must call `run-current --from <terminal>` and, when needed, `run-use --id <run> --from <terminal>` before `worker-start`. Each `run-use` mutation has its own persisted Orca request ID.

Cancellation does not require the coordinator binding and should not wait on this mutex.

## 7. Public and adapter contract changes

### 7.1 Identity and placement types

Add Rust types at the `jcode-command-center` boundary. They are server contracts, not direct browser input.

```rust
id_type!(OrcaRepositoryId);
id_type!(OrcaHostSetupId);
id_type!(OrcaHostId);
id_type!(OrcaRequestId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaCanonicalPlacement {
    pub project_id: OrcaProjectId,
    pub repository_id: OrcaRepositoryId,
    pub host_setup_id: OrcaHostSetupId,
    pub host_id: OrcaHostId,
    pub worktree_id: OrcaWorktreeId,
    pub worktree_selector: String,
    pub coordinator_terminal_id: OrcaTerminalId,
    pub environment: Option<String>,
    pub launcher: OrcaWorkerLauncher,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OrcaWorkerLauncher {
    Agent {
        agent: String,
        model: Option<String>,
        effort: Option<String>,
    },
    ExistingTerminal {
        terminal_id: OrcaTerminalId,
    },
}
```

The initial profile constructs only `Agent`.

### 7.2 Mutation context and requests

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaMutationContext {
    pub command_id: CommandId,
    pub idempotency_key: IdempotencyKey,
    pub correlation_id: CorrelationId,
    pub initiative_id: InitiativeId,
    pub jcode_attempt_id: JcodeRunId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartInitiativeRunRequest {
    pub context: OrcaMutationContext,
    pub objective: String,
    pub task_spec: String,
    pub placement: OrcaCanonicalPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryLinkedRunRequest {
    pub context: OrcaMutationContext,
    pub prior_jcode_attempt_id: JcodeRunId,
    pub orca_run_id: OrcaRunId,
    pub orca_task_id: OrcaTaskId,
    pub retry_of_dispatch_id: OrcaDispatchId,
    pub placement: OrcaCanonicalPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelLinkedRunRequest {
    pub context: OrcaMutationContext,
    pub target_jcode_attempt_id: JcodeRunId,
    pub orca_run_id: OrcaRunId,
    pub orca_task_id: OrcaTaskId,
    pub target_dispatch_id: OrcaDispatchId,
}
```

Change `OrcaAdapter` to accept these requests. The current signatures discard command ID, correlation, placement, Task identity, Dispatch identity, and retry causality, so they cannot implement tasks 4.5c-d safely.

### 7.3 Attempt and lifecycle receipts

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaAttemptIdentity {
    pub run_id: OrcaRunId,
    pub task_id: OrcaTaskId,
    pub dispatch_id: OrcaDispatchId,
    pub retry_of_dispatch_id: Option<OrcaDispatchId>,
    pub worktree_id: OrcaWorktreeId,
    pub terminal_id: Option<OrcaTerminalId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrcaMutationOutcome {
    Ready,
    Failed,
    OutcomeUnknown,
    Stopped,
    Abandoned,
    AlreadySettled,
    Rejected,
    RecoveryRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrcaLifecycleReceipt {
    pub outcome: OrcaMutationOutcome,
    pub attempt: Option<OrcaAttemptIdentity>,
    pub stage: String,
    pub failed_stage: Option<String>,
    pub last_error: Option<String>,
    pub effects: Vec<OrcaEffectReceipt>,
    pub residual_resources: Vec<OrcaEffectReceipt>,
    pub cleanup: Vec<CleanupResourceProjection>,
    pub observed_at: DateTime<Utc>,
}
```

Extend `JcodeRunReference` with migration-safe optional attempt fields:

```rust
#[serde(default)]
pub orca_task_id: Option<OrcaTaskId>,
#[serde(default)]
pub orca_dispatch_id: Option<OrcaDispatchId>,
#[serde(default)]
pub retry_of_jcode_run_id: Option<JcodeRunId>,
#[serde(default)]
pub retry_of_dispatch_id: Option<OrcaDispatchId>,
#[serde(default)]
pub worktree_id: Option<OrcaWorktreeId>,
#[serde(default)]
pub terminal_id: Option<OrcaTerminalId>,
```

Replace or extend `CommandResultPayload::RunAccepted` so the result can carry the lifecycle receipt. Failed and outcome-unknown starts must still return authoritative IDs and effects. They must not be collapsed into a bare `CommandCenterError`.

### 7.4 Command execution result

The service currently maps every successful adapter call to `CommandState::Pending` and every adapter error to `Failed` without an authoritative payload. Replace that split with:

```rust
pub struct RuntimeCommandExecution {
    pub state: CommandState,
    pub payload: Option<CommandResultPayload>,
    pub error: Option<CommandCenterError>,
}
```

Mapping:

| Lifecycle outcome | Command state | Authoritative payload |
|---|---|---|
| start or retry `ready` | `Pending` | new Jcode attempt plus Run/Task/Dispatch/placement receipt |
| start or retry `failed` | `Failed` | failed Jcode attempt plus Run/Task/Dispatch/effects |
| start or retry `outcome_unknown` | `Pending` | known identities and recovery receipt |
| cancel `stopped` or `abandoned` with terminal worker evidence | `Completed` | target attempt plus cancellation and cleanup receipt |
| cancel `stop_unknown` without abandon settlement | `Pending` | target attempt plus recovery receipt |
| precondition rejection | `Failed` | current authoritative attempt when available |

The service must load the initiative and compare `expected_revision` before beginning an Orca operation. The current runtime command path skips that revision check.

## 8. Installed Orca JSON DTOs

Use one generic top-level envelope and command-specific results. Parse stdout even when the process exits nonzero.

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrcaCliEnvelope<T> {
    id: String,
    ok: bool,
    result: Option<T>,
    error: Option<OrcaCliError>,
    #[serde(rename = "_meta")]
    meta: OrcaCliMeta,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrcaCliMeta {
    #[serde(rename = "runtimeId")]
    runtime_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OrcaMutationMeta {
    #[serde(rename = "requestId")]
    request_id: OrcaRequestId,
    replayed: bool,
}
```

Command result DTOs must cover these exact installed fields:

```rust
struct OrcaRunCreateResult {
    run: OrcaRunRow,
    binding: OrcaRunBinding,
    mutation: OrcaMutationMeta,
}

struct OrcaTaskCreateResult {
    task: OrcaTaskRow,
    mutation: OrcaMutationMeta,
}

struct OrcaWorkerStartResult {
    #[serde(rename = "runId")]
    run_id: OrcaRunId,
    #[serde(rename = "taskId")]
    task_id: OrcaTaskId,
    #[serde(rename = "dispatchId")]
    dispatch_id: OrcaDispatchId,
    state: OrcaWorkerStartState,
    stage: String,
    #[serde(rename = "failedStage")]
    failed_stage: Option<String>,
    #[serde(rename = "lastError")]
    last_error: Option<String>,
    setup: OrcaSetupReceipt,
    launch: OrcaLaunchReceipt,
    #[serde(rename = "timeoutMs")]
    timeout_ms: Option<u64>,
    effects: Vec<OrcaEffectReceipt>,
    #[serde(rename = "residualResources")]
    residual_resources: Vec<OrcaEffectReceipt>,
    #[serde(rename = "nextCommands", default)]
    next_commands: Vec<String>,
    warning: Option<String>,
    mutation: OrcaMutationMeta,
}

struct OrcaWorkerStopResult {
    #[serde(rename = "dispatchId")]
    dispatch_id: OrcaDispatchId,
    state: OrcaWorkerStopState,
    #[serde(rename = "alreadySettled")]
    already_settled: bool,
    #[serde(rename = "processAction")]
    process_action: OrcaProcessAction,
    close: Option<serde_json::Value>,
    #[serde(rename = "lastError")]
    last_error: Option<String>,
    mutation: OrcaMutationMeta,
}

struct OrcaWorkerAbandonResult {
    #[serde(rename = "dispatchId")]
    dispatch_id: OrcaDispatchId,
    state: OrcaWorkerState,
    #[serde(rename = "alreadySettled")]
    already_settled: bool,
    stale: bool,
    #[serde(rename = "processAction")]
    process_action: OrcaProcessAction,
    warning: String,
    #[serde(rename = "residualResources")]
    residual_resources: Vec<OrcaEffectReceipt>,
    mutation: OrcaMutationMeta,
}

struct OrcaWorkerReleaseResult {
    #[serde(rename = "dispatchId")]
    dispatch_id: OrcaDispatchId,
    state: OrcaReleaseState,
    reason: Option<String>,
    #[serde(rename = "processAction")]
    process_action: OrcaProcessAction,
    archive: Option<OrcaArchiveReceipt>,
    #[serde(rename = "lastError")]
    last_error: Option<String>,
    recovery: Option<String>,
    mutation: OrcaMutationMeta,
}
```

Closed enum values for the pinned profile:

```text
worker-start response: ready | failed | outcome_unknown
worker-show worker state: starting | ready | start_unknown | succeeded | failed | stopping | stop_unknown | stopped | abandoned
worker-stop response: stopped | stop_unknown | succeeded | failed | abandoned
worker-release response: released | already_released | retained | release_pending | release_unknown
process action: none | unknown | closed_agent_terminal | closed_exited_terminal
```

Any unrecognized enum or field shape is a compatibility failure. Do not silently deserialize it into an `Unknown(String)` variant while advertising mutations.

## 9. Process runner change

The current runner discards stderr and maps every nonzero exit to `OrcaUnavailable`. That loses valid `failed`, `outcome_unknown`, `stop_unknown`, and `release_unknown` JSON receipts.

Replace its return type:

```rust
pub struct OrcaCommandOutput {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[async_trait]
pub trait OrcaCommandRunner {
    async fn run(
        &self,
        command: &str,
        args: &[String],
        current_dir: Option<&Path>,
        timeout: Duration,
    ) -> Result<OrcaCommandOutput, OrcaProcessError>;
}
```

Rules:

- Set `stdin` to null, capture stdout and stderr, set `kill_on_drop(true)`, and run in `working_dir`.
- A spawn failure before the child exists is `OrcaUnavailable` and creates no active attempt.
- A timeout, broken transport, missing stdout, or invalid JSON after the child starts is `OutcomeUnknown`, because effects may exist.
- Parse JSON before interpreting the exit code.
- Validate the expected exit code against the parsed lifecycle state:
  - worker-start `ready` must exit 0, `failed` and `outcome_unknown` must exit nonzero
  - worker-stop `stop_unknown` must exit nonzero, all settled answers must exit 0
  - worker-release `release_unknown` must exit nonzero, all other documented states must exit 0
- Exit and receipt disagreement is a schema mismatch and disables mutation capabilities.

## 10. Stable request identity

Generate one deterministic request ID per Jcode command and composition stage:

```rust
fn orca_request_id(command_id: &CommandId, stage: OrcaMutationStage) -> OrcaRequestId {
    OrcaRequestId(Uuid::new_v5(
        &COMMAND_CENTER_ORCA_REQUEST_NAMESPACE,
        format!("{}:{}", command_id.0, stage.as_str()).as_bytes(),
    ).to_string())
}
```

Required stages:

```text
run_create
run_bind
task_create
worker_start
worker_retry
worker_stop
worker_abandon
worker_release
```

These stage names are persisted contract values.

Before invoking each command:

1. append `OrcaRequestRecord` with the stable request ID, full argument vector, and typed payload
2. commit the store transaction
3. invoke Orca with `--retry-request <stable-id>`
4. append the parsed receipt or typed unknown-result evidence
5. commit identities and recovery obligations atomically

Never generate a replacement request ID for the same stage because a command timed out.

## 11. Deterministic correlation markers

Run and Task creation can become durable while their mutation receipt remains pending. Their pending receipt does not preserve an accepted entity ID in the same way as `worker-start`.

Use deterministic markers so read-side reconciliation can find their effects:

```text
Run objective prefix: [jcode-cc:<command-id>] <initiative title>
Task display name:   jcode-cc:<command-id>:initial
Task spec schema:    jcode.command-center.task.v1
```

The task spec should be compact deterministic JSON containing:

```json
{
  "schema": "jcode.command-center.task.v1",
  "commandId": "...",
  "correlationId": "...",
  "initiativeId": "...",
  "jcodeAttemptId": "...",
  "objective": "...",
  "successCriteria": []
}
```

Reconciliation accepts exactly one matching Run or Task. Zero matches after an `operation_unknown` receipt remains `OutcomeUnknown`. Multiple matches become `RecoveryRequired`. Neither case may create another entity with a new request ID.

## 12. Start command sequence

Clean path, under the coordinator-binding mutex:

```text
1. status --json
2. pinned profile validation and canonical identity reads
3. persist NewOrcaOperation and stable run_create request
4. orchestration run-create
     --objective "[jcode-cc:<command>] <title>"
     --from <coordinator-terminal>
     --retry-request <run-create-request>
     --json
5. persist Run receipt and Orca Run ID
6. orchestration run-current --from <coordinator-terminal> --json
7. if current Run differs, persist run_bind request and call:
   orchestration run-use
     --id <run-id>
     --from <coordinator-terminal>
     --retry-request <run-bind-request>
     --json
8. persist task_create request
9. orchestration task-create
     --spec <deterministic-json>
     --task-title <initiative-title>
     --display-name jcode-cc:<command>:initial
     --run <run-id>
     --from <coordinator-terminal>
     --retry-request <task-create-request>
     --json
10. persist Task receipt and Task ID
11. recheck current Run binding and canonical placement
12. persist worker_start request
13. orchestration worker-start
      --task <task-id>
      --run <run-id>
      --from <coordinator-terminal>
      --worktree id:<worktree-id>
      --agent <agent>
      [--model <model> --effort <effort>]
      --retry-request <worker-start-request>
      --json
14. persist worker-start receipt and Dispatch ID
15. orchestration worker-show --dispatch <dispatch-id> --json
16. verify Run, Task, Dispatch, worktree, terminal, state, and observation identity
17. return authoritative Jcode attempt and lifecycle receipt
```

Do not report acceptance from `worker-start` alone. `worker-show` must confirm the exact Dispatch and placement.

### Start recovery rules

| Durable evidence | Required action |
|---|---|
| no Run ID | replay `run-create` with the same request ID, then reconcile exact objective marker on `operation_unknown` |
| Run ID, no Task ID | verify or restore Run binding, replay `task-create` with the same request ID, then reconcile exact display-name marker |
| Task ID, no Dispatch ID | verify binding and Task status, replay `worker-start` with the same request ID |
| `operation_unknown` contains Dispatch ID | persist it and call `worker-show`; never start another worker |
| Dispatch state `starting` | poll `worker-show` for a bounded interval; after runtime epoch change it should reconcile to `start_unknown` |
| Dispatch state `ready` | verify placement and return `Ready` |
| Dispatch state `failed` | preserve effects and return `Failed` |
| Dispatch state `start_unknown` | return `OutcomeUnknown`; require stop or abandon before retry |
| identity or schema disagreement | return `RecoveryRequired`; advertise no mutation capability until reconciled |

## 13. Retry command sequence

A retry creates a new Jcode attempt before invoking Orca. It reuses the prior Orca Run and Task but never the prior Dispatch identity.

Preconditions:

- the selected prior Jcode attempt exists and belongs to the initiative
- it has exact Orca Run, Task, and Dispatch IDs
- `worker-show` reports the exact prior Dispatch in `failed`, `stopped`, or `abandoned`
- the prior Dispatch is the Task's latest Dispatch
- Task status is `failed` or `blocked`
- placement matches the persisted canonical placement
- retry policy allows one more attempt

Command sequence, under the coordinator-binding mutex:

```text
1. begin or load RetryLinkedRun operation
2. verify prior worker and Task observations
3. ensure coordinator terminal is bound to the existing Run through run-current/run-use
4. persist worker_retry request
5. orchestration worker-start
     --task <same-task-id>
     --run <same-run-id>
     --from <coordinator-terminal>
     --worktree id:<same-worktree-id>
     --agent <same-agent>
     [--model <same-model> --effort <same-effort>]
     --retry-of <exact-prior-dispatch-id>
     --retry-request <worker-retry-request>
     --json
6. persist response
7. require new Dispatch ID != prior Dispatch ID
8. worker-show the new Dispatch and verify exact placement
9. return the new Jcode attempt with retry causality
```

Do not increment retry state when Orca is unavailable before a new Dispatch is known. Retry count is derived from durable Jcode attempts that have a distinct accepted Dispatch.

Retry recovery uses the worker-start recovery rules. It never issues a second replacement with a new request ID.

## 14. Cancel command sequence

Cancellation targets the exact Dispatch recorded on the selected Jcode attempt.

Preflight with `worker-show`:

- `ready` and `start_unknown` are stoppable
- `succeeded` is a stale cancellation and must not be reported as cancelled
- `failed`, `stopped`, and `abandoned` are already terminal, but the result must preserve their real cause rather than relabel them cancelled
- `starting`, `stopping`, and `stop_unknown` require reconciliation before a new lifecycle choice

Primary stop path:

```text
1. begin or load CancelLinkedRun operation
2. worker-show --dispatch <target>
3. persist worker_stop request
4. orchestration worker-stop
     --dispatch <target>
     --retry-request <worker-stop-request>
     --json
5. persist stop receipt
6. worker-show --dispatch <target>
7. if state is stopped, record terminal cancellation evidence and continue to cleanup
8. if state is stop_unknown, continue to abandon path
```

Abandon path:

```text
1. persist worker_abandon request
2. orchestration worker-abandon
     --dispatch <target>
     --retry-request <worker-abandon-request>
     --json
3. require dispatchId == target, state == abandoned, and stale == false
4. worker-show --dispatch <target>
5. require worker state abandoned
6. record every possibly-live resource as RecoveryRequired
7. do not call worker-release
```

A stop transport error or `operation_unknown` does not justify a new stop request. Reconcile with the same request ID and `worker-show`. If termination still cannot be proven, abandon is a distinct recorded mutation with distinct causality.

### Post-stop terminal cleanup

After exact `stopped` evidence, release only the terminal resource owned by that Dispatch:

```text
orchestration worker-release
  --dispatch <target>
  --retry-request <worker-release-request>
  --json
```

Map release results:

| Release state | Cleanup result |
|---|---|
| `released` or `already_released` | terminal `VerifiedReleased` |
| `retained` | cancellation completed, terminal `RecoveryRequired` with reason |
| `release_pending` | cancellation completed, terminal `RecoveryRequired`, recovery may retry the same request |
| `release_unknown` | cancellation completed, terminal `RecoveryRequired`; never substitute broad terminal close |

The worktree, Run, Task, setup terminals, configured tabs, and unrelated processes are never deleted by this command. Record intentional retention or recovery obligations explicitly.

## 15. Crash reconciliation entry point

Add:

```rust
impl OrcaLifecycleAdapter {
    pub async fn reconcile_pending_operations(&self) -> Result<ReconciliationSummary, CommandCenterError>;
}
```

Run it:

- during Command Center startup before advertising mutation capabilities
- before executing any start, retry, or cancel
- after any outcome-unknown result

Algorithm:

1. load operations in `Recorded`, `InProgress`, `OutcomeUnknown`, or `RecoveryRequired`
2. validate the pinned profile and stored canonical identity
3. reconstruct the typed request from `command_payload`
4. inspect completed receipts and known IDs
5. resume only the first incomplete stage
6. use the stage's original stable Orca request ID
7. perform read-side reconciliation before any replay that could create an entity
8. append evidence atomically
9. never overwrite a set-once Run, Task, Dispatch, worktree, or terminal identity with a different value

A reconciliation disagreement is visible and durable. It is not converted to `OrcaUnavailable` or silently retried.

## 16. Error mapping

Add typed errors or equivalent structured reasons for:

```text
orca_profile_mismatch
orca_schema_mismatch
orca_identity_unresolved
orca_identity_drift
orca_coordinator_unavailable
orca_precondition_failed
orca_operation_outcome_unknown
orca_operation_recovery_required
orca_receipt_identity_conflict
```

Map installed Orca errors:

- `operation_unknown`: pending or recovery-required with request and Dispatch evidence
- `request_mismatch`: identity conflict, no replay
- `consumer_fenced`: restore exact Run binding under the mutex, then retry the same stage request only when no effect was created
- `task_not_startable`: stale precondition, no new Dispatch
- `dispatch_not_found`: recovery required when Jcode has a stored Dispatch ID
- `dispatch_inactive`: stale precondition, no new control mutation
- `runtime_unavailable`: unavailable only when no process invocation could have reached Orca, otherwise outcome unknown
- any unknown code or response shape: profile mismatch and capabilities disabled

## 17. Exact test cases

### 17.1 Runner and schema tests

1. Parse `ok: true` plus worker-start `failed` from nonzero exit without returning `OrcaUnavailable`.
2. Parse `ok: true` plus `outcome_unknown` from nonzero exit and preserve residual resources.
3. Parse `ok: false` plus `operation_unknown` and its request/Dispatch recovery data.
4. Treat invalid JSON after child start as outcome unknown.
5. Treat child spawn failure as unavailable and prove no operation stage advanced.
6. Reject exit-state disagreement.
7. Reject unknown lifecycle enum or extra schema shape under the pinned profile.
8. Verify every mutation argv contains the persisted `--retry-request` value.
9. Verify commands run in the adapter working directory.

### 17.2 Start tests

10. Clean start emits exact Run, Task, worker-start, and worker-show sequence.
11. Start uses `id:<worktree-id>`, never `current` or a creation placement.
12. A request record is visible in SQLite before each scripted runner call.
13. Crash after Run creation but before receipt recovers by same request and exact objective marker.
14. Crash after Task creation but before receipt recovers by same request and exact display-name marker.
15. Multiple marker matches produce recovery-required and no mutation.
16. Worker-start `operation_unknown` with Dispatch ID calls worker-show and never creates another worker.
17. Runtime restart changes a stored `starting` Dispatch to `start_unknown` and keeps the command pending.
18. Ready response is not accepted until worker-show confirms Run, Task, Dispatch, worktree, and terminal.
19. Placement mismatch after ready becomes recovery-required.
20. Duplicate browser command returns the existing operation and emits no runner calls.
21. Two concurrent duplicate commands create one operation and one Dispatch.
22. Missing or ambiguous project/setup/worktree identity invokes no mutation.
23. Missing verified coordinator terminal advertises no start capability.

### 17.3 Retry tests

24. Retry argv contains exact prior Dispatch through `--retry-of` and explicit placement.
25. Retry rebinds the coordinator terminal to the existing Run when required.
26. Retry refuses a prior Dispatch that is not the latest Task Dispatch.
27. Retry refuses prior states other than failed, stopped, or abandoned.
28. Retry refuses Task states other than failed or blocked.
29. Replacement Dispatch must differ from the prior Dispatch.
30. Retry preserves prior Jcode attempt and Dispatch causality on the new attempt.
31. Retry does not increment durable retry count before a distinct Dispatch is known.
32. Retry outcome unknown reuses its original request and never starts a second replacement.
33. Retry reconstructs placement from the stored request, not from current focus or live defaults.

### 17.4 Cancel and cleanup tests

34. Ready Dispatch follows stop, worker-show stopped, release, and completed cancellation.
35. `start_unknown` Dispatch is stoppable.
36. Stop `operation_unknown` performs read-side reconciliation without a new stop request.
37. Stop `stop_unknown` performs a separately receipted abandon.
38. Abandon requires `stale == false` and worker-show `abandoned`.
39. Abandoned worker keeps terminal and worktree recovery obligations and does not release.
40. Succeeded worker returns stale cancellation and is never relabelled cancelled.
41. Already failed, stopped, or abandoned worker preserves its actual terminal cause.
42. Release `released` and `already_released` mark only the terminal verified released.
43. Release `retained`, `release_pending`, and `release_unknown` complete cancellation with explicit recovery obligations.
44. Release unknown never invokes `terminal close`.
45. Duplicate cancel key returns the stored result and emits no new stop, abandon, or release.
46. Different cancel key against an already terminal Dispatch does not fabricate a second cancellation.
47. Worktree, setup terminal, configured tabs, and unrelated processes are never listed as released.

### 17.5 Service and capability tests

48. Runtime commands enforce initiative revision before adapter invocation.
49. Direct Orca mutation remains rejected before adapter invocation.
50. Profile or fixture mismatch advertises all mutation capabilities false.
51. Pending recovery at startup advertises mutations false until reconciliation reaches a safe state.
52. Command results preserve authoritative receipt payloads for failed and outcome-unknown attempts.
53. Cancel maps stopped or abandoned terminal evidence to `Completed`, not generic `Pending`.
54. Generated TypeScript contracts are regenerated and the stale-contract script passes.

## 18. File-by-file implementation checklist

### `crates/jcode-command-center/src/lib.rs`

- Add canonical Orca repository, host setup, host, and request ID newtypes.
- Add the request, placement, attempt, receipt, and runtime execution types in section 7.
- Change `OrcaAdapter` mutation methods to accept typed requests and return `RuntimeCommandExecution` or an equivalent receipt-bearing result.
- Extend `JcodeRunReference` and `CommandResultPayload` with migration-safe attempt and receipt fields.
- Enforce `expected_revision` for runtime commands before adapter invocation.
- Preserve authoritative payloads for failed and outcome-unknown mutations.
- Map stopped or abandoned cancel settlement to `CommandState::Completed`.
- Add typed lifecycle error variants.

### `crates/jcode-command-center/src/orca_operation_store.rs`

- Keep the existing set-once identity checks.
- Store the fully typed request in `command_payload`.
- Set `OrcaRequestRecord.orca_request_id` before every CLI invocation.
- Add `list_recoverable()` for `Recorded`, `InProgress`, `OutcomeUnknown`, and `RecoveryRequired` records.
- Add an atomic helper that appends one request or receipt together with identity, state, effects, and obligations.
- Do not add a second dispatch field for retry. Store the new Dispatch in `orca_dispatch_id` and the prior Dispatch in the typed retry request.

### `crates/jcode-app-core/src/command_center.rs`

- Add explicit configuration reads for `JCODE_COMMAND_CENTER_ORCA_COORDINATOR_TERMINAL` and `JCODE_COMMAND_CENTER_ORCA_AGENT`.
- Change `OrcaCommandRunner` to return `OrcaCommandOutput` and accept current directory and timeout.
- Preserve stdout and stderr for nonzero exits.
- Construct `OrcaLifecycleAdapter` with the operation store, pinned profile, working directory, coordinator binding, launcher, and binding mutex.
- Delegate only start, retry, and cancel. Keep every other mutation rejected.
- Keep capability flags false unless profile, configuration, store, startup reconciliation, and the task 4.5e acceptance gate are all verified.

### `crates/jcode-app-core/src/command_center/orca_lifecycle.rs`

- Implement strict installed JSON DTOs and exit-state validation.
- Implement canonical identity resolution.
- Implement stable request ID generation and persist-before-invoke helpers.
- Implement Run binding serialization.
- Implement start, retry, cancel, release, and startup reconciliation state machines.
- Implement authoritative receipt conversion and cleanup obligation mapping.

### Tests and generated contracts

- Put scripted lifecycle tests in the isolated lifecycle test module.
- Consume task 4.5b's pinned fixtures instead of copying JSON literals into many tests.
- Regenerate the TypeScript contract after public DTO changes.
- Do not mark tasks 4.5c-d complete until the focused tests pass. Do not advertise capabilities until task 4.5e also passes.

## 19. Verification commands

After implementation:

```text
cargo test -p jcode-command-center orca_operation_store
cargo test -p jcode-command-center runtime_command
cargo test -p jcode-app-core command_center::orca_lifecycle
cargo test -p jcode-app-core command_center
scripts/check-command-center-contracts.sh
cargo test -p jcode-command-center
cargo test -p jcode-app-core command_center
```

Task 4.5e must then run the isolated live acceptance matrix before `RuntimeMutationCapabilities` returns true outside tests.

## 20. Non-negotiable invariants

1. Persist before invoke.
2. Pass the persisted request ID on the first Orca mutation call.
3. Parse JSON before exit status.
4. Nonzero does not mean unavailable.
5. Never replay with a new request ID after an unknown result.
6. Reconcile known Run, Task, and Dispatch identities before continuing.
7. Use exact existing placement only.
8. Retry the exact prior Dispatch and create a distinct replacement Dispatch.
9. Stop and abandon are distinct mutations with distinct receipts.
10. Abandon never claims the process stopped.
11. Release is cleanup, not cancellation.
12. A Run is a namespace, not terminal outcome authority.
