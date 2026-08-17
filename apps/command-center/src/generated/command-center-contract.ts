declare const idBrand: unique symbol;
export type Brand<Value, Name extends string> = Value & { readonly [idBrand]?: Name };

export type InitiativeId = Brand<string, "InitiativeId">;
export type ScheduleRefId = Brand<string, "ScheduleRefId">;
export type JcodeRunId = Brand<string, "JcodeRunId">;
export type OrcaProjectId = Brand<string, "OrcaProjectId">;
export type OrcaRunId = Brand<string, "OrcaRunId">;
export type StreamId = Brand<string, "StreamId">;
export type CommandId = Brand<string, "CommandId">;
export type IdempotencyKey = Brand<string, "IdempotencyKey">;

export type EntityId = string;
export type Freshness = "live" | "stale" | "unavailable" | "loading" | "error";
export type InitiativeStatus = "active" | "blocked" | "completed" | "archived";
export type StepStatus = "pending" | "running" | "blocked" | "completed";
export type CommandState = "pending" | "completed" | "failed";

export interface ProtocolMeta {
  protocolVersion: "command-center.v1";
  snapshotRevision: number;
  streamId: StreamId;
  sequence: number;
}

export interface MilestoneStep {
  id: EntityId;
  title: string;
  status: StepStatus;
  evidence?: string;
}
export interface Milestone {
  id: EntityId;
  title: string;
  status: StepStatus;
  steps: MilestoneStep[];
}
export interface Checkpoint {
  id: EntityId;
  summary: string;
  createdAt: string;
}
export interface ScheduleProjection {
  id: ScheduleRefId;
  cadence: string;
  timezone: string;
  nextFire?: string;
  lastResult?: string;
  retryState?: string;
  freshness: Freshness;
  evidence?: string;
}
export interface ChildReference {
  id: EntityId;
  kind: "openspec" | "bead" | "session";
  title: string;
  href?: string;
}
export interface AvailableActions {
  checkpoint: boolean;
  updateMilestone: boolean;
  startRun: boolean;
  retryRun: boolean;
  cancelRun: boolean;
}
export interface InitiativeProjection {
  id: InitiativeId;
  title: string;
  outcome: string;
  status: InitiativeStatus;
  revision: number;
  currentMilestone: Milestone;
  successCriteria: string[];
  blockers: string[];
  nextActions: string[];
  children: ChildReference[];
  schedules: ScheduleProjection[];
  checkpoints: Checkpoint[];
  freshness: Freshness;
  updatedAt: string;
  availableActions: AvailableActions;
}
export interface WorkerProjection {
  id: EntityId;
  label: string;
  status: string;
  sessionId?: string;
  attention?: string;
}
export interface GateProjection {
  id: EntityId;
  title: string;
  status: "open" | "approved" | "rejected" | "blocked";
}
export interface TimelineEvent {
  id: EntityId;
  sequence: number;
  timestamp: string;
  source: "jcode" | "orca" | "client";
  message: string;
  severity: "info" | "warning" | "error";
}
export interface RunProjection {
  id: JcodeRunId;
  initiativeId: InitiativeId;
  status: "idle" | "running" | "failed" | "completed" | "canceling";
  health: Freshness;
  orcaProjectId?: OrcaProjectId;
  orcaRunId?: OrcaRunId;
  lastObservedAt?: string;
  workers: WorkerProjection[];
  gates: GateProjection[];
  timeline: TimelineEvent[];
  attention: string[];
  availableActions: Pick<AvailableActions, "startRun" | "retryRun" | "cancelRun">;
}
export interface InitiativeListSnapshot {
  meta: ProtocolMeta;
  initiatives: InitiativeProjection[];
  connection: { state: Freshness; reason?: string; lastConnectedAt?: string };
}
export interface CommandCenterSnapshot extends InitiativeListSnapshot {
  selectedInitiative?: InitiativeProjection;
  selectedRun?: RunProjection;
}
export type DecisionInboxCategory =
  "work_request" | "research_request" | "status_request" | "unrecognized" | "unauthorized";
export type DecisionInboxStatus =
  | "awaiting_approval"
  | "approved"
  | "read_only"
  | "unrecognized"
  | "unauthorized"
  | "deferred"
  | "classification_failed";
export interface DecisionInboxItem {
  recordId: number;
  source: { adapter: string; senderIdentity: string; conversation: string };
  receivedAt: string;
  content?: string;
  category?: DecisionInboxCategory;
  status: DecisionInboxStatus;
  proposal?: { id: number; state: string };
  trackedWork?: number;
  dedupeKey: string;
  duplicateDeliveries: number;
  retryDeliveries: number;
  redacted: boolean;
  rawPayloadRetained: boolean;
}
export interface DecisionInboxSnapshot {
  generatedAt: string;
  items: DecisionInboxItem[];
}
export type EventPayload =
  | { type: "initiative_updated"; initiative: InitiativeProjection }
  | { type: "run_updated"; run: RunProjection }
  | { type: "timeline_appended"; runId: JcodeRunId; event: TimelineEvent }
  | { type: "snapshot_required"; reason: string }
  | ({ type: "unknown"; name: string; requires_snapshot: boolean } & Record<string, unknown>);
export interface EventEnvelope {
  protocolVersion: "command-center.v1";
  streamId: StreamId;
  sequence: number;
  timestamp: string;
  source: "jcode" | "orca";
  entityRefs: EntityId[];
  payload: EventPayload;
}
export type CommandPayload =
  | {
      type: "checkpoint_initiative";
      initiativeId: InitiativeId;
      expectedRevision: number;
      summary: string;
      blockers: string[];
      nextActions: string[];
    }
  | {
      type: "update_step";
      initiativeId: InitiativeId;
      expectedRevision: number;
      milestoneId: EntityId;
      stepId: EntityId;
      status: StepStatus;
    }
  | {
      type: "start_initiative_run" | "retry_linked_run" | "cancel_linked_run";
      initiativeId: InitiativeId;
      runId?: JcodeRunId;
      expectedRevision: number;
    };
export interface CommandEnvelope {
  idempotencyKey: IdempotencyKey;
  payload: CommandPayload;
}
export interface CommandResult {
  state: CommandState;
  correlationId: string;
  snapshot?: CommandCenterSnapshot;
  error?: {
    code: "stale_revision" | "forbidden" | "unavailable" | "invalid" | "reauthentication_required";
    message: string;
    inspect?: string;
  };
}
