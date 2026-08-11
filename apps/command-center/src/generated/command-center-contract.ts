export type EntityId = string;
export type Freshness = "live" | "stale" | "unavailable" | "loading" | "error";
export type InitiativeStatus = "active" | "blocked" | "completed" | "archived";
export type StepStatus = "pending" | "running" | "blocked" | "completed";
export type CommandState = "pending" | "completed" | "failed";

export interface ProtocolMeta {
  protocolVersion: "command-center.v1";
  snapshotRevision: number;
  streamId: string;
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
  id: EntityId;
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
  id: EntityId;
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
  id: EntityId;
  initiativeId: EntityId;
  status: "idle" | "running" | "failed" | "completed" | "canceling";
  health: Freshness;
  orcaProjectId?: string;
  orcaRunId?: string;
  lastObservedAt?: string;
  workers: WorkerProjection[];
  gates: GateProjection[];
  timeline: TimelineEvent[];
  attention: string[];
  availableActions: Pick<AvailableActions, "startRun" | "retryRun" | "cancelRun">;
}
export interface CommandCenterSnapshot {
  meta: ProtocolMeta;
  initiatives: InitiativeProjection[];
  selectedInitiative?: InitiativeProjection;
  selectedRun?: RunProjection;
  connection: { state: Freshness; reason?: string; lastConnectedAt?: string };
}
export type EventPayload =
  | { type: "initiative_updated"; initiative: InitiativeProjection }
  | { type: "run_updated"; run: RunProjection }
  | { type: "timeline_appended"; runId: EntityId; event: TimelineEvent }
  | { type: "snapshot_required"; reason: string }
  | { type: "unknown"; rawType: string };
export interface EventEnvelope {
  protocolVersion: "command-center.v1";
  streamId: string;
  sequence: number;
  timestamp: string;
  source: "jcode" | "orca";
  entityRefs: EntityId[];
  payload: EventPayload;
}
export type CommandPayload =
  | {
      type: "checkpoint_initiative";
      initiativeId: EntityId;
      expectedRevision: number;
      summary: string;
      blockers: string[];
      nextActions: string[];
    }
  | {
      type: "update_step";
      initiativeId: EntityId;
      expectedRevision: number;
      milestoneId: EntityId;
      stepId: EntityId;
      status: StepStatus;
    }
  | {
      type: "start_initiative_run" | "retry_linked_run" | "cancel_linked_run";
      initiativeId: EntityId;
      runId?: EntityId;
      expectedRevision: number;
    };
export interface CommandEnvelope {
  idempotencyKey: string;
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
